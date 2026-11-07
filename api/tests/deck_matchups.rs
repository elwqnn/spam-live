mod common;

use axum::http::StatusCode;
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

fn k(n: u32) -> Uuid {
    format!("00000000-0000-4000-8000-{n:012}").parse().unwrap()
}

fn mine() -> Vec<Uuid> {
    SEAT0_CARDS.iter().map(|c| c.parse().unwrap()).collect()
}

async fn add(pool: &PgPool, n: &mut u64, version: &str, seat0: &[Uuid], seat1: &[Uuid], winner: i16, times: usize) {
    for _ in 0..times {
        *n += 1;
        sqlx::query(
            "INSERT INTO matches (game_uid, app_version, game_mode, is_ai, start_utc, end_utc, duration_ms,
                 turn_count, seat0_cards, seat1_cards, winner_seat, is_draw, disputed)
             VALUES ($1, $2, 10, false, now() - interval '10 minutes', now(), 600000, 10, $3, $4, $5, false, false)",
        )
        .bind(format!("match-{n}"))
        .bind(version)
        .bind(seat0)
        .bind(seat1)
        .bind(winner)
        .execute(pool)
        .await
        .unwrap();
    }
}

async fn record(pool: &PgPool, n: &mut u64, version: &str, opponent: &[Uuid], wins: usize, losses: usize) {
    add(pool, n, version, &mine(), opponent, 0, wins).await;
    add(pool, n, version, opponent, &mine(), 0, losses).await;
}

fn item(card: Uuid, games: i64, wins: i64) -> Value {
    json!({ "card": card, "games": games, "wins": wins, "winRate": wins as f64 / games as f64 })
}

async fn create(app: &axum::Router, cookie: &str) -> i64 {
    let body = json!({ "name": "D", "cards": SEAT0_CARDS, "talents": [0, 0, 0, 0] });
    let res = send(app, req("POST", "/api/decks", cookie, Some(body))).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.body);
    res.body["id"].as_i64().unwrap()
}

fn url(id: i64, query: &str) -> String {
    format!("/api/decks/{id}/matchups{query}")
}

#[sqlx::test]
async fn deck_matchups(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let (_, b) = user(&pool, "76561190000000002").await;
    let deck = create(&app, &a).await;
    let draft = create(&app, &a).await;
    assert!(send(&app, req("POST", &format!("/api/decks/{deck}/publish"), &a, None)).await.status.is_success());

    let n = &mut 0;
    record(&pool, n, "1.29", &[k(11), k(81), k(82), k(83)], 12, 0).await;
    record(&pool, n, "1.29", &[k(12), k(81), k(82), k(83)], 9, 0).await;
    record(&pool, n, "1.29", &[k(13), k(84), k(85), k(86)], 5, 5).await;
    record(&pool, n, "1.29", &[k(21), k(84), k(85), k(86)], 0, 10).await;
    record(&pool, n, "1.29", &[mine()[0], k(84), k(85), k(86)], 0, 10).await;
    record(&pool, n, "1.28.1", &[k(16), k(87), k(88), k(89)], 10, 0).await;

    let res = send(&app, get(&url(deck, ""), "")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    let top = [81, 82, 83].map(|c| item(k(c), 21, 21));
    let weak = [item(k(21), 10, 0), item(k(84), 30, 5), item(k(85), 30, 5), item(k(86), 30, 5)];
    let strong = [&top[..], &[item(k(11), 12, 12), item(k(12), 9, 9)]].concat();
    let latest = json!({ "strong": strong, "weak": weak });
    assert_eq!(res.body, latest);
    assert_eq!(send(&app, get(&url(deck, "?version=1.29"), "")).await.body, latest);
    let old = send(&app, get(&url(deck, "?version=1.28.1"), "")).await.body;
    let old_strong = [16, 87, 88, 89].map(|c| item(k(c), 10, 10));
    assert_eq!(old, json!({ "strong": old_strong, "weak": [] }));
    let all = send(&app, get(&url(deck, "?version=all"), "")).await.body;
    let strong = [&top[..], &[item(k(11), 12, 12), item(k(16), 10, 10)]].concat();
    assert_eq!(all, json!({ "strong": strong, "weak": weak }));

    assert_eq!(send(&app, get(&url(deck, "?version=latest"), "")).await.status, StatusCode::BAD_REQUEST);
    assert_eq!(send(&app, get(&url(999_999, "?version=latest"), "")).await.status, StatusCode::BAD_REQUEST);
    assert_eq!(send(&app, get(&url(999_999, ""), "")).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, get(&url(draft, ""), "")).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, get(&url(draft, ""), &b)).await.status, StatusCode::NOT_FOUND);
    let own = send(&app, get(&url(draft, ""), &a)).await;
    assert_eq!(own.status, StatusCode::OK);
    assert!(own.headers.get("cache-control").is_none());
    assert_eq!(own.body, latest);
}
