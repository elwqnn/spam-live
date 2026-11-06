mod common;

use std::{sync::Arc, time::Duration};

use api::live::Live;
use axum::{
    Router,
    body::BodyDataStream,
    http::{StatusCode, header},
};
use common::*;
use futures_util::StreamExt;
use serde_json::json;
use sqlx::PgPool;
use tokio::time::timeout;
use tower::ServiceExt;

async fn open(app: &Router) -> BodyDataStream {
    let res = app.clone().oneshot(get("/api/decks/events", "")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers()[header::CONTENT_TYPE], "text/event-stream");
    assert_eq!(res.headers()[header::CACHE_CONTROL], "no-store");
    let mut body = res.into_body().into_data_stream();
    let first = timeout(Duration::from_secs(1), body.next()).await.expect("no first frame").unwrap().unwrap();
    assert_eq!(first, "retry: 5000\n\n");
    body
}

async fn read(body: &mut BodyDataStream, n: usize) -> Vec<String> {
    let mut text = String::new();
    while text.matches("\n\n").count() < n {
        let chunk = timeout(Duration::from_secs(5), body.next()).await.expect("no event").unwrap().unwrap();
        text.push_str(std::str::from_utf8(&chunk).unwrap());
    }
    text.split_terminator("\n\n").map(str::to_string).collect()
}

fn signals(ids: &[i64]) -> Vec<String> {
    ids.iter().map(|id| format!("event: decks\ndata: {id}")).collect()
}

async fn create(app: &Router, cookie: &str) -> i64 {
    let body = json!({ "name": "D", "cards": SEAT0_CARDS, "talents": [0, 0, 0, 0] });
    let res = send(app, req("POST", "/api/decks", cookie, Some(body))).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.body);
    res.body["id"].as_i64().unwrap()
}

async fn call(app: &Router, cookie: &str, method: &str, uri: String, body: Option<serde_json::Value>) {
    let res = send(app, req(method, &uri, cookie, body)).await;
    assert!(res.status.is_success(), "{method} {uri}: {} {}", res.status, res.body);
}

#[sqlx::test]
async fn public_deck_changes_send_signals(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let (_, b) = user(&pool, "76561190000000002").await;
    let (_, admin) = admin(&pool, "76561190000000009").await;
    let mut events = open(&app).await;

    let first = create(&app, &a).await;
    call(&app, &a, "POST", format!("/api/decks/{first}/publish"), None).await;
    call(&app, &a, "POST", format!("/api/decks/{first}/publish"), None).await;
    call(&app, &a, "PATCH", format!("/api/decks/{first}"), Some(json!({ "name": "New name" }))).await;
    call(&app, &a, "POST", format!("/api/decks/{first}/unpublish"), None).await;
    call(&app, &a, "POST", format!("/api/decks/{first}/unpublish"), None).await;
    call(&app, &a, "PATCH", format!("/api/decks/{first}"), Some(json!({ "name": "Draft" }))).await;
    call(&app, &a, "DELETE", format!("/api/decks/{first}"), None).await;

    let second = create(&app, &a).await;
    call(&app, &a, "POST", format!("/api/decks/{second}/publish"), None).await;
    call(&app, &a, "DELETE", format!("/api/decks/{second}"), None).await;

    let third = create(&app, &a).await;
    call(&app, &a, "POST", format!("/api/decks/{third}/publish"), None).await;
    call(&app, &b, "PUT", format!("/api/decks/{third}/vote"), Some(json!({ "value": 1 }))).await;
    call(&app, &b, "POST", format!("/api/decks/{third}/comments"), Some(json!({ "body": "nice" }))).await;
    let report = json!({ "targetType": "deck", "targetId": third, "reason": "bad" });
    call(&app, &b, "POST", "/api/reports".into(), Some(report)).await;
    let reports = send(&app, get("/api/admin/reports", &admin)).await.body;
    let resolve = json!({ "action": "delete" });
    call(&app, &admin, "POST", format!("/api/admin/reports/{}/resolve", reports[0]["id"]), Some(resolve)).await;

    let expected = signals(&[first, first, first, second, second, third, third]);
    assert_eq!(read(&mut events, expected.len()).await, expected);
}

#[sqlx::test]
async fn account_deletion_sends_signal(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let deck = create(&app, &a).await;
    call(&app, &a, "POST", format!("/api/decks/{deck}/publish"), None).await;
    let mut events = open(&app).await;

    call(&app, &a, "DELETE", "/api/me".into(), None).await;

    assert_eq!(read(&mut events, 1).await, signals(&[0]));
}

#[sqlx::test]
async fn draft_deck_sends_no_signal(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let mut events = open(&app).await;

    let draft = create(&app, &a).await;
    call(&app, &a, "PATCH", format!("/api/decks/{draft}"), Some(json!({ "name": "Other" }))).await;
    call(&app, &a, "DELETE", format!("/api/decks/{draft}"), None).await;
    let public = create(&app, &a).await;
    call(&app, &a, "POST", format!("/api/decks/{public}/publish"), None).await;

    assert_eq!(read(&mut events, 1).await, signals(&[public]));
}

#[sqlx::test]
async fn stream_limit(pool: PgPool) {
    let mut state = state_with(pool, "https://steamcommunity.com/openid/login", real_catalog());
    state.live = Arc::new(Live::new(1));
    let app = api::router(state);

    let held = open(&app).await;
    let res = send(&app, get("/api/decks/events", "")).await;
    assert_eq!(res.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(res.body, json!({ "error": "too many open streams" }));
    drop(held);
    drop(open(&app).await);
}

#[sqlx::test]
async fn lag_and_close(pool: PgPool) {
    let state = state_with(pool, "https://steamcommunity.com/openid/login", real_catalog());
    let live = state.live.clone();
    let app = api::router(state);
    let mut events = open(&app).await;

    for id in 1..=20 {
        live.notify(id);
    }
    live.close();

    let mut expected = signals(&[0]);
    expected.extend(signals(&(5..=20).collect::<Vec<_>>()));
    assert_eq!(read(&mut events, expected.len()).await, expected);
    assert!(timeout(Duration::from_secs(5), events.next()).await.unwrap().is_none());
}
