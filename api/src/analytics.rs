use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{DateTime, DurationRound, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use utoipa::{IntoParams, ToSchema};
use uuid::{Uuid, Version};

use crate::{
    ApiError, AppState, ErrorBody,
    auth::{self, AnyUser, CurrentUser},
    catalog::parse_version,
    matches::MinVersion,
};

pub const WEB_TTL: Duration = Duration::from_secs(90);
pub const MAX_LIVE: usize = 5_000;
pub const MAX_DISTINCT: usize = 50_000;
pub const CACHE_TTL: Duration = Duration::from_secs(30);

#[derive(Default)]
pub struct Presence(Mutex<PresenceState>);

#[derive(Default)]
struct PresenceState {
    live: HashMap<Uuid, Instant>,
    web_peak: usize,
    app_peak: i64,
    web_ids: HashSet<Uuid>,
    devices: HashSet<i64>,
    totals: Option<(Instant, (i64, i64))>,
}

impl PresenceState {
    fn prune(&mut self, now: Instant) -> usize {
        self.live.retain(|_, t| now.saturating_duration_since(*t) < WEB_TTL);
        self.live.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HourValues {
    pub web_peak: i32,
    pub app_peak: i32,
    pub web_distinct: i32,
    pub app_distinct: i32,
}

fn int(n: impl TryInto<i32>) -> i32 {
    n.try_into().unwrap_or(i32::MAX)
}

impl Presence {
    fn lock(&self) -> MutexGuard<'_, PresenceState> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn ping(&self, id: Uuid, now: Instant) -> usize {
        let mut p = self.lock();
        p.prune(now);
        if p.live.len() < MAX_LIVE || p.live.contains_key(&id) {
            p.live.insert(id, now);
        }
        if p.web_ids.len() < MAX_DISTINCT {
            p.web_ids.insert(id);
        }
        p.live.len()
    }

    pub fn web_online(&self, now: Instant) -> usize {
        self.lock().prune(now)
    }

    pub fn device(&self, id: i64) {
        self.lock().devices.insert(id);
    }

    pub fn peaks(&self, web: usize, app: i64) {
        let mut p = self.lock();
        p.web_peak = p.web_peak.max(web);
        p.app_peak = p.app_peak.max(app);
    }

    pub fn take_hour(&self) -> HourValues {
        let mut p = self.lock();
        let values = HourValues {
            web_peak: int(p.web_peak),
            app_peak: int(p.app_peak),
            web_distinct: int(p.web_ids.len()),
            app_distinct: int(p.devices.len()),
        };
        p.web_peak = 0;
        p.app_peak = 0;
        p.web_ids.clear();
        p.devices.clear();
        values
    }

    pub fn cached_totals(&self, now: Instant) -> Option<(i64, i64)> {
        self.lock().totals.filter(|(at, _)| now.saturating_duration_since(*at) < CACHE_TTL).map(|(_, t)| t)
    }

    pub fn cache_totals(&self, now: Instant, totals: (i64, i64)) {
        self.lock().totals = Some((now, totals));
    }
}

fn no_store<T: Serialize>(body: T) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], Json(body)).into_response()
}

fn bad(message: &str) -> ApiError {
    ApiError::new(StatusCode::BAD_REQUEST, message)
}

pub fn utc_hour(t: DateTime<Utc>) -> DateTime<Utc> {
    t.duration_trunc(TimeDelta::hours(1)).unwrap_or(t)
}

pub async fn app_online(db: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM devices WHERE online_until > now()").fetch_one(db).await
}

pub async fn write_hour(db: &PgPool, hour: DateTime<Utc>, v: HourValues) -> Result<bool, sqlx::Error> {
    let inserted = sqlx::query(
        "INSERT INTO analytics_hourly (hour, web_peak, app_peak, web_distinct, app_distinct, matches, ranked_matches,
             users, linked_accounts, public_decks, devices)
         SELECT $1, $2, $3, $4, $5,
             (SELECT count(*) FROM matches),
             (SELECT count(DISTINCT game_uid) FROM deck_results),
             (SELECT count(*) FROM users),
             (SELECT count(*) FROM users WHERE spam_account_id IS NOT NULL),
             (SELECT count(*) FROM decks WHERE status = 'public'),
             (SELECT count(*) FROM devices)
         ON CONFLICT (hour) DO NOTHING",
    )
    .bind(hour)
    .bind(v.web_peak)
    .bind(v.app_peak)
    .bind(v.web_distinct)
    .bind(v.app_distinct)
    .execute(db)
    .await?
    .rows_affected();
    Ok(inserted == 1)
}

pub struct Hourly {
    pub current: DateTime<Utc>,
    pub pending: Vec<(DateTime<Utc>, HourValues)>,
}

impl Hourly {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self { current: utc_hour(now), pending: Vec::new() }
    }
}

pub async fn tick(s: &AppState, hourly: &mut Hourly, now: DateTime<Utc>) -> Result<(), sqlx::Error> {
    let hour = utc_hour(now);
    if hour != hourly.current {
        let finished = std::mem::replace(&mut hourly.current, hour);
        hourly.pending.push((finished, s.presence.take_hour()));
    }
    let web = s.presence.web_online(Instant::now());
    s.presence.peaks(web, app_online(&s.db).await?);
    while let Some(&(hour, values)) = hourly.pending.first() {
        write_hour(&s.db, hour, values).await?;
        hourly.pending.remove(0);
    }
    Ok(())
}

#[derive(Deserialize, ToSchema)]
pub struct Ping {
    #[schema(format = Uuid)]
    pub id: String,
}

#[derive(Serialize, ToSchema)]
pub struct Online {
    pub online: i64,
    pub matches: i64,
}

#[utoipa::path(post, path = "/api/presence", tag = "analytics", request_body = Ping,
    responses((status = 200, body = Online, description = "Web tabs and desktop apps online now, and the match total. Not cached."),
        (status = 400, body = ErrorBody)))]
pub async fn presence(State(s): State<AppState>, Json(body): Json<Ping>) -> Result<Response, ApiError> {
    let id = Uuid::try_parse(&body.id)
        .ok()
        .filter(|id| id.get_version() == Some(Version::Random))
        .ok_or_else(|| bad("id must be a version 4 UUID"))?;
    let now = Instant::now();
    let web = s.presence.ping(id, now);
    let (app, matches) = match s.presence.cached_totals(now) {
        Some(totals) => totals,
        None => {
            let totals: (i64, i64) = sqlx::query_as(
                "SELECT (SELECT count(*) FROM devices WHERE online_until > now()), (SELECT count(*) FROM matches)",
            )
            .fetch_one(&s.db)
            .await?;
            s.presence.cache_totals(now, totals);
            totals
        }
    };
    Ok(no_store(Online { online: web as i64 + app, matches }))
}

#[derive(Deserialize, ToSchema, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AppEvent {
    Open,
    Check,
    Close,
}

#[derive(Deserialize, ToSchema)]
pub struct AppPresence {
    pub event: AppEvent,
    #[schema(example = "0.1.0")]
    pub version: String,
}

#[utoipa::path(post, path = "/api/desktop/presence", tag = "desktop", request_body = AppPresence,
    responses((status = 200, body = MinVersion, description = "open and check: online for 15 minutes. close: offline. Not cached."),
        (status = 400, body = ErrorBody), (status = 401, body = ErrorBody),
        (status = 403, body = ErrorBody, description = "A cookie session. Only a device token is allowed.")))]
pub async fn desktop_presence(
    State(s): State<AppState>,
    AnyUser(user): AnyUser,
    headers: HeaderMap,
    Json(body): Json<AppPresence>,
) -> Result<Response, ApiError> {
    if user.via_cookie {
        return Err(ApiError::new(StatusCode::FORBIDDEN, "device token required"));
    }
    if parse_version(&body.version).is_none() {
        return Err(bad("version is invalid"));
    }
    let token = auth::bearer(&headers).unwrap_or_default();
    let device: i64 = sqlx::query_scalar(
        "UPDATE devices SET online_until = CASE WHEN $2 THEN now() + interval '15 minutes' END, app_version = $3
         WHERE token_hash = $1 RETURNING id",
    )
    .bind(auth::hash(token))
    .bind(body.event != AppEvent::Close)
    .bind(&body.version)
    .fetch_optional(&s.db)
    .await?
    .ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "login required"))?;
    s.presence.device(device);
    Ok(no_store(MinVersion { min_version: s.config.desktop_min_version.clone() }))
}

#[derive(Deserialize, ToSchema, Clone, Copy, Default)]
pub enum AnalyticsRange {
    #[serde(rename = "24h")]
    Day,
    #[default]
    #[serde(rename = "7d")]
    Week,
    #[serde(rename = "30d")]
    Month,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AnalyticsQuery {
    pub range: Option<AnalyticsRange>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OnlineNow {
    pub web_online: i64,
    pub app_online: i64,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MatchTotals {
    pub total: i64,
    pub ranked: i64,
    pub ai: i64,
    pub disputed: i64,
    pub last24h: i64,
    pub last7d: i64,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Players {
    pub accounts: i64,
    pub linked_accounts: i64,
    pub banned: i64,
    pub signups7d: i64,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Community {
    pub public_decks: i64,
    pub draft_decks: i64,
    pub comments: i64,
    pub votes: i64,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct VersionCount {
    pub version: Option<String>,
    pub devices: i64,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Desktop {
    pub devices: i64,
    pub seen24h: i64,
    #[sqlx(skip)]
    pub versions: Vec<VersionCount>,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct Uploads {
    pub web: i64,
    pub desktop: i64,
    pub unknown: i64,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct HourStats {
    pub hour: DateTime<Utc>,
    pub web_peak: Option<i32>,
    pub app_peak: Option<i32>,
    pub web_distinct: Option<i32>,
    pub app_distinct: Option<i32>,
    pub matches: Option<i64>,
    pub ranked_matches: Option<i64>,
    pub users: Option<i64>,
    pub linked_accounts: Option<i64>,
    pub public_decks: Option<i64>,
    pub devices: Option<i64>,
    pub matches_added: i64,
}

#[derive(Serialize, ToSchema)]
pub struct Analytics {
    pub now: OnlineNow,
    pub matches: MatchTotals,
    pub players: Players,
    pub community: Community,
    pub desktop: Desktop,
    pub uploads: Uploads,
    pub hours: Vec<HourStats>,
}

#[utoipa::path(get, path = "/api/admin/analytics", tag = "admin", params(AnalyticsQuery),
    responses((status = 200, body = Analytics, description = "hours: 1 item per UTC hour of the range, oldest first, the current hour last. A missing hour row gives null fields."),
        (status = 400, body = ErrorBody), (status = 403, body = ErrorBody)))]
pub async fn admin_analytics(
    State(s): State<AppState>,
    user: CurrentUser,
    Query(q): Query<AnalyticsQuery>,
) -> Result<Json<Analytics>, ApiError> {
    user.require_admin()?;
    let hours = match q.range.unwrap_or_default() {
        AnalyticsRange::Day => 24,
        AnalyticsRange::Week => 24 * 7,
        AnalyticsRange::Month => 24 * 30,
    };
    let web_online = s.presence.web_online(Instant::now()) as i64;
    let app_online = app_online(&s.db).await?;
    let matches = sqlx::query_as(
        "SELECT count(*) AS total, (SELECT count(DISTINCT game_uid) FROM deck_results) AS ranked,
             count(*) FILTER (WHERE is_ai) AS ai, count(*) FILTER (WHERE disputed) AS disputed,
             count(*) FILTER (WHERE created_at >= now() - interval '24 hours') AS last24h,
             count(*) FILTER (WHERE created_at >= now() - interval '7 days') AS last7d
         FROM matches",
    )
    .fetch_one(&s.db)
    .await?;
    let players = sqlx::query_as(
        "SELECT count(*) AS accounts, count(spam_account_id) AS linked_accounts, count(banned_at) AS banned,
             count(*) FILTER (WHERE created_at >= now() - interval '7 days') AS signups7d
         FROM users",
    )
    .fetch_one(&s.db)
    .await?;
    let community = sqlx::query_as(
        "SELECT count(*) FILTER (WHERE status = 'public') AS public_decks,
             count(*) FILTER (WHERE status = 'draft') AS draft_decks,
             (SELECT count(*) FROM comments) AS comments, (SELECT count(*) FROM votes) AS votes
         FROM decks",
    )
    .fetch_one(&s.db)
    .await?;
    let mut desktop: Desktop = sqlx::query_as(
        "SELECT count(*) AS devices, count(*) FILTER (WHERE last_used_at >= now() - interval '24 hours') AS seen24h
         FROM devices",
    )
    .fetch_one(&s.db)
    .await?;
    desktop.versions = sqlx::query_as(
        "SELECT app_version AS version, count(*) AS devices FROM devices
         WHERE last_used_at >= now() - interval '7 days' GROUP BY app_version",
    )
    .fetch_all(&s.db)
    .await?;
    desktop.versions.sort_by_cached_key(|v| std::cmp::Reverse(v.version.as_deref().and_then(parse_version)));
    let uploads = sqlx::query_as(
        "SELECT count(*) FILTER (WHERE source = 'web') AS web, count(*) FILTER (WHERE source = 'desktop') AS desktop,
             count(*) FILTER (WHERE source IS NULL) AS unknown
         FROM uploads",
    )
    .fetch_one(&s.db)
    .await?;
    let hours = sqlx::query_as(
        "WITH h AS (
             SELECT generate_series(date_trunc('hour', now(), 'UTC') - make_interval(hours => $1 - 1),
                 date_trunc('hour', now(), 'UTC'), interval '1 hour') AS hour
         ),
         added AS (
             SELECT date_trunc('hour', created_at, 'UTC') AS hour, count(*) AS n FROM matches
             WHERE created_at >= (SELECT min(hour) FROM h) GROUP BY 1
         )
         SELECT h.hour, a.web_peak, a.app_peak, a.web_distinct, a.app_distinct, a.matches, a.ranked_matches, a.users,
             a.linked_accounts, a.public_decks, a.devices, COALESCE(added.n, 0) AS matches_added
         FROM h LEFT JOIN analytics_hourly a USING (hour) LEFT JOIN added USING (hour)
         ORDER BY h.hour",
    )
    .bind(hours)
    .fetch_all(&s.db)
    .await?;
    Ok(Json(Analytics {
        now: OnlineNow { web_online, app_online },
        matches,
        players,
        community,
        desktop,
        uploads,
        hours,
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/presence", post(presence))
        .route("/api/desktop/presence", post(desktop_presence))
        .route("/api/admin/analytics", get(admin_analytics))
}
