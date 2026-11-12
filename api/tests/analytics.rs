mod common;

use std::time::{Duration, Instant};

use api::analytics::{HourValues, Hourly, MAX_LIVE, Presence, WEB_TTL, tick, utc_hour, write_hour};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use chrono::{DateTime, TimeDelta, Utc};
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

const TAB: &str = "6f1c2a4e-8b3d-4c5e-9f7a-1b2c3d4e5f60";

fn v4(n: u128) -> Uuid {
    uuid::Builder::from_random_bytes(n.to_le_bytes()).into_uuid()
}

fn ping(id: &str) -> Request<Body> {
    post_json("/api/presence", "", json!({ "id": id }))
}

fn app_ping(token: Option<&str>, body: Value) -> Request<Body> {
    let mut req = post_json("/api/desktop/presence", "", body);
    req.headers_mut().remove(header::COOKIE);
    if let Some(token) = token {
        req.headers_mut().insert(header::AUTHORIZATION, format!("Bearer {token}").parse().unwrap());
    }
    req
}

async fn device(pool: &PgPool, user_id: i64) -> String {
    let token = api::auth::random_token();
    sqlx::query("INSERT INTO devices (user_id, token_hash) VALUES ($1, $2)")
        .bind(user_id)
        .bind(api::auth::hash(&token))
        .execute(pool)
        .await
        .unwrap();
    token
}

fn cards(ids: [&str; 4]) -> Vec<Uuid> {
    ids.iter().map(|c| Uuid::parse_str(c).unwrap()).collect()
}

async fn insert_match(pool: &PgPool, n: u32, mode: i32, ai: bool, disputed: bool, created_at: DateTime<Utc>) {
    sqlx::query(
        "INSERT INTO matches (game_uid, app_version, game_mode, is_ai, start_utc, end_utc, duration_ms, turn_count,
             seat0_cards, seat1_cards, winner_seat, is_draw, disputed, created_at)
         VALUES ($1, '1.28.1', $2, $3, now() - interval '1 hour', now() - interval '50 minutes', 600000, 10,
             $4, $5, 0, false, $6, $7)",
    )
    .bind(format!("match-{n}"))
    .bind(mode)
    .bind(ai)
    .bind(cards(SEAT0_CARDS))
    .bind(cards(SEAT1_CARDS))
    .bind(disputed)
    .bind(created_at)
    .execute(pool)
    .await
    .unwrap();
}

async fn analytics(app: &Router, cookie: &str, range: &str) -> Value {
    let res = send(app, get(&format!("/api/admin/analytics{range}"), cookie)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.body);
    res.body
}

#[sqlx::test]
async fn presence_counts_web_tabs(pool: PgPool) {
    let app = app(pool.clone());
    for id in ["x", "", "6f1c2a4e-8b3d-1c5e-9f7a-1b2c3d4e5f60", "019a5e3c-8b3d-7c5e-9f7a-1b2c3d4e5f60"] {
        assert_eq!(send(&app, ping(id)).await.status, StatusCode::BAD_REQUEST, "{id}");
    }
    insert_match(&pool, 1, 10, false, false, Utc::now()).await;
    insert_match(&pool, 2, 40, true, false, Utc::now()).await;

    let res = send(&app, ping(TAB)).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers[header::CACHE_CONTROL], "no-store");
    assert!(res.body["online"].as_i64().unwrap() >= 1);
    assert_eq!(res.body["matches"], count(&pool, "matches").await);
    assert_eq!(send(&app, ping(TAB)).await.body["online"], 1);
    assert_eq!(send(&app, ping(&v4(1).to_string())).await.body["online"], 2);
}

#[test]
fn presence_expires_and_caps_web_ids() {
    let presence = Presence::default();
    let t0 = Instant::now();
    let s = |secs| t0 + Duration::from_secs(secs);
    let (a, b) = (v4(1), v4(2));
    assert_eq!(presence.ping(a, s(0)), 1);
    assert_eq!(presence.ping(a, s(10)), 1);
    assert_eq!(presence.ping(b, s(60)), 2);
    assert_eq!(presence.web_online(s(99)), 2);
    assert_eq!(presence.web_online(s(10) + WEB_TTL), 1);
    assert_eq!(presence.web_online(s(60) + WEB_TTL), 0);

    let known = v4(3);
    presence.ping(known, s(200));
    for n in 1..MAX_LIVE as u128 {
        presence.ping(v4(100 + n), s(200));
    }
    assert_eq!(presence.ping(v4(4), s(201)), MAX_LIVE);
    assert_eq!(presence.ping(known, s(250)), MAX_LIVE);
    assert_eq!(presence.web_online(s(200) + WEB_TTL), 1);

    let hour = presence.take_hour();
    assert_eq!(hour.web_distinct as usize, MAX_LIVE + 3);
    assert_eq!(presence.take_hour(), HourValues::default());
}

#[sqlx::test]
async fn desktop_presence(pool: PgPool) {
    let app = app(pool.clone());
    let (_, admin) = admin(&pool, "76561190000000009").await;
    let (id, cookie) = user(&pool, "76561190000000001").await;
    let token = device(&pool, id).await;
    let app_online = async |app: &Router| analytics(app, &admin, "").await["now"]["appOnline"].clone();
    let listed = async |app: &Router| send(app, get("/api/me/devices", &cookie)).await.body[0]["online"].clone();
    assert_eq!(listed(&app).await, false);

    let res = send(&app, app_ping(Some(&token), json!({ "event": "open", "version": "0.1.0" }))).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers[header::CACHE_CONTROL], "no-store");
    assert_eq!(res.body, json!({ "minVersion": "1.2.3" }));
    let (version, online, used): (Option<String>, bool, bool) = sqlx::query_as(
        "SELECT app_version, online_until > now() + interval '14 minutes', last_used_at IS NOT NULL FROM devices",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((version.as_deref(), online, used), (Some("0.1.0"), true, true));
    assert_eq!(app_online(&app).await, 1);
    assert_eq!(listed(&app).await, true);

    let res = send(&app, app_ping(Some(&token), json!({ "event": "close", "version": "0.1.1" }))).await;
    assert_eq!(res.status, StatusCode::OK);
    let (version, until): (Option<String>, Option<DateTime<Utc>>) =
        sqlx::query_as("SELECT app_version, online_until FROM devices").fetch_one(&pool).await.unwrap();
    assert_eq!((version.as_deref(), until), (Some("0.1.1"), None));
    assert_eq!(app_online(&app).await, 0);
    assert_eq!(listed(&app).await, false);

    send(&app, app_ping(Some(&token), json!({ "event": "check", "version": "0.1.1" }))).await;
    assert_eq!(app_online(&app).await, 1);
    assert_eq!(listed(&app).await, true);
    sqlx::query("UPDATE devices SET online_until = now() - interval '1 second'").execute(&pool).await.unwrap();
    assert_eq!(app_online(&app).await, 0);
    assert_eq!(listed(&app).await, false);

    let mut with_cookie = post_json("/api/desktop/presence", &cookie, json!({ "event": "open", "version": "0.1.0" }));
    with_cookie.headers_mut().insert(header::ORIGIN, PUBLIC_URL.parse().unwrap());
    let res = send(&app, with_cookie).await;
    assert_eq!((res.status, res.body["error"].as_str()), (StatusCode::FORBIDDEN, Some("device token required")));
    let body = json!({ "event": "open", "version": "0.1.0" });
    assert_eq!(send(&app, app_ping(None, body.clone())).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(send(&app, app_ping(Some("bad"), body)).await.status, StatusCode::UNAUTHORIZED);
    for version in ["", "1.x", "next", "1..2"] {
        let res = send(&app, app_ping(Some(&token), json!({ "event": "open", "version": version }))).await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST, "{version}");
    }
    let version: Option<String> = sqlx::query_scalar("SELECT app_version FROM devices").fetch_one(&pool).await.unwrap();
    assert_eq!(version.as_deref(), Some("0.1.1"));
}

#[sqlx::test]
async fn upload_source(pool: PgPool) {
    let app = app(pool.clone());
    let (a_id, a) = user(&pool, "76561190000000001").await;
    let (b_id, _) = user(&pool, "76561190000000002").await;
    let token = device(&pool, b_id).await;
    let res = upload(&app, &a, &[entry(1, 0, ACCOUNT_A, ACCOUNT_B, Some(ACCOUNT_A))], &[]).await;
    assert_eq!(statuses(&res.body, "entries"), ["new"]);
    let index = serde_json::to_vec(&[entry(1, 1, ACCOUNT_B, ACCOUNT_A, Some(ACCOUNT_A))]).unwrap();
    let mut req = upload_req("", &index, &[]);
    req.headers_mut().remove(header::COOKIE);
    req.headers_mut().insert(header::AUTHORIZATION, format!("Bearer {token}").parse().unwrap());
    assert_eq!(statuses(&send(&app, req).await.body, "entries"), ["new"]);
    let sources: Vec<(i64, Option<String>)> =
        sqlx::query_as("SELECT user_id, source FROM uploads ORDER BY user_id").fetch_all(&pool).await.unwrap();
    assert_eq!(sources, [(a_id, Some("web".into())), (b_id, Some("desktop".into()))]);
}

#[sqlx::test]
async fn hourly_write(pool: PgPool) {
    let app = app(pool.clone());
    let (id, cookie) = user(&pool, "76561190000000001").await;
    user(&pool, "76561190000000002").await;
    device(&pool, id).await;
    upload(&app, &cookie, &[entry(1, 0, ACCOUNT_A, ACCOUNT_B, Some(ACCOUNT_A))], &[]).await;
    insert_match(&pool, 2, 10, true, false, Utc::now()).await;
    let deck = json!({ "name": "D", "cards": SEAT0_CARDS, "talents": [0, 0, 0, 0] });
    let deck = send(&app, req("POST", "/api/decks", &cookie, Some(deck))).await.body["id"].as_i64().unwrap();
    send(&app, req("POST", &format!("/api/decks/{deck}/publish"), &cookie, None)).await;

    let hour = utc_hour(Utc::now()) - TimeDelta::hours(5);
    let values = HourValues { web_peak: 7, app_peak: 2, web_distinct: 30, app_distinct: 3 };
    assert!(write_hour(&pool, hour, values).await.unwrap());
    insert_match(&pool, 3, 10, false, false, Utc::now()).await;
    assert!(!write_hour(&pool, hour, HourValues { web_peak: 99, ..values }).await.unwrap());

    let row: (DateTime<Utc>, Vec<i32>, Vec<i64>) = sqlx::query_as(
        "SELECT hour, ARRAY[web_peak, app_peak, web_distinct, app_distinct],
             ARRAY[matches, ranked_matches, users, linked_accounts, public_decks, devices] FROM analytics_hourly",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row, (hour, vec![7, 2, 30, 3], vec![2, 1, 2, 1, 1, 1]));
}

#[sqlx::test]
async fn tick_retries_a_failed_write(pool: PgPool) {
    let state = state_with(pool.clone(), "https://steamcommunity.com/openid/login", real_catalog());
    let finished = utc_hour(Utc::now()) - TimeDelta::hours(3);
    let mut hourly = Hourly::new(finished);
    state.presence.ping(v4(1), Instant::now());
    tick(&state, &mut hourly, finished + TimeDelta::minutes(30)).await.unwrap();

    sqlx::query("ALTER TABLE analytics_hourly RENAME TO analytics_off").execute(&pool).await.unwrap();
    assert!(tick(&state, &mut hourly, finished + TimeDelta::minutes(61)).await.is_err());
    assert_eq!(hourly.pending.len(), 1);
    state.presence.ping(v4(2), Instant::now());
    state.presence.ping(v4(3), Instant::now());
    sqlx::query("ALTER TABLE analytics_off RENAME TO analytics_hourly").execute(&pool).await.unwrap();
    tick(&state, &mut hourly, finished + TimeDelta::minutes(62)).await.unwrap();
    assert!(hourly.pending.is_empty());

    let rows: Vec<(DateTime<Utc>, i32, i32)> =
        sqlx::query_as("SELECT hour, web_peak, web_distinct FROM analytics_hourly").fetch_all(&pool).await.unwrap();
    assert_eq!(rows, [(finished, 1, 1)]);
    assert_eq!(state.presence.take_hour().web_distinct, 2);
}

#[sqlx::test]
async fn admin_analytics(pool: PgPool) {
    let app = app(pool.clone());
    let (_, admin) = admin(&pool, "76561190000000009").await;
    let (a_id, a) = user(&pool, "76561190000000001").await;
    let (b_id, b) = user(&pool, "76561190000000002").await;
    let (c_id, _) = user(&pool, "76561190000000003").await;

    for cookie in [&a, ""] {
        let res = send(&app, get("/api/admin/analytics", cookie)).await;
        let other = send(&app, get("/api/admin/reports", cookie)).await;
        assert_eq!((res.status, &res.body), (other.status, &other.body));
    }
    assert_eq!(send(&app, get("/api/admin/analytics", &a)).await.status, StatusCode::FORBIDDEN);
    assert_eq!(send(&app, get("/api/admin/analytics?range=1y", &admin)).await.status, StatusCode::BAD_REQUEST);

    upload(&app, &a, &[entry(1, 0, ACCOUNT_A, ACCOUNT_B, Some(ACCOUNT_A))], &[]).await;
    upload(&app, &b, &[entry(1, 1, ACCOUNT_B, ACCOUNT_A, Some(ACCOUNT_A))], &[]).await;
    sqlx::query("UPDATE uploads SET source = NULL WHERE user_id = $1").bind(b_id).execute(&pool).await.unwrap();
    let now = utc_hour(Utc::now());
    insert_match(&pool, 2, 10, true, false, now - TimeDelta::hours(2)).await;
    insert_match(&pool, 3, 10, false, true, now - TimeDelta::hours(2) + TimeDelta::minutes(59)).await;
    insert_match(&pool, 4, 40, false, false, now - TimeDelta::days(3)).await;
    insert_match(&pool, 5, 10, false, false, now - TimeDelta::days(10)).await;
    sqlx::query("UPDATE matches SET created_at = $1 WHERE game_uid = 'match-1'")
        .bind(now - TimeDelta::hours(23))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE users SET banned_at = now(), created_at = now() - interval '8 days' WHERE id = $1")
        .bind(c_id)
        .execute(&pool)
        .await
        .unwrap();

    let deck = |name: &str| json!({ "name": name, "cards": SEAT0_CARDS, "talents": [0, 0, 0, 0] });
    let public = send(&app, req("POST", "/api/decks", &a, Some(deck("P")))).await.body["id"].as_i64().unwrap();
    send(&app, req("POST", &format!("/api/decks/{public}/publish"), &a, None)).await;
    send(&app, req("POST", "/api/decks", &a, Some(deck("D1")))).await;
    send(&app, req("POST", "/api/decks", &b, Some(deck("D2")))).await;
    send(&app, req("PUT", &format!("/api/decks/{public}/vote"), &b, Some(json!({ "value": 1 })))).await;
    send(&app, req("POST", &format!("/api/decks/{public}/comments"), &b, Some(json!({ "body": "gg" })))).await;

    for (user, version, hours, online) in [
        (a_id, Some("0.2.0"), 1, true),
        (a_id, Some("0.10.0"), 30, false),
        (b_id, Some("0.2.0"), 2, false),
        (b_id, None, 3, false),
        (c_id, Some("0.9.0"), 24 * 8, false),
    ] {
        let token = device(&pool, user).await;
        sqlx::query(
            "UPDATE devices SET app_version = $2, last_used_at = now() - make_interval(hours => $3),
                 online_until = CASE WHEN $4 THEN now() + interval '5 minutes' END
             WHERE token_hash = $1",
        )
        .bind(api::auth::hash(&token))
        .bind(version)
        .bind(hours)
        .bind(online)
        .execute(&pool)
        .await
        .unwrap();
    }
    let values = HourValues { web_peak: 4, app_peak: 1, web_distinct: 9, app_distinct: 2 };
    write_hour(&pool, now - TimeDelta::hours(2), values).await.unwrap();
    write_hour(&pool, now - TimeDelta::days(20), values).await.unwrap();

    let body = analytics(&app, &admin, "").await;
    assert_eq!(body["now"], json!({ "webOnline": 0, "appOnline": 1 }));
    assert_eq!(body["matches"], json!({ "total": 5, "ranked": 2, "ai": 1, "disputed": 1, "last24h": 3, "last7d": 4 }));
    assert_eq!(body["players"], json!({ "accounts": 4, "linkedAccounts": 2, "banned": 1, "signups7d": 3 }));
    assert_eq!(body["community"], json!({ "publicDecks": 1, "draftDecks": 2, "comments": 1, "votes": 1 }));
    assert_eq!(
        body["desktop"],
        json!({ "devices": 5, "seen24h": 3, "versions": [
            { "version": "0.10.0", "devices": 1 },
            { "version": "0.2.0", "devices": 2 },
            { "version": null, "devices": 1 },
        ] })
    );
    assert_eq!(body["uploads"], json!({ "web": 1, "desktop": 0, "unknown": 1 }));
    assert_eq!(body["hours"].as_array().unwrap().len(), 24 * 7);

    let hours = analytics(&app, &admin, "?range=24h").await["hours"].as_array().unwrap().clone();
    assert_eq!(hours.len(), 24);
    let at = |i: usize| hours[i]["hour"].as_str().unwrap().parse::<DateTime<Utc>>().unwrap();
    assert_eq!(at(23), now);
    assert_eq!(at(0), now - TimeDelta::hours(23));
    assert_eq!(hours[21]["webPeak"], 4);
    assert_eq!(hours[21]["matches"], 5);
    assert_eq!(hours[21]["matchesAdded"], 2);
    assert_eq!(hours[0]["matchesAdded"], 1);
    for (i, h) in hours.iter().enumerate().filter(|(i, _)| *i != 21) {
        assert_eq!(h["webPeak"], Value::Null, "{i}");
        assert_eq!(h["devices"], Value::Null, "{i}");
    }
    assert_eq!(hours.iter().map(|h| h["matchesAdded"].as_i64().unwrap()).sum::<i64>(), 3);
    assert_eq!(analytics(&app, &admin, "?range=30d").await["hours"].as_array().unwrap().len(), 24 * 30);
}
