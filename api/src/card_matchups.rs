use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use sqlx::{AssertSqlSafe, FromRow};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{ApiError, AppState, ErrorBody, auth::CurrentUser, catalog::parse_version, public, stats::VersionQuery};

#[derive(Serialize, ToSchema)]
pub struct CardMatchups {
    pub strong: Vec<CardMatchup>,
    pub weak: Vec<CardMatchup>,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CardMatchup {
    pub card: Uuid,
    pub games: i64,
    pub wins: i64,
    pub win_rate: Option<f64>,
}

#[derive(FromRow)]
struct Row {
    strong: bool,
    #[sqlx(flatten)]
    matchup: CardMatchup,
}

fn version(s: &AppState, v: Option<&str>) -> Result<Option<String>, ApiError> {
    match v {
        None => Ok(Some(s.catalog.latest().to_string())),
        Some("all") => Ok(None),
        Some(v) if parse_version(v).is_some() => Ok(Some(v.to_string())),
        Some(_) => Err(ApiError::new(StatusCode::BAD_REQUEST, "version must be an AppVersion or all")),
    }
}

#[utoipa::path(get, path = "/api/stats/cards/{id}/matchups", tag = "stats",
    params(("id" = Uuid, Path, description = "Card UUID"), VersionQuery),
    responses((status = 200, body = CardMatchups,
        description = "For each card of the opposing decks (not this card) with 1 game or more: games, and wins of \
            the deck with this card. strong: winRate above 0.5, sorted by the Wilson lower bound of the wins; weak: \
            winRate below 0.5, sorted by the Wilson lower bound of the losses; then games descending and card. At \
            most 5 each."),
        (status = 400, body = ErrorBody), (status = 404, body = ErrorBody)))]
pub async fn card_matchups(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<VersionQuery>,
) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    if s.catalog.element(&id).is_none() {
        return Err(ApiError::new(StatusCode::NOT_FOUND, "unknown card"));
    }
    Ok(public(matchups(&s, version, "r.deck_key @> $2", &[id]).await?))
}

#[utoipa::path(get, path = "/api/decks/{id}/matchups", tag = "decks",
    params(("id" = i64, Path, description = "Deck ID"), VersionQuery),
    responses((status = 200, body = CardMatchups,
        description = "For each card of the opposing decks (not a card of this deck) with 1 game or more: games, and \
            wins of the deck key of this deck. The same order and limits as the card matchups. A draft is visible only \
            to its author and is not cached."),
        (status = 400, body = ErrorBody), (status = 404, body = ErrorBody)))]
pub async fn deck_matchups(
    State(s): State<AppState>,
    user: Option<CurrentUser>,
    Path(id): Path<i64>,
    Query(q): Query<VersionQuery>,
) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    let (key, open): (Vec<Uuid>, bool) = sqlx::query_as(
        "SELECT deck_key, status = 'public' FROM decks WHERE id = $1 AND (status = 'public' OR author_id = $2)",
    )
    .bind(id)
    .bind(user.map(|u| u.id))
    .fetch_optional(&s.db)
    .await?
    .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "deck not found"))?;
    let body = matchups(&s, version, "r.deck_key = $2", &key).await?;
    Ok(if open { public(body) } else { Json(body).into_response() })
}

async fn matchups(s: &AppState, version: Option<String>, mine: &str, cards: &[Uuid]) -> Result<CardMatchups, ApiError> {
    let rows: Vec<Row> = sqlx::query_as(AssertSqlSafe(format!(
        "WITH g AS (
             SELECT c.card, count(*) AS games, count(*) FILTER (WHERE r.win) AS wins
             FROM deck_results r
             JOIN deck_results o ON o.game_uid = r.game_uid AND o.deck_key <> r.deck_key
             CROSS JOIN LATERAL unnest(o.deck_key) AS c (card)
             WHERE ($1::text IS NULL OR r.app_version = $1) AND {mine} AND c.card <> ALL($2)
             GROUP BY c.card
         ),
         t AS (
             SELECT card, games, wins, win_rate(games, wins) AS win_rate, win_rate(games, wins) > 0.5 AS strong
             FROM g WHERE win_rate(games, wins) <> 0.5
         )
         SELECT strong, card, games, wins, win_rate FROM (
             SELECT t.*, row_number() OVER (PARTITION BY strong
                 ORDER BY win_score(games, CASE WHEN strong THEN wins ELSE games - wins END) DESC, games DESC, card)
                 AS rank
             FROM t
         ) ranked WHERE rank <= 5 ORDER BY strong DESC, rank"
    )))
    .bind(&version)
    .bind(cards)
    .fetch_all(&s.db)
    .await?;
    let (strong, weak): (Vec<Row>, Vec<Row>) = rows.into_iter().partition(|r| r.strong);
    let list = |rows: Vec<Row>| rows.into_iter().map(|r| r.matchup).collect();
    Ok(CardMatchups { strong: list(strong), weak: list(weak) })
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/stats/cards/{id}/matchups", get(card_matchups))
        .route("/api/decks/{id}/matchups", get(deck_matchups))
}
