mod common;

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use api::AppState;
use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Request, StatusCode, header},
    routing,
};
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const STEAM_ID: &str = "76561190000000001";
const TOKEN: &str = "test-token";

#[derive(Default)]
struct Github {
    hits: AtomicUsize,
    last: Mutex<Value>,
}

impl Github {
    fn hits(&self) -> usize {
        self.hits.load(Ordering::SeqCst)
    }

    fn last(&self) -> Value {
        self.last.lock().unwrap().clone()
    }
}

async fn mock_github(status: StatusCode, html_url: &str) -> (String, Arc<Github>) {
    let github = Arc::new(Github::default());
    let shared = github.clone();
    let answer = json!({ "number": 123, "html_url": html_url }).to_string();
    let mock = Router::new().route(
        "/repos/elwqnn/spam-live/issues",
        routing::post(move |headers: HeaderMap, body: String| async move {
            shared.hits.fetch_add(1, Ordering::SeqCst);
            *shared.last.lock().unwrap() = serde_json::from_str(&body).unwrap_or(Value::Null);
            let ok = headers.get(header::ACCEPT).is_some_and(|v| v == "application/vnd.github+json")
                && headers.contains_key(header::USER_AGENT)
                && headers.get(header::AUTHORIZATION).is_some_and(|v| v == "Bearer test-token");
            if ok { (status, answer) } else { (StatusCode::UNAUTHORIZED, String::new()) }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });
    (format!("http://{addr}"), github)
}

fn state(pool: &PgPool, github_url: &str, token: Option<&str>) -> AppState {
    let mut config = config("https://steamcommunity.com/openid/login");
    config.github_api_url = github_url.into();
    config.github_feedback_token = token.map(Into::into);
    AppState::new(pool.clone(), config, real_catalog())
}

fn webp(w: u32, h: u32) -> Vec<u8> {
    let mut b = b"RIFF\x12\0\0\0WEBPVP8L\x05\0\0\0\x2f".to_vec();
    b.extend(((w - 1) | ((h - 1) << 14)).to_le_bytes());
    b.push(0);
    b
}

type Fields = Vec<(&'static str, Vec<u8>)>;

fn valid() -> Fields {
    vec![
        ("kind", b"bug".to_vec()),
        ("body", b"The deck page does not load.".to_vec()),
        ("page", b"/decks/12?tab=cards".to_vec()),
        ("siteVersion", b"abc1234".to_vec()),
    ]
}

fn with(name: &'static str, value: impl Into<Vec<u8>>) -> Fields {
    let mut fields: Fields = valid().into_iter().filter(|(n, _)| *n != name).collect();
    fields.push((name, value.into()));
    fields
}

fn form(cookie: &str, fields: &Fields) -> Request<Body> {
    let boundary = "FeedbackTestBoundary4kq";
    let mut body = Vec::new();
    for (name, data) in fields {
        let file = if *name == "screenshot" { "; filename=\"shot.webp\"" } else { "" };
        body.extend(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"{file}\r\n\r\n").bytes());
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n");
    }
    body.extend(format!("--{boundary}--\r\n").bytes());
    Request::post("/api/feedback")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .header(header::COOKIE, cookie)
        .header(header::USER_AGENT, "Mozilla/5.0 `Test` Browser")
        .body(Body::from(body))
        .unwrap()
}

async fn raw(app: &Router, uri: &str) -> (StatusCode, HeaderMap, Vec<u8>) {
    let res = app.clone().oneshot(get(uri, "")).await.unwrap();
    let (status, headers) = (res.status(), res.headers().clone());
    (status, headers, axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap().to_vec())
}

async fn issue_numbers(pool: &PgPool) -> Vec<Option<i64>> {
    sqlx::query_scalar("SELECT issue_number FROM feedback ORDER BY id").fetch_all(pool).await.unwrap()
}

async fn rows(pool: &PgPool, table: &str) -> i64 {
    let sql = match table {
        "feedback" => "SELECT count(*) FROM feedback",
        "feedback_images" => "SELECT count(*) FROM feedback_images",
        _ => unreachable!(),
    };
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

#[sqlx::test]
async fn login_is_required_and_banned_users_are_403(pool: PgPool) {
    let app = api::router(state(&pool, "http://127.0.0.1:9", None));
    assert_eq!(send(&app, form("", &valid())).await.status, StatusCode::UNAUTHORIZED);
    let (id, cookie) = user(&pool, STEAM_ID).await;
    sqlx::query("UPDATE users SET banned_at = now() WHERE id = $1").bind(id).execute(&pool).await.unwrap();
    assert_eq!(send(&app, form(&cookie, &valid())).await.status, StatusCode::FORBIDDEN);
    assert_eq!(rows(&pool, "feedback").await, 0);
}

#[sqlx::test]
async fn bad_fields_are_400(pool: PgPool) {
    let app = api::router(state(&pool, "http://127.0.0.1:9", None));
    let (_, cookie) = user(&pool, STEAM_ID).await;
    let mut big = webp(10, 10);
    big.resize(3 * 1024 * 1024 + 1, 0);
    let mut duplicate = valid();
    duplicate.push(("kind", b"idea".to_vec()));
    let cases: Vec<Fields> = vec![
        with("kind", "bugs"),
        with("kind", ""),
        with("body", "too short"),
        with("body", "   short     "),
        with("body", "a".repeat(5001)),
        with("body", vec![0xff, 0xfe, 0xfd, 0xfc, 0xfb, 0xfa, 0xf9, 0xf8, 0xf7, 0xf6, 0xf5]),
        with("page", "decks/12"),
        with("page", "//evil.example/x"),
        with("page", "/a b"),
        with("page", "/a`b"),
        with("page", "/a<b"),
        with("page", "/a\"b"),
        with("page", "/a\\b"),
        with("page", "/caf\u{e9}"),
        with("page", format!("/{}", "a".repeat(2000))),
        with("siteVersion", "abc123"),
        with("siteVersion", "ABC1234"),
        with("siteVersion", "abc123g"),
        with("siteVersion", "a".repeat(41)),
        with("element", "not json"),
        with("element", r#"{"selector":"","text":""}"#),
        with("element", json!({ "selector": "a".repeat(301), "text": "" }).to_string()),
        with("element", json!({ "selector": "a", "text": "b".repeat(201) }).to_string()),
        with("element", r#"{"selector":"a","text":"b","extra":1}"#),
        with("element", r#"{"selector":"a"}"#),
        with("body", "ten chars \0 with a NUL"),
        with("element", r#"{"selector":"a\u0000b","text":""}"#),
        with("element", r#"{"selector":"a","text":"b\u0000"}"#),
        with("element", b"{\"selector\":\"a\0\",\"text\":\"\"}".to_vec()),
        with("screenshot", "not an image"),
        with("screenshot", b"GIF89a\x01\0\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_vec()),
        with("screenshot", webp(4097, 10)),
        with("screenshot", big),
        with("other", "x"),
        duplicate,
        valid().into_iter().filter(|(n, _)| *n != "body").collect(),
        valid().into_iter().filter(|(n, _)| *n != "siteVersion").collect(),
    ];
    for (i, fields) in cases.iter().enumerate() {
        let res = send(&app, form(&cookie, fields)).await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST, "case {i}: {}", res.body);
    }
    assert_eq!(rows(&pool, "feedback").await, 0);
    let edge = vec![
        ("kind", b"other".to_vec()),
        ("body", "\u{e9}".repeat(5000).into_bytes()),
        ("page", format!("/{}", "a".repeat(1999)).into_bytes()),
        ("siteVersion", b"dev".to_vec()),
        ("element", json!({ "selector": "a".repeat(300), "text": "b".repeat(200) }).to_string().into_bytes()),
        ("screenshot", webp(4096, 1)),
    ];
    assert_eq!(send(&app, form(&cookie, &edge)).await.status, StatusCode::CREATED);
}

#[sqlx::test]
async fn sixth_feedback_in_24_hours_is_429(pool: PgPool) {
    let app = api::router(state(&pool, "http://127.0.0.1:9", None));
    let (id, cookie) = user(&pool, STEAM_ID).await;
    for _ in 0..4 {
        assert_eq!(send(&app, form(&cookie, &valid())).await.status, StatusCode::CREATED);
    }
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT 1 FROM users WHERE id = $1 FOR UPDATE").bind(id).execute(&mut *tx).await.unwrap();
    let parallel = tokio::spawn({
        let (app, request) = (app.clone(), form(&cookie, &valid()));
        async move { send(&app, request).await.status }
    });
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!parallel.is_finished());
    sqlx::query("INSERT INTO feedback (user_id, kind, body, page, site_version) VALUES ($1, 'bug', 'x', '/', 'dev')")
        .bind(id)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(parallel.await.unwrap(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(send(&app, form(&cookie, &valid())).await.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(rows(&pool, "feedback").await, 5);

    let (_, other) = user(&pool, "76561190000000002").await;
    assert_eq!(send(&app, form(&other, &valid())).await.status, StatusCode::CREATED);

    sqlx::query("UPDATE feedback SET created_at = now() - interval '25 hours'").execute(&pool).await.unwrap();
    assert_eq!(send(&app, form(&cookie, &valid())).await.status, StatusCode::CREATED);
}

#[sqlx::test]
async fn feedback_makes_one_issue_without_identity(pool: PgPool) {
    let (url, github) = mock_github(StatusCode::CREATED, "https://github.com/elwqnn/spam-live/issues/123").await;
    let app = api::router(state(&pool, &url, Some(TOKEN)));
    let (_, cookie) = user(&pool, STEAM_ID).await;
    let first = format!("First line\r ``` with\t@someone and a long tail {}", "x".repeat(60));
    let text = format!("  {first}\nSecond line @other\n  ");
    let image = webp(1692, 1207);
    let fields = vec![
        ("kind", b"idea".to_vec()),
        ("body", text.into_bytes()),
        ("page", b"/fr/decks/12?tab=cards&q=@a".to_vec()),
        ("siteVersion", b"0123456789abcdef0123456789abcdef01234567".to_vec()),
        ("element", br#"{"selector":"main > button.send","text":"Send `now` @x"}"#.to_vec()),
        ("screenshot", image.clone()),
    ];
    let res = send(&app, form(&cookie, &fields)).await;
    assert_eq!(res.status, StatusCode::CREATED);
    let id: i64 = sqlx::query_scalar("SELECT id FROM feedback").fetch_one(&pool).await.unwrap();
    assert_eq!(res.body, json!({ "id": id }));
    assert_eq!(github.hits(), 1);

    let image_id: Uuid = sqlx::query_scalar("SELECT id FROM feedback_images").fetch_one(&pool).await.unwrap();
    let issue = github.last();
    let prefix = "[Idea] First line ``` with@someone and a long tail ";
    assert_eq!(issue["title"], format!("{prefix}{}", "x".repeat(80 - prefix.len())));
    assert!(!issue["title"].as_str().unwrap().chars().any(char::is_control));
    assert_eq!(issue["labels"], json!(["enhancement"]));
    let body = issue["body"].as_str().unwrap();
    let expected = format!(
        "````\n{first}\nSecond line @other\n````\n\n\
         - Page: `{PUBLIC_URL}/fr/decks/12?tab=cards&q=@a`\n\
         - Site version: `0123456789abcdef0123456789abcdef01234567`\n\
         - Browser: `Mozilla/5.0 Test Browser`\n\n\
         Element:\n\n```\nmain > button.send\nSend `now` @x\n```\n\n\
         ![Screenshot]({PUBLIC_URL}/api/feedback/images/{image_id})\n"
    );
    assert_eq!(body, expected);
    assert!(!body.contains(STEAM_ID) && !issue["title"].as_str().unwrap().contains(STEAM_ID));

    let stored: (Option<i64>, Option<String>, Option<String>, String) =
        sqlx::query_as("SELECT issue_number, issue_url, user_agent, body FROM feedback")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored.0, Some(123));
    assert_eq!(stored.1.as_deref(), Some("https://github.com/elwqnn/spam-live/issues/123"));
    assert_eq!(stored.2.as_deref(), Some("Mozilla/5.0 `Test` Browser"));
    assert_eq!(stored.3, format!("{first}\nSecond line @other"));

    let path = body.split("![Screenshot](").nth(1).unwrap().split(')').next().unwrap();
    let (status, headers, bytes) = raw(&app, path.strip_prefix(PUBLIC_URL).unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers[header::CONTENT_TYPE], "image/webp");
    assert_eq!(headers[header::CACHE_CONTROL], "public, max-age=31536000, immutable");
    assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    assert_eq!(bytes, image);
    let size: (i32, i32) = sqlx::query_as("SELECT width, height FROM feedback_images").fetch_one(&pool).await.unwrap();
    assert_eq!(size, (1692, 1207));

    assert_eq!(raw(&app, &format!("/api/feedback/images/{}", Uuid::nil())).await.0, StatusCode::NOT_FOUND);
    assert_eq!(raw(&app, "/api/feedback/images/not-a-uuid").await.0, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn github_failure_or_no_token_keeps_the_feedback_pending(pool: PgPool) {
    let (_, cookie) = user(&pool, STEAM_ID).await;
    let (url, github) = mock_github(StatusCode::INTERNAL_SERVER_ERROR, "https://github.com/x").await;
    let failing = api::router(state(&pool, &url, Some(TOKEN)));
    assert_eq!(send(&failing, form(&cookie, &valid())).await.status, StatusCode::CREATED);
    assert_eq!(github.hits(), 1);

    let no_token = api::router(state(&pool, &url, None));
    assert_eq!(send(&no_token, form(&cookie, &valid())).await.status, StatusCode::CREATED);
    assert_eq!(github.hits(), 1);

    let unreachable = api::router(state(&pool, "http://127.0.0.1:9", Some(TOKEN)));
    assert_eq!(send(&unreachable, form(&cookie, &valid())).await.status, StatusCode::CREATED);
    assert_eq!(issue_numbers(&pool).await, [None, None, None]);

    let (url, _) = mock_github(StatusCode::CREATED, "javascript:alert(1)").await;
    let unsafe_url = api::router(state(&pool, &url, Some(TOKEN)));
    assert_eq!(send(&unsafe_url, form(&cookie, &valid())).await.status, StatusCode::CREATED);
    let last: (Option<i64>, Option<String>) =
        sqlx::query_as("SELECT issue_number, issue_url FROM feedback ORDER BY id DESC LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(last, (Some(123), None));
}

#[sqlx::test]
async fn retry_pending_creates_old_issues_only(pool: PgPool) {
    let (url, github) = mock_github(StatusCode::CREATED, "https://github.com/elwqnn/spam-live/issues/123").await;
    let no_token = state(&pool, &url, None);
    let app = api::router(no_token.clone());
    let (_, cookie) = user(&pool, STEAM_ID).await;
    assert_eq!(send(&app, form(&cookie, &with("screenshot", webp(8, 8)))).await.status, StatusCode::CREATED);
    assert_eq!(send(&app, form(&cookie, &valid())).await.status, StatusCode::CREATED);
    sqlx::query(
        "UPDATE feedback SET created_at = now() - interval '10 minutes' WHERE id = (SELECT min(id) FROM feedback)",
    )
    .execute(&pool)
    .await
    .unwrap();

    api::feedback::retry_pending(&no_token).await;
    assert_eq!(github.hits(), 0);

    let with_token = state(&pool, &url, Some(TOKEN));
    api::feedback::retry_pending(&with_token).await;
    assert_eq!(github.hits(), 1);
    assert_eq!(issue_numbers(&pool).await, [Some(123), None]);
    let body = github.last()["body"].as_str().unwrap().to_string();
    assert!(body.contains("The deck page does not load.") && body.contains("/api/feedback/images/"));

    api::feedback::retry_pending(&with_token).await;
    assert_eq!(github.hits(), 1);
}

#[sqlx::test]
async fn admin_lists_feedback_and_deletes_images(pool: PgPool) {
    let app = api::router(state(&pool, "http://127.0.0.1:9", None));
    let (_, cookie) = user(&pool, STEAM_ID).await;
    let (_, admin_cookie) = admin(&pool, "76561190000000009").await;
    assert_eq!(send(&app, form(&cookie, &with("screenshot", webp(8, 8)))).await.status, StatusCode::CREATED);
    assert_eq!(send(&app, form(&cookie, &with("kind", "other"))).await.status, StatusCode::CREATED);
    let ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM feedback ORDER BY id").fetch_all(&pool).await.unwrap();

    assert_eq!(send(&app, get("/api/admin/feedback", &cookie)).await.status, StatusCode::FORBIDDEN);
    let image_uri = format!("/api/admin/feedback/{}/image", ids[0]);
    assert_eq!(send(&app, req("DELETE", &image_uri, &cookie, None)).await.status, StatusCode::FORBIDDEN);

    let list = send(&app, get("/api/admin/feedback", &admin_cookie)).await;
    assert_eq!(list.status, StatusCode::OK);
    let items = list.body.as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["id"], ids[1]);
    assert_eq!(items[0]["kind"], "other");
    assert_eq!(items[0]["imageId"], Value::Null);
    assert_eq!(items[1]["kind"], "bug");
    assert_eq!(items[1]["body"], "The deck page does not load.");
    assert_eq!(items[1]["page"], "/decks/12?tab=cards");
    assert_eq!(items[1]["siteVersion"], "abc1234");
    assert_eq!(items[1]["issueNumber"], Value::Null);
    assert_eq!(items[1]["issueUrl"], Value::Null);
    assert!(items[1]["createdAt"].is_string());
    let image_id = items[1]["imageId"].as_str().unwrap().to_string();
    assert_eq!(raw(&app, &format!("/api/feedback/images/{image_id}")).await.0, StatusCode::OK);

    assert_eq!(send(&app, req("DELETE", &image_uri, &admin_cookie, None)).await.status, StatusCode::NO_CONTENT);
    assert_eq!(send(&app, req("DELETE", &image_uri, &admin_cookie, None)).await.status, StatusCode::NOT_FOUND);
    let no_image = format!("/api/admin/feedback/{}/image", ids[1]);
    assert_eq!(send(&app, req("DELETE", &no_image, &admin_cookie, None)).await.status, StatusCode::NOT_FOUND);
    assert_eq!(raw(&app, &format!("/api/feedback/images/{image_id}")).await.0, StatusCode::NOT_FOUND);
    let list = send(&app, get("/api/admin/feedback", &admin_cookie)).await;
    assert_eq!(list.body[1]["imageId"], Value::Null);
    assert_eq!(rows(&pool, "feedback").await, 2);
}

#[sqlx::test]
async fn deleting_the_account_deletes_feedback(pool: PgPool) {
    let app = api::router(state(&pool, "http://127.0.0.1:9", None));
    let (_, cookie) = user(&pool, STEAM_ID).await;
    assert_eq!(send(&app, form(&cookie, &with("screenshot", webp(8, 8)))).await.status, StatusCode::CREATED);
    assert_eq!(rows(&pool, "feedback_images").await, 1);
    assert_eq!(send(&app, req("DELETE", "/api/me", &cookie, None)).await.status, StatusCode::NO_CONTENT);
    assert_eq!(rows(&pool, "feedback").await, 0);
    assert_eq!(rows(&pool, "feedback_images").await, 0);
}
