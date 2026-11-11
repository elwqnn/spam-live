mod common;

use axum::Router;
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;

fn replay(e: &Value) -> (String, Vec<u8>) {
    (format!("{}.srpl", e["GameUid"].as_str().unwrap()), match_srpl(e))
}

async fn events(pool: &PgPool) -> [i64; 5] {
    let counts: (i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM shop_buys), (SELECT count(*) FROM talent_picks),
             (SELECT count(*) FROM card_damage), (SELECT count(*) FROM hp_turns),
             (SELECT count(*) FROM match_event_sources)",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    [counts.0, counts.1, counts.2, counts.3, counts.4]
}

async fn sources(pool: &PgPool) -> Vec<(i16, Option<i64>, bool)> {
    sqlx::query_as("SELECT seat, user_id, own FROM match_event_sources ORDER BY seat").fetch_all(pool).await.unwrap()
}

async fn talent_seats(pool: &PgPool) -> Vec<(i16, i64)> {
    sqlx::query_as("SELECT seat, count(*) FROM talent_picks GROUP BY seat ORDER BY seat").fetch_all(pool).await.unwrap()
}

async fn seat1_rows(pool: &PgPool) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT t FROM (
             SELECT format('buy %s %s %s %s', idx, turn, card, tox) AS t FROM shop_buys WHERE seat = 1
             UNION ALL SELECT format('talent %s %s', card, talent) FROM talent_picks WHERE seat = 1
             UNION ALL SELECT format('damage %s %s %s', card, done, taken) FROM card_damage WHERE seat = 1
             UNION ALL SELECT format('hp %s %s', turn, hp) FROM hp_turns WHERE seat = 1
             UNION ALL SELECT format('source %s %s', user_id, own) FROM match_event_sources WHERE seat = 1
         ) r ORDER BY t",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

async fn replay_stats(app: &Router, card: &str) -> Value {
    let res = send(app, get(&format!("/api/stats/cards/{card}/replay?version=1.28.1"), "")).await;
    assert_eq!(res.status, axum::http::StatusCode::OK);
    res.body
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
    let (ua, a) = user(&pool, "76561190000000001").await;
    let (ub, b) = user(&pool, "76561190000000002").await;
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
    assert_eq!(events(&pool).await, [3, 4, 8, 4, 2]);
    assert_eq!(sources(&pool).await, [(0, Some(ua), true), (1, Some(ub), true)]);
    let first = replay_stats(&app, SEAT0_CARDS[0]).await;
    assert_eq!(first["games"], 1);
    let tox: Vec<(&Value, &Value, &Value)> =
        first["tox"].as_array().unwrap().iter().map(|t| (&t["tox"], &t["games"], &t["buyRate"])).collect();
    assert_eq!(tox, [(&json!(TOX[0]), &json!(1), &json!(1.0)), (&json!(TOX[1]), &json!(1), &json!(1.0))]);
    assert_eq!(first["tox"][0]["name"], "Vitalheart");
    assert_eq!(first["damage"], json!({ "games": 1, "done": 500.0, "taken": 300.0 }));
    let second = replay_stats(&app, SEAT0_CARDS[1]).await;
    assert_eq!(second["tox"], json!([]));
    let talents = &second["talents"];
    assert_eq!(talents.as_array().unwrap().len(), 1);
    assert_eq!((&talents[0]["talent"], &talents[0]["games"]), (&json!("Blindage O/L"), &json!(1)));
    assert_eq!((&talents[0]["pickRate"], &talents[0]["winRate"]), (&json!(1.0), &json!(1.0)));
    let opponent = replay_stats(&app, SEAT1_CARDS[1]).await;
    assert_eq!((&opponent["games"], &opponent["tox"][0]["wins"]), (&json!(1), &json!(0)));
}

#[sqlx::test]
async fn seat1_upload_first_keeps_match_seats(pool: PgPool) {
    let app = app(pool.clone());
    let (ua, a) = user(&pool, "76561190000000001").await;
    let (ub, b) = user(&pool, "76561190000000002").await;
    let seat1 = with_stars(entry(1, 1, ACCOUNT_B, ACCOUNT_A, Some(ACCOUNT_B)), 35);
    let seat0 = with_stars(entry(1, 0, ACCOUNT_A, ACCOUNT_B, Some(ACCOUNT_B)), 50);
    upload(&app, &b, std::slice::from_ref(&seat1), &[replay(&seat1)]).await;
    assert_eq!(events(&pool).await, [3, 2, 8, 4, 2]);
    assert_eq!(sources(&pool).await, [(0, Some(ub), false), (1, Some(ub), true)]);
    assert_eq!(talent_seats(&pool).await, [(1, 2)]);
    let buys: Vec<(i16, i32, String)> =
        sqlx::query_as("SELECT seat, idx, card::text FROM shop_buys ORDER BY seat, idx")
            .fetch_all(&pool)
            .await
            .unwrap();
    let buy = |seat, idx, card: &str| (seat, idx, card.to_string());
    assert_eq!(buys, [buy(0, 0, SEAT0_CARDS[0]), buy(0, 1, SEAT0_CARDS[0]), buy(1, 0, SEAT1_CARDS[1])]);
    let hp: Vec<(i16, i32, i32)> =
        sqlx::query_as("SELECT seat, turn, hp FROM hp_turns ORDER BY turn, seat").fetch_all(&pool).await.unwrap();
    assert_eq!(hp, [(0, 1, 3200), (1, 1, 3600), (0, 2, 2800), (1, 2, 0)]);
    let before = seat1_rows(&pool).await;
    assert_eq!(replay_stats(&app, SEAT0_CARDS[1]).await["talents"], json!([]));
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
    assert_eq!(events(&pool).await, [3, 4, 8, 4, 2]);
    assert_eq!(sources(&pool).await, [(0, Some(ua), true), (1, Some(ub), true)]);
    assert_eq!(talent_seats(&pool).await, [(0, 2), (1, 2)]);
    assert_eq!(seat1_rows(&pool).await, before);
    assert_eq!(replay_stats(&app, SEAT0_CARDS[1]).await["talents"][0]["games"], 1);
    let res =
        send(&app, get(&format!("/api/stats/cards/{}/replay?version=1.28.1&minStars=36", SEAT0_CARDS[0]), "")).await;
    assert_eq!((&res.body["games"], &res.body["tox"]), (&json!(0), &json!([])));
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
    assert_eq!(events(&pool).await, [3, 2, 8, 4, 2]);
    assert_eq!(replay_stats(&app, SEAT0_CARDS[0]).await["games"], 1);
}
