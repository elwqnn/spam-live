mod common;

use std::collections::HashMap;

use api::auth::{parse_claimed_id, safe_redirect};
use axum::{
    Form, Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::post,
};
use common::*;
use reqwest::Url;
use serde_json::json;
use sqlx::PgPool;

const STEAM_ID: &str = "76561190000000042";

#[test]
fn claimed_id_parse() {
    let ok = format!("https://steamcommunity.com/openid/id/{STEAM_ID}");
    assert_eq!(parse_claimed_id(&ok), Some(STEAM_ID));
    for bad in [
        "http://steamcommunity.com/openid/id/76561190000000042",
        "https://steamcommunity.com/openid/id/7656119000000004",
        "https://steamcommunity.com/openid/id/765611900000000421",
        "https://steamcommunity.com/openid/id/76561190000000042/",
        "https://steamcommunity.com.evil.com/openid/id/76561190000000042",
        "https://evil.com/?https://steamcommunity.com/openid/id/76561190000000042",
        "https://steamcommunity.com/openid/id/7656119000000004a",
        "",
    ] {
        assert_eq!(parse_claimed_id(bad), None, "{bad}");
    }
}

#[test]
fn open_redirect_guard() {
    assert_eq!(safe_redirect(Some("/decks?x=1")), "/decks?x=1");
    assert_eq!(safe_redirect(Some("/")), "/");
    for bad in ["//evil.com", "/\\evil.com", "https://evil.com", "evil", "", "/a\r\nSet-Cookie: x", "/a b"] {
        assert_eq!(safe_redirect(Some(bad)), "/", "{bad:?}");
    }
    assert_eq!(safe_redirect(None), "/");
}

async fn mock_openid(valid: bool) -> String {
    let app = Router::new().route(
        "/openid/login",
        post(move |Form(form): Form<HashMap<String, String>>| async move {
            let ok = valid
                && form.get("openid.mode").map(String::as_str) == Some("check_authentication")
                && form.contains_key("openid.sig");
            format!("ns:http://specs.openid.net/auth/2.0\nis_valid:{ok}\n")
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}/openid/login")
}

const STATE: &str = "4f1d0c2b9a8e7d6c5b4a39281706f5e4d3c2b1a09f8e7d6c5b4a392817060000";

fn return_to(redirect: &str) -> String {
    format!("{PUBLIC_URL}/api/auth/steam/callback?redirect={redirect}&state={STATE}")
}

fn nonce(offset_secs: i64) -> String {
    let at = chrono::Utc::now() + chrono::TimeDelta::seconds(offset_secs);
    format!("{}abc123", at.format("%Y-%m-%dT%H:%M:%SZ"))
}

fn openid_params(op: &str) -> Vec<(String, String)> {
    let claimed = format!("https://steamcommunity.com/openid/id/{STEAM_ID}");
    [
        ("openid.ns", "http://specs.openid.net/auth/2.0"),
        ("openid.mode", "id_res"),
        ("openid.op_endpoint", op),
        ("openid.claimed_id", &claimed),
        ("openid.identity", &claimed),
        ("openid.return_to", &return_to("%2Fdecks")),
        ("openid.response_nonce", &nonce(0)),
        ("openid.assoc_handle", "1234567890"),
        ("openid.signed", "signed,op_endpoint,claimed_id,identity,return_to,response_nonce,assoc_handle"),
        ("openid.sig", "c2lnbmF0dXJl"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

fn callback_with(params: &[(String, String)], cookie: &str) -> Request<Body> {
    let url = Url::parse_with_params("http://x/api/auth/steam/callback", params).unwrap();
    Request::get(format!("/api/auth/steam/callback?{}", url.query().unwrap()))
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .unwrap()
}

fn callback(params: &[(String, String)]) -> Request<Body> {
    callback_with(params, &format!("theme=dark; oidstate={STATE}"))
}

fn set(params: &mut [(String, String)], key: &str, value: &str) {
    params.iter_mut().find(|(k, _)| k == key).unwrap().1 = value.into();
}

fn set_cookies(res: &Res) -> Vec<String> {
    res.headers.get_all(header::SET_COOKIE).iter().map(|v| v.to_str().unwrap().to_string()).collect()
}

async fn user_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(pool).await.unwrap()
}

#[sqlx::test]
async fn login_redirects_to_steam(pool: PgPool) {
    let app = app(pool);
    let res = send(&app, get("/api/auth/steam?redirect=/decks", "")).await;
    assert_eq!(res.status, StatusCode::SEE_OTHER);
    let cookies = set_cookies(&res);
    assert_eq!(cookies.len(), 1);
    let (state, attrs) = cookies[0].strip_prefix("oidstate=").unwrap().split_once(';').unwrap();
    assert_eq!(state.len(), 64);
    assert_eq!(attrs, " HttpOnly; Secure; SameSite=Lax; Path=/api/auth; Max-Age=600");
    let location = Url::parse(res.headers[header::LOCATION].to_str().unwrap()).unwrap();
    assert_eq!(location.host_str(), Some("steamcommunity.com"));
    let q: HashMap<_, _> = location.query_pairs().into_owned().collect();
    assert_eq!(q["openid.mode"], "checkid_setup");
    assert_eq!(q["openid.realm"], PUBLIC_URL);
    assert_eq!(q["openid.return_to"], format!("{PUBLIC_URL}/api/auth/steam/callback?redirect=%2Fdecks&state={state}"));

    let res = send(&app, get("/api/auth/steam?redirect=//evil.com", "")).await;
    let location = Url::parse(res.headers[header::LOCATION].to_str().unwrap()).unwrap();
    let q: HashMap<_, _> = location.query_pairs().into_owned().collect();
    assert!(q["openid.return_to"].contains("?redirect=%2F&state="));
}

#[sqlx::test]
async fn callback_with_valid_assertion(pool: PgPool) {
    let op = mock_openid(true).await;
    let app = app_with(pool.clone(), &op);
    let res = send(&app, callback(&openid_params(&op))).await;
    assert_eq!(res.status, StatusCode::SEE_OTHER);
    assert_eq!(res.headers[header::LOCATION], "/decks");
    let cookies = set_cookies(&res);
    let session = cookies.iter().find(|c| c.starts_with("session=")).unwrap();
    assert!(session.ends_with("; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=2592000"));
    assert!(cookies.contains(&"oidstate=; HttpOnly; Secure; SameSite=Lax; Path=/api/auth; Max-Age=0".to_string()));
    let token = session.split(';').next().unwrap().to_string();
    assert_eq!(token.len(), "session=".len() + 64);
    let stored: Vec<u8> = sqlx::query_scalar("SELECT token_hash FROM sessions").fetch_one(&pool).await.unwrap();
    assert_eq!(stored, api::auth::hash(&token["session=".len()..]));

    let me = send(&app, get("/api/me", &token)).await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.body["displayName"], STEAM_ID);
    assert_eq!(me.body["isAdmin"], false);
    assert_eq!(me.body["spamAccountId"], serde_json::Value::Null);

    let again = send(&app, callback(&openid_params(&op))).await;
    assert_eq!(again.status, StatusCode::SEE_OTHER);
    assert_eq!(user_count(&pool).await, 1);
}

#[sqlx::test]
async fn callback_with_invalid_assertion(pool: PgPool) {
    let op = mock_openid(false).await;
    let app = app_with(pool.clone(), &op);
    let res = send(&app, callback(&openid_params(&op))).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert!(res.headers.get(header::SET_COOKIE).is_none());
    assert_eq!(user_count(&pool).await, 0);
}

#[sqlx::test]
async fn callback_rejects_bad_parameters(pool: PgPool) {
    let op = mock_openid(true).await;
    let app = app_with(pool.clone(), &op);
    let cases: Vec<(&str, &str)> = vec![
        ("openid.mode", "cancel"),
        ("openid.op_endpoint", "https://evil.com/openid/login"),
        ("openid.return_to", "https://evil.com/api/auth/steam/callback"),
        ("openid.return_to", "http://localhost:5173/api/auth/steam/callbackx"),
        ("openid.claimed_id", "https://evil.com/openid/id/76561190000000042"),
        ("openid.signed", "signed,op_endpoint,identity,return_to,response_nonce"),
    ];
    for (key, value) in cases {
        let mut params = openid_params(&op);
        set(&mut params, key, value);
        let res = send(&app, callback(&params)).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED, "{key}={value}");
    }
    let mut params = openid_params(&op);
    params.push(("openid.claimed_id".into(), "https://steamcommunity.com/openid/id/76561190000000099".into()));
    assert_eq!(send(&app, callback(&params)).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(user_count(&pool).await, 0);

    let mut params = openid_params(&op);
    set(&mut params, "openid.return_to", &return_to("//evil.com"));
    let res = send(&app, callback(&params)).await;
    assert_eq!(res.status, StatusCode::SEE_OTHER);
    assert_eq!(res.headers[header::LOCATION], "/");
}

#[sqlx::test]
async fn callback_requires_login_state(pool: PgPool) {
    let op = mock_openid(true).await;
    let app = app_with(pool.clone(), &op);
    let params = openid_params(&op);
    for cookie in ["", "oidstate=", "oidstate=0000", "session=x"] {
        let res = send(&app, callback_with(&params, cookie)).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED, "{cookie:?}");
        assert_eq!(res.body["error"], "invalid login state");
    }
    let mut params = openid_params(&op);
    set(&mut params, "openid.return_to", &format!("{PUBLIC_URL}/api/auth/steam/callback?redirect=%2F"));
    let res = send(&app, callback_with(&params, "oidstate=")).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(user_count(&pool).await, 0);
}

#[sqlx::test]
async fn callback_requires_fresh_nonce(pool: PgPool) {
    let op = mock_openid(true).await;
    let app = app_with(pool.clone(), &op);
    for value in [nonce(-301), nonce(-3600), nonce(120), "abc".to_string(), "2026-13-01T00:00:00Zx".to_string()] {
        let mut params = openid_params(&op);
        set(&mut params, "openid.response_nonce", &value);
        let res = send(&app, callback(&params)).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED, "{value}");
        assert_eq!(res.body["error"], "OpenID nonce is expired");
    }
    let mut params = openid_params(&op);
    set(&mut params, "openid.response_nonce", &nonce(-240));
    assert_eq!(send(&app, callback(&params)).await.status, StatusCode::SEE_OTHER);
}

fn bearer(method: &str, uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

#[sqlx::test]
async fn device_flow(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, STEAM_ID).await;
    let code = send(&app, post_json("/api/device/code", "", json!({}))).await;
    assert_eq!(code.status, StatusCode::OK);
    let device_code = code.body["deviceCode"].as_str().unwrap().to_string();
    let user_code = code.body["userCode"].as_str().unwrap().to_string();
    assert_eq!(device_code.len(), 64);
    assert_eq!(user_code.len(), 9);
    assert_eq!(&user_code[4..5], "-");
    assert!(user_code.chars().filter(|c| *c != '-').all(|c| "BCDFGHJKLMNPQRSTVWXZ".contains(c)));
    assert_eq!(code.body["verificationUrl"], format!("{PUBLIC_URL}/link?code={user_code}"));
    assert_eq!(code.body["expiresIn"], 600);
    assert_eq!(code.body["interval"], 5);
    let stored: i64 = sqlx::query_scalar("SELECT count(*) FROM device_codes WHERE device_code_hash = $1")
        .bind(api::auth::hash(&device_code))
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, 1);

    let token_req = || post_json("/api/device/token", "", json!({ "deviceCode": device_code }));
    let pending = send(&app, token_req()).await;
    assert_eq!(pending.status, StatusCode::BAD_REQUEST);
    assert_eq!(pending.body["error"], "authorization_pending");

    let confirm = json!({ "userCode": user_code.to_lowercase() });
    assert_eq!(
        send(&app, post_json("/api/device/confirm", &cookie, confirm.clone())).await.status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(send(&app, post_json("/api/device/confirm", &cookie, confirm)).await.status, StatusCode::NOT_FOUND);

    let granted = send(&app, token_req()).await;
    assert_eq!(granted.status, StatusCode::OK);
    assert_eq!(granted.body["tokenType"], "Bearer");
    let token = granted.body["accessToken"].as_str().unwrap().to_string();

    let reused = send(&app, token_req()).await;
    assert_eq!(reused.status, StatusCode::BAD_REQUEST);
    assert_eq!(reused.body["error"], "invalid_grant");

    let me = send(&app, bearer("GET", "/api/me", &token)).await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.body["displayName"], STEAM_ID);

    let other = send(&app, post_json("/api/device/code", "", json!({}))).await;
    let mut req = post_json("/api/device/confirm", "", json!({ "userCode": other.body["userCode"] }));
    req.headers_mut().insert(header::AUTHORIZATION, format!("Bearer {token}").parse().unwrap());
    assert_eq!(send(&app, req).await.status, StatusCode::FORBIDDEN);

    let devices = send(&app, get("/api/me/devices", &cookie)).await;
    let list = devices.body.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert!(list[0]["lastUsedAt"].is_string());
    let id = list[0]["id"].as_i64().unwrap();

    let (_, stranger) = user(&pool, "76561190000000043").await;
    let mut req = get(&format!("/api/me/devices/{id}"), &stranger);
    *req.method_mut() = axum::http::Method::DELETE;
    assert_eq!(send(&app, req).await.status, StatusCode::NOT_FOUND);

    let mut req = get(&format!("/api/me/devices/{id}"), &cookie);
    *req.method_mut() = axum::http::Method::DELETE;
    assert_eq!(send(&app, req).await.status, StatusCode::NO_CONTENT);
    assert_eq!(send(&app, bearer("GET", "/api/me", &token)).await.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn device_code_cap(pool: PgPool) {
    let mut config = config("https://steamcommunity.com/openid/login");
    config.max_pending_device_codes = 2;
    let app = api::router(api::AppState::new(pool.clone(), config, real_catalog()));
    let (_, cookie) = user(&pool, STEAM_ID).await;
    let code = || post_json("/api/device/code", "", json!({}));
    let first = send(&app, code()).await;
    assert_eq!(first.status, StatusCode::OK);
    assert_eq!(send(&app, code()).await.status, StatusCode::OK);
    let full = send(&app, code()).await;
    assert_eq!(full.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(full.body["error"], "too many pending device codes");

    let confirm = json!({ "userCode": first.body["userCode"] });
    assert_eq!(send(&app, post_json("/api/device/confirm", &cookie, confirm)).await.status, StatusCode::NO_CONTENT);
    assert_eq!(send(&app, code()).await.status, StatusCode::OK);
    assert_eq!(send(&app, code()).await.status, StatusCode::TOO_MANY_REQUESTS);

    sqlx::query("UPDATE device_codes SET expires_at = now() - interval '1 second' WHERE user_id IS NULL")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(send(&app, code()).await.status, StatusCode::OK);
}

#[sqlx::test]
async fn device_code_expiry(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, STEAM_ID).await;
    let code = send(&app, post_json("/api/device/code", "", json!({}))).await;
    sqlx::query("UPDATE device_codes SET expires_at = now() - interval '1 second'").execute(&pool).await.unwrap();
    let confirm = json!({ "userCode": code.body["userCode"] });
    assert_eq!(send(&app, post_json("/api/device/confirm", &cookie, confirm)).await.status, StatusCode::NOT_FOUND);
    let token_req = || post_json("/api/device/token", "", json!({ "deviceCode": code.body["deviceCode"] }));
    let expired = send(&app, token_req()).await;
    assert_eq!(expired.body["error"], "expired_token");
    assert_eq!(send(&app, token_req()).await.body["error"], "invalid_grant");

    let confirmed = send(&app, post_json("/api/device/code", "", json!({}))).await;
    let confirm = json!({ "userCode": confirmed.body["userCode"] });
    assert_eq!(send(&app, post_json("/api/device/confirm", &cookie, confirm)).await.status, StatusCode::NO_CONTENT);
    sqlx::query("UPDATE device_codes SET expires_at = now() - interval '1 second'").execute(&pool).await.unwrap();
    let late =
        send(&app, post_json("/api/device/token", "", json!({ "deviceCode": confirmed.body["deviceCode"] }))).await;
    assert_eq!(late.body["error"], "expired_token");
    let devices: i64 = sqlx::query_scalar("SELECT count(*) FROM devices").fetch_one(&pool).await.unwrap();
    assert_eq!(devices, 0);
}

#[sqlx::test]
async fn origin_check(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, STEAM_ID).await;
    let with_origin = |uri: &str, origin: Option<&str>| {
        let mut req = post_json(uri, &cookie, json!({ "userCode": "BBBB-BBBB" }));
        if let Some(o) = origin {
            req.headers_mut().insert(header::ORIGIN, o.parse().unwrap());
        }
        req
    };
    let evil = Some("https://evil.com");
    assert_eq!(send(&app, with_origin("/api/device/confirm", evil)).await.status, StatusCode::FORBIDDEN);
    assert_eq!(send(&app, with_origin("/api/device/confirm", Some("null"))).await.status, StatusCode::FORBIDDEN);
    assert_eq!(send(&app, with_origin("/api/device/confirm", Some(PUBLIC_URL))).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, with_origin("/api/device/confirm", None)).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, with_origin("/api/auth/logout", evil)).await.status, StatusCode::FORBIDDEN);

    let mut get_req = get("/api/me", &cookie);
    get_req.headers_mut().insert(header::ORIGIN, "https://evil.com".parse().unwrap());
    assert_eq!(send(&app, get_req).await.status, StatusCode::OK);

    let device_token = api::auth::random_token();
    let id: i64 = sqlx::query_scalar("SELECT id FROM users").fetch_one(&pool).await.unwrap();
    sqlx::query("INSERT INTO devices (user_id, token_hash) VALUES ($1, $2)")
        .bind(id)
        .bind(api::auth::hash(&device_token))
        .execute(&pool)
        .await
        .unwrap();
    let mut req = upload_req("", b"[]", &[]);
    req.headers_mut().remove(header::COOKIE);
    req.headers_mut().insert(header::AUTHORIZATION, format!("Bearer {device_token}").parse().unwrap());
    req.headers_mut().insert(header::ORIGIN, "https://evil.com".parse().unwrap());
    assert_eq!(send(&app, req).await.status, StatusCode::OK);
}

#[sqlx::test]
async fn logout(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, STEAM_ID).await;
    let mut req = post_json("/api/auth/logout", &cookie, json!({}));
    req.headers_mut().insert(header::ORIGIN, PUBLIC_URL.parse().unwrap());
    let res = send(&app, req).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert!(res.headers[header::SET_COOKIE].to_str().unwrap().contains("Max-Age=0"));
    assert_eq!(send(&app, get("/api/me", &cookie)).await.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn banned_user_can_read_but_not_write(pool: PgPool) {
    let app = app(pool.clone());
    let (id, cookie) = user(&pool, STEAM_ID).await;
    sqlx::query("UPDATE users SET banned_at = now() WHERE id = $1").bind(id).execute(&pool).await.unwrap();
    assert_eq!(send(&app, get("/api/me", &cookie)).await.status, StatusCode::OK);
    let confirm = post_json("/api/device/confirm", &cookie, json!({ "userCode": "BBBB-BBBB" }));
    assert_eq!(send(&app, confirm).await.status, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn expired_session_is_rejected(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, STEAM_ID).await;
    sqlx::query("UPDATE sessions SET expires_at = now() - interval '1 second'").execute(&pool).await.unwrap();
    assert_eq!(send(&app, get("/api/me", &cookie)).await.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn banned_user_deletes_account_and_stays_banned(pool: PgPool) {
    let op = mock_openid(true).await;
    let app = app_with(pool.clone(), &op);
    let (_, admin) = admin(&pool, "76561190000000009").await;
    let (id, _) = user(&pool, STEAM_ID).await;
    let ban = send(&app, post_json(&format!("/api/admin/users/{id}/ban"), &admin, json!({}))).await;
    assert_eq!(ban.status, StatusCode::NO_CONTENT);
    let cookie = new_session(&pool, id).await;
    let deck = json!({ "name": "D", "cards": SEAT0_CARDS, "talents": [0, 0, 0, 0] });
    assert_eq!(send(&app, post_json("/api/decks", &cookie, deck.clone())).await.status, StatusCode::FORBIDDEN);
    let deleted = send(&app, req("DELETE", "/api/me", &cookie, None)).await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert_eq!(user_count(&pool).await, 1);

    let login = send(&app, callback(&openid_params(&op))).await;
    assert_eq!(login.status, StatusCode::SEE_OTHER);
    let session = set_cookies(&login).into_iter().find(|c| c.starts_with("session=")).unwrap();
    let token = session.split(';').next().unwrap().to_string();
    let me = send(&app, get("/api/me", &token)).await;
    assert_eq!(me.status, StatusCode::OK);
    let new_id = me.body["id"].as_i64().unwrap();
    assert_ne!(new_id, id);
    assert_eq!(send(&app, post_json("/api/decks", &token, deck.clone())).await.status, StatusCode::FORBIDDEN);
    let again = send(&app, callback(&openid_params(&op))).await;
    let again = set_cookies(&again).into_iter().find(|c| c.starts_with("session=")).unwrap();
    let again = again.split(';').next().unwrap().to_string();
    assert_eq!(send(&app, post_json("/api/decks", &again, deck.clone())).await.status, StatusCode::FORBIDDEN);

    let unban = send(&app, post_json(&format!("/api/admin/users/{new_id}/unban"), &admin, json!({}))).await;
    assert_eq!(unban.status, StatusCode::NO_CONTENT);
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM banned_steam_ids").fetch_one(&pool).await.unwrap();
    assert_eq!(left, 0);
    assert_eq!(send(&app, post_json("/api/decks", &token, deck.clone())).await.status, StatusCode::CREATED);
    send(&app, callback(&openid_params(&op))).await;
    assert_eq!(send(&app, post_json("/api/decks", &token, deck)).await.status, StatusCode::CREATED);
}
