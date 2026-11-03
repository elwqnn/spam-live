mod common;

use axum::http::StatusCode;
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

fn k(n: u32) -> Uuid {
    format!("00000000-0000-4000-8000-{n:012}").parse().unwrap()
}

#[derive(Clone)]
struct M {
    version: &'static str,
    mode: i32,
    ai: bool,
    disputed: bool,
    seat0: Vec<Uuid>,
    seat1: Vec<Uuid>,
    winner: Option<i16>,
}

fn m(seat0: &[Uuid], seat1: &[Uuid], winner: Option<i16>) -> M {
    M { version: "1.28.1", mode: 10, ai: false, disputed: false, seat0: seat0.to_vec(), seat1: seat1.to_vec(), winner }
}

async fn add(pool: &PgPool, n: &mut u64, m: M, times: usize) {
    for _ in 0..times {
        *n += 1;
        sqlx::query(
            "INSERT INTO matches (game_uid, app_version, game_mode, is_ai, start_utc, end_utc, duration_ms,
                 turn_count, seat0_cards, seat1_cards, winner_seat, is_draw, disputed)
             VALUES ($1, $2, $3, $4, now() - interval '10 minutes', now(), 600000, 10, $5, $6, $7, $8, $9)",
        )
        .bind(format!("match-{n}"))
        .bind(m.version)
        .bind(m.mode)
        .bind(m.ai)
        .bind(&m.seat0)
        .bind(&m.seat1)
        .bind(m.winner)
        .bind(m.winner.is_none())
        .bind(m.disputed)
        .execute(pool)
        .await
        .unwrap();
    }
}

struct Decks {
    a: Vec<Uuid>,
    b: Vec<Uuid>,
    c: Vec<Uuid>,
}

async fn fixture(pool: &PgPool) -> Decks {
    let a = vec![k(1), k(2), k(3), k(4)];
    let b = vec![k(5), k(6), k(7), k(8)];
    let c = vec![k(1), k(5), k(9), k(10)];
    let n = &mut 0;
    add(pool, n, m(&a, &b, Some(0)), 7).await;
    add(pool, n, m(&a, &b, Some(1)), 4).await;
    add(pool, n, m(&a, &b, None), 1).await;
    add(pool, n, m(&b, &a, Some(0)), 3).await;
    add(pool, n, m(&b, &a, Some(1)), 5).await;
    add(pool, n, m(&c, &b, Some(0)), 2).await;
    add(pool, n, m(&c, &b, Some(1)), 1).await;
    add(pool, n, M { ai: true, ..m(&a, &c, Some(1)) }, 1).await;
    add(pool, n, M { mode: 40, ..m(&a, &c, Some(1)) }, 1).await;
    add(pool, n, M { disputed: true, ..m(&a, &c, Some(1)) }, 1).await;
    let reversed: Vec<Uuid> = a.iter().rev().copied().collect();
    add(pool, n, m(&a, &reversed, Some(0)), 1).await;
    add(pool, n, M { version: "1.27.0", ..m(&c, &a, Some(0)) }, 2).await;
    Decks { a, b, c }
}

fn row(cards: &[Uuid], games: i64, wins: i64, win_rate: Option<f64>) -> Value {
    json!({ "cards": cards, "games": games, "wins": wins, "winRate": win_rate })
}

fn card(id: Uuid, games: i64, wins: i64, win_rate: Option<f64>, total: f64) -> Value {
    json!({ "card": id, "games": games, "wins": wins, "winRate": win_rate, "playRate": games as f64 / total })
}

#[sqlx::test]
async fn deck_stats(pool: PgPool) {
    let d = fixture(&pool).await;
    let app = app(pool.clone());
    let res = send(&app, get("/api/stats/decks", "")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    let current =
        json!([row(&d.a, 20, 12, Some(12.0 / 20.0)), row(&d.b, 23, 8, Some(8.0 / 23.0)), row(&d.c, 3, 2, None),]);
    assert_eq!(res.body, current);
    assert_eq!(send(&app, get("/api/stats/decks?version=1.28.1&sort=winrate", "")).await.body, current);

    let by_games = send(&app, get("/api/stats/decks?sort=games", "")).await.body;
    let games: Vec<_> = by_games.as_array().unwrap().iter().map(|r| r["games"].as_i64().unwrap()).collect();
    assert_eq!(games, [23, 20, 3]);
    assert_eq!(
        send(&app, get("/api/stats/decks?sort=games&limit=1", "")).await.body,
        json!([row(&d.b, 23, 8, Some(8.0 / 23.0))])
    );

    let all = send(&app, get("/api/stats/decks?version=all", "")).await.body;
    assert_eq!(
        all,
        json!([row(&d.a, 22, 12, Some(12.0 / 22.0)), row(&d.b, 23, 8, Some(8.0 / 23.0)), row(&d.c, 5, 4, None)])
    );
    let old = send(&app, get("/api/stats/decks?version=1.27.0", "")).await.body;
    assert_eq!(old, json!([row(&d.a, 2, 0, None), row(&d.c, 2, 2, None)]));
    assert_eq!(send(&app, get("/api/stats/decks?version=9.9.9", "")).await.body, json!([]));
    assert_eq!(send(&app, get("/api/stats/decks?version=latest", "")).await.status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn card_stats(pool: PgPool) {
    fixture(&pool).await;
    let app = app(pool.clone());
    let res = send(&app, get("/api/stats/cards", "")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    let t = 46.0;
    let b = |n| card(k(n), 23, 8, Some(8.0 / 23.0), t);
    let a = |n| card(k(n), 20, 12, Some(0.6), t);
    let expected = json!([
        card(k(5), 26, 10, Some(10.0 / 26.0), t),
        card(k(1), 23, 14, Some(14.0 / 23.0), t),
        b(6),
        b(7),
        b(8),
        a(2),
        a(3),
        a(4),
        card(k(9), 3, 2, None, t),
        card(k(10), 3, 2, None, t),
    ]);
    assert_eq!(res.body, expected);
    assert_eq!(send(&app, get("/api/stats/cards?version=1.28.1", "")).await.body, expected);

    let all = send(&app, get("/api/stats/cards?version=all", "")).await.body;
    let find = |id: Uuid| all.as_array().unwrap().iter().find(|c| c["card"] == json!(id)).unwrap().clone();
    assert_eq!(find(k(1)), card(k(1), 27, 16, Some(16.0 / 27.0), 50.0));
    assert_eq!(find(k(9)), card(k(9), 5, 4, None, 50.0));
    assert_eq!(all.as_array().unwrap().len(), 10);
    let old = send(&app, get("/api/stats/cards?version=1.27.0", "")).await.body;
    assert_eq!(old[0], card(k(1), 4, 2, None, 4.0));
    assert_eq!(old.as_array().unwrap().len(), 7);
}

#[sqlx::test]
async fn deck_detail_stats_and_winrate_sort(pool: PgPool) {
    let d = fixture(&pool).await;
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let (_, b) = user(&pool, "76561190000000002").await;
    let shuffled = [d.a[2], d.a[0], d.a[3], d.a[1]];
    let other = [k(11), k(12), k(13), k(14)];
    let mut ids = Vec::new();
    for cards in [&shuffled[..], &d.b, &d.c, &other] {
        let body = json!({ "name": "S", "cards": cards, "talents": [0, 0, 0, 0] });
        ids.push(send(&app, req("POST", "/api/decks", &a, Some(body))).await.body["id"].as_i64().unwrap());
    }
    let detail = send(&app, get(&format!("/api/decks/{}", ids[0]), &a)).await.body;
    assert_eq!(
        detail["stats"],
        json!({
            "version": "1.28.1",
            "current": { "games": 20, "wins": 12, "winRate": 0.6 },
            "all": { "games": 22, "wins": 12, "winRate": 12.0 / 22.0 }
        })
    );
    let none = send(&app, get(&format!("/api/decks/{}", ids[3]), &a)).await.body;
    assert_eq!(none["stats"]["all"], json!({ "games": 0, "wins": 0, "winRate": null }));

    sqlx::query("UPDATE decks SET status = 'public', published_at = now()").execute(&pool).await.unwrap();
    send(&app, req("PUT", &format!("/api/decks/{}/vote", ids[2]), &b, None)).await;
    let sorted = send(&app, get("/api/decks?sort=winrate", "")).await.body;
    let order: Vec<_> = sorted.as_array().unwrap().iter().map(|d| d["id"].as_i64().unwrap()).collect();
    assert_eq!(order, [ids[0], ids[1], ids[2], ids[3]]);
    let list_stats: Vec<Value> =
        sorted.as_array().unwrap().iter().map(|d| json!([d["games"], d["wins"], d["winRate"]])).collect();
    assert_eq!(
        list_stats,
        [json!([20, 12, 0.6]), json!([23, 8, 8.0 / 23.0]), json!([3, 2, null]), json!([0, 0, null])]
    );
    let page = |offset: i64| get(&format!("/api/decks?sort=winrate&limit=2&offset={offset}"), "");
    let pages = [send(&app, page(0)).await.body, send(&app, page(2)).await.body, send(&app, page(4)).await.body];
    let paged: Vec<Vec<i64>> =
        pages.iter().map(|p| p.as_array().unwrap().iter().map(|d| d["id"].as_i64().unwrap()).collect()).collect();
    assert_eq!(paged, [vec![ids[0], ids[1]], vec![ids[2], ids[3]], vec![]]);
}
