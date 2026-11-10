mod common;

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use api::AppState;
use axum::{
    Router,
    extract::Path,
    http::{HeaderMap, StatusCode, header},
    routing,
};
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;

async fn mock_github(status: StatusCode, body: impl FnOnce(&str) -> String) -> (Router, Arc<AtomicUsize>) {
    let (state, hits) = mock_state(status, body).await;
    (api::router(state), hits)
}

async fn mock_state(status: StatusCode, body: impl FnOnce(&str) -> String) -> (AppState, Arc<AtomicUsize>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let body = body(&base);
    let hits = Arc::new(AtomicUsize::new(0));
    let (release_hits, sig_hits) = (hits.clone(), hits.clone());
    let mock = Router::new()
        .route(
            "/releases/latest",
            routing::get(move |headers: HeaderMap| async move {
                release_hits.fetch_add(1, Ordering::SeqCst);
                let ok = headers.get(header::ACCEPT).is_some_and(|v| v == "application/vnd.github+json")
                    && headers.contains_key(header::USER_AGENT);
                if ok { (status, body) } else { (StatusCode::BAD_REQUEST, String::new()) }
            }),
        )
        .route(
            "/sig/{name}",
            routing::get(move |Path(name): Path<String>, headers: HeaderMap| async move {
                sig_hits.fetch_add(1, Ordering::SeqCst);
                if !headers.contains_key(header::USER_AGENT) || name.starts_with("fail") {
                    return (StatusCode::INTERNAL_SERVER_ERROR, String::new());
                }
                let text = match name {
                    n if n.starts_with("large") => "A".repeat(5000),
                    n if n.starts_with("bad") => "not base64!".into(),
                    n => format!("{}\n", signature_of(&n)),
                };
                (StatusCode::OK, text)
            }),
        );
    tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });
    (state_for(&format!("{base}/releases/latest")), hits)
}

fn signature_of(name: &str) -> String {
    name.chars().filter(char::is_ascii_alphanumeric).collect()
}

fn sig(base: &str, name: &str) -> Value {
    json!({
        "name": name,
        "browser_download_url": format!("{base}/sig/{name}"),
        "size": 400,
        "content_type": "application/octet-stream"
    })
}

fn state_for(url: &str) -> AppState {
    let mut config = config("https://steamcommunity.com/openid/login");
    config.github_release_url = url.into();
    let pool = PgPool::connect_lazy("postgres://127.0.0.1:9/none").unwrap();
    AppState::new(pool, config, real_catalog())
}

fn app_for(url: &str) -> Router {
    api::router(state_for(url))
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
    let body = |base: &str| {
        github_release(
            "desktop-v0.2.0",
            vec![
                sig(base, "SPAM.Live_0.2.0_amd64.AppImage.sig"),
                asset("SPAM.Live_0.2.0_amd64.AppImage", 88_000_000),
                asset("SPAM.Live_0.2.0_x64-setup.exe", 4_200_000),
                asset("latest.json", 900),
            ],
        )
    };
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
    assert_eq!(hits.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn missing_or_unsafe_asset_is_null() {
    let mut exe = asset("SPAM.Live_0.3.0_x64-setup.exe", 10);
    exe["browser_download_url"] = json!("javascript:alert(1)");
    let body = github_release("0.3.0", vec![exe, asset("SPAM.Live_0.3.0_amd64.deb", 10)]);
    let (app, _) = mock_github(StatusCode::OK, |_| body).await;
    let res = release(&app).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["version"], "0.3.0");
    assert_eq!(res.body["windows"], Value::Null);
    assert_eq!(res.body["linux"], Value::Null);
}

#[tokio::test]
async fn no_release_is_404_and_cached() {
    let (app, hits) = mock_github(StatusCode::NOT_FOUND, |_| r#"{"message":"Not Found"}"#.into()).await;
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
    let (app, hits) = mock_github(StatusCode::FORBIDDEN, |_| "GitHub secret rate limit text".into()).await;
    for _ in 0..2 {
        let res = release(&app).await;
        assert_eq!(res.status, StatusCode::BAD_GATEWAY);
        assert_eq!(res.body["error"], "release info is not available");
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);

    let (app, _) = mock_github(StatusCode::OK, |_| "not json".into()).await;
    assert_eq!(release(&app).await.status, StatusCode::BAD_GATEWAY);

    let mut body: Value = serde_json::from_str(&github_release("desktop-v1.0.0", vec![])).unwrap();
    body["html_url"] = json!("http://evil.example/");
    let (app, _) = mock_github(StatusCode::OK, |_| body.to_string()).await;
    assert_eq!(release(&app).await.status, StatusCode::BAD_GATEWAY);

    assert_eq!(release(&app_for("http://127.0.0.1:9/releases/latest")).await.status, StatusCode::BAD_GATEWAY);
}

const APPIMAGE: &str = "SPAM.Live_0.2.0_amd64.AppImage";
const SETUP: &str = "SPAM.Live_0.2.0_x64-setup.exe";

fn full_release(base: &str) -> String {
    let mut body: Value = serde_json::from_str(&github_release(
        "desktop-v0.2.0",
        vec![
            asset(APPIMAGE, 88_000_000),
            sig(base, &format!("{APPIMAGE}.sig")),
            asset(SETUP, 4_200_000),
            sig(base, &format!("{SETUP}.sig")),
        ],
    ))
    .unwrap();
    body["body"] = json!("Fixes and new cards.");
    body.to_string()
}

async fn update(app: &Router, path: &str) -> Res {
    send(app, get(&format!("/api/desktop/update/{path}"), "")).await
}

#[tokio::test]
async fn newer_release_gives_the_update_json_and_caches_it() {
    let (app, hits) = mock_github(StatusCode::OK, full_release).await;
    let base = "https://github.com/elwqnn/spam-live/releases/download/desktop-v0.2.0";
    let res = update(&app, "linux/x86_64/0.1.0").await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers[header::CACHE_CONTROL], "public, max-age=300");
    assert_eq!(
        res.body,
        json!({
            "version": "0.2.0",
            "notes": "Fixes and new cards.",
            "pub_date": "2026-11-01T12:30:00Z",
            "url": format!("{base}/{APPIMAGE}"),
            "signature": signature_of(&format!("{APPIMAGE}.sig"))
        })
    );
    let res = update(&app, "windows/x86_64/0.1.9").await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["url"], format!("{base}/{SETUP}"));
    assert_eq!(res.body["signature"], signature_of(&format!("{SETUP}.sig")));
    assert_eq!(update(&app, "linux/x86_64/0.1").await.status, StatusCode::OK);
    assert_eq!(release(&app).await.status, StatusCode::OK);
    assert_eq!(hits.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn equal_or_older_release_is_204() {
    let (app, _) = mock_github(StatusCode::OK, full_release).await;
    for path in [
        "linux/x86_64/0.2.0",
        "linux/x86_64/0.2",
        "windows/x86_64/0.2.0.0",
        "linux/x86_64/0.3.0",
        "windows/x86_64/0.10.0",
        "linux/x86_64/1.0.0",
    ] {
        let res = update(&app, path).await;
        assert_eq!(res.status, StatusCode::NO_CONTENT, "{path}");
        assert_eq!(res.body, Value::Null, "{path}");
    }
}

#[tokio::test]
async fn missing_asset_or_signature_is_204() {
    let body = |base: &str| {
        github_release(
            "desktop-v0.2.0",
            vec![
                asset(APPIMAGE, 10),
                asset(&format!("{APPIMAGE}.tar.gz"), 10),
                sig(base, &format!("{APPIMAGE}.tar.gz.sig")),
                sig(base, &format!("{SETUP}.sig")),
            ],
        )
    };
    let (app, _) = mock_github(StatusCode::OK, body).await;
    assert_eq!(update(&app, "linux/x86_64/0.1.0").await.status, StatusCode::NO_CONTENT);
    assert_eq!(update(&app, "windows/x86_64/0.1.0").await.status, StatusCode::NO_CONTENT);

    let body = |_: &str| {
        let mut unsafe_sig = sig("http://evil.example", &format!("{APPIMAGE}.sig"));
        unsafe_sig["browser_download_url"] = json!(format!("http://evil.example/{APPIMAGE}.sig"));
        let mut exe = asset(SETUP, 10);
        exe["browser_download_url"] = json!(format!("http://github.com/{SETUP}"));
        github_release("desktop-v0.2.0", vec![asset(APPIMAGE, 10), unsafe_sig, exe])
    };
    let (app, hits) = mock_github(StatusCode::OK, body).await;
    assert_eq!(update(&app, "linux/x86_64/0.1.0").await.status, StatusCode::NO_CONTENT);
    assert_eq!(update(&app, "windows/x86_64/0.1.0").await.status, StatusCode::NO_CONTENT);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn bad_version_or_target_is_400() {
    let (app, hits) = mock_github(StatusCode::OK, full_release).await;
    for (path, error) in [
        ("linux/x86_64/0.1.0-beta", "invalid version"),
        ("linux/x86_64/v0.1.0", "invalid version"),
        ("linux/x86_64/1..2", "invalid version"),
        ("linux/x86_64/%20", "invalid version"),
        ("darwin/x86_64/0.1.0", "unsupported target"),
        ("linux/aarch64/0.1.0", "unsupported target"),
        ("windows/i686/0.1.0", "unsupported target"),
        ("Linux/x86_64/0.1.0", "unsupported target"),
    ] {
        let res = update(&app, path).await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST, "{path}");
        assert_eq!(res.body["error"], error, "{path}");
    }
    assert_eq!(hits.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn github_failure_is_502() {
    let (app, _) = mock_github(StatusCode::INTERNAL_SERVER_ERROR, |_| "GitHub text".into()).await;
    let res = update(&app, "linux/x86_64/0.1.0").await;
    assert_eq!(res.status, StatusCode::BAD_GATEWAY);
    assert_eq!(res.body["error"], "release info is not available");
    assert_eq!(
        update(&app_for("http://127.0.0.1:9/releases/latest"), "linux/x86_64/0.1.0").await.status,
        StatusCode::BAD_GATEWAY
    );

    let (app, _) = mock_github(StatusCode::NOT_FOUND, |_| r#"{"message":"Not Found"}"#.into()).await;
    assert_eq!(update(&app, "linux/x86_64/0.1.0").await.status, StatusCode::NO_CONTENT);

    for prefix in ["fail", "large", "bad"] {
        let body = |base: &str| {
            github_release(
                "desktop-v0.2.0",
                vec![
                    asset(APPIMAGE, 10),
                    {
                        let mut s = sig(base, &format!("{APPIMAGE}.sig"));
                        s["browser_download_url"] = json!(format!("{base}/sig/{prefix}.sig"));
                        s
                    },
                    asset(SETUP, 10),
                    sig(base, &format!("{SETUP}.sig")),
                ],
            )
        };
        let (app, _) = mock_github(StatusCode::OK, body).await;
        let res = update(&app, "linux/x86_64/0.1.0").await;
        assert_eq!(res.status, StatusCode::BAD_GATEWAY, "{prefix}");
        assert_eq!(res.body["error"], "release info is not available", "{prefix}");
        assert_eq!(update(&app, "windows/x86_64/0.1.0").await.status, StatusCode::OK, "{prefix}");
        assert_eq!(release(&app).await.body["linux"]["name"], APPIMAGE, "{prefix}");
    }
}

#[tokio::test]
async fn draft_release_offers_no_update() {
    let body = |base: &str| {
        let mut body: Value = serde_json::from_str(&full_release(base)).unwrap();
        body["draft"] = json!(true);
        body["published_at"] = Value::Null;
        body.to_string()
    };
    let (app, _) = mock_github(StatusCode::OK, body).await;
    assert_eq!(update(&app, "linux/x86_64/0.1.0").await.status, StatusCode::NO_CONTENT);
    assert_eq!(release(&app).await.status, StatusCode::NOT_FOUND);

    let body = |base: &str| {
        let mut body: Value = serde_json::from_str(&full_release(base)).unwrap();
        body["draft"] = json!(true);
        body.to_string()
    };
    let (app, _) = mock_github(StatusCode::OK, body).await;
    assert_eq!(update(&app, "windows/x86_64/0.1.0").await.status, StatusCode::NO_CONTENT);
}

fn tagged(tag: &str) -> impl FnOnce(&str) -> String {
    move |base: &str| {
        let mut body: Value = serde_json::from_str(&full_release(base)).unwrap();
        body["tag_name"] = json!(tag);
        body.to_string()
    }
}

#[tokio::test]
async fn tag_with_v_or_suffix() {
    for tag in ["desktop-vv0.2.0", "v0.2.0", "0.2.0"] {
        let (app, _) = mock_github(StatusCode::OK, tagged(tag)).await;
        let res = update(&app, "linux/x86_64/0.1.0").await;
        assert_eq!(res.status, StatusCode::OK, "{tag}");
        assert_eq!(res.body["version"], "0.2.0", "{tag}");
        assert_eq!(update(&app, "linux/x86_64/0.2.0").await.status, StatusCode::NO_CONTENT, "{tag}");
    }
    for tag in ["desktop-v0.3.0-rc1", "desktop-vnext", "latest"] {
        let (app, _) = mock_github(StatusCode::OK, tagged(tag)).await;
        let res = update(&app, "windows/x86_64/0.1.0").await;
        assert_eq!(res.status, StatusCode::NO_CONTENT, "{tag}");
        assert_eq!(release(&app).await.status, StatusCode::OK, "{tag}");
    }
    let (app, _) = mock_github(StatusCode::OK, tagged("desktop-vv0.2.0")).await;
    assert_eq!(release(&app).await.body["version"], "v0.2.0");
}

#[tokio::test]
async fn failed_signature_is_cached_for_one_minute() {
    let ttl = |state: &AppState| {
        let (until, _) = state.release.lock().unwrap().clone().unwrap();
        until.saturating_duration_since(Instant::now())
    };
    let body = |base: &str| {
        let mut body: Value = serde_json::from_str(&full_release(base)).unwrap();
        body["assets"][1]["browser_download_url"] = json!(format!("{base}/sig/fail.sig"));
        body.to_string()
    };
    let (state, hits) = mock_state(StatusCode::OK, body).await;
    let app = api::router(state.clone());
    assert_eq!(update(&app, "linux/x86_64/0.1.0").await.status, StatusCode::BAD_GATEWAY);
    assert!(ttl(&state) <= Duration::from_secs(60));
    assert_eq!(release(&app).await.status, StatusCode::OK);
    assert_eq!(hits.load(Ordering::SeqCst), 3);

    let (state, _) = mock_state(StatusCode::OK, full_release).await;
    assert_eq!(update(&api::router(state.clone()), "linux/x86_64/0.1.0").await.status, StatusCode::OK);
    assert!(ttl(&state) > Duration::from_secs(500));
}
