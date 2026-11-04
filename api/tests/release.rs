mod common;

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use api::AppState;
use axum::{
    Router,
    http::{HeaderMap, StatusCode, header},
    routing,
};
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;

async fn mock_github(status: StatusCode, body: String) -> (Router, Arc<AtomicUsize>) {
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = hits.clone();
    let mock = Router::new().route(
        "/releases/latest",
        routing::get(move |headers: HeaderMap| async move {
            counter.fetch_add(1, Ordering::SeqCst);
            let ok = headers.get(header::ACCEPT).is_some_and(|v| v == "application/vnd.github+json")
                && headers.contains_key(header::USER_AGENT);
            if ok { (status, body) } else { (StatusCode::BAD_REQUEST, String::new()) }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });
    (app_for(&format!("http://{addr}/releases/latest")), hits)
}

fn app_for(url: &str) -> Router {
    let mut config = config("https://steamcommunity.com/openid/login");
    config.github_release_url = url.into();
    let pool = PgPool::connect_lazy("postgres://127.0.0.1:9/none").unwrap();
    api::router(AppState::new(pool, config, real_catalog()))
}

fn asset(name: &str, size: u64) -> Value {
    json!({
        "name": name,
        "browser_download_url": format!("https://github.com/elwqnn/spam-live/releases/download/desktop-v0.2.0/{name}"),
        "size": size,
        "content_type": "application/octet-stream"
    })
}

fn github_release(tag: &str, assets: Vec<Value>) -> String {
    json!({
        "tag_name": tag,
        "name": "SPAM Live",
        "draft": false,
        "published_at": "2026-11-01T12:30:00Z",
        "html_url": format!("https://github.com/elwqnn/spam-live/releases/tag/{tag}"),
        "assets": assets
    })
    .to_string()
}

async fn release(app: &Router) -> Res {
    send(app, get("/api/desktop/release", "")).await
}

#[tokio::test]
async fn parses_the_latest_release_and_caches_it() {
    let body = github_release(
        "desktop-v0.2.0",
        vec![
            asset("SPAM.Live_0.2.0_amd64.AppImage.sig", 400),
            asset("SPAM.Live_0.2.0_amd64.AppImage", 88_000_000),
            asset("SPAM.Live_0.2.0_x64-setup.exe", 4_200_000),
            asset("latest.json", 900),
        ],
    );
    let (app, hits) = mock_github(StatusCode::OK, body).await;
    let res = release(&app).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers[header::CACHE_CONTROL], "public, max-age=300");
    let base = "https://github.com/elwqnn/spam-live/releases";
    assert_eq!(
        res.body,
        json!({
            "version": "0.2.0",
            "publishedAt": "2026-11-01T12:30:00Z",
            "url": format!("{base}/tag/desktop-v0.2.0"),
            "windows": {
                "name": "SPAM.Live_0.2.0_x64-setup.exe",
                "url": format!("{base}/download/desktop-v0.2.0/SPAM.Live_0.2.0_x64-setup.exe"),
                "size": 4_200_000
            },
            "linux": {
                "name": "SPAM.Live_0.2.0_amd64.AppImage",
                "url": format!("{base}/download/desktop-v0.2.0/SPAM.Live_0.2.0_amd64.AppImage"),
                "size": 88_000_000
            }
        })
    );
    let again = release(&app).await;
    assert_eq!(again.status, StatusCode::OK);
    assert_eq!(again.body, res.body);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn missing_or_unsafe_asset_is_null() {
    let mut exe = asset("SPAM.Live_0.3.0_x64-setup.exe", 10);
    exe["browser_download_url"] = json!("javascript:alert(1)");
    let body = github_release("0.3.0", vec![exe, asset("SPAM.Live_0.3.0_amd64.deb", 10)]);
    let (app, _) = mock_github(StatusCode::OK, body).await;
    let res = release(&app).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["version"], "0.3.0");
    assert_eq!(res.body["windows"], Value::Null);
    assert_eq!(res.body["linux"], Value::Null);
}

#[tokio::test]
async fn no_release_is_404_and_cached() {
    let (app, hits) = mock_github(StatusCode::NOT_FOUND, r#"{"message":"Not Found"}"#.into()).await;
    for _ in 0..2 {
        let res = release(&app).await;
        assert_eq!(res.status, StatusCode::NOT_FOUND);
        assert_eq!(res.body["error"], "no release");
        assert!(res.headers.get(header::CACHE_CONTROL).is_none());
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn github_error_is_502_without_its_text() {
    let (app, hits) = mock_github(StatusCode::FORBIDDEN, "GitHub secret rate limit text".into()).await;
    for _ in 0..2 {
        let res = release(&app).await;
        assert_eq!(res.status, StatusCode::BAD_GATEWAY);
        assert_eq!(res.body["error"], "release info is not available");
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);

    let (app, _) = mock_github(StatusCode::OK, "not json".into()).await;
    assert_eq!(release(&app).await.status, StatusCode::BAD_GATEWAY);

    let mut body: Value = serde_json::from_str(&github_release("desktop-v1.0.0", vec![])).unwrap();
    body["html_url"] = json!("http://evil.example/");
    let (app, _) = mock_github(StatusCode::OK, body.to_string()).await;
    assert_eq!(release(&app).await.status, StatusCode::BAD_GATEWAY);

    assert_eq!(release(&app_for("http://127.0.0.1:9/releases/latest")).await.status, StatusCode::BAD_GATEWAY);
}
