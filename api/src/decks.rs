use std::{collections::HashSet, time::Instant};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post, put},
};
use chrono::{DateTime, Utc};
use pulldown_cmark::{Event, Options, Parser, html};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{ApiError, AppState, ErrorBody, auth::CurrentUser, matches::check_account, stats};
use sqlx::AssertSqlSafe;

pub const MAX_IMPORT_BYTES: usize = 1 << 20;
pub const COMMENTS_PER_WINDOW: usize = 10;
pub const COMMENT_WINDOW_SECS: u64 = 600;
const MARKER: &str = "Sending decks to server: ";
const NAME_MAX: usize = 60;
const GUIDE_MAX: usize = 50_000;
const COMMENT_MAX: usize = 2000;
const REASON_MAX: usize = 500;
const DEFAULT_NAME: &str = "Imported deck";
const MAX_OFFSET: i64 = 10_000;

const DECK_COLUMNS: &str = "d.id, d.author_id, u.display_name AS author_name, u.avatar_url AS author_avatar, d.name,
    d.cards, d.talents, d.status, d.app_version, d.game_deck_id,
    (SELECT count(*) FROM votes v WHERE v.deck_id = d.id) AS votes, d.created_at, d.updated_at, d.published_at";
const DECK_FROM: &str = "FROM decks d JOIN users u ON u.id = d.author_id";
const COMMENT_SELECT: &str = "SELECT c.id, c.deck_id, c.author_id, u.display_name AS author_name,
    u.avatar_url AS author_avatar, c.body, c.created_at, c.updated_at
    FROM comments c JOIN users u ON u.id = c.author_id";

fn bad(message: impl Into<String>) -> ApiError {
    ApiError::new(StatusCode::BAD_REQUEST, message)
}

fn deck_not_found() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "deck not found")
}

fn forbidden(message: &str) -> ApiError {
    ApiError::new(StatusCode::FORBIDDEN, message)
}

fn check_cards(cards: &[Uuid], talents: &[i64]) -> Result<(), String> {
    if cards.len() != 4 || cards.iter().collect::<HashSet<_>>().len() != 4 {
        return Err("a deck needs 4 distinct cards".into());
    }
    if talents.len() != 4 || talents.iter().any(|t| !(0..=2).contains(t)) {
        return Err("each card needs 1 talent index from 0 to 2".into());
    }
    Ok(())
}

fn check_text(text: &str, field: &str, max: usize) -> Result<String, ApiError> {
    let text = text.trim();
    let len = text.chars().count();
    if len == 0 || len > max {
        return Err(bad(format!("{field} must have 1 to {max} characters")));
    }
    Ok(text.to_string())
}

fn check_guide(guide: String) -> Result<String, ApiError> {
    if guide.chars().count() > GUIDE_MAX {
        return Err(bad(format!("guide must have at most {GUIDE_MAX} characters")));
    }
    Ok(guide)
}

pub fn render_guide(markdown: &str) -> String {
    let events = Parser::new_ext(markdown, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH).map(|e| match e {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        e => e,
    });
    let mut out = String::new();
    html::push_html(&mut out, events);
    ammonia::Builder::default().link_rel(Some("nofollow ugc noopener")).clean(&out).to_string()
}

#[derive(Serialize, Deserialize, ToSchema, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum DeckStatus {
    Draft,
    Public,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Deck {
    pub id: i64,
    pub author_id: i64,
    pub author_name: String,
    pub author_avatar: Option<String>,
    pub name: String,
    pub cards: Vec<Uuid>,
    pub talents: Vec<i16>,
    pub status: DeckStatus,
    pub app_version: String,
    pub game_deck_id: Option<String>,
    pub votes: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DeckItem {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub deck: Deck,
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub stats: stats::KeyStats,
    pub my_vote: Option<bool>,
}

#[derive(Serialize, ToSchema)]
pub struct DeckStats {
    pub version: String,
    pub current: stats::KeyStats,
    pub all: stats::KeyStats,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeckDetail {
    #[serde(flatten)]
    pub deck: Deck,
    pub guide: String,
    pub guide_html: String,
    pub my_vote: Option<bool>,
    pub stats: DeckStats,
}

#[derive(FromRow)]
struct DetailRow {
    #[sqlx(flatten)]
    deck: Deck,
    guide: String,
    my_vote: Option<bool>,
}

async fn load(s: &AppState, id: i64, viewer: Option<i64>) -> Result<DeckDetail, ApiError> {
    let row: DetailRow = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT {DECK_COLUMNS}, d.guide,
             CASE WHEN $2::bigint IS NOT NULL
                 THEN EXISTS (SELECT 1 FROM votes v WHERE v.deck_id = d.id AND v.user_id = $2) END AS my_vote
         {DECK_FROM} WHERE d.id = $1 AND (d.status = 'public' OR d.author_id = $2)"
    )))
    .bind(id)
    .bind(viewer)
    .fetch_optional(&s.db)
    .await?
    .ok_or_else(deck_not_found)?;
    let version = s.catalog.latest().to_string();
    let current = stats::key_stats(&s.db, &row.deck.cards, Some(&version)).await?;
    let all = stats::key_stats(&s.db, &row.deck.cards, None).await?;
    Ok(DeckDetail {
        guide_html: render_guide(&row.guide),
        deck: row.deck,
        guide: row.guide,
        my_vote: row.my_vote,
        stats: DeckStats { version, current, all },
    })
}

struct Owned {
    status: DeckStatus,
    cards: Vec<Uuid>,
    talents: Vec<i16>,
}

async fn owned(db: &PgPool, id: i64, user: &CurrentUser) -> Result<Owned, ApiError> {
    let row: Option<(i64, DeckStatus, Vec<Uuid>, Vec<i16>)> =
        sqlx::query_as("SELECT author_id, status, cards, talents FROM decks WHERE id = $1")
            .bind(id)
            .fetch_optional(db)
            .await?;
    let (author, status, cards, talents) = row.ok_or_else(deck_not_found)?;
    match (author == user.id, status) {
        (true, _) => Ok(Owned { status, cards, talents }),
        (false, DeckStatus::Public) => Err(forbidden("only the author can change this deck")),
        (false, DeckStatus::Draft) => Err(deck_not_found()),
    }
}

async fn public_deck_author(db: &PgPool, id: i64) -> Result<i64, ApiError> {
    sqlx::query_scalar("SELECT author_id FROM decks WHERE id = $1 AND status = 'public'")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(deck_not_found)
}

fn unknown_cards(s: &AppState, cards: &[Uuid]) -> Result<(), ApiError> {
    let unknown: Vec<String> = cards
        .iter()
        .filter(|c| !s.catalog.knows_card(s.catalog.latest(), c))
        .map(|c| c.to_string()[..8].to_string())
        .collect();
    if unknown.is_empty() {
        return Ok(());
    }
    Err(bad(format!("cards are not in the catalog: {}", unknown.join(", "))))
}

#[derive(Deserialize, ToSchema)]
pub struct NewDeck {
    pub name: String,
    pub cards: Vec<Uuid>,
    pub talents: Vec<i16>,
    #[serde(default)]
    pub guide: String,
}

#[derive(Deserialize, ToSchema)]
pub struct DeckPatch {
    pub name: Option<String>,
    pub cards: Option<Vec<Uuid>>,
    pub talents: Option<Vec<i16>>,
    pub guide: Option<String>,
}

fn wide(talents: &[i16]) -> Vec<i64> {
    talents.iter().map(|&t| i64::from(t)).collect()
}

#[utoipa::path(post, path = "/api/decks", tag = "decks", request_body = NewDeck,
    responses((status = 201, body = DeckDetail), (status = 400, body = ErrorBody), (status = 401, body = ErrorBody),
        (status = 403, body = ErrorBody)))]
pub async fn create_deck(
    State(s): State<AppState>,
    user: CurrentUser,
    Json(body): Json<NewDeck>,
) -> Result<(StatusCode, Json<DeckDetail>), ApiError> {
    let name = check_text(&body.name, "name", NAME_MAX)?;
    check_cards(&body.cards, &wide(&body.talents)).map_err(bad)?;
    let guide = check_guide(body.guide)?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO decks (author_id, name, cards, talents, guide, app_version)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(user.id)
    .bind(name)
    .bind(&body.cards)
    .bind(&body.talents)
    .bind(guide)
    .bind(s.catalog.latest())
    .fetch_one(&s.db)
    .await?;
    Ok((StatusCode::CREATED, Json(load(&s, id, Some(user.id)).await?)))
}

#[derive(Deserialize, ToSchema)]
pub struct GuidePreview {
    pub guide: String,
}

#[derive(Serialize, ToSchema)]
pub struct GuideHtml {
    pub html: String,
}

#[utoipa::path(post, path = "/api/decks/preview", tag = "decks", request_body = GuidePreview,
    responses((status = 200, body = GuideHtml, description = "The same HTML as guideHtml"),
        (status = 400, body = ErrorBody), (status = 401, body = ErrorBody), (status = 403, body = ErrorBody)))]
pub async fn preview(_user: CurrentUser, Json(body): Json<GuidePreview>) -> Result<Json<GuideHtml>, ApiError> {
    Ok(Json(GuideHtml { html: render_guide(&check_guide(body.guide)?) }))
}

#[utoipa::path(get, path = "/api/decks/{id}", tag = "decks", params(("id" = i64, Path, description = "Deck ID")),
    responses((status = 200, body = DeckDetail), (status = 404, body = ErrorBody)))]
pub async fn get_deck(
    State(s): State<AppState>,
    user: Option<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<DeckDetail>, ApiError> {
    Ok(Json(load(&s, id, user.map(|u| u.id)).await?))
}

#[utoipa::path(patch, path = "/api/decks/{id}", tag = "decks", request_body = DeckPatch,
    params(("id" = i64, Path, description = "Deck ID")),
    responses((status = 200, body = DeckDetail), (status = 400, body = ErrorBody), (status = 403, body = ErrorBody),
        (status = 404, body = ErrorBody)))]
pub async fn patch_deck(
    State(s): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(body): Json<DeckPatch>,
) -> Result<Json<DeckDetail>, ApiError> {
    let deck = owned(&s.db, id, &user).await?;
    let name = body.name.map(|n| check_text(&n, "name", NAME_MAX)).transpose()?;
    let guide = body.guide.map(check_guide).transpose()?;
    let cards = body.cards.unwrap_or(deck.cards);
    let talents = body.talents.unwrap_or(deck.talents);
    check_cards(&cards, &wide(&talents)).map_err(bad)?;
    if deck.status == DeckStatus::Public {
        unknown_cards(&s, &cards)?;
    }
    sqlx::query(
        "UPDATE decks SET name = COALESCE($2, name), cards = $3, talents = $4, guide = COALESCE($5, guide),
             updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(name)
    .bind(&cards)
    .bind(&talents)
    .bind(guide)
    .execute(&s.db)
    .await?;
    if deck.status == DeckStatus::Public {
        s.live.notify(id);
    }
    Ok(Json(load(&s, id, Some(user.id)).await?))
}

#[utoipa::path(delete, path = "/api/decks/{id}", tag = "decks", params(("id" = i64, Path, description = "Deck ID")),
    responses((status = 204, description = "Deck deleted"), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody)))]
pub async fn delete_deck(
    State(s): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let deck = owned(&s.db, id, &user).await?;
    sqlx::query("DELETE FROM decks WHERE id = $1").bind(id).execute(&s.db).await?;
    if deck.status == DeckStatus::Public {
        s.live.notify(id);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post, path = "/api/decks/{id}/publish", tag = "decks", params(("id" = i64, Path, description = "Deck ID")),
    responses((status = 200, body = DeckDetail), (status = 400, body = ErrorBody, description = "A card is not in the latest catalog"),
        (status = 403, body = ErrorBody), (status = 404, body = ErrorBody)))]
pub async fn publish(
    State(s): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<DeckDetail>, ApiError> {
    let deck = owned(&s.db, id, &user).await?;
    unknown_cards(&s, &deck.cards)?;
    let changed = sqlx::query(
        "UPDATE decks SET status = 'public', published_at = now(), updated_at = now()
         WHERE id = $1 AND status = 'draft'",
    )
    .bind(id)
    .execute(&s.db)
    .await?
    .rows_affected();
    if changed == 1 {
        s.live.notify(id);
    }
    Ok(Json(load(&s, id, Some(user.id)).await?))
}

#[utoipa::path(post, path = "/api/decks/{id}/unpublish", tag = "decks", params(("id" = i64, Path, description = "Deck ID")),
    responses((status = 200, body = DeckDetail), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody)))]
pub async fn unpublish(
    State(s): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<DeckDetail>, ApiError> {
    owned(&s.db, id, &user).await?;
    let changed =
        sqlx::query("UPDATE decks SET status = 'draft', updated_at = now() WHERE id = $1 AND status = 'public'")
            .bind(id)
            .execute(&s.db)
            .await?
            .rows_affected();
    if changed == 1 {
        s.live.notify(id);
    }
    Ok(Json(load(&s, id, Some(user.id)).await?))
}

#[derive(Deserialize, ToSchema, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum DeckSort {
    #[default]
    Votes,
    Recent,
    Winrate,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DeckQuery {
    pub sort: Option<DeckSort>,
    pub card: Option<Uuid>,
    #[param(value_type = Option<String>, example = "me")]
    pub author: Option<String>,
    pub limit: Option<i64>,
    #[param(maximum = 10_000)]
    pub offset: Option<i64>,
    pub before: Option<DateTime<Utc>>,
    #[param(min_length = 2, max_length = 64)]
    pub q: Option<String>,
}

#[utoipa::path(get, path = "/api/decks", tag = "decks", params(DeckQuery),
    responses((status = 200, body = Vec<DeckItem>,
        description = "Public decks. author=me lists the decks of the caller with drafts. \
            Page with offset for sort=votes and sort=winrate, and with before (publication time) for sort=recent. \
            games, wins and winRate are the stats of the deck key for the latest catalog version. \
            winRate is null at 0 games. winScore is the lower bound of the 95 % Wilson score interval of winRate, \
            null at 0 games. sort=winrate orders by winScore. q matches part of the deck name, without case. \
            myVote is null without login."),
        (status = 400, body = ErrorBody), (status = 401, body = ErrorBody)))]
pub async fn list_decks(
    State(s): State<AppState>,
    user: Option<CurrentUser>,
    Query(q): Query<DeckQuery>,
) -> Result<Json<Vec<DeckItem>>, ApiError> {
    let viewer = user.as_ref().map(|u| u.id);
    let (author, drafts) = match q.author.as_deref() {
        None => (None, false),
        Some("me") => {
            let user = user.ok_or_else(|| ApiError::new(StatusCode::UNAUTHORIZED, "login required"))?;
            (Some(user.id), true)
        }
        Some(id) => (Some(id.parse::<i64>().map_err(|_| bad("author must be me or a user ID"))?), false),
    };
    let offset = q.offset.unwrap_or(0);
    if !(0..=MAX_OFFSET).contains(&offset) {
        return Err(bad(format!("offset must be from 0 to {MAX_OFFSET}")));
    }
    let name = match q.q.as_deref().map(str::trim) {
        None => None,
        Some(t) if (2..=64).contains(&t.chars().count()) => {
            Some(format!("%{}%", t.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")))
        }
        Some(_) => return Err(bad("q must have 2 to 64 characters")),
    };
    let order = match q.sort.unwrap_or_default() {
        DeckSort::Votes => "votes DESC, d.id DESC",
        DeckSort::Recent => "COALESCE(d.published_at, d.created_at) DESC, d.id DESC",
        DeckSort::Winrate => "ws.win_score DESC NULLS LAST, votes DESC, d.id DESC",
    };
    let decks = sqlx::query_as(AssertSqlSafe(format!(
        "WITH ws AS (
             SELECT deck_key, count(*) AS games, count(*) FILTER (WHERE win) AS wins,
                 win_rate(count(*), count(*) FILTER (WHERE win)) AS win_rate,
                 win_score(count(*), count(*) FILTER (WHERE win)) AS win_score
             FROM deck_results WHERE app_version = $6 GROUP BY deck_key
         )
         SELECT {DECK_COLUMNS}, COALESCE(ws.games, 0) AS games, COALESCE(ws.wins, 0) AS wins, ws.win_rate,
             ws.win_score,
             CASE WHEN $9::bigint IS NOT NULL
                 THEN EXISTS (SELECT 1 FROM votes v WHERE v.deck_id = d.id AND v.user_id = $9) END AS my_vote
         {DECK_FROM} LEFT JOIN ws ON ws.deck_key = d.deck_key
         WHERE ($1::bigint IS NULL OR d.author_id = $1) AND (d.status = 'public' OR $2)
             AND ($3::uuid IS NULL OR $3 = ANY (d.cards))
             AND ($4::timestamptz IS NULL OR COALESCE(d.published_at, d.created_at) < $4)
             AND ($8::text IS NULL OR d.name ILIKE $8)
         ORDER BY {order} LIMIT $5 OFFSET $7"
    )))
    .bind(author)
    .bind(drafts)
    .bind(q.card)
    .bind(q.before)
    .bind(q.limit.unwrap_or(50).clamp(1, 100))
    .bind(s.catalog.latest())
    .bind(offset)
    .bind(name)
    .bind(viewer)
    .fetch_all(&s.db)
    .await?;
    Ok(Json(decks))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeckLine {
    user_id: Uuid,
    added: Option<Vec<GameDeck>>,
    updated: Option<Vec<GameDeck>>,
}

#[derive(Deserialize)]
struct GameDeck {
    id: String,
    name: Option<String>,
    cards: Vec<GameCard>,
}

#[derive(Deserialize)]
struct GameCard {
    id: Uuid,
    talent: i64,
}

struct ImportDeck {
    name: String,
    cards: Vec<Uuid>,
    talents: Vec<i16>,
}

fn import_deck(deck: GameDeck) -> Result<ImportDeck, String> {
    if deck.id.is_empty() || deck.id.chars().count() > 64 {
        return Err("invalid deck id".into());
    }
    let cards: Vec<Uuid> = deck.cards.iter().map(|c| c.id).collect();
    let talents: Vec<i64> = deck.cards.iter().map(|c| c.talent).collect();
    check_cards(&cards, &talents)?;
    let name: String = deck.name.as_deref().unwrap_or_default().trim().chars().take(NAME_MAX).collect();
    Ok(ImportDeck {
        name: if name.is_empty() { DEFAULT_NAME.into() } else { name },
        cards,
        talents: talents.into_iter().map(|t| t as i16).collect(),
    })
}

#[derive(Serialize, ToSchema, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ImportStatus {
    New,
    Updated,
    Unchanged,
    Skipped,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ImportedDeck {
    pub game_deck_id: String,
    pub deck_id: Option<i64>,
    pub name: Option<String>,
    pub status: ImportStatus,
    pub reason: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ImportResult {
    pub decks: Vec<ImportedDeck>,
}

async fn store_import(
    s: &AppState,
    user_id: i64,
    game_deck_id: &str,
    d: &ImportDeck,
) -> Result<(i64, ImportStatus, Option<String>), ApiError> {
    let inserted: Option<i64> = sqlx::query_scalar(
        "INSERT INTO decks (author_id, game_deck_id, name, cards, talents, app_version)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (author_id, game_deck_id) DO NOTHING RETURNING id",
    )
    .bind(user_id)
    .bind(game_deck_id)
    .bind(&d.name)
    .bind(&d.cards)
    .bind(&d.talents)
    .bind(s.catalog.latest())
    .fetch_optional(&s.db)
    .await?;
    if let Some(id) = inserted {
        return Ok((id, ImportStatus::New, None));
    }
    let (id, status, same): (i64, DeckStatus, bool) = sqlx::query_as(
        "SELECT id, status, name = $3 AND cards = $4 AND talents = $5 FROM decks
         WHERE author_id = $1 AND game_deck_id = $2",
    )
    .bind(user_id)
    .bind(game_deck_id)
    .bind(&d.name)
    .bind(&d.cards)
    .bind(&d.talents)
    .fetch_one(&s.db)
    .await?;
    let published = || Ok((id, ImportStatus::Skipped, Some("published".into())));
    match (status, same) {
        (DeckStatus::Public, _) => published(),
        (DeckStatus::Draft, true) => Ok((id, ImportStatus::Unchanged, None)),
        (DeckStatus::Draft, false) => {
            let updated = sqlx::query(
                "UPDATE decks SET name = $2, cards = $3, talents = $4, updated_at = now()
                 WHERE id = $1 AND status = 'draft'",
            )
            .bind(id)
            .bind(&d.name)
            .bind(&d.cards)
            .bind(&d.talents)
            .execute(&s.db)
            .await?
            .rows_affected();
            if updated == 0 { published() } else { Ok((id, ImportStatus::Updated, None)) }
        }
    }
}

#[utoipa::path(post, path = "/api/decks/import", tag = "decks",
    request_body(content = String, content_type = "text/plain",
        description = "Player-prev.log then Player.log lines with the marker 'Sending decks to server: '"),
    responses((status = 200, body = ImportResult), (status = 400, body = ErrorBody), (status = 401, body = ErrorBody),
        (status = 403, body = ErrorBody), (status = 413, body = ErrorBody)))]
pub async fn import(
    State(s): State<AppState>,
    user: CurrentUser,
    body: String,
) -> Result<Json<ImportResult>, ApiError> {
    let mut lines = Vec::new();
    for (n, line) in body.lines().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
        let json = line.split_once(MARKER).ok_or_else(|| bad(format!("line {} has no deck marker", n + 1)))?.1;
        let parsed: DeckLine =
            serde_json::from_str(json).map_err(|e| bad(format!("line {} is not valid deck JSON: {e}", n + 1)))?;
        lines.push(parsed);
    }
    let mut linked = user.spam_account_id;
    let mut latest: Vec<(String, Result<ImportDeck, String>)> = Vec::new();
    for line in lines {
        let account = check_account(&s.db, user.id, &mut linked, line.user_id).await?;
        for deck in line.added.into_iter().chain(line.updated).flatten() {
            let id = deck.id.clone();
            let parsed = account.clone().and_then(|()| import_deck(deck));
            match latest.iter_mut().find(|(known, _)| *known == id) {
                Some(slot) => slot.1 = parsed,
                None => latest.push((id, parsed)),
            }
        }
    }
    let mut decks = Vec::with_capacity(latest.len());
    for (game_deck_id, parsed) in latest {
        let (deck_id, name, status, reason) = match parsed {
            Ok(d) => {
                let (id, status, reason) = store_import(&s, user.id, &game_deck_id, &d).await?;
                (Some(id), Some(d.name), status, reason)
            }
            Err(reason) => (None, None, ImportStatus::Skipped, Some(reason)),
        };
        decks.push(ImportedDeck { game_deck_id, deck_id, name, status, reason });
    }
    Ok(Json(ImportResult { decks }))
}

#[utoipa::path(put, path = "/api/decks/{id}/vote", tag = "votes", params(("id" = i64, Path, description = "Deck ID")),
    responses((status = 204, description = "Vote stored"), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody)))]
pub async fn put_vote(
    State(s): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    if public_deck_author(&s.db, id).await? == user.id {
        return Err(forbidden("you cannot vote for your own deck"));
    }
    sqlx::query("INSERT INTO votes (deck_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(id)
        .bind(user.id)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(delete, path = "/api/decks/{id}/vote", tag = "votes", params(("id" = i64, Path, description = "Deck ID")),
    responses((status = 204, description = "Vote removed"), (status = 404, body = ErrorBody)))]
pub async fn delete_vote(
    State(s): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    public_deck_author(&s.db, id).await?;
    sqlx::query("DELETE FROM votes WHERE deck_id = $1 AND user_id = $2").bind(id).bind(user.id).execute(&s.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, ToSchema, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: i64,
    pub deck_id: i64,
    pub author_id: i64,
    pub author_name: String,
    pub author_avatar: Option<String>,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct CommentInput {
    pub body: String,
}

async fn comment(db: &PgPool, id: i64) -> Result<Comment, ApiError> {
    Ok(sqlx::query_as(AssertSqlSafe(format!("{COMMENT_SELECT} WHERE c.id = $1"))).bind(id).fetch_one(db).await?)
}

#[utoipa::path(get, path = "/api/decks/{id}/comments", tag = "comments", params(("id" = i64, Path, description = "Deck ID")),
    responses((status = 200, body = Vec<Comment>, description = "Oldest first"), (status = 404, body = ErrorBody)))]
pub async fn list_comments(State(s): State<AppState>, Path(id): Path<i64>) -> Result<Json<Vec<Comment>>, ApiError> {
    public_deck_author(&s.db, id).await?;
    let comments = sqlx::query_as(AssertSqlSafe(format!("{COMMENT_SELECT} WHERE c.deck_id = $1 ORDER BY c.id")))
        .bind(id)
        .fetch_all(&s.db)
        .await?;
    Ok(Json(comments))
}

#[utoipa::path(post, path = "/api/decks/{id}/comments", tag = "comments", request_body = CommentInput,
    params(("id" = i64, Path, description = "Deck ID")),
    responses((status = 201, body = Comment), (status = 400, body = ErrorBody), (status = 404, body = ErrorBody),
        (status = 429, body = ErrorBody)))]
pub async fn create_comment(
    State(s): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(input): Json<CommentInput>,
) -> Result<(StatusCode, Json<Comment>), ApiError> {
    let body = check_text(&input.body, "comment", COMMENT_MAX)?;
    public_deck_author(&s.db, id).await?;
    if !s.comment_limiter.allow(user.id, Instant::now()) {
        return Err(ApiError::new(StatusCode::TOO_MANY_REQUESTS, "comment limit is 10 per 10 minutes"));
    }
    let comment_id: i64 =
        sqlx::query_scalar("INSERT INTO comments (deck_id, author_id, body) VALUES ($1, $2, $3) RETURNING id")
            .bind(id)
            .bind(user.id)
            .bind(body)
            .fetch_one(&s.db)
            .await?;
    Ok((StatusCode::CREATED, Json(comment(&s.db, comment_id).await?)))
}

async fn own_comment(db: &PgPool, id: i64, user: &CurrentUser, public_only: bool) -> Result<(), ApiError> {
    let row: Option<(i64, DeckStatus)> =
        sqlx::query_as("SELECT c.author_id, d.status FROM comments c JOIN decks d ON d.id = c.deck_id WHERE c.id = $1")
            .bind(id)
            .fetch_optional(db)
            .await?;
    match row {
        Some((_, DeckStatus::Draft)) if public_only => Err(ApiError::new(StatusCode::NOT_FOUND, "comment not found")),
        Some((author, _)) if author != user.id => Err(forbidden("only the author can change this comment")),
        Some(_) => Ok(()),
        None => Err(ApiError::new(StatusCode::NOT_FOUND, "comment not found")),
    }
}

#[utoipa::path(patch, path = "/api/comments/{id}", tag = "comments", request_body = CommentInput,
    params(("id" = i64, Path, description = "Comment ID")),
    responses((status = 200, body = Comment), (status = 400, body = ErrorBody), (status = 403, body = ErrorBody),
        (status = 404, body = ErrorBody)))]
pub async fn patch_comment(
    State(s): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(input): Json<CommentInput>,
) -> Result<Json<Comment>, ApiError> {
    let body = check_text(&input.body, "comment", COMMENT_MAX)?;
    own_comment(&s.db, id, &user, true).await?;
    sqlx::query("UPDATE comments SET body = $2, updated_at = now() WHERE id = $1")
        .bind(id)
        .bind(body)
        .execute(&s.db)
        .await?;
    Ok(Json(comment(&s.db, id).await?))
}

#[utoipa::path(delete, path = "/api/comments/{id}", tag = "comments", params(("id" = i64, Path, description = "Comment ID")),
    responses((status = 204, description = "Comment deleted"), (status = 403, body = ErrorBody), (status = 404, body = ErrorBody)))]
pub async fn delete_comment(
    State(s): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    own_comment(&s.db, id, &user, false).await?;
    sqlx::query("DELETE FROM comments WHERE id = $1").bind(id).execute(&s.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, Deserialize, ToSchema, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum TargetType {
    Deck,
    Comment,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewReport {
    pub target_type: TargetType,
    pub target_id: i64,
    pub reason: String,
}

#[utoipa::path(post, path = "/api/reports", tag = "reports", request_body = NewReport,
    responses((status = 204, description = "Report stored. A second open report on the same target changes nothing."),
        (status = 400, body = ErrorBody), (status = 404, body = ErrorBody)))]
pub async fn create_report(
    State(s): State<AppState>,
    user: CurrentUser,
    Json(input): Json<NewReport>,
) -> Result<StatusCode, ApiError> {
    let reason = check_text(&input.reason, "reason", REASON_MAX)?;
    let (deck_id, comment_id) = match input.target_type {
        TargetType::Deck => {
            public_deck_author(&s.db, input.target_id).await?;
            (Some(input.target_id), None)
        }
        TargetType::Comment => {
            let visible: bool = sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM comments c JOIN decks d ON d.id = c.deck_id
                 WHERE c.id = $1 AND d.status = 'public')",
            )
            .bind(input.target_id)
            .fetch_one(&s.db)
            .await?;
            if !visible {
                return Err(ApiError::new(StatusCode::NOT_FOUND, "comment not found"));
            }
            (None, Some(input.target_id))
        }
    };
    sqlx::query(
        "INSERT INTO reports (reporter_id, deck_id, comment_id, reason) VALUES ($1, $2, $3, $4)
         ON CONFLICT (reporter_id, deck_id, comment_id) WHERE status = 'open' DO NOTHING",
    )
    .bind(user.id)
    .bind(deck_id)
    .bind(comment_id)
    .bind(reason)
    .execute(&s.db)
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/decks", get(list_decks).post(create_deck))
        .route("/api/decks/import", post(import).layer(DefaultBodyLimit::max(MAX_IMPORT_BYTES)))
        .route("/api/decks/preview", post(preview))
        .route("/api/decks/{id}", get(get_deck).patch(patch_deck).delete(delete_deck))
        .route("/api/decks/{id}/publish", post(publish))
        .route("/api/decks/{id}/unpublish", post(unpublish))
        .route("/api/decks/{id}/vote", put(put_vote).delete(delete_vote))
        .route("/api/decks/{id}/comments", get(list_comments).post(create_comment))
        .route("/api/comments/{id}", patch(patch_comment).delete(delete_comment))
        .route("/api/reports", post(create_report))
}
