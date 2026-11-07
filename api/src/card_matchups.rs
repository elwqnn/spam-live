use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Response,
    routing::get,
};
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{ApiError, AppState, ErrorBody, catalog::parse_version, public, stats::VersionQuery};

const MIN_GAMES: i64 = 10;

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
        description = "For each card of the opposing decks (not this card) with 10 games or more: games, and wins of \
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
    let rows: Vec<Row> = sqlx::query_as(
        "WITH g AS (
             SELECT c.card, count(*) AS games, count(*) FILTER (WHERE r.win) AS wins
             FROM deck_results r
             JOIN deck_results o ON o.game_uid = r.game_uid AND o.deck_key <> r.deck_key
             CROSS JOIN LATERAL unnest(o.deck_key) AS c (card)
             WHERE ($1::text IS NULL OR r.app_version = $1) AND $2 = ANY(r.deck_key) AND c.card <> $2
             GROUP BY c.card HAVING count(*) >= $3
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
         ) ranked WHERE rank <= 5 ORDER BY strong DESC, rank",
    )
    .bind(&version)
    .bind(id)
    .bind(MIN_GAMES)
    .fetch_all(&s.db)
    .await?;
    let (strong, weak): (Vec<Row>, Vec<Row>) = rows.into_iter().partition(|r| r.strong);
    let list = |rows: Vec<Row>| rows.into_iter().map(|r| r.matchup).collect();
    Ok(public(CardMatchups { strong: list(strong), weak: list(weak) }))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/stats/cards/{id}/matchups", get(card_matchups))
}
