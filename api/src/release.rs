use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use axum::{
    Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

use crate::{ApiError, AppState, catalog::parse_version, public};

const OK_TTL: Duration = Duration::from_secs(600);
const ERROR_TTL: Duration = Duration::from_secs(60);
const TIMEOUT: Duration = Duration::from_secs(5);
const SIGNATURE_MAX: usize = 4096;

pub type Cache = Mutex<Option<(Instant, Result<Release, StatusCode>)>>;

#[derive(Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub version: String,
    pub published_at: DateTime<Utc>,
    pub url: String,
    pub windows: Option<Asset>,
    pub linux: Option<Asset>,
    #[serde(skip)]
    pub notes: String,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub size: u64,
    #[serde(skip)]
    pub signature: Result<Option<String>, StatusCode>,
}

#[derive(Serialize, ToSchema)]
pub struct Update {
    pub version: String,
    pub notes: String,
    pub pub_date: DateTime<Utc>,
    pub url: String,
    pub signature: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    #[serde(default)]
    draft: bool,
    published_at: Option<DateTime<Utc>>,
    html_url: String,
    body: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

async fn asset(s: &AppState, assets: &[GithubAsset], suffix: &str) -> Option<Asset> {
    let found = assets.iter().find(|a| a.name.ends_with(suffix) && a.browser_download_url.starts_with("https://"))?;
    let sig = format!("{}.sig", found.name);
    let signature = match assets.iter().find(|a| a.name == sig && trusted(&a.browser_download_url, s)) {
        Some(sig) => signature(s, &sig.browser_download_url).await.map(Some),
        None => Ok(None),
    };
    Some(Asset { name: found.name.clone(), url: found.browser_download_url.clone(), size: found.size, signature })
}

fn trusted(url: &str, s: &AppState) -> bool {
    let origin = |u: &str| Url::parse(u).ok().map(|u| u.origin());
    url.starts_with("https://") || origin(url).is_some_and(|o| Some(o) == origin(&s.config.github_release_url))
}

async fn signature(s: &AppState, url: &str) -> Result<String, StatusCode> {
    let mut res = s
        .http
        .get(url)
        .header(header::USER_AGENT, "spam-live")
        .timeout(TIMEOUT)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| bad_gateway(e.without_url()))?;
    let mut body = Vec::new();
    while let Some(chunk) = res.chunk().await.map_err(|e| bad_gateway(e.without_url()))? {
        body.extend_from_slice(&chunk);
        if body.len() > SIGNATURE_MAX {
            return Err(bad_gateway("signature is too large"));
        }
    }
    let text = String::from_utf8(body).map_err(bad_gateway)?.trim().to_string();
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_alphanumeric() || b"+/=".contains(&b)) {
        return Err(bad_gateway("signature is not base64"));
    }
    Ok(text)
}

fn bad_gateway(e: impl std::fmt::Display) -> StatusCode {
    tracing::warn!(error = %e, "GitHub release fetch failed");
    StatusCode::BAD_GATEWAY
}

fn newer(latest: &[u64], current: &[u64]) -> bool {
    let len = latest.len().max(current.len());
    let pad = |v: &[u64]| {
        let mut v = v.to_vec();
        v.resize(len, 0);
        v
    };
    pad(latest) > pad(current)
}

async fn fetch(s: &AppState) -> Result<Release, StatusCode> {
    let res = s
        .http
        .get(&s.config.github_release_url)
        .header(header::USER_AGENT, "spam-live")
        .header(header::ACCEPT, "application/vnd.github+json")
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|e| bad_gateway(e.without_url()))?;
    if res.status() == StatusCode::NOT_FOUND {
        return Err(StatusCode::NOT_FOUND);
    }
    let bytes = res
        .error_for_status()
        .map_err(|e| bad_gateway(e.without_url()))?
        .bytes()
        .await
        .map_err(|e| bad_gateway(e.without_url()))?;
    let raw: GithubRelease = serde_json::from_slice(&bytes).map_err(bad_gateway)?;
    let Some(published_at) = raw.published_at.filter(|_| !raw.draft) else {
        return Err(StatusCode::NOT_FOUND);
    };
    if !raw.html_url.starts_with("https://") {
        return Err(bad_gateway("release url is not https"));
    }
    let (windows, linux) = tokio::join!(asset(s, &raw.assets, "-setup.exe"), asset(s, &raw.assets, ".AppImage"));
    Ok(Release {
        version: raw.tag_name.strip_prefix("desktop-v").unwrap_or(&raw.tag_name).to_string(),
        published_at,
        windows,
        linux,
        url: raw.html_url,
        notes: raw.body.unwrap_or_default(),
    })
}

async fn cached(s: &AppState) -> Result<Release, StatusCode> {
    let cached = s.release.lock().unwrap_or_else(|e| e.into_inner()).clone();
    match cached {
        Some((until, result)) if Instant::now() < until => result,
        _ => {
            let result = fetch(s).await;
            let signed = |r: &Release| [&r.windows, &r.linux].into_iter().flatten().all(|a| a.signature.is_ok());
            let ttl = if result.as_ref().is_ok_and(signed) { OK_TTL } else { ERROR_TTL };
            *s.release.lock().unwrap_or_else(|e| e.into_inner()) = Some((Instant::now() + ttl, result.clone()));
            result
        }
    }
}

fn unavailable() -> ApiError {
    ApiError::new(StatusCode::BAD_GATEWAY, "release info is not available")
}

#[utoipa::path(get, path = "/api/desktop/release", tag = "desktop",
    responses((status = 200, body = Release, description = "The latest desktop release on GitHub. \
        windows and linux are null when the release has no such asset."),
        (status = 404, body = crate::ErrorBody, description = "No release"),
        (status = 502, body = crate::ErrorBody)))]
pub async fn release(State(s): State<AppState>) -> Result<Response, ApiError> {
    match cached(&s).await {
        Ok(release) => Ok(public(release)),
        Err(StatusCode::NOT_FOUND) => Err(ApiError::new(StatusCode::NOT_FOUND, "no release")),
        Err(_) => Err(unavailable()),
    }
}

#[utoipa::path(get, path = "/api/desktop/update/{target}/{arch}/{current_version}", tag = "desktop",
    params(("target" = String, Path, description = "windows or linux"),
        ("arch" = String, Path, description = "x86_64"),
        ("current_version" = String, Path, description = "The installed app version")),
    responses((status = 200, body = Update, description = "The Tauri updater JSON of the latest published release \
        when it is newer than current_version"),
        (status = 204, description = "No newer release with a signed asset for this target"),
        (status = 400, body = crate::ErrorBody, description = "Unsupported target or invalid version"),
        (status = 502, body = crate::ErrorBody)))]
pub async fn update(
    State(s): State<AppState>,
    Path((target, arch, current)): Path<(String, String, String)>,
) -> Result<Response, ApiError> {
    let windows = match (target.as_str(), arch.as_str()) {
        ("windows", "x86_64") => true,
        ("linux", "x86_64") => false,
        _ => return Err(ApiError::new(StatusCode::BAD_REQUEST, "unsupported target")),
    };
    let current = parse_version(&current).ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "invalid version"))?;
    let release = match cached(&s).await {
        Ok(release) => release,
        Err(StatusCode::NOT_FOUND) => return Ok(StatusCode::NO_CONTENT.into_response()),
        Err(_) => return Err(unavailable()),
    };
    let version = release.version.strip_prefix('v').unwrap_or(&release.version).to_string();
    let asset = if windows { release.windows } else { release.linux };
    let newer = parse_version(&version).is_some_and(|latest| newer(&latest, &current));
    let Some(asset) = asset.filter(|_| newer) else {
        return Ok(StatusCode::NO_CONTENT.into_response());
    };
    match asset.signature {
        Ok(Some(signature)) => Ok(public(Update {
            version,
            notes: release.notes,
            pub_date: release.published_at,
            url: asset.url,
            signature,
        })),
        Ok(None) => Ok(StatusCode::NO_CONTENT.into_response()),
        Err(_) => Err(unavailable()),
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/desktop/release", get(release))
        .route("/api/desktop/update/{target}/{arch}/{current_version}", get(update))
}
