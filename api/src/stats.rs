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
    #[sqlx(skip)]
    pub adjusted_win_rate: Option<f64>,
    #[sqlx(skip)]
    pub tier: Option<Tier>,
}

#[derive(Serialize, ToSchema, Clone, Copy, PartialEq, Debug)]
pub enum Tier {
    S,
    A,
    B,
    C,
    D,
}

const TIER_K: f64 = 30.0;
const TIER_MIN_GAMES: i64 = 20;
const TIER_MIN_CARDS: usize = 5;
const TIER_CUTS: (f64, f64) = (1.5, 0.5);

pub fn rank(cards: &mut [CardStats]) {
    for c in cards.iter_mut() {
        c.adjusted_win_rate = (c.games > 0).then(|| (c.wins as f64 + TIER_K * 0.5) / (c.games as f64 + TIER_K));
    }
    let ranked = |c: &CardStats| c.adjusted_win_rate.filter(|_| c.games >= TIER_MIN_GAMES);
    let adj: Vec<f64> = cards.iter().filter_map(ranked).collect();
    let n = adj.len() as f64;
    let mu = adj.iter().sum::<f64>() / n;
    let sigma = (adj.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / n).sqrt();
    let flat = adj.len() < TIER_MIN_CARDS || adj.iter().all(|&x| x == adj[0]);
    let (s, a) = TIER_CUTS;
    for c in cards.iter_mut() {
        c.tier = ranked(c).map(|x| match (x - mu) / sigma {
            _ if flat => Tier::B,
            z if z >= s => Tier::S,
            z if z >= a => Tier::A,
            z if z > -a => Tier::B,
            z if z > -s => Tier::C,
            _ => Tier::D,
        });
    }
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
            and in version mode for all, for the oldest version and for a version that is not in the catalog. \
            adjustedWinRate = (wins + 15) / (games + 30), null at 0 games. tier: S to D from the z-score of \
            adjustedWinRate over the cards with 20 games or more, null for the other cards."),
        (status = 400, body = ErrorBody)))]
pub async fn card_stats(State(s): State<AppState>, Query(q): Query<CardStatsQuery>) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    let week = matches!(q.trend.unwrap_or_default(), Trend::Week);
    let previous = if week { None } else { version.as_deref().and_then(|v| s.catalog.previous(v)) };
    let mut rows: Vec<CardStats> = sqlx::query_as(
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
    rank(&mut rows);
    Ok(public(rows))
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct VersionQuery {
    #[param(example = "all")]
    pub version: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct Overview {
    pub matches: i64,
    pub players: i64,
    pub decks: i64,
    pub days: Vec<DayMatches>,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct DayMatches {
    pub day: NaiveDate,
    pub matches: i64,
}

#[utoipa::path(get, path = "/api/stats/overview", tag = "stats", params(VersionQuery),
    responses((status = 200, body = Overview,
        description = "matches: distinct ranked matches. players: distinct users that uploaded these matches (a count \
            only). decks: distinct deck keys. days: the ranked matches per UTC day of the match end, for the 30 UTC days \
            that end today, oldest first, 0 for a day with no match."),
        (status = 400, body = ErrorBody)))]
pub async fn overview(State(s): State<AppState>, Query(q): Query<VersionQuery>) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    let (matches, players, decks): (i64, i64, i64) = sqlx::query_as(
        "WITH r AS (SELECT game_uid, deck_key FROM deck_results WHERE $1::text IS NULL OR app_version = $1)
         SELECT (SELECT count(DISTINCT game_uid) FROM r),
             (SELECT count(DISTINCT user_id) FROM uploads WHERE game_uid IN (SELECT game_uid FROM r)),
             (SELECT count(DISTINCT deck_key) FROM r)",
    )
    .bind(&version)
    .fetch_one(&s.db)
    .await?;
    let days: Vec<DayMatches> = sqlx::query_as(
        "WITH t AS (SELECT (now() AT TIME ZONE 'UTC')::date AS today),
         d AS (SELECT generate_series(today - 29, today, interval '1 day')::date AS day FROM t),
         m AS (
             SELECT (end_utc AT TIME ZONE 'UTC')::date AS day, count(DISTINCT game_uid) AS n FROM deck_results, t
             WHERE ($1::text IS NULL OR app_version = $1) AND (end_utc AT TIME ZONE 'UTC')::date >= today - 29
             GROUP BY 1
         )
         SELECT d.day, coalesce(m.n, 0) AS matches FROM d LEFT JOIN m USING (day) ORDER BY d.day",
    )
    .bind(&version)
    .fetch_all(&s.db)
    .await?;
    Ok(public(Overview { matches, players, decks, days }))
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Duo {
    pub cards: Vec<Uuid>,
    pub games: i64,
    pub wins: i64,
    pub win_rate: Option<f64>,
    pub win_score: Option<f64>,
}

#[utoipa::path(get, path = "/api/stats/duos", tag = "stats", params(VersionQuery),
    responses((status = 200, body = Vec<Duo>,
        description = "Pairs of 2 different cards in the same deck result (cards sorted), with 20 games or more in \
            2 different decks or more. Sorted by winScore, winRate and games, descending, then the cards. At most 24."),
        (status = 400, body = ErrorBody)))]
pub async fn duos(State(s): State<AppState>, Query(q): Query<VersionQuery>) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    let rows: Vec<Duo> = sqlx::query_as(
        "SELECT ARRAY[a.card, b.card] AS cards, count(*) AS games, count(*) FILTER (WHERE r.win) AS wins,
             win_rate(count(*), count(*) FILTER (WHERE r.win)) AS win_rate,
             win_score(count(*), count(*) FILTER (WHERE r.win)) AS win_score
         FROM deck_results r
         CROSS JOIN LATERAL unnest(r.deck_key) AS a (card)
         CROSS JOIN LATERAL unnest(r.deck_key) AS b (card)
         WHERE ($1::text IS NULL OR r.app_version = $1) AND a.card < b.card
         GROUP BY a.card, b.card HAVING count(*) >= 20 AND count(DISTINCT r.deck_key) >= 2
         ORDER BY win_score DESC, win_rate DESC, games DESC, a.card, b.card LIMIT 24",
    )
    .bind(&version)
    .fetch_all(&s.db)
    .await?;
    Ok(public(rows))
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CountersQuery {
    #[param(example = "all")]
    pub version: Option<String>,
    #[param(example = "0c8c383b-2207-4d18-b378-4f2a3795a58e,f16bc1f8-5cea-4acf-9e2a-0614062cb9b8")]
    pub against: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct Counters {
    pub games: i64,
    pub cards: Vec<Counter>,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Counter {
    pub card: Uuid,
    pub games: i64,
    pub wins: i64,
    pub win_rate: Option<f64>,
    pub win_score: Option<f64>,
}

fn against(v: Option<&str>) -> Result<Vec<Uuid>, ApiError> {
    let bad = || ApiError::new(StatusCode::BAD_REQUEST, "against must be 1 to 4 different card ids");
    let ids: Vec<Uuid> = v.unwrap_or("").split(',').map(str::parse).collect::<Result<_, _>>().map_err(|_| bad())?;
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    if ids.len() > 4 || unique.len() != ids.len() {
        return Err(bad());
    }
    Ok(ids)
}

#[utoipa::path(get, path = "/api/stats/counters", tag = "stats", params(CountersQuery),
    responses((status = 200, body = Counters,
        description = "Targets: the deck results whose deck has all the against cards. games: the count of targets. \
            cards: each card of the opposing deck that is not in against, with 10 games or more. wins: the games that \
            the opposing deck won. Sorted by winScore, winRate and games, descending, then card. At most 24."),
        (status = 400, body = ErrorBody)))]
pub async fn counters(State(s): State<AppState>, Query(q): Query<CountersQuery>) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    let against = against(q.against.as_deref())?;
    let games: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM deck_results WHERE ($1::text IS NULL OR app_version = $1) AND deck_key @> $2",
    )
    .bind(&version)
    .bind(&against)
    .fetch_one(&s.db)
    .await?;
    let cards: Vec<Counter> = sqlx::query_as(
        "SELECT c.card, count(*) AS games, count(*) FILTER (WHERE o.win) AS wins,
             win_rate(count(*), count(*) FILTER (WHERE o.win)) AS win_rate,
             win_score(count(*), count(*) FILTER (WHERE o.win)) AS win_score
         FROM deck_results r
         JOIN deck_results o ON o.game_uid = r.game_uid AND o.deck_key <> r.deck_key
         CROSS JOIN LATERAL unnest(o.deck_key) AS c (card)
         WHERE ($1::text IS NULL OR r.app_version = $1) AND r.deck_key @> $2 AND NOT c.card = ANY($2)
         GROUP BY c.card HAVING count(*) >= 10
         ORDER BY win_score DESC, win_rate DESC, games DESC, c.card LIMIT 24",
    )
    .bind(&version)
    .bind(&against)
    .fetch_all(&s.db)
    .await?;
    Ok(public(Counters { games, cards }))
}

#[derive(Serialize, ToSchema)]
pub struct PaceStats {
    pub turns: Vec<TurnBucket>,
    pub median: Option<i32>,
    pub short: Vec<CardPace>,
    pub long: Vec<CardPace>,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct TurnBucket {
    pub turns: i32,
    pub matches: i64,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardPace {
    pub card: Uuid,
    pub short_games: i64,
    pub short_wins: i64,
    pub short_win_rate: f64,
    pub long_games: i64,
    pub long_wins: i64,
    pub long_win_rate: f64,
    pub gap: f64,
    pub margin: f64,
    pub curve: Vec<TurnGroup>,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct TurnGroup {
    pub start: i32,
    pub games: i64,
    pub wins: i64,
}

#[derive(FromRow)]
struct PaceRow {
    card: Uuid,
    short_games: i64,
    short_wins: i64,
    long_games: i64,
    long_wins: i64,
}

#[derive(FromRow)]
struct CardGroup {
    card: Uuid,
    start: i32,
    games: i64,
    wins: i64,
}

const PACE_GAMES: i64 = 20;
const PACE_Z: f64 = 3.29;
const PACE_LIST: usize = 10;

fn pace_lists(rows: Vec<PaceRow>) -> (Vec<CardPace>, Vec<CardPace>) {
    let mut cards: Vec<CardPace> = rows
        .into_iter()
        .filter(|r| r.short_games >= PACE_GAMES && r.long_games >= PACE_GAMES)
        .map(|r| {
            let (n1, n2) = (r.short_games as f64, r.long_games as f64);
            let (p1, p2) = (r.short_wins as f64 / n1, r.long_wins as f64 / n2);
            CardPace {
                card: r.card,
                short_games: r.short_games,
                short_wins: r.short_wins,
                short_win_rate: p1,
                long_games: r.long_games,
                long_wins: r.long_wins,
                long_win_rate: p2,
                gap: p1 - p2,
                margin: PACE_Z * (p1 * (1.0 - p1) / n1 + p2 * (1.0 - p2) / n2).sqrt(),
                curve: Vec::new(),
            }
        })
        .filter(|c| c.gap.abs() > c.margin)
        .collect();
    let lead = |c: &CardPace| c.gap.abs() - c.margin;
    cards.sort_by(|a, b| lead(b).total_cmp(&lead(a)).then(a.card.cmp(&b.card)));
    let (mut short, mut long): (Vec<_>, Vec<_>) = cards.into_iter().partition(|c| c.gap > 0.0);
    short.truncate(PACE_LIST);
    long.truncate(PACE_LIST);
    (short, long)
}

const PACE_MATCHES: &str = "SELECT DISTINCT r.game_uid, r.app_version, m.turn_count FROM deck_results r
    JOIN matches m ON m.game_uid = r.game_uid WHERE $1::text IS NULL OR r.app_version = $1";

#[utoipa::path(get, path = "/api/stats/pace", tag = "stats", params(VersionQuery),
    responses((status = 200, body = PaceStats,
        description = "turns: the match count for each turn count. median: the median turn count of the version \
            (percentile_disc 0.5), null for all and when there is no match. A match is short when its turn count is \
            at most the median of its own version, else long. short and long: at most 10 cards with 20 games or more \
            in each part whose gap (shortWinRate - longWinRate) is larger than its 99.9 % margin of error (z = 3.29, strict \
            because about 150 cards are tested at once); short has a \
            positive gap, long a negative gap, both sorted by |gap| - margin descending, then card. curve: the games \
            and wins of the card by group of 3 turns (start = t - (t - 1) % 3)."),
        (status = 400, body = ErrorBody)))]
pub async fn pace(State(s): State<AppState>, Query(q): Query<VersionQuery>) -> Result<Response, ApiError> {
    let version = version(&s, q.version.as_deref())?;
    let turns: Vec<TurnBucket> = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT turn_count AS turns, count(*) AS matches FROM ({PACE_MATCHES}) g GROUP BY turn_count ORDER BY turn_count"
    )))
    .bind(&version)
    .fetch_all(&s.db)
    .await?;
    let median: Option<i32> = match &version {
        Some(_) => {
            sqlx::query_scalar(AssertSqlSafe(format!(
                "SELECT percentile_disc(0.5) WITHIN GROUP (ORDER BY turn_count) FROM ({PACE_MATCHES}) g"
            )))
            .bind(&version)
            .fetch_one(&s.db)
            .await?
        }
        None => None,
    };
    let rows: Vec<PaceRow> = sqlx::query_as(AssertSqlSafe(format!(
        "WITH g AS ({PACE_MATCHES}),
         v AS (SELECT app_version, percentile_disc(0.5) WITHIN GROUP (ORDER BY turn_count) AS median
             FROM g GROUP BY app_version),
         p AS (SELECT g.game_uid, g.turn_count <= v.median AS short FROM g JOIN v USING (app_version))
         SELECT c.card, count(*) FILTER (WHERE p.short) AS short_games,
             count(*) FILTER (WHERE p.short AND r.win) AS short_wins,
             count(*) FILTER (WHERE NOT p.short) AS long_games,
             count(*) FILTER (WHERE NOT p.short AND r.win) AS long_wins
         FROM deck_results r JOIN p ON p.game_uid = r.game_uid
         CROSS JOIN LATERAL unnest(r.deck_key) AS c (card)
         GROUP BY c.card"
    )))
    .bind(&version)
    .fetch_all(&s.db)
    .await?;
    let (mut short, mut long) = pace_lists(rows);
    let listed: Vec<Uuid> = short.iter().chain(&long).map(|c| c.card).collect();
    let groups: Vec<CardGroup> = sqlx::query_as(
        "SELECT c.card, m.turn_count - (m.turn_count - 1) % 3 AS start, count(*) AS games,
             count(*) FILTER (WHERE r.win) AS wins
         FROM deck_results r JOIN matches m ON m.game_uid = r.game_uid
         CROSS JOIN LATERAL unnest(r.deck_key) AS c (card)
         WHERE ($1::text IS NULL OR r.app_version = $1) AND c.card = ANY($2)
         GROUP BY 1, 2 ORDER BY 2",
    )
    .bind(&version)
    .bind(&listed)
    .fetch_all(&s.db)
    .await?;
    for card in short.iter_mut().chain(long.iter_mut()) {
        card.curve = groups
            .iter()
            .filter(|g| g.card == card.card)
            .map(|g| TurnGroup { start: g.start, games: g.games, wins: g.wins })
            .collect();
    }
    Ok(public(PaceStats { turns, median, short, long }))
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
        .route("/api/stats/overview", get(overview))
        .route("/api/stats/duos", get(duos))
        .route("/api/stats/counters", get(counters))
        .route("/api/stats/pace", get(pace))
        .route("/api/stats/history", get(history))
}
