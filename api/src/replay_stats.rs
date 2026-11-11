use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Response,
    routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::{AssertSqlSafe, FromRow};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    ApiError, AppState, ErrorBody, public,
    stats::{min_stars, version},
};

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ReplayStatsQuery {
    #[param(example = "all")]
    pub version: Option<String>,
    #[serde(rename = "minStars")]
    #[param(minimum = 0, example = 30)]
    pub min_stars: Option<i32>,
    #[param(example = "en")]
    pub lang: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct CardReplayStats {
    pub games: i64,
    pub tox: Vec<ToxStats>,
    pub talents: Vec<TalentStats>,
    pub damage: DamageStats,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ToxStats {
    pub tox: Uuid,
    #[sqlx(skip)]
    pub name: Option<String>,
    #[sqlx(skip)]
    pub rarity: Option<String>,
    #[sqlx(skip)]
    pub image: bool,
    pub games: i64,
    pub wins: i64,
    pub buy_rate: f64,
    pub win_rate: Option<f64>,
    pub win_score: Option<f64>,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TalentStats {
    pub talent: String,
    #[sqlx(skip)]
    pub name: Option<String>,
    pub games: i64,
    pub wins: i64,
    pub pick_rate: f64,
    pub win_rate: Option<f64>,
    pub win_score: Option<f64>,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct DamageStats {
    pub games: i64,
    pub done: Option<f64>,
    pub taken: Option<f64>,
}

const BASE: &str = "WITH b AS (
    SELECT r.game_uid, r.seat, r.win, r.app_version, s.own
    FROM deck_results r JOIN match_event_sources s ON s.game_uid = r.game_uid AND s.seat = r.seat
    WHERE ($1::text IS NULL OR r.app_version = $1) AND ($2::int IS NULL OR r.stars >= $2) AND $3 = ANY(r.deck_key)
)";

const RATES: &str = "count(*) AS games, count(*) FILTER (WHERE t.win) AS wins,
    win_rate(count(*), count(*) FILTER (WHERE t.win)) AS win_rate,
    win_score(count(*), count(*) FILTER (WHERE t.win)) AS win_score";

#[utoipa::path(get, path = "/api/stats/cards/{id}/replay", tag = "stats",
    params(("id" = Uuid, Path, description = "Card UUID"), ReplayStatsQuery),
    responses((status = 200, body = CardReplayStats,
        description = "From the decoded replays of ranked human matches, not disputed, no mirror. Base: the seats \
            whose deck has the card and that have replay events; games is their count. tox: each Tox bought on \
            this card, games = the base seats with 1 buy or more, buyRate = games / base. talents: base = the seats \
            with a replay of their own player; each talent id picked on this card, pickRate = games / base. \
            name is in lang (default en). Both lists are sorted by winScore, games, descending, then id. damage: \
            the mean damage done and taken by this card, null at 0 games."),
        (status = 400, body = ErrorBody), (status = 404, body = ErrorBody)))]
pub async fn card_replay(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<ReplayStatsQuery>,
) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    let stars = min_stars(q.min_stars)?;
    if s.catalog.element(&id).is_none() {
        return Err(ApiError::new(StatusCode::NOT_FOUND, "unknown card"));
    }
    let lang = q.lang.as_deref().unwrap_or("en");
    let (games, damage_games, done, taken): (i64, i64, Option<f64>, Option<f64>) =
        sqlx::query_as(AssertSqlSafe(format!(
            "{BASE} SELECT count(*), count(d.card), avg(d.done)::float8, avg(d.taken)::float8 FROM b
             LEFT JOIN card_damage d ON d.game_uid = b.game_uid AND d.seat = b.seat AND d.card = $3"
        )))
        .bind(&version)
        .bind(stars)
        .bind(id)
        .fetch_one(&s.db)
        .await?;
    let damage = DamageStats { games: damage_games, done, taken };
    let mut tox: Vec<ToxStats> = sqlx::query_as(AssertSqlSafe(format!(
        "{BASE}, n AS (SELECT count(*) AS n FROM b),
         t AS (SELECT DISTINCT b.game_uid, b.seat, b.win, x.tox FROM b
             JOIN shop_buys x ON x.game_uid = b.game_uid AND x.seat = b.seat AND x.card = $3)
         SELECT t.tox, {RATES}, count(*)::float8 / n.n AS buy_rate
         FROM t CROSS JOIN n GROUP BY t.tox, n.n ORDER BY win_score DESC, games DESC, t.tox"
    )))
    .bind(&version)
    .bind(stars)
    .bind(id)
    .fetch_all(&s.db)
    .await?;
    for t in &mut tox {
        if let Some(face) = s.catalog.tox(&t.tox.to_string(), lang) {
            (t.name, t.rarity, t.image) = (Some(face.name), face.rarity, face.image);
        }
    }
    let (versions, slots, talent_ids) = s.catalog.talent_slots(&id);
    let mut talents: Vec<TalentStats> = sqlx::query_as(AssertSqlSafe(format!(
        "{BASE}, n AS (SELECT count(*) AS n FROM b WHERE own),
         k AS (SELECT * FROM unnest($4::text[], $5::smallint[], $6::text[]) AS k (app_version, talent, id)),
         t AS (SELECT k.id, b.win FROM b
             JOIN talent_picks p ON p.game_uid = b.game_uid AND p.seat = b.seat AND p.card = $3
             JOIN k ON k.talent = p.talent
                 AND k.app_version = CASE WHEN b.app_version = ANY($4) THEN b.app_version ELSE $7 END
             WHERE b.own)
         SELECT t.id AS talent, {RATES}, count(*)::float8 / n.n AS pick_rate
         FROM t CROSS JOIN n GROUP BY t.id, n.n ORDER BY win_score DESC, games DESC, t.id"
    )))
    .bind(&version)
    .bind(stars)
    .bind(id)
    .bind(&versions)
    .bind(&slots)
    .bind(&talent_ids)
    .bind(s.catalog.latest())
    .fetch_all(&s.db)
    .await?;
    for t in &mut talents {
        t.name = s.catalog.talent_name(&t.talent, lang);
    }
    Ok(public(CardReplayStats { games, tox, talents, damage }))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/stats/cards/{id}/replay", get(card_replay))
}
