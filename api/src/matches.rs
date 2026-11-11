use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::{Read, Write},
    sync::Mutex,
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Query, State, multipart::Field},
    http::StatusCode,
    response::Response,
    routing::{get, post},
};
use chrono::{DateTime, TimeDelta, Utc};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    ApiError, AppState, ErrorBody,
    auth::CurrentUser,
    catalog::{Catalog, parse_version},
    decks::DeckStatus,
    public, replay,
};

pub const MAX_INDEX_BYTES: usize = 1 << 20;
pub const MAX_REPLAY_BYTES: usize = 512 << 10;
pub const MAX_REPLAY_OUTPUT: u64 = 5 << 20;
pub const MAX_ENTRIES: usize = 50;
pub const UPLOADS_PER_HOUR: usize = 30;
const NOT_LINKED: &str = "AccountId is not the SPAM account linked to your Steam account";

pub struct RateLimiter {
    max: usize,
    window: Duration,
    times: Mutex<HashMap<i64, VecDeque<Instant>>>,
}

impl RateLimiter {
    pub fn new(max: usize, window: Duration) -> Self {
        Self { max, window, times: Mutex::default() }
    }

    pub fn allow(&self, user_id: i64, now: Instant) -> bool {
        let mut map = self.times.lock().unwrap_or_else(|e| e.into_inner());
        let times = map.entry(user_id).or_default();
        while times.front().is_some_and(|t| now.duration_since(*t) >= self.window) {
            times.pop_front();
        }
        if times.len() >= self.max {
            return false;
        }
        times.push_back(now);
        true
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct IndexEntry {
    app_version: String,
    game_uid: String,
    account_id: String,
    seat: i64,
    game_mode: i32,
    opponent_id: Option<String>,
    opponent_is_ai: bool,
    winner_id: Option<String>,
    is_draw: bool,
    my_cards: Vec<String>,
    opponent_cards: Vec<String>,
    start_utc: DateTime<Utc>,
    end_utc: DateTime<Utc>,
    duration_ms: i64,
    turn_count: i32,
    loading: Option<Loading>,
}

#[derive(Deserialize)]
struct Loading {
    me: Option<LoadingMe>,
}

#[derive(Deserialize)]
struct LoadingMe {
    stars: Option<i32>,
}

pub(crate) struct Match {
    game_uid: String,
    app_version: String,
    account_id: Uuid,
    seat: i16,
    game_mode: i32,
    is_ai: bool,
    cards: [Vec<Uuid>; 2],
    winner_seat: Option<i16>,
    start_utc: DateTime<Utc>,
    end_utc: DateTime<Utc>,
    duration_ms: i64,
    turn_count: i32,
    stars: Option<i32>,
}

pub fn valid_game_uid(s: &str) -> bool {
    s.len() <= 32 && s.strip_prefix("match-").is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
}

fn deck(cards: &[String], side: &str) -> Result<Vec<Uuid>, String> {
    let invalid = || format!("{side} must contain 4 distinct card UUIDs");
    let ids = cards.iter().map(|c| Uuid::parse_str(c)).collect::<Result<Vec<_>, _>>().map_err(|_| invalid())?;
    if ids.len() != 4 || sorted(&ids).windows(2).any(|w| w[0] == w[1]) {
        return Err(invalid());
    }
    Ok(ids)
}

fn sorted(ids: &[Uuid]) -> Vec<Uuid> {
    let mut ids = ids.to_vec();
    ids.sort();
    ids
}

pub(crate) fn parse_entry(raw: Value, now: DateTime<Utc>) -> Result<Match, String> {
    match raw.get("FormatVersion").and_then(Value::as_i64) {
        Some(2) => {}
        v => {
            let v = v.map_or("missing".into(), |v| v.to_string());
            tracing::warn!(
                game_uid = raw.get("GameUid").and_then(|v| v.as_str()),
                format_version = v,
                "rejected index entry with unsupported FormatVersion"
            );
            return Err(format!("unsupported FormatVersion {v}"));
        }
    }
    let e: IndexEntry = serde_json::from_value(raw).map_err(|e| format!("invalid entry: {e}"))?;
    if !valid_game_uid(&e.game_uid) {
        return Err("invalid GameUid".into());
    }
    if parse_version(&e.app_version).is_none() {
        return Err("invalid AppVersion".into());
    }
    let account_id = Uuid::parse_str(&e.account_id).map_err(|_| "invalid AccountId")?;
    let seat = match e.seat {
        0 => 0,
        1 => 1,
        _ => return Err("Seat must be 0 or 1".into()),
    };
    let mine = deck(&e.my_cards, "MyCards")?;
    let theirs = deck(&e.opponent_cards, "OpponentCards")?;
    if e.end_utc <= e.start_utc {
        return Err("EndUtc must be after StartUtc".into());
    }
    if e.start_utc > now + TimeDelta::minutes(5) {
        return Err("StartUtc is in the future".into());
    }
    if e.duration_ms.abs_diff((e.end_utc - e.start_utc).num_milliseconds()) > 5000 {
        return Err("DurationMs does not match EndUtc - StartUtc".into());
    }
    if e.turn_count < 1 {
        return Err("TurnCount must be at least 1".into());
    }
    let winner_seat = if e.is_draw {
        None
    } else if e.winner_id.as_deref() == Some(e.account_id.as_str()) {
        Some(seat)
    } else if e.opponent_is_ai || (e.winner_id.is_some() && e.winner_id == e.opponent_id) {
        Some(1 - seat)
    } else {
        return Err("WinnerId is not a player of the match".into());
    };
    let cards = if seat == 0 { [mine, theirs] } else { [theirs, mine] };
    Ok(Match {
        game_uid: e.game_uid,
        app_version: e.app_version,
        account_id,
        seat,
        game_mode: e.game_mode,
        is_ai: e.opponent_is_ai,
        cards,
        winner_seat,
        start_utc: e.start_utc,
        end_utc: e.end_utc,
        duration_ms: e.duration_ms,
        turn_count: e.turn_count,
        stars: e.loading.and_then(|l| l.me).and_then(|m| m.stars),
    })
}

fn varint(bytes: &[u8]) -> Option<(usize, &[u8])> {
    let mut n = 0u64;
    for (i, &b) in bytes.iter().enumerate().take(5) {
        n |= u64::from(b & 0x7f) << (7 * i);
        if b & 0x80 == 0 {
            return Some((usize::try_from(n).ok()?, &bytes[i + 1..]));
        }
    }
    None
}

fn push_varint(out: &mut Vec<u8>, mut n: usize) {
    while n >= 0x80 {
        out.push((n & 0x7f) as u8 | 0x80);
        n >>= 7;
    }
    out.push(n as u8);
}

fn blank(body: &mut [u8], prefix: &[u8], name: &[u8], filler: &[u8]) {
    let needle = [prefix, name].concat();
    let mut i = 0;
    while let Some(p) = body[i..].windows(needle.len()).position(|w| w == needle) {
        let at = i + p + prefix.len();
        body[at..at + name.len()].copy_from_slice(filler);
        i = at + name.len();
    }
}

const NAMES: [&str; 4] =
    ["/OpponentName", "/Loading/me/username", "/Loading/opponent/username", "/Loading/ally/username"];

pub(crate) fn gunzip(gz: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    GzDecoder::new(gz).take(MAX_REPLAY_OUTPUT + 1).read_to_end(&mut out).map_err(|_| "replay is not valid gzip")?;
    if out.len() as u64 > MAX_REPLAY_OUTPUT {
        return Err("replay is larger than 5 MB after gunzip".into());
    }
    Ok(out)
}

pub(crate) fn split_replay(raw: &[u8]) -> Result<(&[u8], &[u8]), String> {
    let rest = raw.strip_prefix(b"SRPL").ok_or("replay magic is not SRPL")?;
    if rest.get(..4) != Some(&2u32.to_le_bytes()[..]) {
        return Err("unsupported replay version".into());
    }
    let (len, rest) = varint(&rest[4..]).ok_or("invalid replay header length")?;
    let header = rest.get(..len).ok_or("replay header is truncated")?;
    Ok((header, &rest[len..]))
}

pub fn clean_replay(gz: &[u8]) -> Result<(String, Vec<u8>), String> {
    let out = gunzip(gz)?;
    let (header, rest) = split_replay(&out)?;
    let mut header: Value = serde_json::from_slice(header).map_err(|_| "invalid replay header")?;
    let game_uid = header.get("GameUid").and_then(Value::as_str).ok_or("invalid replay header")?.to_owned();
    let names: Vec<String> =
        NAMES.iter().filter_map(|p| header.pointer(p)?.as_str()).filter(|n| !n.is_empty()).map(str::to_owned).collect();
    for path in NAMES.iter().chain(&["/Loading/ticket"]) {
        if let Some((parent, key)) = path.rsplit_once('/')
            && let Some(o) = header.pointer_mut(parent).and_then(Value::as_object_mut)
        {
            o.remove(key);
        }
    }
    let header = serde_json::to_vec(&header).map_err(|_| "invalid replay header")?;
    let mut raw = b"SRPL".to_vec();
    raw.extend(2u32.to_le_bytes());
    push_varint(&mut raw, header.len());
    raw.extend(header);
    let start = raw.len();
    raw.extend(rest);
    let body = &mut raw[start..];
    for name in &names {
        let mut prefix = Vec::new();
        push_varint(&mut prefix, 2 * (name.len() + 1));
        blank(body, &prefix, name.as_bytes(), &vec![b'_'; name.len()]);
        let units: Vec<u16> = name.encode_utf16().collect();
        let utf16: Vec<u8> = units.iter().flat_map(|u| u.to_le_bytes()).collect();
        blank(body, &(units.len() as u32).to_le_bytes(), &utf16, &b"_\0".repeat(units.len()));
    }
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    let gz = enc.write_all(&raw).and_then(|()| enc.finish()).map_err(|e| e.to_string())?;
    Ok((game_uid, gz))
}

#[derive(Serialize, ToSchema, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    New,
    Known,
    Rejected,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EntryResult {
    pub game_uid: Option<String>,
    pub status: Status,
    pub reason: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReplayResult {
    pub file_name: Option<String>,
    pub game_uid: Option<String>,
    pub status: Status,
    pub reason: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct UploadResult {
    pub entries: Vec<EntryResult>,
    pub replays: Vec<ReplayResult>,
}

#[derive(ToSchema)]
pub struct UploadForm {
    #[schema(value_type = String, format = Binary)]
    pub index: Vec<u8>,
    #[schema(value_type = Vec<String>, format = Binary, required = false)]
    pub replay: Vec<Vec<u8>>,
}

pub async fn check_account(
    db: &PgPool,
    user_id: i64,
    linked: &mut Option<Uuid>,
    account_id: Uuid,
) -> Result<Result<(), String>, ApiError> {
    match *linked {
        Some(id) if id != account_id => Ok(Err(NOT_LINKED.into())),
        Some(_) => Ok(Ok(())),
        None => {
            let result = link_account(db, user_id, account_id).await?;
            if result.is_ok() {
                *linked = Some(account_id);
            }
            Ok(result)
        }
    }
}

async fn link_account(db: &PgPool, user_id: i64, account_id: Uuid) -> Result<Result<(), String>, ApiError> {
    let linked = sqlx::query_scalar::<_, Option<Uuid>>(
        "UPDATE users SET spam_account_id = COALESCE(spam_account_id, $2) WHERE id = $1 RETURNING spam_account_id",
    )
    .bind(user_id)
    .bind(account_id)
    .fetch_one(db)
    .await;
    match linked {
        Ok(Some(id)) if id == account_id => Ok(Ok(())),
        Ok(_) => Ok(Err(NOT_LINKED.into())),
        Err(e) if e.as_database_error().is_some_and(|d| d.is_unique_violation()) => {
            Ok(Err("AccountId is linked to another Steam account".into()))
        }
        Err(e) => Err(e.into()),
    }
}

pub(crate) async fn store(
    db: &PgPool,
    catalog: &Catalog,
    user_id: i64,
    m: &Match,
    source: &str,
) -> Result<Status, sqlx::Error> {
    let mut stars = [None, None];
    stars[m.seat as usize] = m.stars;
    let mut tx = db.begin().await?;
    let inserted = sqlx::query(
        "INSERT INTO matches (game_uid, app_version, game_mode, is_ai, start_utc, end_utc, duration_ms,
             turn_count, seat0_cards, seat1_cards, winner_seat, is_draw, seat0_stars, seat1_stars)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
         ON CONFLICT (game_uid) DO NOTHING",
    )
    .bind(&m.game_uid)
    .bind(&m.app_version)
    .bind(m.game_mode)
    .bind(m.is_ai)
    .bind(m.start_utc)
    .bind(m.end_utc)
    .bind(m.duration_ms)
    .bind(m.turn_count)
    .bind(&m.cards[0])
    .bind(&m.cards[1])
    .bind(m.winner_seat)
    .bind(m.winner_seat.is_none())
    .bind(stars[0])
    .bind(stars[1])
    .execute(&mut *tx)
    .await?
    .rows_affected()
        == 1;
    if inserted {
        let mut unknown: Vec<Uuid> =
            m.cards.iter().flatten().copied().filter(|id| !catalog.knows_card(&m.app_version, id)).collect();
        unknown.sort();
        unknown.dedup();
        sqlx::query(
            "INSERT INTO unknown_cards (card_id, seen_count, first_seen, last_seen)
             SELECT id, 1, now(), now() FROM unnest($1::uuid[]) AS id
             ON CONFLICT (card_id) DO UPDATE SET seen_count = unknown_cards.seen_count + 1, last_seen = now()",
        )
        .bind(&unknown)
        .execute(&mut *tx)
        .await?;
    } else {
        let (seat0, seat1, winner): (Vec<Uuid>, Vec<Uuid>, Option<i16>) =
            sqlx::query_as("SELECT seat0_cards, seat1_cards, winner_seat FROM matches WHERE game_uid = $1 FOR UPDATE")
                .bind(&m.game_uid)
                .fetch_one(&mut *tx)
                .await?;
        let uploads: Vec<(i64, i16)> = sqlx::query_as("SELECT user_id, seat FROM uploads WHERE game_uid = $1")
            .bind(&m.game_uid)
            .fetch_all(&mut *tx)
            .await?;
        if uploads.iter().any(|(u, _)| *u == user_id) {
            return Ok(Status::Known);
        }
        let disputed = uploads.iter().any(|(_, seat)| *seat == m.seat)
            || sorted(&seat0) != sorted(&m.cards[0])
            || sorted(&seat1) != sorted(&m.cards[1])
            || winner != m.winner_seat;
        if disputed {
            stars = [None, None];
        }
        sqlx::query(
            "UPDATE matches SET disputed = disputed OR $2,
                 seat0_stars = COALESCE(seat0_stars, $3), seat1_stars = COALESCE(seat1_stars, $4)
             WHERE game_uid = $1",
        )
        .bind(&m.game_uid)
        .bind(disputed)
        .bind(stars[0])
        .bind(stars[1])
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("INSERT INTO uploads (game_uid, user_id, seat, source) VALUES ($1, $2, $3, $4)")
        .bind(&m.game_uid)
        .bind(user_id)
        .bind(m.seat)
        .bind(source)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Status::New)
}

pub(crate) fn multipart_error(e: axum::extract::multipart::MultipartError) -> ApiError {
    ApiError::new(e.status(), e.body_text())
}

pub(crate) async fn read_field(field: &mut Field<'_>, max: usize) -> Result<Option<Vec<u8>>, ApiError> {
    let mut buf = Vec::new();
    let mut too_large = false;
    while let Some(chunk) = field.chunk().await.map_err(multipart_error)? {
        if too_large || buf.len() + chunk.len() > max {
            too_large = true;
            buf = Vec::new();
        } else {
            buf.extend_from_slice(&chunk);
        }
    }
    Ok((!too_large).then_some(buf))
}

type ReplayFile = (Option<String>, Result<(String, Vec<u8>), String>);

#[utoipa::path(post, path = "/api/matches", tag = "matches",
    request_body(content = UploadForm, content_type = "multipart/form-data"),
    responses((status = 200, body = UploadResult), (status = 400, body = ErrorBody), (status = 401, body = ErrorBody),
        (status = 403, body = ErrorBody), (status = 413, body = ErrorBody), (status = 429, body = ErrorBody)))]
pub async fn upload(
    State(s): State<AppState>,
    user: CurrentUser,
    mut multipart: Multipart,
) -> Result<Json<UploadResult>, ApiError> {
    if !s.upload_limiter.allow(user.id, Instant::now()) {
        return Err(ApiError::new(StatusCode::TOO_MANY_REQUESTS, "upload limit is 30 requests per hour"));
    }
    let bad = |m: &str| ApiError::new(StatusCode::BAD_REQUEST, m);
    let mut index = None;
    let mut files = Vec::new();
    while let Some(mut field) = multipart.next_field().await.map_err(multipart_error)? {
        match field.name() {
            Some("index") if index.is_none() => {
                index =
                    Some(read_field(&mut field, MAX_INDEX_BYTES).await?.ok_or_else(|| {
                        ApiError::new(StatusCode::PAYLOAD_TOO_LARGE, "index.json is larger than 1 MB")
                    })?);
            }
            Some("replay") if files.len() < MAX_ENTRIES => {
                let name = field.file_name().map(str::to_owned);
                files.push((name, read_field(&mut field, MAX_REPLAY_BYTES).await?));
            }
            Some("replay") => return Err(bad("too many replay files")),
            _ => return Err(bad("unexpected multipart field")),
        }
    }
    let index = index.ok_or_else(|| bad("missing index field"))?;
    let entries: Vec<Value> =
        serde_json::from_slice(&index).map_err(|e| bad(&format!("index.json is not a JSON array: {e}")))?;
    if entries.len() > MAX_ENTRIES {
        return Err(bad("index.json has more than 50 entries"));
    }
    let replays: Vec<ReplayFile> = tokio::task::spawn_blocking(move || {
        files
            .into_iter()
            .map(|(name, data)| {
                let parsed =
                    data.ok_or_else(|| "replay is larger than 512 KB".to_string()).and_then(|d| clean_replay(&d));
                (name, parsed)
            })
            .collect()
    })
    .await
    .map_err(|_| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "internal error"))?;

    let now = Utc::now();
    let source = if user.via_cookie { "web" } else { "desktop" };
    let mut linked = user.spam_account_id;
    let mut in_index = HashSet::new();
    let mut accepted = HashSet::new();
    let mut entry_results = Vec::with_capacity(entries.len());
    for raw in entries {
        let game_uid = raw.get("GameUid").and_then(Value::as_str).map(str::to_owned);
        in_index.extend(game_uid.clone());
        let result = match parse_entry(raw, now) {
            Err(reason) => Err(reason),
            Ok(m) => check_account(&s.db, user.id, &mut linked, m.account_id).await?.map(|()| m),
        };
        let (status, reason) = match result {
            Ok(m) => {
                let status = store(&s.db, &s.catalog, user.id, &m, source).await?;
                accepted.insert(m.game_uid);
                (status, None)
            }
            Err(reason) => (Status::Rejected, Some(reason)),
        };
        entry_results.push(EntryResult { game_uid, status, reason });
    }

    let mut replay_results = Vec::with_capacity(replays.len());
    let mut stored = Vec::new();
    for (file_name, parsed) in replays {
        let (game_uid, status, reason) = match parsed {
            Err(reason) => (None, Status::Rejected, Some(reason)),
            Ok((uid, _)) if !in_index.contains(&uid) => {
                (Some(uid), Status::Rejected, Some("GameUid is not in index.json".into()))
            }
            Ok((uid, _)) if !accepted.contains(&uid) => {
                (Some(uid), Status::Rejected, Some("index entry is rejected".into()))
            }
            Ok((uid, data)) => {
                let inserted = sqlx::query(
                    "INSERT INTO replays (game_uid, user_id, data, clean, expires_at)
                     VALUES ($1, $2, $3, true, now() + interval '12 months') ON CONFLICT DO NOTHING",
                )
                .bind(&uid)
                .bind(user.id)
                .bind(data)
                .execute(&s.db)
                .await?
                .rows_affected();
                if inserted == 1 {
                    stored.push(uid.clone());
                }
                (Some(uid), if inserted == 1 { Status::New } else { Status::Known }, None)
            }
        };
        replay_results.push(ReplayResult { file_name, game_uid, status, reason });
    }
    for uid in &stored {
        if let Err(e) = replay::decode_match(&s.db, uid).await {
            tracing::error!(game_uid = uid, error = %e, "replay decode failed");
        }
    }
    Ok(Json(UploadResult { entries: entry_results, replays: replay_results }))
}

pub async fn clean_stored_replays(db: &PgPool) -> Result<(u64, u64), Box<dyn std::error::Error + Send + Sync>> {
    let (mut cleaned, mut deleted) = (0, 0);
    loop {
        let rows: Vec<(String, i64, Vec<u8>)> =
            sqlx::query_as("SELECT game_uid, user_id, data FROM replays WHERE NOT clean LIMIT 50")
                .fetch_all(db)
                .await?;
        if rows.is_empty() {
            return Ok((cleaned, deleted));
        }
        let rows = tokio::task::spawn_blocking(move || {
            rows.into_iter().map(|(uid, user_id, data)| (uid, user_id, clean_replay(&data))).collect::<Vec<_>>()
        })
        .await?;
        for (uid, user_id, result) in rows {
            if let Ok((_, data)) = result {
                sqlx::query("UPDATE replays SET data = $3, clean = true WHERE game_uid = $1 AND user_id = $2")
                    .bind(&uid)
                    .bind(user_id)
                    .bind(data)
                    .execute(db)
                    .await?;
                cleaned += 1;
            } else {
                sqlx::query("DELETE FROM replays WHERE game_uid = $1 AND user_id = $2")
                    .bind(&uid)
                    .bind(user_id)
                    .execute(db)
                    .await?;
                tracing::warn!(game_uid = uid, "deleted a stored replay that cannot be cleaned");
                deleted += 1;
            }
        }
    }
}

#[derive(Serialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum MatchResult {
    Win,
    Loss,
    Draw,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MyMatch {
    pub game_uid: String,
    pub app_version: String,
    pub game_mode: i32,
    pub is_ai: bool,
    pub start_utc: DateTime<Utc>,
    pub duration_ms: i64,
    pub turn_count: i32,
    pub my_seat: i16,
    pub my_cards: Vec<Uuid>,
    pub opponent_cards: Vec<Uuid>,
    pub result: MatchResult,
    pub disputed: bool,
    #[sqlx(json)]
    pub hp: Vec<HpPoint>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HpPoint {
    pub turn: i32,
    pub mine: i32,
    pub opponent: i32,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
    pub before: Option<DateTime<Utc>>,
}

#[utoipa::path(get, path = "/api/me/matches", tag = "matches", params(HistoryQuery),
    responses((status = 200, body = Vec<MyMatch>,
        description = "Newest first. hp: the HP of each seat at the end of each turn, from the decoded replay, \
            empty when the match has no replay events."),
        (status = 401, body = ErrorBody)))]
pub async fn my_matches(
    State(s): State<AppState>,
    user: CurrentUser,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<Vec<MyMatch>>, ApiError> {
    let rows = sqlx::query_as(
        "SELECT m.game_uid, m.app_version, m.game_mode, m.is_ai, m.start_utc, m.duration_ms, m.turn_count,
             u.seat AS my_seat,
             CASE u.seat WHEN 0 THEN m.seat0_cards ELSE m.seat1_cards END AS my_cards,
             CASE u.seat WHEN 0 THEN m.seat1_cards ELSE m.seat0_cards END AS opponent_cards,
             CASE WHEN m.is_draw THEN 'draw' WHEN m.winner_seat = u.seat THEN 'win' ELSE 'loss' END AS result,
             m.disputed,
             COALESCE((SELECT json_agg(json_build_object('turn', a.turn, 'mine', a.hp, 'opponent', o.hp)
                           ORDER BY a.turn)
                       FROM hp_turns a JOIN hp_turns o ON o.game_uid = a.game_uid AND o.turn = a.turn
                           AND o.seat = 1 - a.seat
                       WHERE a.game_uid = m.game_uid AND a.seat = u.seat), '[]') AS hp
         FROM uploads u JOIN matches m USING (game_uid)
         WHERE u.user_id = $1 AND ($2::timestamptz IS NULL OR m.start_utc < $2)
         ORDER BY m.start_utc DESC
         LIMIT $3",
    )
    .bind(user.id)
    .bind(q.before)
    .bind(q.limit.unwrap_or(50).clamp(1, 100))
    .fetch_all(&s.db)
    .await?;
    Ok(Json(rows))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SavedDeck {
    pub id: i64,
    pub name: String,
    pub status: DeckStatus,
    pub cards: Vec<Uuid>,
    pub talents: Vec<i16>,
    pub guide: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopDeck {
    pub cards: Vec<Uuid>,
    pub games: i64,
    pub wins: i64,
    pub win_rate: Option<f64>,
    pub deck: Option<SavedDeck>,
}

#[derive(FromRow)]
struct TopRow {
    cards: Vec<Uuid>,
    games: i64,
    wins: i64,
    win_rate: Option<f64>,
    deck_id: Option<i64>,
    deck_name: Option<String>,
    deck_status: Option<DeckStatus>,
    deck_cards: Option<Vec<Uuid>>,
    deck_talents: Option<Vec<i16>>,
    deck_guide: Option<String>,
}

#[utoipa::path(get, path = "/api/me/top-decks", tag = "matches",
    responses((status = 200, body = Vec<TopDeck>, description = "Most played first, no AI or disputed match"),
        (status = 401, body = ErrorBody)))]
pub async fn my_top_decks(State(s): State<AppState>, user: CurrentUser) -> Result<Json<Vec<TopDeck>>, ApiError> {
    let rows: Vec<TopRow> = sqlx::query_as(
        "SELECT g.cards, g.games, g.wins, win_rate(g.games, g.wins) AS win_rate,
             d.id AS deck_id, d.name AS deck_name, d.status AS deck_status, d.cards AS deck_cards,
             d.talents AS deck_talents, d.guide AS deck_guide
         FROM (SELECT cards, count(*) AS games, count(*) FILTER (WHERE win) AS wins, max(start_utc) AS last
               FROM (SELECT array_sort(CASE u.seat WHEN 0 THEN m.seat0_cards ELSE m.seat1_cards END) AS cards,
                         m.winner_seat = u.seat AS win, m.start_utc
                     FROM uploads u JOIN matches m USING (game_uid)
                     WHERE u.user_id = $1 AND NOT m.is_ai AND NOT m.disputed) mine
               GROUP BY cards
               ORDER BY games DESC, last DESC
               LIMIT 3) g
         LEFT JOIN LATERAL (SELECT id, name, status, cards, talents, guide FROM decks
                            WHERE author_id = $1 AND deck_key = g.cards
                            ORDER BY updated_at DESC, id DESC LIMIT 1) d ON true
         ORDER BY g.games DESC, g.last DESC",
    )
    .bind(user.id)
    .fetch_all(&s.db)
    .await?;
    let top = rows
        .into_iter()
        .map(|r| TopDeck {
            deck: match (r.deck_id, r.deck_name, r.deck_status, r.deck_cards, r.deck_talents, r.deck_guide) {
                (Some(id), Some(name), Some(status), Some(cards), Some(talents), Some(guide)) => {
                    Some(SavedDeck { id, name, status, cards, talents, guide })
                }
                _ => None,
            },
            cards: r.cards,
            games: r.games,
            wins: r.wins,
            win_rate: r.win_rate,
        })
        .collect();
    Ok(Json(top))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MinVersion {
    pub min_version: String,
}

#[utoipa::path(get, path = "/api/desktop/min-version", tag = "desktop",
    responses((status = 200, body = MinVersion)))]
pub async fn min_version(State(s): State<AppState>) -> Response {
    public(MinVersion { min_version: s.config.desktop_min_version.clone() })
}

pub fn routes() -> Router<AppState> {
    let body_limit = MAX_INDEX_BYTES + MAX_ENTRIES * MAX_REPLAY_BYTES + (1 << 20);
    Router::new()
        .route("/api/matches", post(upload).layer(DefaultBodyLimit::max(body_limit)))
        .route("/api/me/matches", get(my_matches))
        .route("/api/me/top-decks", get(my_top_decks))
        .route("/api/desktop/min-version", get(min_version))
}
