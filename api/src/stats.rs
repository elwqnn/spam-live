use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::Response,
    routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{ApiError, AppState, ErrorBody, catalog::parse_version, public};
use sqlx::AssertSqlSafe;

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct KeyStats {
    pub games: i64,
    pub wins: i64,
    pub win_rate: Option<f64>,
}

pub async fn key_stats(db: &PgPool, cards: &[Uuid], version: Option<&str>) -> Result<KeyStats, sqlx::Error> {
    sqlx::query_as(
        "SELECT count(*) AS games, count(*) FILTER (WHERE win) AS wins,
             win_rate(count(*), count(*) FILTER (WHERE win)) AS win_rate
         FROM deck_results WHERE deck_key = array_sort($1::uuid[]) AND ($2::text IS NULL OR app_version = $2)",
    )
    .bind(cards)
    .bind(version)
    .fetch_one(db)
    .await
}

fn version(s: &AppState, v: Option<&str>) -> Result<Option<String>, ApiError> {
    match v {
        None => Ok(Some(s.catalog.latest().to_string())),
        Some("all") => Ok(None),
        Some(v) if parse_version(v).is_some() => Ok(Some(v.to_string())),
        Some(_) => Err(ApiError::new(StatusCode::BAD_REQUEST, "version must be an AppVersion or all")),
    }
}

#[derive(Deserialize, ToSchema, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum StatsSort {
    #[default]
    Winrate,
    Games,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DeckStatsQuery {
    #[param(example = "1.28.1")]
    pub version: Option<String>,
    pub sort: Option<StatsSort>,
    pub limit: Option<i64>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CardStatsQuery {
    #[param(example = "all")]
    pub version: Option<String>,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DeckKeyStats {
    pub cards: Vec<Uuid>,
    pub games: i64,
    pub wins: i64,
    pub win_rate: Option<f64>,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CardStats {
    pub card: Uuid,
    pub games: i64,
    pub wins: i64,
    pub win_rate: Option<f64>,
    pub play_rate: f64,
}

#[utoipa::path(get, path = "/api/stats/decks", tag = "stats", params(DeckStatsQuery),
    responses((status = 200, body = Vec<DeckKeyStats>,
        description = "Ranked human matches, not disputed, no mirror. winRate is null below 20 games."),
        (status = 400, body = ErrorBody)))]
pub async fn deck_stats(State(s): State<AppState>, Query(q): Query<DeckStatsQuery>) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    let order = match q.sort.unwrap_or_default() {
        StatsSort::Winrate => "win_rate DESC NULLS LAST, games DESC, cards",
        StatsSort::Games => "games DESC, wins DESC, cards",
    };
    let rows: Vec<DeckKeyStats> = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT deck_key AS cards, count(*) AS games, count(*) FILTER (WHERE win) AS wins,
             win_rate(count(*), count(*) FILTER (WHERE win)) AS win_rate
         FROM deck_results WHERE ($1::text IS NULL OR app_version = $1)
         GROUP BY deck_key ORDER BY {order} LIMIT $2"
    )))
    .bind(version)
    .bind(q.limit.unwrap_or(100).clamp(1, 1000))
    .fetch_all(&s.db)
    .await?;
    Ok(public(rows))
}

#[utoipa::path(get, path = "/api/stats/cards", tag = "stats", params(CardStatsQuery),
    responses((status = 200, body = Vec<CardStats>,
        description = "playRate = deck results with the card / all deck results. winRate is null below 20 games."),
        (status = 400, body = ErrorBody)))]
pub async fn card_stats(State(s): State<AppState>, Query(q): Query<CardStatsQuery>) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    let rows: Vec<CardStats> = sqlx::query_as(
        "WITH r AS (SELECT deck_key, win FROM deck_results WHERE ($1::text IS NULL OR app_version = $1))
         SELECT c.card, count(*) AS games, count(*) FILTER (WHERE r.win) AS wins,
             win_rate(count(*), count(*) FILTER (WHERE r.win)) AS win_rate,
             count(*)::float8 / (SELECT count(*) FROM r) AS play_rate
         FROM r CROSS JOIN LATERAL unnest(r.deck_key) AS c (card)
         GROUP BY c.card ORDER BY games DESC, c.card",
    )
    .bind(version)
    .fetch_all(&s.db)
    .await?;
    Ok(public(rows))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/stats/decks", get(deck_stats)).route("/api/stats/cards", get(card_stats))
}
