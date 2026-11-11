mod common;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use api::{
    AppState,
    analytics::utc_hour,
    steam::{poll_news, poll_players},
};
use axum::{
    Router,
    extract::Query,
    http::{StatusCode, header},
    routing,
};
use chrono::{DateTime, DurationRound, TimeDelta, Utc};
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;

type Reply = Arc<Mutex<(StatusCode, String)>>;

fn set(reply: &Reply, status: StatusCode, body: impl Into<String>) {
    *reply.lock().unwrap() = (status, body.into());
}

fn route(reply: Reply, expected: &'static [(&'static str, &'static str)]) -> routing::MethodRouter {
    routing::get(move |Query(q): Query<HashMap<String, String>>| async move {
        if expected.iter().any(|(k, v)| q.get(*k).map(String::as_str) != Some(*v)) {
            return (StatusCode::BAD_REQUEST, String::new());
        }
        reply.lock().unwrap().clone()
    })
}

async fn mock(pool: PgPool) -> (AppState, Router, Reply, Reply) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let players: Reply = Arc::new(Mutex::new((StatusCode::OK, String::new())));
    let news: Reply = Arc::new(Mutex::new((StatusCode::OK, String::new())));
    let server = Router::new()
        .route("/ISteamUserStats/GetNumberOfCurrentPlayers/v1/", route(players.clone(), &[("appid", "2902720")]))
        .route("/ISteamNews/GetNewsForApp/v2/", route(news.clone(), &[("appid", "2902720"), ("count", "20")]));
    tokio::spawn(async move { axum::serve(listener, server).await.unwrap() });
    let mut config = config("https://steamcommunity.com/openid/login");
    config.steam_api_url = base;
    let state = AppState::new(pool, config, real_catalog());
    (state.clone(), api::router(state), players, news)
}

fn count_body(players: i64) -> String {
    json!({ "response": { "player_count": players, "result": 1 } }).to_string()
}

fn item(gid: &str, title: &str, url: &str, date: i64) -> Value {
    json!({
        "gid": gid,
        "title": title,
        "url": url,
        "is_external_url": true,
        "author": "Studio author",
        "contents": "Hidden body text of the patch notes",
        "feedlabel": "Community Announcements",
        "date": date,
        "feedname": "steam_community_announcements",
        "feed_type": 1,
        "appid": 2902720
    })
}

fn news_body(items: Vec<Value>) -> String {
    json!({ "appnews": { "appid": 2902720, "newsitems": items, "count": 3 } }).to_string()
}

async fn rows(pool: &PgPool, table: &str) -> i64 {
    let sql = match table {
        "steam_players" => "SELECT count(*) FROM steam_players",
        "steam_news" => "SELECT count(*) FROM steam_news",
        _ => unreachable!(),
    };
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

async fn get_ok(app: &Router, uri: &str) -> Value {
    let res = send(app, get(uri, "")).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.body);
    assert_eq!(res.headers[header::CACHE_CONTROL], "public, max-age=300");
    res.body
}

fn hour_max(body: &Value, hour: DateTime<Utc>) -> Value {
    let hourly = body["hourly"].as_array().unwrap();
    let found = hourly.iter().find(|h| h["hour"].as_str().unwrap().parse::<DateTime<Utc>>().unwrap() == hour);
    found.unwrap_or_else(|| panic!("no hour {hour}"))["max"].clone()
}

fn now() -> DateTime<Utc> {
    Utc::now().duration_trunc(TimeDelta::seconds(1)).unwrap()
}

#[sqlx::test]
async fn players_poll_stores_one_row_per_time(pool: PgPool) {
    let (state, app, players, _) = mock(pool.clone()).await;
    let empty = get_ok(&app, "/api/steam/players").await;
    assert_eq!(empty["current"], Value::Null);
    assert_eq!(empty["hourly"].as_array().unwrap().len(), 168);
    assert!(empty["hourly"].as_array().unwrap().iter().all(|h| h["max"].is_null()));

    let at = now();
    set(&players, StatusCode::OK, count_body(911));
    poll_players(&state, at).await;
    assert_eq!(rows(&pool, "steam_players").await, 1);
    set(&players, StatusCode::OK, count_body(999));
    poll_players(&state, at).await;
    assert_eq!(rows(&pool, "steam_players").await, 1);

    let earlier = utc_hour(at) - TimeDelta::hours(2);
    for (minutes, n) in [(5, 500), (15, 700), (25, 600)] {
        set(&players, StatusCode::OK, count_body(n));
        poll_players(&state, earlier + TimeDelta::minutes(minutes)).await;
    }
    set(&players, StatusCode::OK, count_body(4000));
    poll_players(&state, at - TimeDelta::days(8)).await;
    poll_players(&state, at - TimeDelta::days(31)).await;
    assert_eq!(rows(&pool, "steam_players").await, 6);

    let body = get_ok(&app, "/api/steam/players").await;
    assert_eq!(body["current"]["players"], 911);
    let hourly = body["hourly"].as_array().unwrap();
    assert_eq!(hourly.len(), 168);
    let hours: Vec<DateTime<Utc>> = hourly.iter().map(|h| h["hour"].as_str().unwrap().parse().unwrap()).collect();
    assert!(hours.windows(2).all(|w| w[1] - w[0] == TimeDelta::hours(1)));
    assert_eq!(hour_max(&body, utc_hour(at)), 911);
    assert_eq!(hour_max(&body, earlier), 700);
    assert_eq!(hour_max(&body, earlier + TimeDelta::hours(1)), Value::Null);
    assert!(hourly.iter().all(|h| h["max"] != 4000));

    api::cleanup(&pool).await.unwrap();
    assert_eq!(rows(&pool, "steam_players").await, 5);
}

#[sqlx::test]
async fn news_poll_upserts_items_without_the_body(pool: PgPool) {
    let (state, app, _, news) = mock(pool.clone()).await;
    assert_eq!(get_ok(&app, "/api/steam/news").await, json!([]));

    let community = "https://steamcommunity.com/games/2902720/announcements/detail/1002";
    let items = |first: &str| {
        vec![
            item(
                "1001",
                first,
                "https://steamstore-a.akamaihd.net/news/externalpost/steam_community/1001",
                1_793_818_334,
            ),
            item("1002", "Patch 1.2", community, 1_793_000_000),
            item("1003", "Version 11.29 is out", "javascript:alert(1)", 1_794_000_000),
            item("../1004", "Patch 1.29", community, 1_794_000_001),
        ]
    };
    set(&news, StatusCode::OK, news_body(items("Patch 1.29: Balance and fixes")));
    poll_news(&state).await;
    assert_eq!(rows(&pool, "steam_news").await, 3);

    let columns: Vec<String> = sqlx::query_scalar(
        "SELECT column_name::text FROM information_schema.columns WHERE table_name = 'steam_news' ORDER BY ordinal_position",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(columns, ["gid", "title", "url", "published_at", "feed"]);
    let leaked: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM steam_news WHERE concat_ws(' ', title, url, feed) ~ 'Hidden body|Studio author'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(leaked, 0);

    let body = get_ok(&app, "/api/steam/news").await;
    let view = "https://store.steampowered.com/news/app/2902720/view";
    assert_eq!(
        body,
        json!([
            {
                "gid": "1003",
                "title": "Version 11.29 is out",
                "url": format!("{view}/1003"),
                "publishedAt": "2026-11-06T21:20:00Z",
                "feed": "Community Announcements",
                "version": null
            },
            {
                "gid": "1001",
                "title": "Patch 1.29: Balance and fixes",
                "url": format!("{view}/1001"),
                "publishedAt": "2026-11-04T18:52:14Z",
                "feed": "Community Announcements",
                "version": "1.29"
            },
            {
                "gid": "1002",
                "title": "Patch 1.2",
                "url": community,
                "publishedAt": "2026-10-26T07:33:20Z",
                "feed": "Community Announcements",
                "version": null
            }
        ])
    );

    set(&news, StatusCode::OK, news_body(items("Patch 1.29 and 1.28.1: new cards")));
    poll_news(&state).await;
    assert_eq!(rows(&pool, "steam_news").await, 3);
    let body = get_ok(&app, "/api/steam/news").await;
    assert_eq!(body[1]["title"], "Patch 1.29 and 1.28.1: new cards");
    assert_eq!(body[1]["version"], "1.29");
}

#[sqlx::test]
async fn news_poll_skips_bad_items_and_links_versions(pool: PgPool) {
    let (state, app, _, news) = mock(pool.clone()).await;
    let url = "https://steamcommunity.com/games/2902720/announcements/detail/1";
    let mut no_url = item("3001", "Patch 1.29", url, 1_793_000_001);
    no_url.as_object_mut().unwrap().remove("url");
    let mut bad_title = item("3002", "Patch 1.29", url, 1_793_000_002);
    bad_title["title"] = json!(42);
    let items = vec![
        no_url,
        bad_title,
        item("3003", "Patch 1.28: Secret of the Oniyx", url, 1_793_000_003),
        item("3004", "Hotfix 11.28", url, 1_793_000_004),
        item("3005", "From 1.28 to 1.29", url, 1_793_000_005),
        item("3006", "Patch 1.28.1.2", url, 1_793_000_006),
        item("3007", "1 new card", url, 1_793_000_007),
    ];
    set(&news, StatusCode::OK, news_body(items));
    poll_news(&state).await;
    assert_eq!(rows(&pool, "steam_news").await, 5);

    let body = get_ok(&app, "/api/steam/news").await;
    let versions: Vec<(&str, &Value)> =
        body.as_array().unwrap().iter().map(|n| (n["gid"].as_str().unwrap(), &n["version"])).collect();
    assert_eq!(
        versions,
        [
            ("3007", &Value::Null),
            ("3006", &Value::Null),
            ("3005", &json!("1.29")),
            ("3004", &Value::Null),
            ("3003", &json!("1.28.1")),
        ]
    );
}

#[sqlx::test]
async fn steam_errors_keep_the_stored_values(pool: PgPool) {
    let (state, app, players, news) = mock(pool.clone()).await;
    set(&players, StatusCode::OK, count_body(911));
    poll_players(&state, now() - TimeDelta::minutes(1)).await;
    set(
        &news,
        StatusCode::OK,
        news_body(vec![item("1001", "Patch 1.29", "https://steamcommunity.com/x", 1_793_818_334)]),
    );
    poll_news(&state).await;
    let stored_players = get_ok(&app, "/api/steam/players").await;
    let stored_news = get_ok(&app, "/api/steam/news").await;
    assert_eq!(stored_players["current"]["players"], 911);
    assert_eq!(stored_news.as_array().unwrap().len(), 1);

    let bad_players = [
        (StatusCode::INTERNAL_SERVER_ERROR, count_body(5)),
        (StatusCode::OK, "not json".into()),
        (StatusCode::OK, json!({ "response": { "player_count": 5, "result": 42 } }).to_string()),
        (StatusCode::OK, json!({ "response": { "result": 1 } }).to_string()),
        (StatusCode::OK, count_body(-5)),
    ];
    for (status, body) in bad_players {
        set(&players, status, body);
        poll_players(&state, now()).await;
    }
    let bad_news = [
        (StatusCode::INTERNAL_SERVER_ERROR, news_body(vec![item("2001", "New", "https://steamcommunity.com/y", 1)])),
        (StatusCode::OK, "{\"appnews\":".into()),
        (StatusCode::OK, json!({ "response": {} }).to_string()),
    ];
    for (status, body) in bad_news {
        set(&news, status, body);
        poll_news(&state).await;
    }

    assert_eq!(rows(&pool, "steam_players").await, 1);
    assert_eq!(rows(&pool, "steam_news").await, 1);
    assert_eq!(get_ok(&app, "/api/steam/players").await, stored_players);
    assert_eq!(get_ok(&app, "/api/steam/news").await, stored_news);
}
