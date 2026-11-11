mod common;

use axum::Router;
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;

fn replay(e: &Value) -> (String, Vec<u8>) {
    (format!("{}.srpl", e["GameUid"].as_str().unwrap()), srpl(e))
}

fn with_stars(mut e: Value, stars: i32) -> Value {
    e["Loading"]["me"]["stars"] = json!(stars);
    e
}

async fn rows(pool: &PgPool) -> [i64; 3] {
    [count(pool, "matches").await, count(pool, "uploads").await, count(pool, "replays").await]
}

async fn stats(app: &Router, query: &str) -> (Value, i64) {
    let overview = send(app, get(&format!("/api/stats/overview?version=1.28.1{query}"), "")).await.body;
    let cards = send(app, get(&format!("/api/stats/cards?version=1.28.1{query}"), "")).await.body;
    let card = cards.as_array().unwrap().iter().find(|c| c["card"] == SEAT0_CARDS[0]);
    (overview["matches"].clone(), card.map_or(0, |c| c["games"].as_i64().unwrap()))
}

#[sqlx::test]
async fn both_players_upload_one_match_counts_once(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let (_, b) = user(&pool, "76561190000000002").await;
    let seat0 = entry(1, 0, ACCOUNT_A, ACCOUNT_B, Some(ACCOUNT_A));
    let seat1 = entry(1, 1, ACCOUNT_B, ACCOUNT_A, Some(ACCOUNT_A));
    let ra = upload(&app, &a, std::slice::from_ref(&seat0), &[replay(&seat0)]).await;
    let rb = upload(&app, &b, std::slice::from_ref(&seat1), &[replay(&seat1)]).await;
    for res in [&ra, &rb] {
        assert_eq!(statuses(&res.body, "entries"), ["new"]);
        assert_eq!(statuses(&res.body, "replays"), ["new"]);
    }
    assert_eq!(rows(&pool).await, [1, 2, 2]);
    for query in ["", "&minStars=0", "&minStars=20"] {
        assert_eq!(stats(&app, query).await, (json!(1), 1), "{query}");
    }
    let overview = send(&app, get("/api/stats/overview?version=all", "")).await.body;
    assert_eq!((&overview["matches"], &overview["players"]), (&json!(1), &json!(2)));
    let decks = send(&app, get("/api/stats/decks?version=1.28.1&minStars=20", "")).await.body;
    let games: Vec<&Value> = decks.as_array().unwrap().iter().map(|d| &d["games"]).collect();
    assert_eq!(games, [&json!(1), &json!(1)]);
}

#[sqlx::test]
async fn seat1_upload_first_keeps_match_seats(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let (_, b) = user(&pool, "76561190000000002").await;
    let seat1 = with_stars(entry(1, 1, ACCOUNT_B, ACCOUNT_A, Some(ACCOUNT_B)), 35);
    let seat0 = with_stars(entry(1, 0, ACCOUNT_A, ACCOUNT_B, Some(ACCOUNT_B)), 50);
    upload(&app, &b, std::slice::from_ref(&seat1), &[replay(&seat1)]).await;
    let (s0, s1): (Option<i32>, Option<i32>) =
        sqlx::query_as("SELECT seat0_stars, seat1_stars FROM matches").fetch_one(&pool).await.unwrap();
    assert_eq!((s0, s1), (None, Some(35)));
    upload(&app, &a, std::slice::from_ref(&seat0), &[replay(&seat0)]).await;
    let (s0, s1, disputed): (Option<i32>, Option<i32>, bool) =
        sqlx::query_as("SELECT seat0_stars, seat1_stars, disputed FROM matches").fetch_one(&pool).await.unwrap();
    assert_eq!((s0, s1, disputed), (Some(50), Some(35), false));
    let stars: Vec<Option<i32>> = sqlx::query_scalar("SELECT stars FROM deck_results").fetch_all(&pool).await.unwrap();
    assert_eq!(stars, [Some(35), Some(35)]);
    assert_eq!(rows(&pool).await, [1, 2, 2]);
    assert_eq!(stats(&app, "&minStars=35").await, (json!(1), 1));
    assert_eq!(stats(&app, "&minStars=36").await, (json!(0), 0));
}

#[sqlx::test]
async fn same_user_upload_twice_adds_no_row(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let e = entry(1, 0, ACCOUNT_A, ACCOUNT_B, Some(ACCOUNT_A));
    let first = upload(&app, &a, std::slice::from_ref(&e), &[replay(&e)]).await;
    assert_eq!(statuses(&first.body, "entries"), ["new"]);
    assert_eq!(statuses(&first.body, "replays"), ["new"]);
    assert_eq!(rows(&pool).await, [1, 1, 1]);
    let second = upload(&app, &a, std::slice::from_ref(&e), &[replay(&e)]).await;
    assert_eq!(statuses(&second.body, "entries"), ["known"]);
    assert_eq!(statuses(&second.body, "replays"), ["known"]);
    assert_eq!(rows(&pool).await, [1, 1, 1]);
    assert_eq!(stats(&app, "").await, (json!(1), 1));
}
