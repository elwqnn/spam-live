pub mod admin;
pub mod analytics;
pub mod auth;
pub mod catalog;
pub mod decks;
pub mod live;
pub mod matches;
pub mod release;
#[cfg(feature = "seed")]
pub mod seed;
pub mod stats;

use std::{sync::Arc, time::Duration};

use axum::{
    Json, Router,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

pub struct Config {
    pub public_url: String,
    pub steam_api_key: Option<String>,
    pub steam_openid_url: String,
    pub github_release_url: String,
    pub desktop_min_version: String,
    pub max_pending_device_codes: i64,
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
    pub catalog: Arc<catalog::Catalog>,
    pub http: reqwest::Client,
    pub upload_limiter: Arc<matches::RateLimiter>,
    pub comment_limiter: Arc<matches::RateLimiter>,
    pub release: Arc<release::Cache>,
    pub presence: Arc<analytics::Presence>,
    pub live: Arc<live::Live>,
}

impl AppState {
    pub fn new(db: PgPool, config: Config, catalog: catalog::Catalog) -> Self {
        Self {
            db,
            config: Arc::new(config),
            catalog: Arc::new(catalog),
            http: reqwest::Client::new(),
            upload_limiter: Arc::new(matches::RateLimiter::new(matches::UPLOADS_PER_HOUR, Duration::from_secs(3600))),
            comment_limiter: Arc::new(matches::RateLimiter::new(
                decks::COMMENTS_PER_WINDOW,
                Duration::from_secs(decks::COMMENT_WINDOW_SECS),
            )),
            release: Arc::default(),
            presence: Arc::default(),
            live: Arc::default(),
        }
    }
}

#[derive(Debug)]
pub struct ApiError(pub StatusCode, pub String);

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self(status, message.into())
    }
}

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(ErrorBody { error: self.1 })).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(error = %e, "database error");
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal error")
    }
}

pub fn public<T: Serialize>(body: T) -> Response {
    ([(header::CACHE_CONTROL, "public, max-age=300")], Json(body)).into_response()
}

#[derive(OpenApi)]
#[openapi(
    info(title = "SPAM fan site API"),
    paths(
        auth::steam_login,
        auth::steam_callback,
        auth::logout,
        auth::me,
        auth::delete_me,
        auth::device_code,
        auth::device_confirm,
        auth::device_token,
        auth::list_devices,
        auth::delete_device,
        catalog::versions,
        catalog::list_cards,
        catalog::get_card,
        catalog::artists,
        catalog::glossary,
        catalog::glossary_keywords,
        matches::upload,
        matches::my_matches,
        matches::min_version,
        release::release,
        decks::import,
        decks::list_decks,
        decks::create_deck,
        decks::preview,
        decks::get_deck,
        decks::patch_deck,
        decks::delete_deck,
        decks::publish,
        decks::unpublish,
        decks::put_vote,
        decks::delete_vote,
        decks::list_comments,
        decks::create_comment,
        decks::patch_comment,
        decks::delete_comment,
        decks::create_report,
        live::deck_events,
        stats::deck_stats,
        stats::card_stats,
        stats::matchups,
        stats::pace,
        stats::turns,
        stats::history,
        admin::reports,
        admin::resolve,
        admin::ban,
        admin::unban,
        admin::unlink,
        admin::disputed,
        admin::unknown_cards,
        analytics::presence,
        analytics::desktop_presence,
        analytics::admin_analytics,
    ),
    components(schemas(decks::DeckSort, stats::StatsSort, stats::Trend, analytics::AnalyticsRange))
)]
pub struct ApiDoc;

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/openapi.json", get(openapi_json))
        .merge(auth::routes())
        .merge(catalog::routes())
        .merge(matches::routes())
        .merge(release::routes())
        .merge(decks::routes())
        .merge(live::routes())
        .merge(stats::routes())
        .merge(admin::routes())
        .merge(analytics::routes())
        .with_state(state)
}

pub async fn cleanup(db: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM replays WHERE expires_at < now()").execute(db).await?;
    sqlx::query("DELETE FROM sessions WHERE expires_at < now()").execute(db).await?;
    sqlx::query("DELETE FROM device_codes WHERE expires_at < now()").execute(db).await?;
    Ok(())
}
