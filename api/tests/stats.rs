mod common;

use api::catalog::Catalog;
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
    days: i32,
    turns: i32,
    end: Option<String>,
}

fn m(seat0: &[Uuid], seat1: &[Uuid], winner: Option<i16>) -> M {
    M {
        version: "1.29",
        mode: 10,
        ai: false,
        disputed: false,
        seat0: seat0.to_vec(),
        seat1: seat1.to_vec(),
        winner,
        days: 0,
        turns: 10,
        end: None,
    }
}

async fn add(pool: &PgPool, n: &mut u64, m: M, times: usize) {
    for _ in 0..times {
        *n += 1;
        sqlx::query(
            "INSERT INTO matches (game_uid, app_version, game_mode, is_ai, start_utc, end_utc, duration_ms,
                 turn_count, seat0_cards, seat1_cards, winner_seat, is_draw, disputed)
             SELECT $1, $2, $3, $4, e - interval '10 minutes', e, 600000, $11, $5, $6, $7, $8, $9
             FROM (SELECT coalesce($12::timestamptz, now() - make_interval(days => $10)) AS e) t",
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
        .bind(m.days)
        .bind(m.turns)
        .bind(&m.end)
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

fn wilson(games: i64, wins: i64) -> Option<f64> {
    let (n, w) = (games as f64, wins as f64);
    (games > 0).then(|| ((w + 1.9208 - 1.96 * (w * (n - w) / n + 0.9604).sqrt()) / (n + 3.8416)).max(0.0))
}

fn rounded(v: Value) -> Value {
    match v {
        Value::Array(a) => Value::Array(a.into_iter().map(rounded).collect()),
        Value::Object(o) => Value::Object(
            o.into_iter()
                .map(|(k, v)| match v.as_f64() {
                    Some(f) if k == "winScore" => (k, json!((f * 1e9).round())),
                    _ => (k, rounded(v)),
                })
                .collect(),
        ),
        v => v,
    }
}

fn row(cards: &[Uuid], games: i64, wins: i64, win_rate: Option<f64>) -> Value {
    json!({ "cards": cards, "games": games, "wins": wins, "winRate": win_rate, "winScore": wilson(games, wins) })
}

fn card(id: Uuid, games: i64, wins: i64, win_rate: Option<f64>, total: f64) -> Value {
    json!({
        "card": id, "games": games, "wins": wins, "winRate": win_rate, "playRate": games as f64 / total,
        "winRateChange": null, "playRateChange": null
    })
}

#[sqlx::test]
async fn deck_stats(pool: PgPool) {
    let d = fixture(&pool).await;
    let app = app(pool.clone());
    let res = send(&app, get("/api/stats/decks", "")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    let current = rounded(json!([
        row(&d.a, 20, 12, Some(12.0 / 20.0)),
        row(&d.c, 3, 2, Some(2.0 / 3.0)),
        row(&d.b, 23, 8, Some(8.0 / 23.0)),
    ]));
    assert_eq!(rounded(res.body), current);
    assert_eq!(rounded(send(&app, get("/api/stats/decks?version=1.29&sort=winrate", "")).await.body), current);

    let by_games = send(&app, get("/api/stats/decks?sort=games", "")).await.body;
    let games: Vec<_> = by_games.as_array().unwrap().iter().map(|r| r["games"].as_i64().unwrap()).collect();
    assert_eq!(games, [23, 20, 3]);
    assert_eq!(
        rounded(send(&app, get("/api/stats/decks?sort=games&limit=1", "")).await.body),
        rounded(json!([row(&d.b, 23, 8, Some(8.0 / 23.0))]))
    );

    let all = send(&app, get("/api/stats/decks?version=all", "")).await.body;
    assert_eq!(
        rounded(all),
        rounded(json!([
            row(&d.c, 5, 4, Some(0.8)),
            row(&d.a, 22, 12, Some(12.0 / 22.0)),
            row(&d.b, 23, 8, Some(8.0 / 23.0))
        ]))
    );
    let old = send(&app, get("/api/stats/decks?version=1.27.0", "")).await.body;
    assert_eq!(rounded(old), rounded(json!([row(&d.c, 2, 2, Some(1.0)), row(&d.a, 2, 0, Some(0.0))])));
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
        card(k(9), 3, 2, Some(2.0 / 3.0), t),
        card(k(10), 3, 2, Some(2.0 / 3.0), t),
    ]);
    assert_eq!(res.body, expected);
    assert_eq!(send(&app, get("/api/stats/cards?version=1.29", "")).await.body, expected);

    let all = send(&app, get("/api/stats/cards?version=all", "")).await.body;
    let find = |id: Uuid| all.as_array().unwrap().iter().find(|c| c["card"] == json!(id)).unwrap().clone();
    assert_eq!(find(k(1)), card(k(1), 27, 16, Some(16.0 / 27.0), 50.0));
    assert_eq!(find(k(9)), card(k(9), 5, 4, Some(0.8), 50.0));
    assert_eq!(all.as_array().unwrap().len(), 10);
    let old = send(&app, get("/api/stats/cards?version=1.27.0", "")).await.body;
    assert_eq!(old[0], card(k(1), 4, 2, Some(0.5), 4.0));
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
    for cards in [&shuffled[..], &d.b, &d.c, &other, &d.c] {
        let body = json!({ "name": "S", "cards": cards, "talents": [0, 0, 0, 0] });
        ids.push(send(&app, req("POST", "/api/decks", &a, Some(body))).await.body["id"].as_i64().unwrap());
    }
    let detail = send(&app, get(&format!("/api/decks/{}", ids[0]), &a)).await.body;
    assert_eq!(
        rounded(detail["stats"].clone()),
        rounded(json!({
            "version": "1.29",
            "current": { "games": 20, "wins": 12, "winRate": 0.6, "winScore": wilson(20, 12) },
            "all": { "games": 22, "wins": 12, "winRate": 12.0 / 22.0, "winScore": wilson(22, 12) }
        }))
    );
    let none = send(&app, get(&format!("/api/decks/{}", ids[3]), &a)).await.body;
    assert_eq!(none["stats"]["all"], json!({ "games": 0, "wins": 0, "winRate": null, "winScore": null }));

    sqlx::query("UPDATE decks SET status = 'public', published_at = now()").execute(&pool).await.unwrap();
    send(&app, req("PUT", &format!("/api/decks/{}/vote", ids[2]), &b, Some(json!({ "value": 1 })))).await;
    let sorted = send(&app, get("/api/decks?sort=winrate", "")).await.body;
    let order: Vec<_> = sorted.as_array().unwrap().iter().map(|d| d["id"].as_i64().unwrap()).collect();
    assert_eq!(order, [ids[0], ids[2], ids[4], ids[1], ids[3]]);
    let list_stats: Vec<Value> = sorted
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            rounded(
                json!({ "games": d["games"], "wins": d["wins"], "winRate": d["winRate"], "winScore": d["winScore"] }),
            )
        })
        .collect();
    let stats = |games: i64, wins: i64, win_rate: Option<f64>| {
        rounded(json!({ "games": games, "wins": wins, "winRate": win_rate, "winScore": wilson(games, wins) }))
    };
    assert_eq!(
        list_stats,
        [
            stats(20, 12, Some(0.6)),
            stats(3, 2, Some(2.0 / 3.0)),
            stats(3, 2, Some(2.0 / 3.0)),
            stats(23, 8, Some(8.0 / 23.0)),
            stats(0, 0, None)
        ]
    );
    let page = |offset: i64| get(&format!("/api/decks?sort=winrate&limit=2&offset={offset}"), "");
    let pages = [send(&app, page(0)).await.body, send(&app, page(2)).await.body, send(&app, page(4)).await.body];
    let paged: Vec<Vec<i64>> =
        pages.iter().map(|p| p.as_array().unwrap().iter().map(|d| d["id"].as_i64().unwrap()).collect()).collect();
    assert_eq!(paged, [vec![ids[0], ids[2]], vec![ids[4], ids[1]], vec![ids[3]]]);
}

#[sqlx::test]
async fn win_score_function(pool: PgPool) {
    let score = async |games: i64, wins: i64| -> Option<f64> {
        sqlx::query_scalar("SELECT win_score($1, $2)").bind(games).bind(wins).fetch_one(&pool).await.unwrap()
    };
    let three = |v: Option<f64>| v.map(|v| (v * 1000.0).round() / 1000.0);
    assert_eq!(score(0, 0).await, None);
    assert_eq!(score(2, 0).await, Some(0.0));
    assert_eq!(three(score(1, 1).await), Some(0.207));
    assert_eq!(three(score(5, 5).await), Some(0.566));
    assert_eq!(three(score(20, 12).await), Some(0.387));
    assert_eq!(three(score(151, 92).await), Some(0.530));
    for (games, wins) in [(1, 1), (5, 5), (20, 12), (151, 92)] {
        assert!((score(games, wins).await.unwrap() - wilson(games, wins).unwrap()).abs() < 1e-12);
    }
}

fn ids(cards: [&str; 4]) -> Vec<Uuid> {
    cards.iter().map(|c| c.parse().unwrap()).collect()
}

fn changes(body: &Value, id: Uuid) -> [Option<f64>; 2] {
    let row = body.as_array().unwrap().iter().find(|c| c["card"] == json!(id)).unwrap();
    ["winRateChange", "playRateChange"].map(|k| row[k].as_f64().map(|v| (v * 1e9).round()))
}

fn pts(win: f64, play: f64) -> [Option<f64>; 2] {
    [win, play].map(|v| Some((v * 1e9).round()))
}

fn at(days: i32, seat0: &[Uuid], seat1: &[Uuid], winner: i16) -> M {
    M { days, ..m(seat0, seat1, Some(winner)) }
}

fn without_changes(body: &Value) -> Vec<Value> {
    let strip = |c: &Value| json!([c["card"], c["games"], c["wins"], c["winRate"], c["playRate"]]);
    body.as_array().unwrap().iter().map(strip).collect()
}

#[sqlx::test]
async fn card_stats_week_trend(pool: PgPool) {
    let (a, b) = (ids(SEAT0_CARDS), ids(SEAT1_CARDS));
    let c = vec![a[0], a[1], b[0], b[1]];
    let n = &mut 0;
    add(&pool, n, at(1, &a, &b, 0), 12).await;
    add(&pool, n, at(1, &a, &b, 1), 8).await;
    add(&pool, n, at(1, &c, &b, 0), 5).await;
    add(&pool, n, at(8, &a, &b, 0), 7).await;
    add(&pool, n, at(8, &a, &b, 1), 12).await;
    add(&pool, n, at(13, &a, &c, 0), 1).await;
    add(&pool, n, at(15, &a, &b, 0), 5).await;
    add(&pool, n, M { version: "1.27.0", ..at(1, &a, &c, 0) }, 20).await;
    let app = app(pool.clone());

    let res = send(&app, get("/api/stats/cards", "")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    assert_eq!(changes(&res.body, a[2]), pts(12.0 / 20.0 - 8.0 / 20.0, 20.0 / 50.0 - 20.0 / 40.0));
    assert_eq!(changes(&res.body, a[0]), pts(17.0 / 25.0 - 8.0 / 21.0, 25.0 / 50.0 - 21.0 / 40.0));
    assert_eq!(changes(&res.body, b[2]), pts(8.0 / 25.0 - 12.0 / 19.0, 25.0 / 50.0 - 19.0 / 40.0));
    assert_eq!(send(&app, get("/api/stats/cards?trend=week&version=1.29", "")).await.body, res.body);

    let all = send(&app, get("/api/stats/cards?version=all", "")).await.body;
    assert_eq!(changes(&all, a[2]), pts(32.0 / 40.0 - 8.0 / 20.0, 40.0 / 90.0 - 20.0 / 40.0));
    let old = send(&app, get("/api/stats/cards?version=1.27.0", "")).await.body;
    assert_eq!(changes(&old, a[2]), [None, None]);
    assert_eq!(send(&app, get("/api/stats/cards?trend=other", "")).await.status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn card_stats_week_trend_one_game(pool: PgPool) {
    let deck = |s: u32| (s..s + 4).map(k).collect::<Vec<_>>();
    let (a, b, c, d) = (deck(1), deck(5), deck(9), deck(13));
    let n = &mut 0;
    add(&pool, n, at(1, &a, &b, 0), 1).await;
    add(&pool, n, at(8, &a, &b, 1), 1).await;
    add(&pool, n, at(1, &c, &d, 0), 1).await;
    let app = app(pool.clone());

    let res = send(&app, get("/api/stats/cards", "")).await.body;
    assert_eq!(changes(&res, a[0]), pts(1.0 / 1.0 - 0.0 / 1.0, 1.0 / 4.0 - 1.0 / 2.0));
    assert_eq!(changes(&res, b[0]), pts(0.0 / 1.0 - 1.0 / 1.0, 1.0 / 4.0 - 1.0 / 2.0));
    assert_eq!(changes(&res, c[0]), [None, None]);
    assert_eq!(changes(&res, d[0]), [None, None]);
}

fn test_catalog(versions: &[&str]) -> Catalog {
    let dir = std::env::temp_dir().join(format!("spam-stats-catalog-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for v in versions {
        let file =
            json!({ "appVersion": v, "languages": ["en"], "cards": [], "abilities": [], "talents": [], "tox": [] });
        std::fs::write(dir.join(format!("{v}.json")), file.to_string()).unwrap();
    }
    let catalog = Catalog::load(&dir).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    catalog
}

#[sqlx::test]
async fn card_stats_version_trend(pool: PgPool) {
    let (a, b) = (ids(SEAT0_CARDS), ids(SEAT1_CARDS));
    let c = vec![a[0], a[1], b[0], b[1]];
    let n = &mut 0;
    let on = |version, seat0: &[Uuid], seat1: &[Uuid], winner| M { version, ..m(seat0, seat1, Some(winner)) };
    add(&pool, n, on("1.28.1", &a, &b, 0), 12).await;
    add(&pool, n, on("1.28.1", &a, &b, 1), 8).await;
    add(&pool, n, on("1.27.0", &a, &b, 0), 5).await;
    add(&pool, n, on("1.27.0", &a, &b, 1), 15).await;
    add(&pool, n, on("1.27.0", &c, &b, 0), 10).await;
    add(&pool, n, on("1.9.0", &a, &b, 0), 20).await;
    add(&pool, n, on("2.0.0", &a, &b, 0), 20).await;
    let catalog = test_catalog(&["1.9.0", "1.27.0", "1.28.1"]);
    let app = api::router(state_with(pool.clone(), "https://steamcommunity.com/openid/login", catalog));
    let cards = |q: &str| get(&format!("/api/stats/cards?{q}"), "");

    let res = send(&app, cards("trend=version")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(changes(&res.body, a[2]), pts(12.0 / 20.0 - 5.0 / 20.0, 20.0 / 40.0 - 20.0 / 60.0));
    assert_eq!(changes(&res.body, b[2]), pts(8.0 / 20.0 - 15.0 / 30.0, 20.0 / 40.0 - 30.0 / 60.0));
    let week = send(&app, cards("")).await.body;
    assert_eq!(without_changes(&res.body), without_changes(&week));
    assert_eq!(week.as_array().unwrap().len(), 8);

    let older = send(&app, cards("trend=version&version=1.27.0")).await.body;
    assert_eq!(changes(&older, a[2]), pts(5.0 / 20.0 - 20.0 / 20.0, 20.0 / 60.0 - 20.0 / 40.0));
    for q in ["version=1.9.0", "version=all", "version=2.0.0"] {
        let body = send(&app, cards(&format!("trend=version&{q}"))).await.body;
        assert_eq!(changes(&body, a[2]), [None, None], "{q}");
    }
}

fn cell(body: &Value, card: u32, opponent: u32) -> Option<Value> {
    let found = body["cells"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["card"] == json!(k(card)) && c["opponent"] == json!(k(opponent)));
    found.map(|c| json!([c["games"], c["wins"], c["winRate"]]))
}

#[sqlx::test]
async fn matchup_stats(pool: PgPool) {
    let app = app(pool.clone());
    assert_eq!(send(&app, get("/api/stats/matchups", "")).await.body, json!({ "cards": [], "cells": [] }));
    fixture(&pool).await;

    let res = send(&app, get("/api/stats/matchups", "")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    let body = res.body;
    assert_eq!(body["cards"], json!([5, 1, 6, 7, 8, 2, 3, 4, 9, 10].map(k)));
    assert_eq!(body["cells"].as_array().unwrap().len(), 54);
    assert_eq!(cell(&body, 1, 5), Some(json!([23, 14, 14.0 / 23.0])));
    assert_eq!(cell(&body, 5, 1), Some(json!([23, 8, 8.0 / 23.0])));
    assert_eq!(cell(&body, 2, 6), Some(json!([20, 12, 0.6])));
    assert_eq!(cell(&body, 6, 2), Some(json!([20, 7, 0.35])));
    assert_eq!(cell(&body, 9, 6), Some(json!([3, 2, 2.0 / 3.0])));
    assert_eq!(cell(&body, 6, 9), Some(json!([3, 1, 1.0 / 3.0])));
    assert_eq!(cell(&body, 5, 6), Some(json!([3, 2, 2.0 / 3.0])));
    for (card, opponent) in [(5, 5), (1, 1), (1, 2), (2, 9), (9, 1)] {
        assert_eq!(cell(&body, card, opponent), None, "{card} {opponent}");
    }
    assert_eq!(send(&app, get("/api/stats/matchups?version=1.29", "")).await.body, body);

    let old = send(&app, get("/api/stats/matchups?version=1.27.0", "")).await.body;
    assert_eq!(old["cards"], json!([1, 2, 3, 4, 5, 9, 10].map(k)));
    assert_eq!(old["cells"].as_array().unwrap().len(), 30);
    assert_eq!(cell(&old, 1, 2), Some(json!([2, 2, 1.0])));
    assert_eq!(cell(&old, 2, 1), Some(json!([2, 0, 0.0])));
    let all = send(&app, get("/api/stats/matchups?version=all", "")).await.body;
    assert_eq!(cell(&all, 1, 5), Some(json!([25, 14, 14.0 / 25.0])));
    assert_eq!(send(&app, get("/api/stats/matchups?version=latest", "")).await.status, StatusCode::BAD_REQUEST);
}

fn pace_card(id: u32, short: (i64, i64, Option<f64>), long: (i64, i64, Option<f64>)) -> Value {
    json!({
        "card": k(id), "shortGames": short.0, "shortWins": short.1, "shortWinRate": short.2,
        "longGames": long.0, "longWins": long.1, "longWinRate": long.2
    })
}

#[sqlx::test]
async fn pace_stats(pool: PgPool) {
    let app = app(pool.clone());
    assert_eq!(send(&app, get("/api/stats/pace", "")).await.body, json!({ "turns": [], "median": null, "cards": [] }));
    let deck = |s: u32| (s..s + 4).map(k).collect::<Vec<_>>();
    let (a, b, c) = (deck(1), deck(5), deck(9));
    let n = &mut 0;
    let at = |turns, seat0: &[Uuid], seat1: &[Uuid], winner| M { turns, ..m(seat0, seat1, winner) };
    add(&pool, n, at(6, &a, &b, Some(0)), 3).await;
    add(&pool, n, at(6, &c, &b, Some(0)), 1).await;
    add(&pool, n, at(8, &a, &b, Some(0)), 1).await;
    add(&pool, n, at(8, &b, &a, Some(0)), 1).await;
    add(&pool, n, at(12, &b, &a, Some(0)), 3).await;
    add(&pool, n, at(12, &a, &b, None), 1).await;
    add(&pool, n, M { ai: true, ..at(30, &a, &b, Some(0)) }, 2).await;
    let reversed: Vec<Uuid> = a.iter().rev().copied().collect();
    add(&pool, n, at(30, &a, &reversed, Some(0)), 2).await;
    add(&pool, n, M { version: "1.27.0", ..at(20, &a, &b, Some(0)) }, 2).await;

    let res = send(&app, get("/api/stats/pace", "")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    let short_b = (6, 1, Some(1.0 / 6.0));
    let long_b = (4, 3, Some(0.75));
    let short_a = (5, 4, Some(0.8));
    let long_a = (4, 0, Some(0.0));
    let short_c = (1, 1, Some(1.0));
    let none = (0, 0, None);
    let expected = json!({
        "turns": [{ "turns": 6, "matches": 4 }, { "turns": 8, "matches": 2 }, { "turns": 12, "matches": 4 }],
        "median": 8,
        "cards": [
            pace_card(5, short_b, long_b), pace_card(6, short_b, long_b), pace_card(7, short_b, long_b),
            pace_card(8, short_b, long_b), pace_card(1, short_a, long_a), pace_card(2, short_a, long_a),
            pace_card(3, short_a, long_a), pace_card(4, short_a, long_a), pace_card(9, short_c, none),
            pace_card(10, short_c, none), pace_card(11, short_c, none), pace_card(12, short_c, none),
        ]
    });
    assert_eq!(res.body, expected);
    assert_eq!(send(&app, get("/api/stats/pace?version=1.29", "")).await.body, expected);

    let old = send(&app, get("/api/stats/pace?version=1.27.0", "")).await.body;
    assert_eq!(old["turns"], json!([{ "turns": 20, "matches": 2 }]));
    assert_eq!(old["median"], 20);
    assert_eq!(old["cards"][0], pace_card(1, (2, 2, Some(1.0)), none));
    assert_eq!(old["cards"][4], pace_card(5, (2, 0, Some(0.0)), none));
    let all = send(&app, get("/api/stats/pace?version=all", "")).await.body;
    assert_eq!(all["median"], 8);
    assert_eq!(all["turns"][3], json!({ "turns": 20, "matches": 2 }));
    assert_eq!(all["cards"][0], pace_card(5, short_b, (6, 3, Some(0.5))));
    assert_eq!(send(&app, get("/api/stats/pace?version=latest", "")).await.status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn history_stats(pool: PgPool) {
    let app = app(pool.clone());
    assert_eq!(send(&app, get("/api/stats/history", "")).await.body, json!({ "weeks": [], "versions": [] }));
    let x = ids([
        "059b42d0-2de6-46b1-baab-fc61f0bb2ac3",
        "0c8c383b-2207-4d18-b378-4f2a3795a58e",
        "1c57782f-6347-4b61-b052-a263ce519f5c",
        "0b940993-1523-46b8-ac0b-cde93989e01f",
    ]);
    let y = vec![
        "0d26bb73-4cd0-44fb-aba6-5b14d93d6249".parse().unwrap(),
        "165dce19-5697-4e44-a200-c1478fe44a03".parse().unwrap(),
        "2bb3b47f-7ac9-4a86-9ec9-7db626b6b56f".parse().unwrap(),
        k(1),
    ];
    let n = &mut 0;
    let on = |end: &str, version, winner| M { end: Some(end.into()), version, ..m(&x, &y, winner) };
    add(&pool, n, on("2026-03-04T12:00:00Z", "1.27.0", Some(0)), 1).await;
    add(&pool, n, on("2026-03-08T23:30:00Z", "1.28.1", Some(1)), 1).await;
    add(&pool, n, on("2026-03-09T00:30:00Z", "1.28.1", None), 1).await;
    add(&pool, n, M { ai: true, ..on("2026-03-09T00:30:00Z", "1.28.1", Some(0)) }, 1).await;

    let res = send(&app, get("/api/stats/history", "")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    let week = |week, element, games: i64, wins: i64| json!({ "week": week, "element": element, "games": games, "wins": wins, "winRate": wins as f64 / games as f64 });
    assert_eq!(
        res.body,
        json!({
            "weeks": [
                week("2026-03-02", "earth", 2, 1),
                week("2026-03-02", "fire", 4, 2),
                week("2026-03-02", "shadow", 6, 3),
                week("2026-03-02", "water", 2, 1),
                week("2026-03-09", "earth", 1, 0),
                week("2026-03-09", "fire", 2, 0),
                week("2026-03-09", "shadow", 3, 0),
                week("2026-03-09", "water", 1, 0),
            ],
            "versions": [
                { "version": "1.27.0", "start": "2026-03-04T12:00:00Z" },
                { "version": "1.28.1", "start": "2026-03-08T23:30:00Z" },
            ]
        })
    );
}

fn card_turns(id: u32, turns: &[(i32, i64, i64)]) -> Value {
    let turns: Vec<Value> = turns.iter().map(|&(t, g, w)| json!({ "turns": t, "games": g, "wins": w })).collect();
    json!({ "card": k(id), "turns": turns })
}

#[sqlx::test]
async fn turn_stats(pool: PgPool) {
    let app = app(pool.clone());
    assert_eq!(send(&app, get("/api/stats/turns", "")).await.body, json!({ "cards": [] }));
    let deck = |s: u32| (s..s + 4).map(k).collect::<Vec<_>>();
    let (a, b) = (deck(1), deck(5));
    let c = vec![k(1), k(9), k(10), k(11)];
    let n = &mut 0;
    let at = |turns, seat0: &[Uuid], seat1: &[Uuid], winner| M { turns, ..m(seat0, seat1, winner) };
    add(&pool, n, at(5, &a, &b, Some(0)), 2).await;
    add(&pool, n, at(5, &b, &a, Some(0)), 1).await;
    add(&pool, n, at(9, &a, &b, None), 1).await;
    add(&pool, n, at(9, &c, &b, Some(0)), 1).await;
    let reversed: Vec<Uuid> = a.iter().rev().copied().collect();
    add(&pool, n, at(5, &a, &reversed, Some(0)), 1).await;
    add(&pool, n, M { version: "1.27.0", ..at(12, &a, &b, Some(0)) }, 2).await;

    let res = send(&app, get("/api/stats/turns", "")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    let (a_turns, b_turns) = ([(5, 3, 2), (9, 1, 0)], [(5, 3, 1), (9, 2, 0)]);
    let expected = json!({
        "cards": [
            card_turns(1, &[(5, 3, 2), (9, 2, 1)]),
            card_turns(5, &b_turns),
            card_turns(6, &b_turns),
            card_turns(7, &b_turns),
            card_turns(8, &b_turns),
            card_turns(2, &a_turns),
            card_turns(3, &a_turns),
            card_turns(4, &a_turns),
            card_turns(9, &[(9, 1, 1)]),
            card_turns(10, &[(9, 1, 1)]),
            card_turns(11, &[(9, 1, 1)]),
        ]
    });
    assert_eq!(res.body, expected);
    assert_eq!(send(&app, get("/api/stats/turns?version=1.29", "")).await.body, expected);

    let old = send(&app, get("/api/stats/turns?version=1.27.0", "")).await.body;
    let order: Vec<Value> = old["cards"].as_array().unwrap().iter().map(|c| c["card"].clone()).collect();
    assert_eq!(order, [1, 2, 3, 4, 5, 6, 7, 8].map(|i| json!(k(i))));
    assert_eq!(old["cards"][0], card_turns(1, &[(12, 2, 2)]));
    assert_eq!(old["cards"][4], card_turns(5, &[(12, 2, 0)]));
    let all = send(&app, get("/api/stats/turns?version=all", "")).await.body;
    assert_eq!(all["cards"][0], card_turns(1, &[(5, 3, 2), (9, 2, 1), (12, 2, 2)]));
    assert_eq!(send(&app, get("/api/stats/turns?version=latest", "")).await.status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn overview_stats(pool: PgPool) {
    let app = app(pool.clone());
    let empty = send(&app, get("/api/stats/overview", "")).await.body;
    assert_eq!((&empty["matches"], &empty["players"], &empty["decks"]), (&json!(0), &json!(0), &json!(0)));
    let today = chrono::Utc::now().date_naive();
    let days = |body: &Value| -> Vec<(String, i64)> {
        body["days"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| (d["day"].as_str().unwrap().to_string(), d["matches"].as_i64().unwrap()))
            .collect()
    };
    let empty_days = days(&empty);
    assert_eq!(empty_days.len(), 30);
    assert_eq!(empty_days[0].0, (today - chrono::Days::new(29)).to_string());
    assert_eq!(empty_days[29].0, today.to_string());
    assert!(empty_days.iter().all(|d| d.1 == 0));

    let deck = |s: u32| (s..s + 4).map(k).collect::<Vec<_>>();
    let (a, b, c) = (deck(1), deck(5), deck(9));
    let n = &mut 0;
    let late = format!("{}T23:30:00Z", today - chrono::Days::new(2));
    let early = format!("{}T00:30:00Z", today - chrono::Days::new(1));
    add(&pool, n, M { end: Some(late), ..m(&a, &b, Some(0)) }, 1).await;
    add(&pool, n, M { end: Some(early), ..m(&a, &c, Some(1)) }, 2).await;
    add(&pool, n, M { ai: true, ..m(&a, &b, Some(0)) }, 1).await;
    add(&pool, n, M { version: "1.27.0", days: 40, ..m(&b, &c, Some(0)) }, 1).await;
    let (_, _) = user(&pool, "76561190000000001").await;
    let (_, _) = user(&pool, "76561190000000002").await;
    let (_, _) = user(&pool, "76561190000000003").await;
    sqlx::query(
        "INSERT INTO uploads (game_uid, user_id, seat)
         SELECT 'match-1', id, 0 FROM users WHERE steam_id = '76561190000000001'
         UNION ALL SELECT 'match-1', id, 1 FROM users WHERE steam_id = '76561190000000002'
         UNION ALL SELECT 'match-2', id, 0 FROM users WHERE steam_id = '76561190000000001'
         UNION ALL SELECT 'match-4', id, 0 FROM users WHERE steam_id = '76561190000000003'
         UNION ALL SELECT 'match-5', id, 0 FROM users WHERE steam_id = '76561190000000003'",
    )
    .execute(&pool)
    .await
    .unwrap();

    let res = send(&app, get("/api/stats/overview", "")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    assert_eq!((&res.body["matches"], &res.body["players"], &res.body["decks"]), (&json!(3), &json!(2), &json!(3)));
    let got = days(&res.body);
    assert_eq!(got.len(), 30);
    assert_eq!(got[27], ((today - chrono::Days::new(2)).to_string(), 1));
    assert_eq!(got[28], ((today - chrono::Days::new(1)).to_string(), 2));
    assert_eq!(got.iter().map(|d| d.1).sum::<i64>(), 3);

    let old = send(&app, get("/api/stats/overview?version=1.27.0", "")).await.body;
    assert_eq!((&old["matches"], &old["players"], &old["decks"]), (&json!(1), &json!(1), &json!(2)));
    assert!(days(&old).iter().all(|d| d.1 == 0));
    let all = send(&app, get("/api/stats/overview?version=all", "")).await.body;
    assert_eq!((&all["matches"], &all["players"], &all["decks"]), (&json!(4), &json!(3), &json!(3)));
    assert_eq!(send(&app, get("/api/stats/overview?version=latest", "")).await.status, StatusCode::BAD_REQUEST);
}

fn duo(a: u32, b: u32, games: i64, wins: i64) -> Value {
    json!({
        "cards": [k(a), k(b)], "games": games, "wins": wins,
        "winRate": wins as f64 / games as f64, "winScore": wilson(games, wins)
    })
}

#[sqlx::test]
async fn duo_stats(pool: PgPool) {
    let app = app(pool.clone());
    assert_eq!(send(&app, get("/api/stats/duos", "")).await.body, json!([]));
    let deck = |s: u32| (s..s + 4).map(k).collect::<Vec<_>>();
    let (a, b, c) = (deck(1), deck(5), deck(9));
    let d = vec![k(1), k(2), k(13), k(14)];
    let b2 = vec![k(5), k(6), k(15), k(16)];
    let c2 = vec![k(9), k(10), k(17), k(18)];
    let e = deck(19);
    let n = &mut 0;
    add(&pool, n, m(&a, &b, Some(0)), 6).await;
    add(&pool, n, m(&a, &b, Some(1)), 4).await;
    add(&pool, n, m(&a, &c, Some(0)), 6).await;
    add(&pool, n, m(&a, &c, Some(1)), 4).await;
    add(&pool, n, m(&d, &b2, Some(0)), 10).await;
    add(&pool, n, m(&c2, &e, Some(0)), 5).await;
    add(&pool, n, m(&c2, &e, Some(1)), 4).await;
    add(&pool, n, M { version: "1.27.0", ..m(&b, &c, Some(0)) }, 10).await;
    add(&pool, n, M { version: "1.27.0", ..m(&b2, &c2, Some(0)) }, 10).await;

    let res = send(&app, get("/api/stats/duos", "")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    let expected = json!([duo(1, 2, 30, 22), duo(5, 6, 20, 4)]);
    assert_eq!(rounded(res.body), rounded(expected.clone()));
    assert_eq!(rounded(send(&app, get("/api/stats/duos?version=1.29", "")).await.body), rounded(expected));

    let old = send(&app, get("/api/stats/duos?version=1.27.0", "")).await.body;
    assert_eq!(rounded(old), rounded(json!([duo(5, 6, 20, 20), duo(9, 10, 20, 0)])));
    let all = send(&app, get("/api/stats/duos?version=all", "")).await.body;
    assert_eq!(rounded(json!(all.as_array().unwrap()[..2])), rounded(json!([duo(1, 2, 30, 22), duo(5, 6, 40, 24)])));
    assert_eq!((&all[2]["cards"], &all[2]["games"], &all[2]["wins"]), (&json!([k(9), k(10)]), &json!(39), &json!(9)));
    assert_eq!(all.as_array().unwrap().len(), 3);
    assert_eq!(send(&app, get("/api/stats/duos?version=latest", "")).await.status, StatusCode::BAD_REQUEST);
}

fn counter(id: u32, games: i64, wins: i64) -> Value {
    json!({
        "card": k(id), "games": games, "wins": wins,
        "winRate": wins as f64 / games as f64, "winScore": wilson(games, wins)
    })
}

#[sqlx::test]
async fn counter_stats(pool: PgPool) {
    let app = app(pool.clone());
    let q = |query: &str| get(&format!("/api/stats/counters?{query}"), "");
    let one = format!("against={}", k(1));
    let two = format!("against={},{}", k(1), k(2));
    assert_eq!(send(&app, q(&one)).await.body, json!({ "games": 0, "cards": [] }));
    let deck = |s: u32| (s..s + 4).map(k).collect::<Vec<_>>();
    let (a, b, d) = (deck(1), deck(5), deck(14));
    let c = vec![k(1), k(9), k(10), k(11)];
    let n = &mut 0;
    add(&pool, n, m(&a, &b, Some(1)), 6).await;
    add(&pool, n, m(&a, &b, Some(0)), 4).await;
    add(&pool, n, m(&c, &b, Some(1)), 5).await;
    add(&pool, n, m(&a, &c, Some(1)), 7).await;
    add(&pool, n, m(&a, &c, Some(0)), 3).await;
    add(&pool, n, m(&d, &a, Some(0)), 9).await;
    add(&pool, n, M { version: "1.27.0", ..m(&a, &b, Some(0)) }, 10).await;

    let res = send(&app, q(&one)).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    let b_cards = |g, w| (5..9).map(|i| counter(i, g, w)).collect::<Vec<_>>();
    let c_cards = |g, w| (9..12).map(|i| counter(i, g, w)).collect::<Vec<_>>();
    let a_cards = |g, w| (2..5).map(|i| counter(i, g, w)).collect::<Vec<_>>();
    let cards = [b_cards(15, 11), c_cards(10, 7), a_cards(10, 3)].concat();
    assert_eq!(rounded(res.body), rounded(json!({ "games": 44, "cards": cards })));

    let both = send(&app, q(&two)).await.body;
    let cards = [c_cards(10, 7), b_cards(10, 6)].concat();
    assert_eq!(rounded(both), rounded(json!({ "games": 29, "cards": cards })));
    let reversed = send(&app, q(&format!("against={},{}&version=1.29", k(2), k(1)))).await.body;
    assert_eq!(reversed["games"], 29);

    let old = send(&app, q(&format!("{one}&version=1.27.0"))).await.body;
    assert_eq!(rounded(old), rounded(json!({ "games": 10, "cards": b_cards(10, 0) })));
    let all = send(&app, q(&format!("{one}&version=all"))).await.body;
    assert_eq!(all["games"], 54);
    assert!(rounded(all["cards"].clone()).as_array().unwrap().contains(&rounded(counter(5, 25, 11))));

    let five = (1..6).map(|i| k(i).to_string()).collect::<Vec<_>>().join(",");
    for bad in [
        String::new(),
        "against=".into(),
        format!("against={five}"),
        "against=nope".into(),
        format!("against={},{}", k(1), k(1)),
        format!("{one}&version=latest"),
    ] {
        assert_eq!(send(&app, q(&bad)).await.status, StatusCode::BAD_REQUEST, "{bad}");
    }
}
