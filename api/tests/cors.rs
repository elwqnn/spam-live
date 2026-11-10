mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::*;
use sqlx::PgPool;

fn from(method: &str, uri: &str, origin: &str) -> Request<Body> {
    Request::builder().method(method).uri(uri).header(header::ORIGIN, origin).body(Body::empty()).unwrap()
}

fn preflight(origin: &str, method: &str) -> Request<Body> {
    Request::builder()
        .method("OPTIONS")
        .uri("/api/cards")
        .header(header::ORIGIN, origin)
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, method)
        .body(Body::empty())
        .unwrap()
}

fn allowed(res: &Res) -> Option<&str> {
    res.headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN).and_then(|v| v.to_str().ok())
}

fn methods(res: &Res) -> String {
    res.headers
        .get(header::ACCESS_CONTROL_ALLOW_METHODS)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_ascii_uppercase()
}

#[sqlx::test]
async fn allows_get_from_the_app_origins(pool: PgPool) {
    let app = app(pool);
    for origin in ["tauri://localhost", "http://tauri.localhost", DEV_ORIGIN] {
        for method in ["GET", "HEAD"] {
            let res = send(&app, from(method, "/api/cards", origin)).await;
            assert_eq!(res.status, StatusCode::OK, "{method} {origin}");
            assert_eq!(allowed(&res), Some(origin), "{method} {origin}");
            assert!(res.headers.get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS).is_none());
        }
    }
}

#[sqlx::test]
async fn refuses_other_origins(pool: PgPool) {
    let app = app(pool);
    for origin in ["https://evil.example", "http://localhost:5173", "tauri://evil", "http://localhost:1421", "null"] {
        let res = send(&app, from("GET", "/api/cards", origin)).await;
        assert_eq!(res.status, StatusCode::OK);
        assert_eq!(allowed(&res), None, "{origin}");
        let res = send(&app, preflight(origin, "GET")).await;
        assert_eq!(allowed(&res), None, "{origin}");
    }
}

#[sqlx::test]
async fn preflight_allows_only_get_and_head(pool: PgPool) {
    let app = app(pool);
    let res = send(&app, preflight("tauri://localhost", "GET")).await;
    assert_eq!(allowed(&res), Some("tauri://localhost"));
    let list = methods(&res);
    assert!(list.contains("GET") && list.contains("HEAD"), "{list}");
    for method in ["POST", "PUT", "PATCH", "DELETE"] {
        let res = send(&app, preflight("tauri://localhost", method)).await;
        assert!(!methods(&res).contains(method), "{method}");
    }
    assert!(res.headers.get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS).is_none());
}
