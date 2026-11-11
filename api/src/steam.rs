use std::time::Duration;

use axum::{Router, extract::State, http::header, response::Response, routing::get};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sqlx::FromRow;
use url::Url;
use utoipa::ToSchema;

use crate::{ApiError, AppState, catalog::Catalog, public};

pub const APP_ID: u32 = 2902720;
const TIMEOUT: Duration = Duration::from_secs(10);
const NEWS_LIMIT: i64 = 50;

#[derive(Deserialize)]
struct PlayersBody {
    response: PlayersResult,
}

#[derive(Deserialize)]
struct PlayersResult {
    result: i64,
    player_count: Option<i32>,
}

#[derive(Deserialize)]
struct NewsBody {
    appnews: AppNews,
}

#[derive(Deserialize)]
struct AppNews {
    #[serde(default)]
    newsitems: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct RawNews {
    gid: String,
    title: String,
    url: String,
    date: i64,
    feedlabel: Option<String>,
    feedname: Option<String>,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct PlayerCount {
    pub at: DateTime<Utc>,
    pub players: i32,
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct PlayerHour {
    pub hour: DateTime<Utc>,
    pub max: Option<i32>,
}

#[derive(Serialize, ToSchema)]
pub struct SteamPlayers {
    pub current: Option<PlayerCount>,
    pub hourly: Vec<PlayerHour>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SteamNews {
    pub gid: String,
    pub title: String,
    pub url: String,
    pub published_at: DateTime<Utc>,
    pub feed: String,
    pub version: Option<String>,
}

async fn fetch<T: DeserializeOwned>(s: &AppState, path: &str) -> Result<T, String> {
    let bytes = s
        .http
        .get(format!("{}{path}", s.config.steam_api_url))
        .header(header::USER_AGENT, "spam-live")
        .timeout(TIMEOUT)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| e.without_url().to_string())?
        .bytes()
        .await
        .map_err(|e| e.without_url().to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

pub async fn poll_players(s: &AppState, at: DateTime<Utc>) {
    let path = format!("/ISteamUserStats/GetNumberOfCurrentPlayers/v1/?appid={APP_ID}");
    let players = match fetch::<PlayersBody>(s, &path).await {
        Ok(PlayersBody { response: PlayersResult { result: 1, player_count: Some(n) } }) if n >= 0 => n,
        Ok(body) => return tracing::warn!(result = body.response.result, "Steam player count is not valid"),
        Err(e) => return tracing::warn!(error = %e, "Steam player count fetch failed"),
    };
    let insert = sqlx::query("INSERT INTO steam_players (at, players) VALUES ($1, $2) ON CONFLICT (at) DO NOTHING")
        .bind(at)
        .bind(players)
        .execute(&s.db)
        .await;
    if let Err(e) = insert {
        tracing::warn!(error = %e, "Steam player count write failed");
    }
}

pub async fn poll_news(s: &AppState) {
    let path = format!("/ISteamNews/GetNewsForApp/v2/?appid={APP_ID}&count=20");
    let items = match fetch::<NewsBody>(s, &path).await {
        Ok(body) => body.appnews.newsitems,
        Err(e) => return tracing::warn!(error = %e, "Steam news fetch failed"),
    };
    for item in items {
        let Ok(item) = serde_json::from_value::<RawNews>(item) else { continue };
        let Some(published_at) = DateTime::from_timestamp(item.date, 0) else { continue };
        if item.gid.is_empty() || !item.gid.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let feed = item.feedlabel.filter(|f| !f.is_empty()).or(item.feedname).unwrap_or_default();
        let upsert = sqlx::query(
            "INSERT INTO steam_news (gid, title, url, published_at, feed) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (gid) DO UPDATE SET title = EXCLUDED.title, url = EXCLUDED.url,
                 published_at = EXCLUDED.published_at, feed = EXCLUDED.feed",
        )
        .bind(&item.gid)
        .bind(&item.title)
        .bind(&item.url)
        .bind(published_at)
        .bind(&feed)
        .execute(&s.db)
        .await;
        if let Err(e) = upsert {
            tracing::warn!(error = %e, gid = item.gid, "Steam news write failed");
        }
    }
}

fn link(gid: &str, url: &str) -> String {
    let steam = Url::parse(url).is_ok_and(|u| {
        u.scheme() == "https" && matches!(u.host_str(), Some("store.steampowered.com" | "steamcommunity.com"))
    });
    if steam { url.to_string() } else { format!("https://store.steampowered.com/news/app/{APP_ID}/view/{gid}") }
}

fn title_version(catalog: &Catalog, title: &str) -> Option<String> {
    let names: Vec<&str> = catalog.version_names().collect();
    let tokens = || title.split(|c: char| !c.is_ascii_digit() && c != '.').map(|t| t.trim_matches('.'));
    let exact = tokens().find(|t| names.contains(t));
    let prefix = || {
        tokens().filter(|t| t.contains('.')).find_map(|t| {
            let mut found = names.iter().filter(|n| n.strip_prefix(t).is_some_and(|rest| rest.starts_with('.')));
            match (found.next(), found.next()) {
                (Some(n), None) => Some(*n),
                _ => None,
            }
        })
    };
    exact.or_else(prefix).map(str::to_string)
}

#[utoipa::path(get, path = "/api/steam/players", tag = "steam",
    responses((status = 200, body = SteamPlayers, description = "The Steam player count of the game. current: the last \
        count of the last 30 minutes, else null. hourly: 1 item per UTC hour of the last 7 days, oldest first, the \
        current hour last, with the highest count of the hour or null.")))]
pub async fn players(State(s): State<AppState>) -> Result<Response, ApiError> {
    let current = sqlx::query_as(
        "SELECT at, players FROM steam_players WHERE at > now() - interval '30 minutes' ORDER BY at DESC LIMIT 1",
    )
    .fetch_optional(&s.db)
    .await?;
    let hourly = sqlx::query_as(
        "WITH h AS (
             SELECT generate_series(date_trunc('hour', now(), 'UTC') - interval '167 hours',
                 date_trunc('hour', now(), 'UTC'), interval '1 hour') AS hour
         )
         SELECT h.hour, max(p.players) AS max
         FROM h LEFT JOIN steam_players p ON p.at >= h.hour AND p.at < h.hour + interval '1 hour'
         GROUP BY h.hour ORDER BY h.hour",
    )
    .fetch_all(&s.db)
    .await?;
    Ok(public(SteamPlayers { current, hourly }))
}

#[utoipa::path(get, path = "/api/steam/news", tag = "steam",
    responses((status = 200, body = Vec<SteamNews>, description = "The Steam news of the game, newest first, max 50. \
        url is an https Steam page. version: a catalog version that is a token of the title; else the only catalog \
        version that starts with a dotted token of the title and a dot (1.28 gives 1.28.1); else null.")))]
pub async fn news(State(s): State<AppState>) -> Result<Response, ApiError> {
    let rows: Vec<(String, String, String, DateTime<Utc>, String)> = sqlx::query_as(
        "SELECT gid, title, url, published_at, feed FROM steam_news ORDER BY published_at DESC, gid DESC LIMIT $1",
    )
    .bind(NEWS_LIMIT)
    .fetch_all(&s.db)
    .await?;
    let items: Vec<_> = rows
        .into_iter()
        .map(|(gid, title, url, published_at, feed)| SteamNews {
            url: link(&gid, &url),
            version: title_version(&s.catalog, &title),
            gid,
            title,
            published_at,
            feed,
        })
        .collect();
    Ok(public(items))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/steam/players", get(players)).route("/api/steam/news", get(news))
}
