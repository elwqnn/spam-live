mod common;

use axum::{Router, http::StatusCode};
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;

async fn public_deck(app: &Router, cookie: &str, cards: [&str; 4]) -> i64 {
    let body = json!({ "name": "D", "cards": cards, "talents": [0, 0, 0, 0] });
    let id = send(app, req("POST", "/api/decks", cookie, Some(body))).await.body["id"].as_i64().unwrap();
    let res = send(app, req("POST", &format!("/api/decks/{id}/publish"), cookie, None)).await;
    assert_eq!(res.status, StatusCode::OK);
    id
}

async fn comment(app: &Router, cookie: &str, deck: i64, body: &str) -> i64 {
    let res =
        send(app, req("POST", &format!("/api/decks/{deck}/comments"), cookie, Some(json!({ "body": body })))).await;
    assert_eq!(res.status, StatusCode::CREATED);
    res.body["id"].as_i64().unwrap()
}

async fn report(app: &Router, cookie: &str, kind: &str, id: i64) {
    let body = json!({ "targetType": kind, "targetId": id, "reason": "bad" });
    assert_eq!(send(app, req("POST", "/api/reports", cookie, Some(body))).await.status, StatusCode::NO_CONTENT);
}

fn resolve(cookie: &str, id: &Value, action: &str) -> axum::http::Request<axum::body::Body> {
    req("POST", &format!("/api/admin/reports/{id}/resolve"), cookie, Some(json!({ "action": action })))
}

#[sqlx::test]
async fn non_admin_is_forbidden(pool: PgPool) {
    let app = app(pool.clone());
    let (id, cookie) = user(&pool, "76561190000000001").await;
    let calls = [
        ("GET", "/api/admin/reports".to_string(), None),
        ("POST", "/api/admin/reports/1/resolve".to_string(), Some(json!({ "action": "dismiss" }))),
        ("POST", format!("/api/admin/users/{id}/ban"), None),
        ("POST", format!("/api/admin/users/{id}/unban"), None),
        ("POST", format!("/api/admin/users/{id}/unlink"), None),
        ("GET", "/api/admin/matches/disputed".to_string(), None),
        ("GET", "/api/admin/unknown-cards".to_string(), None),
    ];
    for (method, uri, body) in calls {
        assert_eq!(send(&app, req(method, &uri, &cookie, body.clone())).await.status, StatusCode::FORBIDDEN, "{uri}");
        assert_eq!(send(&app, req(method, &uri, "", body)).await.status, StatusCode::UNAUTHORIZED, "{uri}");
    }
    let banned: bool = sqlx::query_scalar("SELECT banned_at IS NOT NULL FROM users").fetch_one(&pool).await.unwrap();
    assert!(!banned);
}

#[sqlx::test]
async fn resolve_reports(pool: PgPool) {
    let app = app(pool.clone());
    let (_, admin) = admin(&pool, "76561190000000009").await;
    let (a_id, a) = user(&pool, "76561190000000001").await;
    let (b_id, b) = user(&pool, "76561190000000002").await;
    let (c_id, c) = user(&pool, "76561190000000003").await;
    let deck = public_deck(&app, &a, SEAT0_CARDS).await;
    let note = comment(&app, &b, deck, "rude words").await;
    report(&app, &c, "deck", deck).await;
    report(&app, &c, "comment", note).await;

    let open = send(&app, get("/api/admin/reports?status=open", &admin)).await.body;
    assert_eq!(open.as_array().unwrap().len(), 2);
    assert_eq!(open[0]["targetType"], "deck");
    assert_eq!(open[0]["targetId"], deck);
    assert_eq!(open[0]["targetAuthorId"], a_id);
    assert_eq!(open[0]["targetText"], "D");
    assert_eq!(open[0]["reporterId"], c_id);
    assert_eq!(open[0]["status"], "open");
    assert_eq!(open[1]["targetType"], "comment");
    assert_eq!(open[1]["targetId"], note);
    assert_eq!(open[1]["deckId"], deck);
    assert_eq!(open[1]["targetAuthorId"], b_id);
    assert_eq!(open[1]["targetText"], "rude words");

    assert_eq!(send(&app, resolve(&admin, &open[1]["id"], "delete")).await.status, StatusCode::NO_CONTENT);
    assert_eq!(count(&pool, "comments").await, 0);
    assert_eq!(send(&app, resolve(&admin, &open[0]["id"], "dismiss")).await.status, StatusCode::NO_CONTENT);
    assert_eq!(send(&app, resolve(&admin, &open[0]["id"], "dismiss")).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, resolve(&admin, &json!(999_999), "delete")).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, get("/api/admin/reports", &admin)).await.body, json!([]));
    let dismissed = send(&app, get("/api/admin/reports?status=dismissed", &admin)).await.body;
    assert_eq!(dismissed.as_array().unwrap().len(), 1);
    assert!(dismissed[0]["resolvedAt"].is_string());
    assert_eq!(send(&app, get(&format!("/api/decks/{deck}"), "")).await.status, StatusCode::OK);

    report(&app, &c, "deck", deck).await;
    let open = send(&app, get("/api/admin/reports", &admin)).await.body;
    assert_eq!(open.as_array().unwrap().len(), 1);
    assert_eq!(send(&app, resolve(&admin, &open[0]["id"], "delete")).await.status, StatusCode::NO_CONTENT);
    assert_eq!(send(&app, get(&format!("/api/decks/{deck}"), "")).await.status, StatusCode::NOT_FOUND);
    assert_eq!(count(&pool, "decks").await, 0);
    assert_eq!(count(&pool, "reports").await, 0);
}

#[sqlx::test]
async fn ban_deletes_sessions_and_blocks_writes(pool: PgPool) {
    let app = app(pool.clone());
    let (_, admin) = admin(&pool, "76561190000000009").await;
    let (b_id, b) = user(&pool, "76561190000000002").await;
    sqlx::query("INSERT INTO devices (user_id, token_hash) VALUES ($1, $2)")
        .bind(b_id)
        .bind(api::auth::hash("device"))
        .execute(&pool)
        .await
        .unwrap();
    let uri = |action: &str| format!("/api/admin/users/{b_id}/{action}");
    assert_eq!(send(&app, req("POST", &uri("ban"), &admin, None)).await.status, StatusCode::NO_CONTENT);
    let left: i64 = sqlx::query_scalar(
        "SELECT (SELECT count(*) FROM sessions WHERE user_id = $1) + (SELECT count(*) FROM devices WHERE user_id = $1)",
    )
    .bind(b_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(left, 0);
    assert_eq!(count(&pool, "sessions").await, 1);
    assert_eq!(send(&app, get("/api/me", &b)).await.status, StatusCode::UNAUTHORIZED);
    let banned_ids = || async {
        sqlx::query_scalar::<_, String>("SELECT steam_id FROM banned_steam_ids").fetch_all(&pool).await.unwrap()
    };
    assert_eq!(banned_ids().await, ["76561190000000002"]);
    assert_eq!(send(&app, req("POST", &uri("ban"), &admin, None)).await.status, StatusCode::NO_CONTENT);
    assert_eq!(banned_ids().await, ["76561190000000002"]);

    let b = new_session(&pool, b_id).await;
    assert_eq!(send(&app, get("/api/me", &b)).await.status, StatusCode::OK);
    let deck = json!({ "name": "D", "cards": SEAT0_CARDS, "talents": [0, 0, 0, 0] });
    assert_eq!(send(&app, req("POST", "/api/decks", &b, Some(deck.clone()))).await.status, StatusCode::FORBIDDEN);
    assert_eq!(send(&app, text_req("/api/decks/import", &b, String::new())).await.status, StatusCode::FORBIDDEN);

    assert_eq!(send(&app, req("POST", &uri("unban"), &admin, None)).await.status, StatusCode::NO_CONTENT);
    assert!(banned_ids().await.is_empty());
    assert_eq!(send(&app, req("POST", "/api/decks", &b, Some(deck))).await.status, StatusCode::CREATED);
    for action in ["ban", "unban", "unlink"] {
        let res = send(&app, req("POST", &format!("/api/admin/users/999999/{action}"), &admin, None)).await;
        assert_eq!(res.status, StatusCode::NOT_FOUND);
    }
}

#[sqlx::test]
async fn unlink(pool: PgPool) {
    let app = app(pool.clone());
    let (_, admin) = admin(&pool, "76561190000000009").await;
    let (b_id, b) = user(&pool, "76561190000000002").await;
    upload(&app, &b, &[entry(1, 0, ACCOUNT_A, ACCOUNT_B, None)], &[]).await;
    let other = [entry(2, 0, ACCOUNT_B, ACCOUNT_A, None)];
    assert_eq!(statuses(&upload(&app, &b, &other, &[]).await.body, "entries"), ["rejected"]);
    let res = send(&app, req("POST", &format!("/api/admin/users/{b_id}/unlink"), &admin, None)).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert_eq!(send(&app, get("/api/me", &b)).await.body["spamAccountId"], Value::Null);
    assert_eq!(statuses(&upload(&app, &b, &other, &[]).await.body, "entries"), ["new"]);
    assert_eq!(send(&app, get("/api/me", &b)).await.body["spamAccountId"], ACCOUNT_B);
}

#[sqlx::test]
async fn disputed_matches_and_unknown_cards(pool: PgPool) {
    let app = app(pool.clone());
    let (_, admin) = admin(&pool, "76561190000000009").await;
    let (a_id, a) = user(&pool, "76561190000000001").await;
    let (b_id, b) = user(&pool, "76561190000000002").await;
    let unknown = "00000000-0000-4000-8000-0000000000c1";
    let mut first = entry(1, 0, ACCOUNT_A, ACCOUNT_B, Some(ACCOUNT_A));
    first["OpponentCards"][0] = json!(unknown);
    upload(&app, &a, &[first], &[]).await;
    upload(&app, &b, &[entry(1, 1, ACCOUNT_B, ACCOUNT_A, Some(ACCOUNT_B))], &[]).await;
    upload(&app, &a, &[entry(2, 0, ACCOUNT_A, ACCOUNT_B, None)], &[]).await;
    sqlx::query("INSERT INTO unknown_cards VALUES ($1::uuid, 1, now(), now())")
        .bind(SEAT0_CARDS[0])
        .execute(&pool)
        .await
        .unwrap();

    let disputed = send(&app, get("/api/admin/matches/disputed", &admin)).await.body;
    assert_eq!(disputed.as_array().unwrap().len(), 1);
    assert_eq!(disputed[0]["gameUid"], "match-1");
    assert_eq!(disputed[0]["uploaderIds"], json!([a_id, b_id]));
    assert_eq!(disputed[0]["winnerSeat"], 0);
    let cards = send(&app, get("/api/admin/unknown-cards", &admin)).await.body;
    assert_eq!(cards.as_array().unwrap().len(), 1);
    assert_eq!(cards[0]["cardId"], unknown);
    assert_eq!(cards[0]["seenCount"], 1);
}

#[sqlx::test]
async fn account_deletion(pool: PgPool) {
    let app = app(pool.clone());
    let (a_id, a) = user(&pool, "76561190000000001").await;
    let (b_id, b) = user(&pool, "76561190000000002").await;
    sqlx::query("INSERT INTO devices (user_id, token_hash) VALUES ($1, $2)")
        .bind(a_id)
        .bind(api::auth::hash("device"))
        .execute(&pool)
        .await
        .unwrap();
    let ea = entry(1, 0, ACCOUNT_A, ACCOUNT_B, Some(ACCOUNT_A));
    let eb = entry(1, 1, ACCOUNT_B, ACCOUNT_A, Some(ACCOUNT_A));
    upload(&app, &a, std::slice::from_ref(&ea), &[("a.srpl".into(), srpl(&ea))]).await;
    upload(&app, &b, std::slice::from_ref(&eb), &[("b.srpl".into(), srpl(&eb))]).await;
    sqlx::query("INSERT INTO device_codes VALUES ($1, $2, $3, now() + interval '5 minutes')")
        .bind(api::auth::hash("dc"))
        .bind(api::auth::hash("uc"))
        .bind(a_id)
        .execute(&pool)
        .await
        .unwrap();

    let deck_a = public_deck(&app, &a, SEAT0_CARDS).await;
    let draft = json!({ "name": "Draft", "cards": SEAT1_CARDS, "talents": [0, 0, 0, 0] });
    send(&app, req("POST", "/api/decks", &a, Some(draft))).await;
    let deck_b = public_deck(&app, &b, SEAT1_CARDS).await;
    comment(&app, &a, deck_a, "own").await;
    comment(&app, &a, deck_b, "on b").await;
    comment(&app, &b, deck_a, "b on a").await;
    let kept = comment(&app, &b, deck_b, "b on b").await;
    send(&app, req("PUT", &format!("/api/decks/{deck_b}/vote"), &a, None)).await;
    send(&app, req("PUT", &format!("/api/decks/{deck_a}/vote"), &b, None)).await;
    report(&app, &a, "deck", deck_b).await;
    report(&app, &b, "deck", deck_a).await;

    let res = send(&app, req("DELETE", "/api/me", &a, None)).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    let cookie = res.headers["set-cookie"].to_str().unwrap();
    assert!(cookie.starts_with("session=;") && cookie.contains("Max-Age=0"), "{cookie}");
    assert_eq!(send(&app, get("/api/me", &a)).await.status, StatusCode::UNAUTHORIZED);

    for sql in [
        "SELECT count(*) FROM users WHERE id = $1",
        "SELECT count(*) FROM sessions WHERE user_id = $1",
        "SELECT count(*) FROM devices WHERE user_id = $1",
        "SELECT count(*) FROM device_codes WHERE user_id = $1",
        "SELECT count(*) FROM uploads WHERE user_id = $1",
        "SELECT count(*) FROM replays WHERE user_id = $1",
        "SELECT count(*) FROM decks WHERE author_id = $1",
        "SELECT count(*) FROM comments WHERE author_id = $1",
        "SELECT count(*) FROM votes WHERE user_id = $1",
        "SELECT count(*) FROM reports WHERE reporter_id = $1",
    ] {
        let n: i64 = sqlx::query_scalar(sql).bind(a_id).fetch_one(&pool).await.unwrap();
        assert_eq!(n, 0, "{sql}");
    }
    assert_eq!(count(&pool, "matches").await, 1);
    assert_eq!(count(&pool, "uploads").await, 1);
    assert_eq!(count(&pool, "replays").await, 1);
    assert_eq!(count(&pool, "users").await, 1);
    assert_eq!(count(&pool, "sessions").await, 1);
    assert_eq!(count(&pool, "decks").await, 1);
    assert_eq!(count(&pool, "votes").await, 0);
    assert_eq!(count(&pool, "reports").await, 0);
    let comments = send(&app, get(&format!("/api/decks/{deck_b}/comments"), "")).await.body;
    assert_eq!(comments.as_array().unwrap().len(), 1);
    assert_eq!(comments[0]["id"], kept);
    assert_eq!(count(&pool, "comments").await, 1);
    assert_eq!(send(&app, get("/api/me", &b)).await.body["id"], b_id);
}
