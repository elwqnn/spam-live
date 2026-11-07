mod common;

use axum::http::StatusCode;
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

fn k(n: u32) -> Uuid {
    format!("00000000-0000-4000-8000-{n:012}").parse().unwrap()
}

fn me() -> Uuid {
    SEAT0_CARDS[0].parse().unwrap()
}

fn mine() -> Vec<Uuid> {
    vec![me(), k(2), k(3), k(4)]
}

fn deck(n: u32) -> Vec<Uuid> {
    vec![k(n), k(91), k(92), k(93)]
}

async fn add(
    pool: &PgPool,
    n: &mut u64,
    version: &str,
    seat0: &[Uuid],
    seat1: &[Uuid],
    winner: Option<i16>,
    times: usize,
) {
    for _ in 0..times {
        *n += 1;
        sqlx::query(
            "INSERT INTO matches (game_uid, app_version, game_mode, is_ai, start_utc, end_utc, duration_ms,
                 turn_count, seat0_cards, seat1_cards, winner_seat, is_draw, disputed)
             VALUES ($1, $2, 10, false, now() - interval '10 minutes', now(), 600000, 10, $3, $4, $5, $6, false)",
        )
        .bind(format!("match-{n}"))
        .bind(version)
        .bind(seat0)
        .bind(seat1)
        .bind(winner)
        .bind(winner.is_none())
        .execute(pool)
        .await
        .unwrap();
    }
}

async fn record(pool: &PgPool, n: &mut u64, version: &str, opponent: u32, wins: usize, losses: usize) {
    add(pool, n, version, &mine(), &deck(opponent), Some(0), wins).await;
    add(pool, n, version, &deck(opponent), &mine(), Some(0), losses).await;
}

fn item(card: u32, games: i64, wins: i64) -> Value {
    json!({ "card": k(card), "games": games, "wins": wins, "winRate": wins as f64 / games as f64 })
}

fn url(id: Uuid, query: &str) -> String {
    format!("/api/stats/cards/{id}/matchups{query}")
}

#[sqlx::test]
async fn card_matchups(pool: PgPool) {
    let app = app(pool.clone());
    let n = &mut 0;
    record(&pool, n, "1.29", 11, 12, 0).await;
    record(&pool, n, "1.29", 12, 40, 10).await;
    record(&pool, n, "1.29", 13, 9, 1).await;
    record(&pool, n, "1.29", 14, 9, 0).await;
    record(&pool, n, "1.29", 15, 5, 5).await;
    record(&pool, n, "1.29", 21, 0, 12).await;
    record(&pool, n, "1.29", 22, 10, 40).await;
    record(&pool, n, "1.29", 23, 1, 9).await;
    record(&pool, n, "1.29", 24, 0, 9).await;
    record(&pool, n, "1.28.1", 16, 10, 0).await;
    record(&pool, n, "1.28.1", 26, 0, 10).await;

    let res = send(&app, get(&url(me(), ""), "")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    let latest = json!({
        "strong": [item(11, 12, 12), item(12, 50, 40), item(13, 10, 9)],
        "weak": [item(21, 12, 0), item(22, 50, 10), item(23, 10, 1)]
    });
    assert_eq!(res.body, latest);
    assert_eq!(send(&app, get(&url(me(), "?version=1.29"), "")).await.body, latest);

    let score = async |games: i64, wins: i64| -> f64 {
        sqlx::query_scalar("SELECT win_score($1, $2)").bind(games).bind(wins).fetch_one(&pool).await.unwrap()
    };
    assert!(score(12, 12).await > score(50, 40).await);
    assert!(score(50, 40).await > score(10, 9).await);

    assert_eq!(
        send(&app, get(&url(me(), "?version=1.28.1"), "")).await.body,
        json!({ "strong": [item(16, 10, 10)], "weak": [item(26, 10, 0)] })
    );
    assert_eq!(
        send(&app, get(&url(me(), "?version=all"), "")).await.body,
        json!({
            "strong": [item(11, 12, 12), item(16, 10, 10), item(12, 50, 40), item(13, 10, 9)],
            "weak": [item(21, 12, 0), item(26, 10, 0), item(22, 50, 10), item(23, 10, 1)]
        })
    );
    assert_eq!(send(&app, get(&url(me(), "?version=9.9.9"), "")).await.body, json!({ "strong": [], "weak": [] }));
    assert_eq!(send(&app, get(&url(me(), "?version=latest"), "")).await.status, StatusCode::BAD_REQUEST);
    assert_eq!(send(&app, get(&url(k(1), ""), "")).await.status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn card_matchups_limit_and_self(pool: PgPool) {
    let app = app(pool.clone());
    let other = vec![me(), k(50), k(51), k(52)];
    add(&pool, &mut 0, "1.29", &mine(), &other, None, 10).await;
    let body = send(&app, get(&url(me(), ""), "")).await.body;
    let weak = [2, 3, 4, 50, 51].map(|c| item(c, 10, 0));
    assert_eq!(body, json!({ "strong": [], "weak": weak }));
}
