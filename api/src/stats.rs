use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::Response,
    routing::get,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::collections::BTreeMap;
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
    pub win_score: Option<f64>,
}

pub async fn key_stats(db: &PgPool, cards: &[Uuid], version: Option<&str>) -> Result<KeyStats, sqlx::Error> {
    sqlx::query_as(
        "SELECT count(*) AS games, count(*) FILTER (WHERE win) AS wins,
             win_rate(count(*), count(*) FILTER (WHERE win)) AS win_rate,
             win_score(count(*), count(*) FILTER (WHERE win)) AS win_score
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
    pub win_score: Option<f64>,
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
        description = "Ranked human matches, not disputed, no mirror. winRate is null at 0 games. \
            winScore is the lower bound of the 95 % Wilson score interval of winRate, null at 0 games. \
            sort=winrate orders by winScore."),
        (status = 400, body = ErrorBody)))]
pub async fn deck_stats(State(s): State<AppState>, Query(q): Query<DeckStatsQuery>) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    let order = match q.sort.unwrap_or_default() {
        StatsSort::Winrate => "win_score DESC NULLS LAST, win_rate DESC NULLS LAST, games DESC, cards",
        StatsSort::Games => "games DESC, wins DESC, cards",
    };
    let rows: Vec<DeckKeyStats> = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT deck_key AS cards, count(*) AS games, count(*) FILTER (WHERE win) AS wins,
             win_rate(count(*), count(*) FILTER (WHERE win)) AS win_rate,
             win_score(count(*), count(*) FILTER (WHERE win)) AS win_score
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

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct VersionQuery {
    #[param(example = "all")]
    pub version: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct MatchupStats {
    pub cards: Vec<Uuid>,
    pub cells: Vec<Matchup>,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Matchup {
    pub card: Uuid,
    pub opponent: Uuid,
    pub games: i64,
    pub wins: i64,
    pub win_rate: Option<f64>,
}

async fn top_cards(db: &PgPool, version: Option<&str>) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT c.card FROM deck_results r CROSS JOIN LATERAL unnest(r.deck_key) AS c (card)
         WHERE $1::text IS NULL OR r.app_version = $1
         GROUP BY c.card ORDER BY count(*) DESC, c.card LIMIT 12",
    )
    .bind(version)
    .fetch_all(db)
    .await
}

#[utoipa::path(get, path = "/api/stats/matchups", tag = "stats", params(VersionQuery),
    responses((status = 200, body = MatchupStats,
        description = "cards: the 12 most played cards (deck results with the card, then card id). cells: 1 item for \
            each ordered pair of 2 different cards of cards with 1 game or more. games: the deck results where the deck \
            has card and the opposing deck has opponent. wins: the part of games where the deck with card won."),
        (status = 400, body = ErrorBody)))]
pub async fn matchups(State(s): State<AppState>, Query(q): Query<VersionQuery>) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    let cards = top_cards(&s.db, version.as_deref()).await?;
    let cells: Vec<Matchup> = sqlx::query_as(
        "SELECT a.card, b.card AS opponent, count(*) AS games, count(*) FILTER (WHERE r.win) AS wins,
             win_rate(count(*), count(*) FILTER (WHERE r.win)) AS win_rate
         FROM deck_results r
         JOIN deck_results o ON o.game_uid = r.game_uid AND o.deck_key <> r.deck_key
         CROSS JOIN LATERAL unnest(r.deck_key) AS a (card)
         CROSS JOIN LATERAL unnest(o.deck_key) AS b (card)
         WHERE ($1::text IS NULL OR r.app_version = $1) AND a.card = ANY($2) AND b.card = ANY($2) AND a.card <> b.card
         GROUP BY a.card, b.card ORDER BY a.card, b.card",
    )
    .bind(&version)
    .bind(&cards)
    .fetch_all(&s.db)
    .await?;
    Ok(public(MatchupStats { cards, cells }))
}

#[derive(Serialize, ToSchema)]
pub struct PaceStats {
    pub turns: Vec<TurnBucket>,
    pub median: Option<i32>,
    pub cards: Vec<CardPace>,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct TurnBucket {
    pub turns: i32,
    pub matches: i64,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CardPace {
    pub card: Uuid,
    pub short_games: i64,
    pub short_wins: i64,
    pub short_win_rate: Option<f64>,
    pub long_games: i64,
    pub long_wins: i64,
    pub long_win_rate: Option<f64>,
}

const PACE_MATCHES: &str = "SELECT DISTINCT r.game_uid, m.turn_count FROM deck_results r
    JOIN matches m ON m.game_uid = r.game_uid WHERE $1::text IS NULL OR r.app_version = $1";

#[utoipa::path(get, path = "/api/stats/pace", tag = "stats", params(VersionQuery),
    responses((status = 200, body = PaceStats,
        description = "turns: the match count for each turn count. median: the median turn count \
            (percentile_disc 0.5), null when there is no match. cards: short is turn count <= median, long is turn \
            count > median. The win rates are null at 0 games."),
        (status = 400, body = ErrorBody)))]
pub async fn pace(State(s): State<AppState>, Query(q): Query<VersionQuery>) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    let turns: Vec<TurnBucket> = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT turn_count AS turns, count(*) AS matches FROM ({PACE_MATCHES}) g GROUP BY turn_count ORDER BY turn_count"
    )))
    .bind(&version)
    .fetch_all(&s.db)
    .await?;
    let median: Option<i32> = sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT percentile_disc(0.5) WITHIN GROUP (ORDER BY turn_count) FROM ({PACE_MATCHES}) g"
    )))
    .bind(&version)
    .fetch_one(&s.db)
    .await?;
    let cards: Vec<CardPace> = sqlx::query_as(
        "SELECT card, short_games, short_wins, win_rate(short_games, short_wins) AS short_win_rate,
             long_games, long_wins, win_rate(long_games, long_wins) AS long_win_rate
         FROM (
             SELECT c.card, count(*) FILTER (WHERE m.turn_count <= $2) AS short_games,
                 count(*) FILTER (WHERE m.turn_count <= $2 AND r.win) AS short_wins,
                 count(*) FILTER (WHERE m.turn_count > $2) AS long_games,
                 count(*) FILTER (WHERE m.turn_count > $2 AND r.win) AS long_wins
             FROM deck_results r JOIN matches m ON m.game_uid = r.game_uid
             CROSS JOIN LATERAL unnest(r.deck_key) AS c (card)
             WHERE $1::text IS NULL OR r.app_version = $1
             GROUP BY c.card
         ) t
         ORDER BY short_games + long_games DESC, card",
    )
    .bind(&version)
    .bind(median)
    .fetch_all(&s.db)
    .await?;
    Ok(public(PaceStats { turns, median, cards }))
}

#[derive(Serialize, ToSchema)]
pub struct TurnStats {
    pub cards: Vec<CardTurns>,
}

#[derive(Serialize, ToSchema)]
pub struct CardTurns {
    pub card: Uuid,
    pub turns: Vec<TurnResult>,
}

#[derive(Serialize, ToSchema)]
pub struct TurnResult {
    pub turns: i32,
    pub games: i64,
    pub wins: i64,
}

#[derive(FromRow)]
struct CardTurn {
    card: Uuid,
    turns: i32,
    games: i64,
    wins: i64,
}

#[utoipa::path(get, path = "/api/stats/turns", tag = "stats", params(VersionQuery),
    responses((status = 200, body = TurnStats,
        description = "cards: the 12 most played cards, in the matchups order. turns: 1 item for each match turn count \
            with 1 game or more for the card, sorted by turns. games: the deck results with the card in matches with \
            this turn count. wins: the part of games that the deck won."),
        (status = 400, body = ErrorBody)))]
pub async fn turns(State(s): State<AppState>, Query(q): Query<VersionQuery>) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    let cards = top_cards(&s.db, version.as_deref()).await?;
    let rows: Vec<CardTurn> = sqlx::query_as(
        "SELECT c.card, m.turn_count AS turns, count(*) AS games, count(*) FILTER (WHERE r.win) AS wins
         FROM deck_results r JOIN matches m ON m.game_uid = r.game_uid
         CROSS JOIN LATERAL unnest(r.deck_key) AS c (card)
         WHERE ($1::text IS NULL OR r.app_version = $1) AND c.card = ANY($2)
         GROUP BY c.card, m.turn_count ORDER BY m.turn_count",
    )
    .bind(&version)
    .bind(&cards)
    .fetch_all(&s.db)
    .await?;
    let cards = cards
        .into_iter()
        .map(|card| CardTurns {
            card,
            turns: rows
                .iter()
                .filter(|r| r.card == card)
                .map(|r| TurnResult { turns: r.turns, games: r.games, wins: r.wins })
                .collect(),
        })
        .collect();
    Ok(public(TurnStats { cards }))
}

#[derive(Serialize, ToSchema)]
pub struct HistoryStats {
    pub weeks: Vec<ElementWeek>,
    pub versions: Vec<VersionStart>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ElementWeek {
    pub week: NaiveDate,
    pub element: String,
    pub games: i64,
    pub wins: i64,
    pub win_rate: f64,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct VersionStart {
    pub version: String,
    pub start: DateTime<Utc>,
}

#[derive(FromRow)]
struct WeekCard {
    week: NaiveDate,
    card: Uuid,
    games: i64,
    wins: i64,
}

#[utoipa::path(get, path = "/api/stats/history", tag = "stats",
    responses((status = 200, body = HistoryStats,
        description = "All versions. weeks: 1 item for each UTC week (the Monday of the match end) and element with 1 \
            game or more. games: card slots, 1 for each card of the element in a deck result. wins: the slots in a won \
            deck result. The element comes from the newest catalog version that has the card. Cards that no catalog \
            version has are not counted. versions: the first match end of each version.")))]
pub async fn history(State(s): State<AppState>) -> Result<Response, ApiError> {
    let rows: Vec<WeekCard> = sqlx::query_as(
        "SELECT date_trunc('week', r.end_utc AT TIME ZONE 'UTC')::date AS week, c.card,
             count(*) AS games, count(*) FILTER (WHERE r.win) AS wins
         FROM deck_results r CROSS JOIN LATERAL unnest(r.deck_key) AS c (card)
         GROUP BY 1, 2",
    )
    .fetch_all(&s.db)
    .await?;
    let mut sums: BTreeMap<(NaiveDate, &str), (i64, i64)> = BTreeMap::new();
    for row in &rows {
        if let Some(element) = s.catalog.element(&row.card) {
            let sum = sums.entry((row.week, element)).or_default();
            sum.0 += row.games;
            sum.1 += row.wins;
        }
    }
    let weeks = sums
        .into_iter()
        .map(|((week, element), (games, wins))| ElementWeek {
            week,
            element: element.to_string(),
            games,
            wins,
            win_rate: wins as f64 / games as f64,
        })
        .collect();
    let versions: Vec<VersionStart> = sqlx::query_as(
        "SELECT app_version AS version, min(end_utc) AS start FROM deck_results
         GROUP BY app_version ORDER BY start, app_version",
    )
    .fetch_all(&s.db)
    .await?;
    Ok(public(HistoryStats { weeks, versions }))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/stats/decks", get(deck_stats))
        .route("/api/stats/cards", get(card_stats))
        .route("/api/stats/matchups", get(matchups))
        .route("/api/stats/pace", get(pace))
        .route("/api/stats/turns", get(turns))
        .route("/api/stats/history", get(history))
}
