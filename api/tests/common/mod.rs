#![allow(dead_code)]

use std::{io::Write, path::Path};

use api::{AppState, Config, catalog::Catalog};
use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Request, StatusCode, header},
};
use flate2::{Compression, write::GzEncoder};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

pub const PUBLIC_URL: &str = "http://localhost:5173";
pub const ACCOUNT_A: &str = "00000000-0000-4000-8000-00000000000a";
pub const ACCOUNT_B: &str = "00000000-0000-4000-8000-00000000000b";
pub const SEAT0_CARDS: [&str; 4] = [
    "0c8c383b-2207-4d18-b378-4f2a3795a58e",
    "f16bc1f8-5cea-4acf-9e2a-0614062cb9b8",
    "9e9dfca9-0e53-496f-b471-f7c1ae152b80",
    "4b2ece95-feb4-410c-a53d-9a1e3b0a7a24",
];
pub const SEAT1_CARDS: [&str; 4] = [
    "75491473-57fa-4b8f-aced-9d61d0404e65",
    "869a57eb-3787-423b-b244-df864406c065",
    "d2e2e893-6688-4a1a-9d92-1d239e8d0a75",
    "4dc96cb5-d342-4ef6-8808-fcc79c63690b",
];

pub fn config(openid_url: &str) -> Config {
    Config {
        public_url: PUBLIC_URL.into(),
        steam_api_key: None,
        steam_openid_url: openid_url.into(),
        desktop_min_version: "1.2.3".into(),
        max_pending_device_codes: 1000,
    }
}

pub fn state_with(pool: PgPool, openid_url: &str, catalog: Catalog) -> AppState {
    AppState::new(pool, config(openid_url), catalog)
}

pub fn real_catalog() -> Catalog {
    Catalog::load(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../catalog"))).unwrap()
}

pub fn app_with(pool: PgPool, openid_url: &str) -> Router {
    api::router(state_with(pool, openid_url, real_catalog()))
}

pub fn app(pool: PgPool) -> Router {
    app_with(pool, "https://steamcommunity.com/openid/login")
}

pub async fn user(pool: &PgPool, steam_id: &str) -> (i64, String) {
    let id: i64 = sqlx::query_scalar("INSERT INTO users (steam_id, display_name) VALUES ($1, $1) RETURNING id")
        .bind(steam_id)
        .fetch_one(pool)
        .await
        .unwrap();
    (id, new_session(pool, id).await)
}

pub struct Res {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Value,
}

pub async fn send(app: &Router, req: Request<Body>) -> Res {
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let headers = res.headers().clone();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    Res { status, headers, body }
}

pub fn get(uri: &str, cookie: &str) -> Request<Body> {
    Request::get(uri).header(header::COOKIE, cookie).body(Body::empty()).unwrap()
}

pub fn post_json(uri: &str, cookie: &str, body: Value) -> Request<Body> {
    Request::post(uri)
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

pub fn req(method: &str, uri: &str, cookie: &str, body: Option<Value>) -> Request<Body> {
    let builder = Request::builder().method(method).uri(uri).header(header::COOKIE, cookie);
    match body {
        Some(body) => {
            builder.header(header::CONTENT_TYPE, "application/json").body(Body::from(body.to_string())).unwrap()
        }
        None => builder.body(Body::empty()).unwrap(),
    }
}

pub fn text_req(uri: &str, cookie: &str, body: String) -> Request<Body> {
    Request::post(uri)
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(body))
        .unwrap()
}

pub async fn admin(pool: &PgPool, steam_id: &str) -> (i64, String) {
    let (id, cookie) = user(pool, steam_id).await;
    sqlx::query("UPDATE users SET is_admin = true WHERE id = $1").bind(id).execute(pool).await.unwrap();
    (id, cookie)
}

pub async fn new_session(pool: &PgPool, user_id: i64) -> String {
    let token = api::auth::random_token();
    sqlx::query("INSERT INTO sessions (token_hash, user_id, expires_at) VALUES ($1, $2, now() + interval '1 day')")
        .bind(api::auth::hash(&token))
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
    format!("session={token}")
}

pub fn entry(n: u64, seat: usize, me: &str, opponent: &str, winner: Option<&str>) -> Value {
    let (mine, theirs) = if seat == 0 { (SEAT0_CARDS, SEAT1_CARDS) } else { (SEAT1_CARDS, SEAT0_CARDS) };
    json!({
        "FormatVersion": 2,
        "AppVersion": "1.28.1",
        "Capabilities": 1023,
        "GameUid": format!("match-{n}"),
        "AccountId": me,
        "Seat": seat,
        "GameMode": 10,
        "Level": "",
        "OpponentId": opponent,
        "OpponentName": "FakeOpponent",
        "OpponentIsAi": false,
        "WinnerId": winner,
        "IsDraw": winner.is_none(),
        "MyCards": mine,
        "OpponentCards": theirs,
        "StartUtc": "2026-01-01T10:00:00.1234567Z",
        "EndUtc": "2026-01-01T10:10:00.1234567Z",
        "DurationMs": 600_004,
        "MessageCount": 100,
        "TurnCount": 12,
        "Loading": {
            "matchId": format!("match-{n}"),
            "ticket": "fake-ticket",
            "opponent": { "username": "FakeOpponent", "stars": 20, "rank": 1 },
            "me": { "username": "FakeMe", "stars": 20 + seat, "rank": 1 }
        }
    })
}

pub fn gzip(data: &[u8]) -> Vec<u8> {
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(data).unwrap();
    enc.finish().unwrap()
}

pub fn srpl_raw(header: &Value, padding: usize) -> Vec<u8> {
    let header = serde_json::to_vec(header).unwrap();
    let mut raw = b"SRPL".to_vec();
    raw.extend(2u32.to_le_bytes());
    let mut n = header.len();
    loop {
        let b = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            raw.push(b);
            break;
        }
        raw.push(b | 0x80);
    }
    raw.extend(header);
    raw.resize(raw.len() + padding, 0);
    raw
}

pub fn srpl(header: &Value) -> Vec<u8> {
    gzip(&srpl_raw(header, 256))
}

pub fn upload_req(cookie: &str, index: &[u8], replays: &[(String, Vec<u8>)]) -> Request<Body> {
    let boundary = "SpamTestBoundary7MA4YWxkTrZu0gW";
    let mut body = Vec::new();
    let mut part = |name: &str, file: &str, data: &[u8]| {
        write!(
            body,
            "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"{file}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
        )
        .unwrap();
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n");
    };
    part("index", "index.json", index);
    for (file, data) in replays {
        part("replay", file, data);
    }
    write!(body, "--{boundary}--\r\n").unwrap();
    Request::post("/api/matches")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .header(header::COOKIE, cookie)
        .body(Body::from(body))
        .unwrap()
}

pub async fn upload(app: &Router, cookie: &str, entries: &[Value], replays: &[(String, Vec<u8>)]) -> Res {
    let index = serde_json::to_vec(entries).unwrap();
    send(app, upload_req(cookie, &index, replays)).await
}

pub fn statuses(body: &Value, key: &str) -> Vec<String> {
    body[key].as_array().unwrap().iter().map(|e| e["status"].as_str().unwrap().to_string()).collect()
}

pub fn reason(body: &Value, key: &str, i: usize) -> String {
    body[key][i]["reason"].as_str().unwrap_or_default().to_string()
}

pub async fn history(app: &Router, cookie: &str) -> Vec<Value> {
    let res = send(app, get("/api/me/matches", cookie)).await;
    assert_eq!(res.status, StatusCode::OK);
    res.body.as_array().unwrap().clone()
}

pub async fn count(pool: &PgPool, table: &str) -> i64 {
    let sql = match table {
        "matches" => "SELECT count(*) FROM matches",
        "uploads" => "SELECT count(*) FROM uploads",
        "replays" => "SELECT count(*) FROM replays",
        "users" => "SELECT count(*) FROM users",
        "sessions" => "SELECT count(*) FROM sessions",
        "devices" => "SELECT count(*) FROM devices",
        "decks" => "SELECT count(*) FROM decks",
        "votes" => "SELECT count(*) FROM votes",
        "comments" => "SELECT count(*) FROM comments",
        "reports" => "SELECT count(*) FROM reports",
        _ => unreachable!(),
    };
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}
