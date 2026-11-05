use std::collections::HashSet;

use axum::{
    Json, Router,
    extract::{FromRequestParts, OptionalFromRequestParts, Path, Query, State},
    http::{HeaderMap, Method, StatusCode, header, request::Parts},
    response::{AppendHeaders, IntoResponse, Redirect, Response},
    routing::{delete, get, post},
};
use chrono::{DateTime, TimeDelta, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{ApiError, AppState, ErrorBody};

pub const CLAIMED_ID_PREFIX: &str = "https://steamcommunity.com/openid/id/";
const OPENID_NS: &str = "http://specs.openid.net/auth/2.0";
const IDENTIFIER_SELECT: &str = "http://specs.openid.net/auth/2.0/identifier_select";
const SESSION_MAX_AGE: i64 = 30 * 24 * 3600;
const STATE_COOKIE: &str = "oidstate";
const USER_CODE_ALPHABET: &[u8] = b"BCDFGHJKLMNPQRSTVWXZ";
const DEVICE_CODE_TTL: i64 = 600;
const CLEAR_SESSION: &str = "session=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0";

fn err(status: StatusCode, message: &str) -> ApiError {
    ApiError::new(status, message)
}

pub fn random_token() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("system random generator");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

pub fn parse_claimed_id(claimed_id: &str) -> Option<&str> {
    let id = claimed_id.strip_prefix(CLAIMED_ID_PREFIX)?;
    (id.len() == 17 && id.bytes().all(|b| b.is_ascii_digit())).then_some(id)
}

pub fn safe_redirect(redirect: Option<&str>) -> &str {
    match redirect {
        Some(p)
            if p.starts_with('/')
                && !p.starts_with("//")
                && !p.starts_with("/\\")
                && p.bytes().all(|b| b.is_ascii_graphic()) =>
        {
            p
        }
        _ => "/",
    }
}

fn cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get_all(header::COOKIE).iter().filter_map(|v| v.to_str().ok()).flat_map(|v| v.split(';')).find_map(|pair| {
        let (k, v) = pair.trim().split_once('=')?;
        (k == name).then_some(v)
    })
}

fn check_origin(s: &AppState, method: &Method, headers: &HeaderMap) -> Result<(), ApiError> {
    if matches!(*method, Method::GET | Method::HEAD) {
        return Ok(());
    }
    match headers.get(header::ORIGIN) {
        Some(origin) if origin.as_bytes() != s.config.public_url.as_bytes() => {
            Err(err(StatusCode::FORBIDDEN, "Origin is not allowed"))
        }
        _ => Ok(()),
    }
}

pub struct CurrentUser {
    pub id: i64,
    pub spam_account_id: Option<Uuid>,
    pub is_admin: bool,
    pub via_cookie: bool,
}

#[derive(FromRow)]
struct UserRow {
    id: i64,
    banned: bool,
    spam_account_id: Option<Uuid>,
    is_admin: bool,
}

pub struct AnyUser(pub CurrentUser);

pub fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("Bearer "))
}

async fn authenticate(parts: &mut Parts, s: &AppState) -> Result<(CurrentUser, bool), ApiError> {
    let (row, via_cookie) = if let Some(token) = bearer(&parts.headers) {
        let row = sqlx::query_as::<_, UserRow>(
            "UPDATE devices d SET last_used_at = now() FROM users u
             WHERE d.token_hash = $1 AND u.id = d.user_id
             RETURNING u.id, u.banned_at IS NOT NULL AS banned, u.spam_account_id, u.is_admin",
        )
        .bind(hash(token))
        .fetch_optional(&s.db)
        .await?;
        (row, false)
    } else if let Some(token) = cookie(&parts.headers, "session") {
        check_origin(s, &parts.method, &parts.headers)?;
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT u.id, u.banned_at IS NOT NULL AS banned, u.spam_account_id, u.is_admin
             FROM sessions s JOIN users u ON u.id = s.user_id
             WHERE s.token_hash = $1 AND s.expires_at > now()",
        )
        .bind(hash(token))
        .fetch_optional(&s.db)
        .await?;
        (row, true)
    } else {
        (None, false)
    };
    let row = row.ok_or_else(|| err(StatusCode::UNAUTHORIZED, "login required"))?;
    let user = CurrentUser { id: row.id, spam_account_id: row.spam_account_id, is_admin: row.is_admin, via_cookie };
    Ok((user, row.banned))
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, s: &AppState) -> Result<Self, ApiError> {
        let (user, banned) = authenticate(parts, s).await?;
        if banned && !matches!(parts.method, Method::GET | Method::HEAD) {
            return Err(err(StatusCode::FORBIDDEN, "account is banned"));
        }
        Ok(user)
    }
}

impl FromRequestParts<AppState> for AnyUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, s: &AppState) -> Result<Self, ApiError> {
        Ok(Self(authenticate(parts, s).await?.0))
    }
}

impl OptionalFromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, s: &AppState) -> Result<Option<Self>, ApiError> {
        match <Self as FromRequestParts<AppState>>::from_request_parts(parts, s).await {
            Ok(user) => Ok(Some(user)),
            Err(ApiError(StatusCode::UNAUTHORIZED, _)) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl CurrentUser {
    pub fn require_admin(&self) -> Result<(), ApiError> {
        if self.is_admin { Ok(()) } else { Err(err(StatusCode::FORBIDDEN, "admin only")) }
    }
}

fn callback_url(s: &AppState) -> String {
    format!("{}/api/auth/steam/callback", s.config.public_url)
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct LoginQuery {
    pub redirect: Option<String>,
}

#[utoipa::path(get, path = "/api/auth/steam", tag = "auth", params(LoginQuery),
    responses((status = 303, description = "Redirect to Steam")))]
pub async fn steam_login(State(s): State<AppState>, Query(q): Query<LoginQuery>) -> Result<Response, ApiError> {
    let internal = |_| err(StatusCode::INTERNAL_SERVER_ERROR, "invalid URL configuration");
    let state = random_token();
    let return_to = Url::parse_with_params(
        &callback_url(&s),
        &[("redirect", safe_redirect(q.redirect.as_deref())), ("state", state.as_str())],
    )
    .map_err(internal)?;
    let url = Url::parse_with_params(
        &s.config.steam_openid_url,
        &[
            ("openid.ns", OPENID_NS),
            ("openid.mode", "checkid_setup"),
            ("openid.return_to", return_to.as_str()),
            ("openid.realm", &s.config.public_url),
            ("openid.identity", IDENTIFIER_SELECT),
            ("openid.claimed_id", IDENTIFIER_SELECT),
        ],
    )
    .map_err(internal)?;
    let cookie = format!("{STATE_COOKIE}={state}; HttpOnly; Secure; SameSite=Lax; Path=/api/auth; Max-Age=600");
    Ok(([(header::SET_COOKIE, cookie)], Redirect::to(url.as_str())).into_response())
}

fn fresh_nonce(nonce: Option<&str>, now: DateTime<Utc>) -> bool {
    nonce
        .and_then(|n| n.get(..20))
        .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
        .map(|t| now - t.to_utc())
        .is_some_and(|age| age <= TimeDelta::minutes(5) && age >= -TimeDelta::minutes(1))
}

async fn check_authentication(s: &AppState, params: &[(String, String)]) -> Result<bool, ApiError> {
    let form: Vec<(&str, &str)> = params
        .iter()
        .filter(|(k, _)| k.starts_with("openid."))
        .map(|(k, v)| (k.as_str(), if k == "openid.mode" { "check_authentication" } else { v }))
        .collect();
    let bad_gateway = |e: reqwest::Error| {
        tracing::warn!(error = %e, "Steam check_authentication failed");
        err(StatusCode::BAD_GATEWAY, "Steam verification failed")
    };
    let body = s
        .http
        .post(&s.config.steam_openid_url)
        .form(&form)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(bad_gateway)?
        .text()
        .await
        .map_err(bad_gateway)?;
    Ok(body.lines().any(|l| l.trim() == "is_valid:true"))
}

async fn player_summary(s: &AppState, steam_id: &str) -> Option<(String, Option<String>)> {
    #[derive(Deserialize)]
    struct Body {
        response: Players,
    }
    #[derive(Deserialize)]
    struct Players {
        players: Vec<Player>,
    }
    #[derive(Deserialize)]
    struct Player {
        personaname: String,
        avatarfull: Option<String>,
    }
    let key = s.config.steam_api_key.as_deref()?;
    let url = Url::parse_with_params(
        "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/",
        &[("key", key), ("steamids", steam_id)],
    )
    .ok()?;
    let res = s.http.get(url).send().await.and_then(|r| r.error_for_status());
    let text = match res {
        Ok(r) => r.text().await.ok()?,
        Err(e) => {
            tracing::warn!(error = %e.without_url(), "GetPlayerSummaries failed");
            return None;
        }
    };
    let player = serde_json::from_str::<Body>(&text).ok()?.response.players.into_iter().next()?;
    Some((player.personaname, player.avatarfull))
}

async fn create_session(db: &PgPool, user_id: i64) -> Result<String, sqlx::Error> {
    let token = random_token();
    sqlx::query("INSERT INTO sessions (token_hash, user_id, expires_at) VALUES ($1, $2, now() + interval '30 days')")
        .bind(hash(&token))
        .bind(user_id)
        .execute(db)
        .await?;
    Ok(token)
}

#[utoipa::path(get, path = "/api/auth/steam/callback", tag = "auth",
    responses((status = 303, description = "Session cookie set, redirect to the page"), (status = 401, body = ErrorBody)))]
pub async fn steam_callback(
    State(s): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<Vec<(String, String)>>,
) -> Result<Response, ApiError> {
    let denied = |m: &str| err(StatusCode::UNAUTHORIZED, m);
    let mut seen = HashSet::new();
    if !params.iter().all(|(k, _)| seen.insert(k.as_str())) {
        return Err(denied("duplicate OpenID parameter"));
    }
    let get = |k: &str| params.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
    if get("openid.ns") != Some(OPENID_NS) || get("openid.mode") != Some("id_res") {
        return Err(denied("invalid OpenID response"));
    }
    if get("openid.op_endpoint") != Some(s.config.steam_openid_url.as_str()) {
        return Err(denied("invalid OpenID endpoint"));
    }
    let callback = callback_url(&s);
    let return_to = get("openid.return_to").unwrap_or_default();
    if return_to != callback && !return_to.starts_with(&format!("{callback}?")) {
        return Err(denied("invalid OpenID return_to"));
    }
    let signed: Vec<&str> = get("openid.signed").unwrap_or_default().split(',').collect();
    if !["op_endpoint", "claimed_id", "identity", "return_to", "response_nonce"].iter().all(|f| signed.contains(f)) {
        return Err(denied("OpenID response is not fully signed"));
    }
    let return_query: Vec<(String, String)> =
        Url::parse(return_to).map(|u| u.query_pairs().into_owned().collect()).unwrap_or_default();
    let query = |k: &str| return_query.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
    if query("state").is_none() || query("state") != cookie(&headers, STATE_COOKIE) {
        return Err(denied("invalid login state"));
    }
    if !fresh_nonce(get("openid.response_nonce"), Utc::now()) {
        return Err(denied("OpenID nonce is expired"));
    }
    let steam_id = get("openid.claimed_id").and_then(parse_claimed_id).ok_or_else(|| denied("invalid Steam ID"))?;
    if !check_authentication(&s, &params).await? {
        return Err(denied("Steam did not validate the login"));
    }
    let redirect = safe_redirect(query("redirect")).to_string();

    let (name, avatar) = player_summary(&s, steam_id).await.unzip();
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (steam_id, display_name, avatar_url, banned_at)
         VALUES ($1, COALESCE($2, $1), $3, (SELECT banned_at FROM banned_steam_ids WHERE steam_id = $1))
         ON CONFLICT (steam_id) DO UPDATE
         SET display_name = COALESCE($2, users.display_name), avatar_url = COALESCE($3, users.avatar_url),
             banned_at = COALESCE(users.banned_at, EXCLUDED.banned_at)
         RETURNING id",
    )
    .bind(steam_id)
    .bind(name)
    .bind(avatar.flatten())
    .fetch_one(&s.db)
    .await?;
    let token = create_session(&s.db, user_id).await?;
    let cookie = format!("session={token}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={SESSION_MAX_AGE}");
    let clear_state = format!("{STATE_COOKIE}=; HttpOnly; Secure; SameSite=Lax; Path=/api/auth; Max-Age=0");
    Ok((
        StatusCode::SEE_OTHER,
        AppendHeaders([(header::LOCATION, redirect), (header::SET_COOKIE, cookie), (header::SET_COOKIE, clear_state)]),
    )
        .into_response())
}

#[utoipa::path(post, path = "/api/auth/logout", tag = "auth",
    responses((status = 204, description = "Session deleted"), (status = 403, body = ErrorBody)))]
pub async fn logout(State(s): State<AppState>, method: Method, headers: HeaderMap) -> Result<Response, ApiError> {
    if let Some(token) = cookie(&headers, "session") {
        check_origin(&s, &method, &headers)?;
        sqlx::query("DELETE FROM sessions WHERE token_hash = $1").bind(hash(token)).execute(&s.db).await?;
    }
    Ok((StatusCode::NO_CONTENT, [(header::SET_COOKIE, CLEAR_SESSION)]).into_response())
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Me {
    pub id: i64,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub spam_account_id: Option<Uuid>,
    pub is_admin: bool,
}

#[utoipa::path(get, path = "/api/me", tag = "auth",
    responses((status = 200, body = Me), (status = 401, body = ErrorBody)))]
pub async fn me(State(s): State<AppState>, user: CurrentUser) -> Result<Json<Me>, ApiError> {
    let me = sqlx::query_as("SELECT id, display_name, avatar_url, spam_account_id, is_admin FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(&s.db)
        .await?;
    Ok(Json(me))
}

#[utoipa::path(delete, path = "/api/me", tag = "auth",
    responses((status = 204, description = "Account and all its data deleted, session cookie cleared. Banned users can also do this."),
        (status = 401, body = ErrorBody), (status = 403, body = ErrorBody)))]
pub async fn delete_me(State(s): State<AppState>, AnyUser(user): AnyUser) -> Result<Response, ApiError> {
    sqlx::query("DELETE FROM users WHERE id = $1").bind(user.id).execute(&s.db).await?;
    Ok((StatusCode::NO_CONTENT, [(header::SET_COOKIE, CLEAR_SESSION)]).into_response())
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub expires_in: i64,
    pub interval: i64,
}

fn new_user_code() -> String {
    let mut code = String::with_capacity(8);
    while code.len() < 8 {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).expect("system random generator");
        for b in bytes {
            if code.len() < 8 && usize::from(b) < 256 - 256 % USER_CODE_ALPHABET.len() {
                code.push(char::from(USER_CODE_ALPHABET[usize::from(b) % USER_CODE_ALPHABET.len()]));
            }
        }
    }
    code
}

fn normalize_user_code(code: &str) -> String {
    code.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_uppercase()).collect()
}

#[utoipa::path(post, path = "/api/device/code", tag = "device",
    responses((status = 200, body = DeviceCode), (status = 429, body = ErrorBody)))]
pub async fn device_code(State(s): State<AppState>) -> Result<Json<DeviceCode>, ApiError> {
    let pending: i64 =
        sqlx::query_scalar("SELECT count(*) FROM device_codes WHERE user_id IS NULL AND expires_at > now()")
            .fetch_one(&s.db)
            .await?;
    if pending >= s.config.max_pending_device_codes {
        return Err(err(StatusCode::TOO_MANY_REQUESTS, "too many pending device codes"));
    }
    let device_code = random_token();
    for _ in 0..5 {
        let code = new_user_code();
        let inserted = sqlx::query(
            "INSERT INTO device_codes (device_code_hash, user_code_hash, expires_at)
             VALUES ($1, $2, now() + make_interval(secs => $3))
             ON CONFLICT (user_code_hash) DO NOTHING",
        )
        .bind(hash(&device_code))
        .bind(hash(&code))
        .bind(DEVICE_CODE_TTL as f64)
        .execute(&s.db)
        .await?
        .rows_affected();
        if inserted == 1 {
            let user_code = format!("{}-{}", &code[..4], &code[4..]);
            return Ok(Json(DeviceCode {
                verification_url: format!("{}/link?code={user_code}", s.config.public_url),
                device_code,
                user_code,
                expires_in: DEVICE_CODE_TTL,
                interval: 5,
            }));
        }
    }
    Err(err(StatusCode::SERVICE_UNAVAILABLE, "no free user code"))
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfirm {
    pub user_code: String,
}

#[utoipa::path(post, path = "/api/device/confirm", tag = "device", request_body = DeviceConfirm,
    responses((status = 204, description = "Device code confirmed"), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody)))]
pub async fn device_confirm(
    State(s): State<AppState>,
    user: CurrentUser,
    Json(body): Json<DeviceConfirm>,
) -> Result<StatusCode, ApiError> {
    if !user.via_cookie {
        return Err(err(StatusCode::FORBIDDEN, "a web session is required"));
    }
    let updated = sqlx::query(
        "UPDATE device_codes SET user_id = $1
         WHERE user_code_hash = $2 AND user_id IS NULL AND expires_at > now()",
    )
    .bind(user.id)
    .bind(hash(&normalize_user_code(&body.user_code)))
    .execute(&s.db)
    .await?
    .rows_affected();
    if updated == 0 {
        return Err(err(StatusCode::NOT_FOUND, "unknown or expired code"));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTokenRequest {
    pub device_code: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeviceToken {
    pub access_token: String,
    pub token_type: String,
}

#[utoipa::path(post, path = "/api/device/token", tag = "device", request_body = DeviceTokenRequest,
    responses((status = 200, body = DeviceToken),
        (status = 400, body = ErrorBody, description = "error is authorization_pending, expired_token or invalid_grant")))]
pub async fn device_token(
    State(s): State<AppState>,
    Json(body): Json<DeviceTokenRequest>,
) -> Result<Json<DeviceToken>, ApiError> {
    let code_hash = hash(&body.device_code);
    let mut tx = s.db.begin().await?;
    let row: Option<(Option<i64>, bool)> = sqlx::query_as(
        "DELETE FROM device_codes
         WHERE device_code_hash = $1 AND (user_id IS NOT NULL OR expires_at <= now())
         RETURNING user_id, expires_at > now()",
    )
    .bind(&code_hash)
    .fetch_optional(&mut *tx)
    .await?;
    let user_id = match row {
        Some((Some(user_id), true)) => user_id,
        Some(_) => {
            tx.commit().await?;
            return Err(err(StatusCode::BAD_REQUEST, "expired_token"));
        }
        None => {
            let pending: bool =
                sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM device_codes WHERE device_code_hash = $1)")
                    .bind(&code_hash)
                    .fetch_one(&mut *tx)
                    .await?;
            let code = if pending { "authorization_pending" } else { "invalid_grant" };
            return Err(err(StatusCode::BAD_REQUEST, code));
        }
    };
    let token = random_token();
    sqlx::query("INSERT INTO devices (user_id, token_hash) VALUES ($1, $2)")
        .bind(user_id)
        .bind(hash(&token))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(DeviceToken { access_token: token, token_type: "Bearer".into() }))
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[utoipa::path(get, path = "/api/me/devices", tag = "device",
    responses((status = 200, body = Vec<Device>), (status = 401, body = ErrorBody)))]
pub async fn list_devices(State(s): State<AppState>, user: CurrentUser) -> Result<Json<Vec<Device>>, ApiError> {
    let devices = sqlx::query_as("SELECT id, created_at, last_used_at FROM devices WHERE user_id = $1 ORDER BY id")
        .bind(user.id)
        .fetch_all(&s.db)
        .await?;
    Ok(Json(devices))
}

#[utoipa::path(delete, path = "/api/me/devices/{id}", tag = "device",
    params(("id" = i64, Path, description = "Device ID")),
    responses((status = 204, description = "Device revoked"), (status = 404, body = ErrorBody)))]
pub async fn delete_device(
    State(s): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let deleted = sqlx::query("DELETE FROM devices WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&s.db)
        .await?
        .rows_affected();
    if deleted == 0 {
        return Err(err(StatusCode::NOT_FOUND, "unknown device"));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth/steam", get(steam_login))
        .route("/api/auth/steam/callback", get(steam_callback))
        .route("/api/auth/logout", post(logout))
        .route("/api/me", get(me).delete(delete_me))
        .route("/api/me/devices", get(list_devices))
        .route("/api/me/devices/{id}", delete(delete_device))
        .route("/api/device/code", post(device_code))
        .route("/api/device/confirm", post(device_confirm))
        .route("/api/device/token", post(device_token))
}
