use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{ApiError, AppState, ErrorBody, auth::CurrentUser, decks::TargetType};

fn not_found(message: &str) -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, message)
}

#[derive(Serialize, Deserialize, ToSchema, sqlx::Type, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum ReportStatus {
    #[default]
    Open,
    Dismissed,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ReportQuery {
    pub status: Option<ReportStatus>,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub id: i64,
    pub reporter_id: i64,
    pub reporter_name: String,
    pub target_type: TargetType,
    pub target_id: i64,
    pub deck_id: i64,
    pub target_author_id: i64,
    pub target_text: String,
    pub reason: String,
    pub status: ReportStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[utoipa::path(get, path = "/api/admin/reports", tag = "admin", params(ReportQuery),
    responses((status = 200, body = Vec<Report>, description = "Oldest first. targetText is the deck name or the comment body."),
        (status = 403, body = ErrorBody)))]
pub async fn reports(
    State(s): State<AppState>,
    user: CurrentUser,
    Query(q): Query<ReportQuery>,
) -> Result<Json<Vec<Report>>, ApiError> {
    user.require_admin()?;
    let rows = sqlx::query_as(
        "SELECT r.id, r.reporter_id, ru.display_name AS reporter_name,
             CASE WHEN r.deck_id IS NOT NULL THEN 'deck' ELSE 'comment' END AS target_type,
             COALESCE(r.deck_id, r.comment_id) AS target_id, COALESCE(r.deck_id, c.deck_id) AS deck_id,
             COALESCE(d.author_id, c.author_id) AS target_author_id, COALESCE(d.name, c.body) AS target_text,
             r.reason, r.status, r.created_at, r.resolved_at
         FROM reports r JOIN users ru ON ru.id = r.reporter_id
         LEFT JOIN decks d ON d.id = r.deck_id
         LEFT JOIN comments c ON c.id = r.comment_id
         WHERE r.status = $1 ORDER BY r.id",
    )
    .bind(q.status.unwrap_or_default())
    .fetch_all(&s.db)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ResolveAction {
    Dismiss,
    Delete,
}

#[derive(Deserialize, ToSchema)]
pub struct Resolve {
    pub action: ResolveAction,
}

#[utoipa::path(post, path = "/api/admin/reports/{id}/resolve", tag = "admin", request_body = Resolve,
    params(("id" = i64, Path, description = "Report ID")),
    responses((status = 204, description = "dismiss closes the report. delete removes the target and all its reports."),
        (status = 403, body = ErrorBody), (status = 404, body = ErrorBody)))]
pub async fn resolve(
    State(s): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(body): Json<Resolve>,
) -> Result<StatusCode, ApiError> {
    user.require_admin()?;
    let target: (Option<i64>, Option<i64>) =
        sqlx::query_as("SELECT deck_id, comment_id FROM reports WHERE id = $1 AND status = 'open'")
            .bind(id)
            .fetch_optional(&s.db)
            .await?
            .ok_or_else(|| not_found("open report not found"))?;
    match (body.action, target) {
        (ResolveAction::Dismiss, _) => {
            sqlx::query("UPDATE reports SET status = 'dismissed', resolved_at = now() WHERE id = $1")
                .bind(id)
                .execute(&s.db)
                .await?;
        }
        (ResolveAction::Delete, (Some(deck), _)) => {
            sqlx::query("DELETE FROM decks WHERE id = $1").bind(deck).execute(&s.db).await?;
            s.live.notify(deck);
        }
        (ResolveAction::Delete, (_, comment)) => {
            let deck: Option<i64> = sqlx::query_scalar("DELETE FROM comments WHERE id = $1 RETURNING deck_id")
                .bind(comment)
                .fetch_optional(&s.db)
                .await?;
            if let Some(deck) = deck {
                s.live.comments(&s.db, deck).await;
            }
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/api/admin/users/{id}/ban", tag = "admin", params(("id" = i64, Path, description = "User ID")),
    responses((status = 204, description = "User banned, sessions and devices deleted. The SteamID stays banned after account deletion."),
        (status = 403, body = ErrorBody),
        (status = 404, body = ErrorBody)))]
pub async fn ban(State(s): State<AppState>, user: CurrentUser, Path(id): Path<i64>) -> Result<StatusCode, ApiError> {
    user.require_admin()?;
    let mut tx = s.db.begin().await?;
    let found: i64 = sqlx::query_scalar(
        "WITH u AS (UPDATE users SET banned_at = COALESCE(banned_at, now()) WHERE id = $1 RETURNING steam_id, banned_at),
             b AS (INSERT INTO banned_steam_ids (steam_id, banned_at) SELECT steam_id, banned_at FROM u
                   ON CONFLICT (steam_id) DO NOTHING)
         SELECT count(*) FROM u",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    if found == 0 {
        return Err(not_found("user not found"));
    }
    for sql in [
        "DELETE FROM sessions WHERE user_id = $1",
        "DELETE FROM devices WHERE user_id = $1",
        "DELETE FROM device_codes WHERE user_id = $1",
    ] {
        sqlx::query(sql).bind(id).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/api/admin/users/{id}/unban", tag = "admin", params(("id" = i64, Path, description = "User ID")),
    responses((status = 204, description = "User unbanned"), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody)))]
pub async fn unban(State(s): State<AppState>, user: CurrentUser, Path(id): Path<i64>) -> Result<StatusCode, ApiError> {
    user.require_admin()?;
    let found: i64 = sqlx::query_scalar(
        "WITH u AS (UPDATE users SET banned_at = NULL WHERE id = $1 RETURNING steam_id),
             b AS (DELETE FROM banned_steam_ids WHERE steam_id IN (SELECT steam_id FROM u))
         SELECT count(*) FROM u",
    )
    .bind(id)
    .fetch_one(&s.db)
    .await?;
    if found == 0 {
        return Err(not_found("user not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/api/admin/users/{id}/unlink", tag = "admin", params(("id" = i64, Path, description = "User ID")),
    responses((status = 204, description = "SPAM AccountId link removed"), (status = 403, body = ErrorBody),
        (status = 404, body = ErrorBody)))]
pub async fn unlink(State(s): State<AppState>, user: CurrentUser, Path(id): Path<i64>) -> Result<StatusCode, ApiError> {
    user.require_admin()?;
    let updated = sqlx::query("UPDATE users SET spam_account_id = NULL WHERE id = $1")
        .bind(id)
        .execute(&s.db)
        .await?
        .rows_affected();
    if updated == 0 {
        return Err(not_found("user not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DisputedMatch {
    pub game_uid: String,
    pub app_version: String,
    pub game_mode: i32,
    pub is_ai: bool,
    pub start_utc: DateTime<Utc>,
    pub seat0_cards: Vec<Uuid>,
    pub seat1_cards: Vec<Uuid>,
    pub winner_seat: Option<i16>,
    pub uploader_ids: Vec<i64>,
}

#[utoipa::path(get, path = "/api/admin/matches/disputed", tag = "admin",
    responses((status = 200, body = Vec<DisputedMatch>, description = "Newest first, cards and winner of the first upload"),
        (status = 403, body = ErrorBody)))]
pub async fn disputed(State(s): State<AppState>, user: CurrentUser) -> Result<Json<Vec<DisputedMatch>>, ApiError> {
    user.require_admin()?;
    let rows = sqlx::query_as(
        "SELECT m.game_uid, m.app_version, m.game_mode, m.is_ai, m.start_utc, m.seat0_cards, m.seat1_cards,
             m.winner_seat,
             ARRAY(SELECT u.user_id FROM uploads u WHERE u.game_uid = m.game_uid ORDER BY u.uploaded_at) AS uploader_ids
         FROM matches m WHERE m.disputed ORDER BY m.start_utc DESC LIMIT 500",
    )
    .fetch_all(&s.db)
    .await?;
    Ok(Json(rows))
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UnknownCard {
    pub card_id: Uuid,
    pub seen_count: i64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[utoipa::path(get, path = "/api/admin/unknown-cards", tag = "admin",
    responses((status = 200, body = Vec<UnknownCard>, description = "Cards that the latest catalog does not have"),
        (status = 403, body = ErrorBody)))]
pub async fn unknown_cards(State(s): State<AppState>, user: CurrentUser) -> Result<Json<Vec<UnknownCard>>, ApiError> {
    user.require_admin()?;
    let rows: Vec<UnknownCard> =
        sqlx::query_as("SELECT * FROM unknown_cards ORDER BY last_seen DESC").fetch_all(&s.db).await?;
    let latest = s.catalog.latest();
    Ok(Json(rows.into_iter().filter(|c| !s.catalog.knows_card(latest, &c.card_id)).collect()))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/admin/reports", get(reports))
        .route("/api/admin/reports/{id}/resolve", post(resolve))
        .route("/api/admin/users/{id}/ban", post(ban))
        .route("/api/admin/users/{id}/unban", post(unban))
        .route("/api/admin/users/{id}/unlink", post(unlink))
        .route("/api/admin/matches/disputed", get(disputed))
        .route("/api/admin/unknown-cards", get(unknown_cards))
}
