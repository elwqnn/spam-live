use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use axum::{
    Router,
    extract::State,
    http::{StatusCode, header},
    response::Response,
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{ApiError, AppState, public};

const OK_TTL: Duration = Duration::from_secs(600);
const ERROR_TTL: Duration = Duration::from_secs(60);

pub type Cache = Mutex<Option<(Instant, Result<Release, StatusCode>)>>;

#[derive(Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub version: String,
    pub published_at: DateTime<Utc>,
    pub url: String,
    pub windows: Option<Asset>,
    pub linux: Option<Asset>,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    published_at: DateTime<Utc>,
    html_url: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

fn asset(assets: &[GithubAsset], suffix: &str) -> Option<Asset> {
    assets.iter().find(|a| a.name.ends_with(suffix) && a.browser_download_url.starts_with("https://")).map(|a| Asset {
        name: a.name.clone(),
        url: a.browser_download_url.clone(),
        size: a.size,
    })
}

fn bad_gateway(e: impl std::fmt::Display) -> StatusCode {
    tracing::warn!(error = %e, "GitHub release fetch failed");
    StatusCode::BAD_GATEWAY
}

async fn fetch(s: &AppState) -> Result<Release, StatusCode> {
    let res = s
        .http
        .get(&s.config.github_release_url)
        .header(header::USER_AGENT, "spam-live")
        .header(header::ACCEPT, "application/vnd.github+json")
        .timeout(Duration::from_secs(5))
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
    if !raw.html_url.starts_with("https://") {
        return Err(bad_gateway("release url is not https"));
    }
    Ok(Release {
        version: raw.tag_name.strip_prefix("desktop-v").unwrap_or(&raw.tag_name).to_string(),
        published_at: raw.published_at,
        windows: asset(&raw.assets, "-setup.exe"),
        linux: asset(&raw.assets, ".AppImage"),
        url: raw.html_url,
    })
}

#[utoipa::path(get, path = "/api/desktop/release", tag = "desktop",
    responses((status = 200, body = Release, description = "The latest desktop release on GitHub. \
        windows and linux are null when the release has no such asset."),
        (status = 404, body = crate::ErrorBody, description = "No release"),
        (status = 502, body = crate::ErrorBody)))]
pub async fn release(State(s): State<AppState>) -> Result<Response, ApiError> {
    let cached = s.release.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let result = match cached {
        Some((until, result)) if Instant::now() < until => result,
        _ => {
            let result = fetch(&s).await;
            let ttl = if result.is_ok() { OK_TTL } else { ERROR_TTL };
            *s.release.lock().unwrap_or_else(|e| e.into_inner()) = Some((Instant::now() + ttl, result.clone()));
            result
        }
    };
    match result {
        Ok(release) => Ok(public(release)),
        Err(StatusCode::NOT_FOUND) => Err(ApiError::new(StatusCode::NOT_FOUND, "no release")),
        Err(_) => Err(ApiError::new(StatusCode::BAD_GATEWAY, "release info is not available")),
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/desktop/release", get(release))
}
