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
pub const DEV_ORIGIN: &str = "http://localhost:1420";
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
        steam_api_url: "http://127.0.0.1:9".into(),
        github_release_url: "http://127.0.0.1:9/releases/latest".into(),
        github_api_url: "http://127.0.0.1:9".into(),
        github_feedback_repo: "elwqnn/spam-live".into(),
        github_feedback_token: None,
        desktop_min_version: "1.2.3".into(),
        max_pending_device_codes: 1000,
        cors_origins: vec![DEV_ORIGIN.into()],
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

pub const TOX: [&str; 2] = ["0004a1e3-755d-4fe7-8890-316214c67fcd", "04dedce0-2c15-4302-807b-6935254e92f3"];

fn uv(mut n: u64) -> Vec<u8> {
    let mut out = Vec::new();
    while n >= 0x80 {
        out.push((n & 0x7f) as u8 | 0x80);
        n >>= 7;
    }
    out.push(n as u8);
    out
}

fn zz(v: i64) -> Vec<u8> {
    uv(((v << 1) ^ (v >> 63)) as u64)
}

fn str8(s: &str) -> Vec<u8> {
    [uv(2 * (s.len() as u64 + 1)), s.as_bytes().to_vec()].concat()
}

fn str16(s: &str) -> Vec<u8> {
    let units: Vec<u8> = s.encode_utf16().flat_map(u16::to_le_bytes).collect();
    [(units.len() as u32 / 2).to_le_bytes().to_vec(), units].concat()
}

pub struct CardState<'a> {
    pub card: &'a str,
    pub uid: &'a str,
    pub player: i64,
    pub talent: i64,
    pub damage: i64,
    pub hp: i64,
    pub done: i64,
    pub taken: i64,
}

pub fn card_block(c: &CardState) -> Vec<u8> {
    let mut b = vec![0xC5, 1];
    b.extend(str8(c.card));
    b.extend(str8(c.uid));
    b.extend(zz(c.player));
    b.extend(str8("cardstyle-standard"));
    b.extend(zz(c.talent));
    b.extend([zz(0), zz(0), zz(0)].concat());
    b.push(0);
    for v in [c.damage, c.hp, 0, c.done, 0, 0, 0, c.done, c.taken] {
        b.extend(zz(v));
    }
    b.extend([0; 14]);
    let len = (b.len() as u32).to_le_bytes();
    [len.to_vec(), len.to_vec(), b].concat()
}

pub fn message(kind: u16, body: &[u8]) -> Vec<u8> {
    [kind.to_le_bytes().as_slice(), body].concat()
}

pub fn new_turn(player: i32, turn: i32) -> Vec<u8> {
    message(2003, &[player.to_le_bytes(), 30f32.to_le_bytes(), turn.to_le_bytes()].concat())
}

pub fn shop_buy(player: i32, uid: &str, tox: &str) -> Vec<u8> {
    message(2020, &[player.to_le_bytes().to_vec(), str16(uid), str16(tox), str16(tox), str16("")].concat())
}

pub fn states(blocks: &[CardState]) -> Vec<u8> {
    message(2207, &[vec![blocks.len() as u8], blocks.iter().flat_map(card_block).collect()].concat())
}

pub fn srpl_messages(header: &Value, messages: &[Vec<u8>]) -> Vec<u8> {
    let mut header = header.clone();
    header["MessageCount"] = json!(messages.len());
    let mut raw = srpl_raw(&header, 0);
    raw.extend((messages.len() as u32).to_le_bytes());
    for (i, payload) in messages.iter().enumerate() {
        raw.extend((i as u32 * 250).to_le_bytes());
        raw.extend((payload.len() as u32).to_le_bytes());
        raw.extend(payload);
    }
    raw
}

pub fn match_messages(seat: usize, buys: bool) -> Vec<Vec<u8>> {
    let decks = [SEAT0_CARDS, SEAT1_CARDS];
    let uids: Vec<Vec<String>> = (0..2).map(|s| (0..4).map(|i| format!("uid-{s}-{i}")).collect()).collect();
    let all = |damage: [i64; 2]| {
        let blocks: Vec<CardState> = (0..2)
            .flat_map(|s| {
                let uids = &uids;
                (0..4).map(move |i| CardState {
                    card: decks[s][i],
                    uid: &uids[s][i],
                    player: s as i64,
                    talent: if s == seat { i as i64 % 3 } else { 0 },
                    damage: damage[s],
                    hp: 1000,
                    done: damage[1 - s] / 2,
                    taken: damage[s],
                })
            })
            .collect();
        states(&blocks)
    };
    let mut out = vec![all([0, 0]), new_turn(1, 1), new_turn(0, 1)];
    if buys {
        out.extend([shop_buy(0, "uid-0-0", TOX[0]), shop_buy(1, "uid-1-1", TOX[1])]);
    }
    out.extend([all([200, 100]), new_turn(1, 2), new_turn(0, 2)]);
    if buys {
        out.extend([shop_buy(0, "uid-0-0", TOX[1]), shop_buy(0, "uid-unknown", TOX[0])]);
    }
    out.push(all([300, 1000]));
    out
}

pub fn match_srpl(header: &Value) -> Vec<u8> {
    let seat = header["Seat"].as_u64().unwrap() as usize;
    gzip(&srpl_messages(header, &match_messages(seat, true)))
}
