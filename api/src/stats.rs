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

#[derive(Deserialize, ToSchema, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum Trend {
    #[default]
    Week,
    Version,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CardStatsQuery {
    #[param(example = "all")]
    pub version: Option<String>,
    pub trend: Option<Trend>,
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
    pub win_rate_change: Option<f64>,
    pub play_rate_change: Option<f64>,
}

#[utoipa::path(get, path = "/api/stats/decks", tag = "stats", params(DeckStatsQuery),
    responses((status = 200, body = Vec<DeckKeyStats>,
        description = "Ranked human matches, not disputed, no mirror. winRate is null at 0 games."),
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
        description = "playRate = deck results with the card / all deck results. winRate is null at 0 games. \
            winRateChange and playRateChange = period A - period B, as fractions. trend week (default): A is the last \
            7 days and B is the 7 days before, on the match end time, in the selected version. trend version: A is the \
            selected version and B is the catalog version before it. The changes are null when A or B has 0 games, \
            and in version mode for all, for the oldest version and for a version that is not in the catalog."),
        (status = 400, body = ErrorBody)))]
pub async fn card_stats(State(s): State<AppState>, Query(q): Query<CardStatsQuery>) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    let week = matches!(q.trend.unwrap_or_default(), Trend::Week);
    let previous = if week { None } else { version.as_deref().and_then(|v| s.catalog.previous(v)) };
    let rows: Vec<CardStats> = sqlx::query_as(
        "WITH r AS (
             SELECT deck_key, win, $1::text IS NULL OR app_version = $1 AS cur,
                 CASE WHEN NOT $3 THEN CASE app_version WHEN $1 THEN 1 WHEN $2 THEN 2 END
                     WHEN end_utc >= now() - interval '7 days' AND end_utc < now() THEN 1
                     WHEN end_utc >= now() - interval '14 days' AND end_utc < now() - interval '7 days' THEN 2
                 END AS p
             FROM deck_results WHERE $1::text IS NULL OR app_version IN ($1, $2)
         ),
         t AS (SELECT count(*) FILTER (WHERE cur) AS n, count(*) FILTER (WHERE p = 1) AS na,
             count(*) FILTER (WHERE p = 2) AS nb FROM r),
         g AS (
             SELECT c.card, count(*) FILTER (WHERE r.cur) AS games, count(*) FILTER (WHERE r.cur AND r.win) AS wins,
                 count(*) FILTER (WHERE r.p = 1) AS ga, count(*) FILTER (WHERE r.p = 1 AND r.win) AS wa,
                 count(*) FILTER (WHERE r.p = 2) AS gb, count(*) FILTER (WHERE r.p = 2 AND r.win) AS wb
             FROM r CROSS JOIN LATERAL unnest(r.deck_key) AS c (card) GROUP BY c.card
         )
         SELECT g.card, games, wins, win_rate(games, wins) AS win_rate, games::float8 / t.n AS play_rate,
             win_rate(ga, wa) - win_rate(gb, wb) AS win_rate_change,
             CASE WHEN ga > 0 AND gb > 0 THEN ga::float8 / t.na - gb::float8 / t.nb END AS play_rate_change
         FROM g CROSS JOIN t WHERE games > 0 ORDER BY games DESC, g.card",
    )
    .bind(version)
    .bind(previous)
    .bind(week)
    .fetch_all(&s.db)
    .await?;
    Ok(public(rows))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/stats/decks", get(deck_stats)).route("/api/stats/cards", get(card_stats))
}
