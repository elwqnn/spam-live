mod common;

use axum::{Router, http::StatusCode};
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;

const OPP: &str = "00000000-0000-4000-8000-0000000000ff";

fn play(n: u64, cards: [&str; 4], winner: Option<&str>, day: u32) -> Value {
    let mut e = entry(n, 0, ACCOUNT_A, OPP, winner);
    e["MyCards"] = json!(cards);
    e["StartUtc"] = json!(format!("2026-01-{day:02}T10:00:00Z"));
    e["EndUtc"] = json!(format!("2026-01-{day:02}T10:10:00Z"));
    e["DurationMs"] = json!(600_000);
    e
}

fn key(cards: [&str; 4]) -> Value {
    let mut cards = cards.to_vec();
    cards.sort();
    json!(cards)
}

async fn create(app: &Router, cookie: &str, name: &str, cards: [&str; 4]) -> i64 {
    let body = json!({ "name": name, "cards": cards, "talents": [0, 1, 2, 0], "guide": "Play it slow." });
    let res = send(app, req("POST", "/api/decks", cookie, Some(body))).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.body);
    res.body["id"].as_i64().unwrap()
}

#[sqlx::test]
async fn top_decks_need_login(pool: PgPool) {
    let app = app(pool);
    assert_eq!(send(&app, get("/api/me/top-decks", "")).await.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn top_decks(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let (_, b) = user(&pool, "76561190000000002").await;
    let [s0, s1] = [SEAT0_CARDS, SEAT1_CARDS];
    let x = s0;
    let y = s1;
    let z = [s0[0], s0[1], s1[0], s1[1]];
    let w = [s0[2], s0[3], s1[2], s1[3]];
    let reversed = [x[3], x[2], x[1], x[0]];

    let mut ai = play(4, x, Some(ACCOUNT_A), 8);
    ai["OpponentIsAi"] = json!(true);
    let entries = [
        play(1, x, Some(ACCOUNT_A), 1),
        play(2, x, Some(ACCOUNT_A), 2),
        play(3, reversed, Some(OPP), 3),
        ai,
        play(5, x, Some(ACCOUNT_A), 9),
        play(6, y, Some(ACCOUNT_A), 4),
        play(7, y, None, 5),
        play(8, z, Some(OPP), 6),
        play(9, w, Some(ACCOUNT_A), 7),
    ];
    let res = upload(&app, &a, &entries, &[]).await;
    assert_eq!(statuses(&res.body, "entries"), ["new"; 9]);
    sqlx::query("UPDATE matches SET disputed = true WHERE game_uid = 'match-5'").execute(&pool).await.unwrap();

    let old = create(&app, &a, "Old", x).await;
    let saved = create(&app, &a, "Mine", reversed).await;
    sqlx::query("UPDATE decks SET updated_at = now() - interval '1 day' WHERE id = $1")
        .bind(old)
        .execute(&pool)
        .await
        .unwrap();
    let public = create(&app, &a, "Public", w).await;
    let res = send(&app, req("POST", &format!("/api/decks/{public}/publish"), &a, None)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.body);
    create(&app, &b, "Other", y).await;

    let res = send(&app, get("/api/me/top-decks", &a)).await;
    assert_eq!(res.status, StatusCode::OK);
    let top = res.body.as_array().unwrap();
    let keys: Vec<_> = top.iter().map(|t| t["cards"].clone()).collect();
    assert_eq!(keys, [key(x), key(y), key(w)]);
    assert_eq!((top[0]["games"].as_i64(), top[0]["wins"].as_i64()), (Some(3), Some(2)));
    assert!((top[0]["winRate"].as_f64().unwrap() - 2.0 / 3.0).abs() < 1e-9);
    assert_eq!((top[1]["games"].as_i64(), top[1]["wins"].as_i64()), (Some(2), Some(1)));
    assert_eq!(top[1]["winRate"], 0.5);
    assert_eq!(top[2]["winRate"], 1.0);
    assert_eq!(
        top[0]["deck"],
        json!({ "id": saved, "name": "Mine", "status": "draft", "cards": reversed,
            "talents": [0, 1, 2, 0], "guide": "Play it slow." })
    );
    assert_eq!(top[1]["deck"], Value::Null);
    assert_eq!(top[2]["deck"]["id"], public);
    assert_eq!(top[2]["deck"]["status"], "public");

    let res = send(&app, get("/api/me/top-decks", &b)).await;
    assert_eq!(res.body, json!([]));
}
