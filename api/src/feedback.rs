use std::time::Duration;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    ApiError, AppState, ErrorBody,
    auth::CurrentUser,
    matches::{multipart_error, read_field},
};

pub const MAX_IMAGE_BYTES: usize = 3 * 1024 * 1024;
const MAX_TEXT_BYTES: usize = 32 << 10;
const MAX_SIDE: u32 = 4096;
const FEEDBACK_PER_DAY: i64 = 5;

#[derive(Serialize, Deserialize, ToSchema, sqlx::Type, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum FeedbackKind {
    Bug,
    Idea,
    Other,
}

impl FeedbackKind {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "bug" => Some(Self::Bug),
            "idea" => Some(Self::Idea),
            "other" => Some(Self::Other),
            _ => None,
        }
    }

    fn github(self) -> (&'static str, &'static str) {
        match self {
            Self::Bug => ("[Bug] ", "bug"),
            Self::Idea => ("[Idea] ", "enhancement"),
            Self::Other => ("[Other] ", "question"),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Element {
    selector: String,
    text: String,
}

#[derive(ToSchema)]
pub struct FeedbackForm {
    pub kind: FeedbackKind,
    #[schema(min_length = 10, max_length = 5000)]
    pub body: String,
    #[schema(max_length = 2000)]
    pub page: String,
    #[schema(rename = "siteVersion")]
    pub site_version: String,
    #[schema(required = false)]
    pub element: String,
    #[schema(value_type = String, format = Binary, required = false)]
    pub screenshot: Vec<u8>,
}

#[derive(Serialize, ToSchema)]
pub struct FeedbackCreated {
    pub id: i64,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Feedback {
    pub id: i64,
    pub kind: FeedbackKind,
    pub body: String,
    pub page: String,
    pub site_version: String,
    pub created_at: DateTime<Utc>,
    pub issue_number: Option<i64>,
    pub issue_url: Option<String>,
    pub image_id: Option<Uuid>,
}

#[derive(FromRow)]
struct Pending {
    id: i64,
    kind: FeedbackKind,
    body: String,
    page: String,
    site_version: String,
    user_agent: Option<String>,
    element: Option<String>,
    image_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct Issue {
    number: i64,
    html_url: String,
}

fn bad(message: impl Into<String>) -> ApiError {
    ApiError::new(StatusCode::BAD_REQUEST, message)
}

fn le(b: &[u8], at: usize, n: usize) -> Option<u32> {
    Some(b.get(at..at + n)?.iter().rev().fold(0, |acc, &x| acc << 8 | u32::from(x)))
}

pub fn image_size(b: &[u8]) -> Option<(&'static str, u32, u32)> {
    if b.starts_with(b"\x89PNG\r\n\x1a\n") && b.get(12..16) == Some(&b"IHDR"[..]) {
        let be = |at: usize| Some(u32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?));
        return Some(("image/png", be(16)?, be(20)?));
    }
    if b.get(..4) != Some(&b"RIFF"[..]) || b.get(8..12) != Some(&b"WEBP"[..]) {
        return None;
    }
    let (w, h) = match b.get(12..16)? {
        b"VP8 " if b.get(23..26) == Some(&[0x9d, 0x01, 0x2a][..]) => (le(b, 26, 2)? & 0x3fff, le(b, 28, 2)? & 0x3fff),
        b"VP8L" if b.get(20) == Some(&0x2f) => {
            let bits = le(b, 21, 4)?;
            ((bits & 0x3fff) + 1, (bits >> 14 & 0x3fff) + 1)
        }
        b"VP8X" => (le(b, 24, 3)? + 1, le(b, 27, 3)? + 1),
        _ => return None,
    };
    Some(("image/webp", w, h))
}

fn valid_page(p: &str) -> bool {
    p.starts_with('/')
        && !p.starts_with("//")
        && p.len() <= 2000
        && p.bytes().all(|b| b.is_ascii_graphic() && !b"`<>\"\\".contains(&b))
}

fn valid_version(v: &str) -> bool {
    v == "dev" || ((7..=40).contains(&v.len()) && v.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')))
}

fn code_block(text: &str) -> String {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat(3.max(longest + 1));
    format!("{fence}\n{text}\n{fence}")
}

fn issue(public_url: &str, f: &Pending) -> Value {
    let (prefix, label) = f.kind.github();
    let first = f.body.lines().next().unwrap_or_default().trim_end();
    let title: String = format!("{prefix}{first}").chars().filter(|c| !c.is_control()).take(80).collect();
    let browser = f.user_agent.as_deref().map(|ua| ua.replace('`', "")).filter(|ua| !ua.trim().is_empty());
    let mut body = format!(
        "{}\n\n- Page: `{public_url}{}`\n- Site version: `{}`\n- Browser: {}\n",
        code_block(&f.body),
        f.page,
        f.site_version,
        browser.map_or("unknown".into(), |b| format!("`{b}`")),
    );
    if let Some(e) = f.element.as_deref().and_then(|e| serde_json::from_str::<Element>(e).ok()) {
        body += &format!("\nElement:\n\n{}\n", code_block(&format!("{}\n{}", e.selector, e.text)));
    }
    if let Some(id) = f.image_id {
        body += &format!("\n![Screenshot]({public_url}/api/feedback/images/{id})\n");
    }
    json!({ "title": title, "body": body, "labels": [label] })
}

async fn create_issue(s: &AppState, token: &str, f: &Pending) -> Result<(i64, Option<String>), String> {
    let fail = |e: reqwest::Error| e.without_url().to_string();
    let bytes = s
        .http
        .post(format!("{}/repos/{}/issues", s.config.github_api_url, s.config.github_feedback_repo))
        .header(header::USER_AGENT, "spam-live")
        .header(header::ACCEPT, "application/vnd.github+json")
        .header(header::CONTENT_TYPE, "application/json")
        .bearer_auth(token)
        .body(issue(&s.config.public_url, f).to_string())
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(fail)?
        .error_for_status()
        .map_err(fail)?
        .bytes()
        .await
        .map_err(fail)?;
    let issue: Issue = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    Ok((issue.number, issue.html_url.starts_with("https://").then_some(issue.html_url)))
}

async fn publish(s: &AppState, f: &Pending) {
    let Some(token) = &s.config.github_feedback_token else {
        tracing::warn!(feedback = f.id, "GITHUB_FEEDBACK_TOKEN is not set, so no GitHub issue");
        return;
    };
    let (number, url) = match create_issue(s, token, f).await {
        Ok(issue) => issue,
        Err(e) => {
            tracing::warn!(error = %e, feedback = f.id, "GitHub issue creation failed");
            return;
        }
    };
    let updated = sqlx::query("UPDATE feedback SET issue_number = $2, issue_url = $3 WHERE id = $1")
        .bind(f.id)
        .bind(number)
        .bind(url)
        .execute(&s.db)
        .await;
    if let Err(e) = updated {
        tracing::error!(error = %e, feedback = f.id, "feedback issue update failed");
    }
}

pub async fn retry_pending(s: &AppState) {
    if s.config.github_feedback_token.is_none() {
        return;
    }
    let rows = sqlx::query_as::<_, Pending>(
        "SELECT f.id, f.kind, f.body, f.page, f.site_version, f.user_agent, f.element::text AS element,
             i.id AS image_id
         FROM feedback f LEFT JOIN feedback_images i ON i.feedback_id = f.id
         WHERE f.issue_number IS NULL AND f.created_at < now() - interval '5 minutes'
         ORDER BY f.created_at, f.id LIMIT 20",
    )
    .fetch_all(&s.db)
    .await;
    match rows {
        Ok(rows) => {
            for f in &rows {
                publish(s, f).await;
            }
        }
        Err(e) => tracing::error!(error = %e, "pending feedback query failed"),
    }
}

fn text(value: Option<Vec<u8>>, name: &str) -> Result<String, ApiError> {
    let value = String::from_utf8(value.ok_or_else(|| bad(format!("missing {name} field")))?)
        .map_err(|_| bad(format!("{name} is not UTF-8")))?;
    if value.contains('\0') {
        return Err(bad(format!("{name} contains a NUL character")));
    }
    Ok(value)
}

#[utoipa::path(post, path = "/api/feedback", tag = "feedback",
    request_body(content = FeedbackForm, content_type = "multipart/form-data",
        description = "element is JSON { selector, text }. screenshot is a WebP or PNG file, max 3 MB, each side 1 to 4096 px."),
    responses((status = 201, body = FeedbackCreated, description = "Saved. The GitHub issue can come later."),
        (status = 400, body = ErrorBody), (status = 401, body = ErrorBody), (status = 403, body = ErrorBody),
        (status = 413, body = ErrorBody), (status = 429, body = ErrorBody, description = "Max 5 feedbacks in 24 hours")))]
pub async fn create(
    State(s): State<AppState>,
    user: CurrentUser,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<FeedbackCreated>), ApiError> {
    let [mut kind, mut body, mut page, mut version, mut element, mut screenshot] = [None, None, None, None, None, None];
    while let Some(mut field) = multipart.next_field().await.map_err(multipart_error)? {
        let name = field.name().unwrap_or_default().to_owned();
        let (slot, max) = match name.as_str() {
            "kind" => (&mut kind, MAX_TEXT_BYTES),
            "body" => (&mut body, MAX_TEXT_BYTES),
            "page" => (&mut page, MAX_TEXT_BYTES),
            "siteVersion" => (&mut version, MAX_TEXT_BYTES),
            "element" => (&mut element, MAX_TEXT_BYTES),
            "screenshot" => (&mut screenshot, MAX_IMAGE_BYTES),
            _ => return Err(bad("unexpected multipart field")),
        };
        if slot.is_some() {
            return Err(bad(format!("duplicate {name} field")));
        }
        *slot = Some(read_field(&mut field, max).await?.ok_or_else(|| bad(format!("{name} is too large")))?);
    }
    let kind = FeedbackKind::parse(&text(kind, "kind")?).ok_or_else(|| bad("kind must be bug, idea or other"))?;
    let body = text(body, "body")?.trim().to_owned();
    if !(10..=5000).contains(&body.chars().count()) {
        return Err(bad("body must have 10 to 5000 characters"));
    }
    let page = text(page, "page")?;
    if !valid_page(&page) {
        return Err(bad("page must be a path of the site"));
    }
    let site_version = text(version, "siteVersion")?;
    if !valid_version(&site_version) {
        return Err(bad("siteVersion must be dev or a git commit SHA"));
    }
    let element = element
        .map(|raw| {
            let e: Element = serde_json::from_str(&text(Some(raw), "element")?)
                .map_err(|_| bad("element must be JSON { selector, text }"))?;
            if e.selector.contains('\0') || e.text.contains('\0') {
                return Err(bad("element contains a NUL character"));
            }
            if !(1..=300).contains(&e.selector.chars().count()) || e.text.chars().count() > 200 {
                return Err(bad("element selector must have 1 to 300 characters and text max 200"));
            }
            Ok(json!(e).to_string())
        })
        .transpose()?;
    let image = screenshot
        .map(|bytes| {
            let (mime, w, h) = image_size(&bytes).ok_or_else(|| bad("screenshot must be a WebP or PNG image"))?;
            if !(1..=MAX_SIDE).contains(&w) || !(1..=MAX_SIDE).contains(&h) {
                return Err(bad("screenshot sides must be 1 to 4096 px"));
            }
            Ok((bytes, mime, w as i32, h as i32))
        })
        .transpose()?;
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.is_empty())
        .map(|v| v.chars().take(512).collect::<String>());

    let mut tx = s.db.begin().await?;
    sqlx::query("SELECT 1 FROM users WHERE id = $1 FOR UPDATE").bind(user.id).execute(&mut *tx).await?;
    let recent: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM feedback WHERE user_id = $1 AND created_at > now() - interval '24 hours'",
    )
    .bind(user.id)
    .fetch_one(&mut *tx)
    .await?;
    if recent >= FEEDBACK_PER_DAY {
        return Err(ApiError::new(StatusCode::TOO_MANY_REQUESTS, "feedback limit is 5 per 24 hours"));
    }
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO feedback (user_id, kind, body, page, site_version, user_agent, element)
         VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb) RETURNING id",
    )
    .bind(user.id)
    .bind(kind)
    .bind(&body)
    .bind(&page)
    .bind(&site_version)
    .bind(&user_agent)
    .bind(&element)
    .fetch_one(&mut *tx)
    .await?;
    let image_id = match image {
        Some((bytes, mime, w, h)) => Some(
            sqlx::query_scalar(
                "INSERT INTO feedback_images (feedback_id, bytes, mime, width, height)
                 VALUES ($1, $2, $3, $4, $5) RETURNING id",
            )
            .bind(id)
            .bind(bytes)
            .bind(mime)
            .bind(w)
            .bind(h)
            .fetch_one(&mut *tx)
            .await?,
        ),
        None => None,
    };
    tx.commit().await?;

    publish(&s, &Pending { id, kind, body, page, site_version, user_agent, element, image_id }).await;
    Ok((StatusCode::CREATED, Json(FeedbackCreated { id })))
}

#[utoipa::path(get, path = "/api/feedback/images/{id}", tag = "feedback",
    params(("id" = Uuid, Path, description = "Image ID")),
    responses((status = 200, description = "The WebP or PNG bytes, cached for 1 year"), (status = 404, body = ErrorBody)))]
pub async fn image(State(s): State<AppState>, Path(id): Path<Uuid>) -> Result<Response, ApiError> {
    let (bytes, mime): (Vec<u8>, String) = sqlx::query_as("SELECT bytes, mime FROM feedback_images WHERE id = $1")
        .bind(id)
        .fetch_optional(&s.db)
        .await?
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "image not found"))?;
    let cache =
        [(header::CACHE_CONTROL, "public, max-age=31536000, immutable"), (header::X_CONTENT_TYPE_OPTIONS, "nosniff")];
    Ok(([(header::CONTENT_TYPE, mime)], cache, bytes).into_response())
}

#[utoipa::path(get, path = "/api/admin/feedback", tag = "admin",
    responses((status = 200, body = Vec<Feedback>, description = "Newest first, max 200"), (status = 403, body = ErrorBody)))]
pub async fn list(State(s): State<AppState>, user: CurrentUser) -> Result<Json<Vec<Feedback>>, ApiError> {
    user.require_admin()?;
    let rows = sqlx::query_as(
        "SELECT f.id, f.kind, f.body, f.page, f.site_version, f.created_at, f.issue_number, f.issue_url,
             i.id AS image_id
         FROM feedback f LEFT JOIN feedback_images i ON i.feedback_id = f.id
         ORDER BY f.created_at DESC, f.id DESC LIMIT 200",
    )
    .fetch_all(&s.db)
    .await?;
    Ok(Json(rows))
}

#[utoipa::path(delete, path = "/api/admin/feedback/{id}/image", tag = "admin",
    params(("id" = i64, Path, description = "Feedback ID")),
    responses((status = 204, description = "Image deleted"), (status = 403, body = ErrorBody),
        (status = 404, body = ErrorBody)))]
pub async fn delete_image(
    State(s): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    user.require_admin()?;
    let deleted = sqlx::query("DELETE FROM feedback_images WHERE feedback_id = $1")
        .bind(id)
        .execute(&s.db)
        .await?
        .rows_affected();
    if deleted == 0 {
        return Err(ApiError::new(StatusCode::NOT_FOUND, "image not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/feedback", post(create).layer(DefaultBodyLimit::max(MAX_IMAGE_BYTES + (256 << 10))))
        .route("/api/feedback/images/{id}", get(image))
        .route("/api/admin/feedback", get(list))
        .route("/api/admin/feedback/{id}/image", delete(delete_image))
}

#[cfg(test)]
mod tests {
    use super::image_size;

    fn riff(chunk: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut b = b"RIFF".to_vec();
        b.extend((4 + 8 + payload.len() as u32).to_le_bytes());
        b.extend(b"WEBP");
        b.extend(chunk);
        b.extend((payload.len() as u32).to_le_bytes());
        b.extend(payload);
        b
    }

    #[test]
    fn reads_png_and_webp_sizes() {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        png.extend(1692u32.to_be_bytes());
        png.extend(1207u32.to_be_bytes());
        png.extend([8, 6, 0, 0, 0]);
        assert_eq!(image_size(&png), Some(("image/png", 1692, 1207)));

        let mut vp8 = vec![0x10, 0x02, 0x00, 0x9d, 0x01, 0x2a];
        vp8.extend((640u16 | 0x4000).to_le_bytes());
        vp8.extend(480u16.to_le_bytes());
        assert_eq!(image_size(&riff(b"VP8 ", &vp8)), Some(("image/webp", 640, 480)));

        let bits: u32 = (1692 - 1) | ((1207 - 1) << 14);
        let mut vp8l = vec![0x2f];
        vp8l.extend(bits.to_le_bytes());
        assert_eq!(image_size(&riff(b"VP8L", &vp8l)), Some(("image/webp", 1692, 1207)));

        let mut vp8x = vec![0x10, 0, 0, 0];
        vp8x.extend(&4095u32.to_le_bytes()[..3]);
        vp8x.extend(&0u32.to_le_bytes()[..3]);
        assert_eq!(image_size(&riff(b"VP8X", &vp8x)), Some(("image/webp", 4096, 1)));

        assert_eq!(image_size(b"GIF89a\x01\0\x01\0"), None);
        assert_eq!(image_size(&riff(b"VP8 ", &[0x10, 0x02, 0x00, 0, 0, 0, 1, 0, 1, 0])), None);
        assert_eq!(image_size(&png[..20]), None);
    }
}
