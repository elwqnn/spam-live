mod common;

use axum::{Router, http::StatusCode};
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;

const UNKNOWN: &str = "00000000-0000-4000-8000-0000000000c1";
const G1: &str = "00000000-0000-4000-8000-0000000000d1";
const G2: &str = "00000000-0000-4000-8000-0000000000d2";
const G3: &str = "00000000-0000-4000-8000-0000000000d3";

fn game_deck(id: &str, name: &str, cards: &[&str], talents: &[i64]) -> Value {
    let cards: Vec<Value> = cards
        .iter()
        .zip(talents)
        .map(|(c, t)| json!({ "id": c, "talent": t, "talents": [0], "cost": 100, "style": "x", "isAlive": true }))
        .collect();
    json!({ "id": id, "name": name, "order": 0, "locked": false, "ownedTalentsVerified": true, "cards": cards })
}

fn line(account: &str, added: &[Value], updated: &[Value], removed: &[Value]) -> String {
    let body = json!({ "userId": account, "added": added, "updated": updated, "removed": removed });
    format!("<color=#FFA000>[UserData]</color> Sending decks to server: {body}")
}

async fn import(app: &Router, cookie: &str, lines: &[String]) -> Res {
    send(app, text_req("/api/decks/import", cookie, lines.join("\n"))).await
}

fn results(res: &Res) -> Vec<(String, String)> {
    res.body["decks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| (d["gameDeckId"].as_str().unwrap().to_string(), d["status"].as_str().unwrap().to_string()))
        .collect()
}

async fn deck(app: &Router, cookie: &str, id: &Value) -> Res {
    send(app, get(&format!("/api/decks/{id}"), cookie)).await
}

async fn create(app: &Router, cookie: &str, name: &str, cards: [&str; 4]) -> i64 {
    let body = json!({ "name": name, "cards": cards, "talents": [0, 1, 2, 0] });
    let res = send(app, req("POST", "/api/decks", cookie, Some(body))).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.body);
    res.body["id"].as_i64().unwrap()
}

async fn action(app: &Router, cookie: &str, id: i64, name: &str) -> Res {
    send(app, req("POST", &format!("/api/decks/{id}/{name}"), cookie, None)).await
}

fn ids(body: &Value) -> Vec<i64> {
    body.as_array().unwrap().iter().map(|d| d["id"].as_i64().unwrap()).collect()
}

#[sqlx::test]
async fn import_new_unchanged_updated(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let first = line(ACCOUNT_A, &[game_deck(G1, "  My deck  ", &SEAT0_CARDS, &[0, 1, 2, 0])], &[], &[]);
    let res = import(&app, &cookie, std::slice::from_ref(&first)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.body);
    assert_eq!(results(&res), [(G1.to_string(), "new".to_string())]);
    assert_eq!(res.body["decks"][0]["name"], "My deck");
    let id = res.body["decks"][0]["deckId"].clone();
    let d = deck(&app, &cookie, &id).await.body;
    assert_eq!(d["name"], "My deck");
    assert_eq!(d["status"], "draft");
    assert_eq!(d["cards"], json!(SEAT0_CARDS));
    assert_eq!(d["talents"], json!([0, 1, 2, 0]));
    assert_eq!(d["gameDeckId"], G1);
    assert_eq!(d["appVersion"], "1.29");

    let again = import(&app, &cookie, &[first]).await;
    assert_eq!(results(&again)[0].1, "unchanged");
    assert_eq!(again.body["decks"][0]["deckId"], id);

    let changed = line(ACCOUNT_A, &[], &[game_deck(G1, "My deck", &SEAT0_CARDS, &[0, 1, 2, 1])], &[]);
    let res = import(&app, &cookie, &[changed]).await;
    assert_eq!(results(&res)[0].1, "updated");
    assert_eq!(res.body["decks"][0]["deckId"], id);
    assert_eq!(deck(&app, &cookie, &id).await.body["talents"], json!([0, 1, 2, 1]));
    assert_eq!(count(&pool, "decks").await, 1);
}

#[sqlx::test]
async fn import_later_line_wins_and_removed_is_ignored(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let lines = [
        line(
            ACCOUNT_A,
            &[game_deck(G1, "One", &SEAT0_CARDS, &[0, 0, 0, 0]), game_deck(G2, "Two", &SEAT1_CARDS, &[1, 1, 1, 1])],
            &[],
            &[game_deck(G3, "Gone", &SEAT1_CARDS, &[0, 0, 0, 0])],
        ),
        line(ACCOUNT_A, &[], &[game_deck(G1, "One", &SEAT0_CARDS, &[2, 2, 2, 2])], &[]),
    ];
    let res = import(&app, &cookie, &lines).await;
    assert_eq!(results(&res), [(G1.to_string(), "new".to_string()), (G2.to_string(), "new".to_string())]);
    let d = deck(&app, &cookie, &res.body["decks"][0]["deckId"]).await.body;
    assert_eq!(d["talents"], json!([2, 2, 2, 2]));
    assert_eq!(count(&pool, "decks").await, 2);
}

#[sqlx::test]
async fn import_skips_invalid_decks(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let five = [SEAT0_CARDS[0], SEAT0_CARDS[1], SEAT0_CARDS[2], SEAT0_CARDS[3], SEAT1_CARDS[0]];
    let dup = [SEAT0_CARDS[0], SEAT0_CARDS[0], SEAT0_CARDS[1], SEAT0_CARDS[2]];
    let long = format!("  {}  ", "n".repeat(70));
    let decks = [
        game_deck("three", "x", &SEAT0_CARDS[..3], &[0, 0, 0]),
        game_deck("five", "x", &five, &[0, 0, 0, 0, 0]),
        game_deck("dup", "x", &dup, &[0, 0, 0, 0]),
        game_deck("talent3", "x", &SEAT0_CARDS, &[0, 1, 2, 3]),
        game_deck("negative", "x", &SEAT0_CARDS, &[0, -1, 0, 0]),
        game_deck("empty", "   ", &SEAT0_CARDS, &[0, 0, 0, 0]),
        game_deck("long", &long, &SEAT1_CARDS, &[0, 0, 0, 0]),
    ];
    let res = import(&app, &cookie, &[line(ACCOUNT_A, &decks, &[], &[])]).await;
    let statuses: Vec<String> = results(&res).into_iter().map(|r| r.1).collect();
    assert_eq!(statuses, ["skipped", "skipped", "skipped", "skipped", "skipped", "new", "new"]);
    for i in 0..3 {
        assert_eq!(res.body["decks"][i]["reason"], "a deck needs 4 distinct cards");
        assert_eq!(res.body["decks"][i]["deckId"], Value::Null);
        assert_eq!(res.body["decks"][i]["name"], Value::Null);
    }
    for i in 3..5 {
        assert_eq!(res.body["decks"][i]["reason"], "each card needs 1 talent index from 0 to 2");
    }
    let empty = deck(&app, &cookie, &res.body["decks"][5]["deckId"]).await.body;
    assert_eq!(empty["name"], "Imported deck");
    let long = deck(&app, &cookie, &res.body["decks"][6]["deckId"]).await.body;
    assert_eq!(long["name"], "n".repeat(60));
    assert_eq!(count(&pool, "decks").await, 2);
}

#[sqlx::test]
async fn import_skips_published_deck(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let res =
        import(&app, &cookie, &[line(ACCOUNT_A, &[game_deck(G1, "D", &SEAT0_CARDS, &[0, 0, 0, 0])], &[], &[])]).await;
    let id = res.body["decks"][0]["deckId"].as_i64().unwrap();
    assert_eq!(action(&app, &cookie, id, "publish").await.status, StatusCode::OK);
    let res =
        import(&app, &cookie, &[line(ACCOUNT_A, &[], &[game_deck(G1, "D", &SEAT0_CARDS, &[1, 1, 1, 1])], &[])]).await;
    assert_eq!(results(&res)[0].1, "skipped");
    assert_eq!(res.body["decks"][0]["reason"], "published");
    assert_eq!(res.body["decks"][0]["deckId"], id);
    let d = deck(&app, &cookie, &json!(id)).await.body;
    assert_eq!((d["status"].clone(), d["talents"].clone()), (json!("public"), json!([0, 0, 0, 0])));
}

#[sqlx::test]
async fn import_account_link(pool: PgPool) {
    let app = app(pool.clone());
    let (a_id, a) = user(&pool, "76561190000000001").await;
    let (b_id, b) = user(&pool, "76561190000000002").await;
    let deck_a = || line(ACCOUNT_A, &[game_deck(G1, "D", &SEAT0_CARDS, &[0, 0, 0, 0])], &[], &[]);
    let deck_b = line(ACCOUNT_B, &[game_deck(G2, "D", &SEAT0_CARDS, &[0, 0, 0, 0])], &[], &[]);
    assert_eq!(results(&import(&app, &a, &[deck_a()]).await)[0].1, "new");
    let link = |id: i64| {
        let pool = pool.clone();
        async move {
            sqlx::query_scalar::<_, Option<uuid::Uuid>>("SELECT spam_account_id FROM users WHERE id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap()
                .map(|u| u.to_string())
        }
    };
    assert_eq!(link(a_id).await.as_deref(), Some(ACCOUNT_A));

    let res = import(&app, &a, &[deck_b]).await;
    assert_eq!(results(&res)[0].1, "skipped");
    assert!(res.body["decks"][0]["reason"].as_str().unwrap().contains("linked to your Steam account"));

    let res = import(&app, &b, &[deck_a()]).await;
    assert_eq!(results(&res)[0].1, "skipped");
    assert!(res.body["decks"][0]["reason"].as_str().unwrap().contains("another Steam account"));
    assert_eq!(link(b_id).await, None);
    assert_eq!(count(&pool, "decks").await, 1);
}

#[sqlx::test]
async fn import_unknown_card_blocks_publication(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let cards = [UNKNOWN, SEAT0_CARDS[1], SEAT0_CARDS[2], SEAT0_CARDS[3]];
    let res = import(&app, &cookie, &[line(ACCOUNT_A, &[game_deck(G1, "D", &cards, &[0, 0, 0, 0])], &[], &[])]).await;
    assert_eq!(results(&res)[0].1, "new");
    let id = res.body["decks"][0]["deckId"].as_i64().unwrap();
    let blocked = action(&app, &cookie, id, "publish").await;
    assert_eq!(blocked.status, StatusCode::BAD_REQUEST);
    assert_eq!(blocked.body["error"], "cards are not in the catalog: 00000000");
    assert_eq!(deck(&app, &cookie, &json!(id)).await.body["status"], "draft");
    let patch =
        send(&app, req("PATCH", &format!("/api/decks/{id}"), &cookie, Some(json!({ "cards": SEAT0_CARDS })))).await;
    assert_eq!(patch.status, StatusCode::OK);
    assert_eq!(action(&app, &cookie, id, "publish").await.body["status"], "public");
}

#[sqlx::test]
async fn import_rejects_bad_body(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let valid = line(ACCOUNT_A, &[], &[], &[]);
    assert_eq!(import(&app, "", std::slice::from_ref(&valid)).await.status, StatusCode::UNAUTHORIZED);
    let no_marker = import(&app, &cookie, &[valid.clone(), "hello".into()]).await;
    assert_eq!(no_marker.status, StatusCode::BAD_REQUEST);
    assert_eq!(no_marker.body["error"], "line 2 has no deck marker");
    let bad_json = import(&app, &cookie, &["Sending decks to server: {".into()]).await;
    assert_eq!(bad_json.status, StatusCode::BAD_REQUEST);
    let empty = import(&app, &cookie, &["".into(), "  ".into()]).await;
    assert_eq!(empty.status, StatusCode::OK);
    assert_eq!(empty.body["decks"], json!([]));
    let huge = format!("{valid}\n{}", " ".repeat(1 << 20));
    assert_eq!(send(&app, text_req("/api/decks/import", &cookie, huge)).await.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(count(&pool, "users").await, 1);
    let linked: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT spam_account_id FROM users").fetch_one(&pool).await.unwrap();
    assert_eq!(linked, None);
}

#[sqlx::test]
async fn deck_changes_are_author_only(pool: PgPool) {
    let app = app(pool.clone());
    let (a_id, a) = user(&pool, "76561190000000001").await;
    let (_, b) = user(&pool, "76561190000000002").await;
    let body = json!({ "name": "Mine", "cards": SEAT0_CARDS, "talents": [0, 1, 2, 0], "guide": "Play **well**" });
    let created = send(&app, req("POST", "/api/decks", &a, Some(body))).await;
    assert_eq!(created.status, StatusCode::CREATED);
    assert_eq!(created.body["status"], "draft");
    assert_eq!(created.body["authorId"], a_id);
    assert_eq!(created.body["authorName"], "76561190000000001");
    assert_eq!(created.body["guideHtml"], "<p>Play <strong>well</strong></p>\n");
    assert_eq!((created.body["upvotes"].clone(), created.body["downvotes"].clone()), (json!(0), json!(0)));
    let id = created.body["id"].as_i64().unwrap();
    let uri = format!("/api/decks/{id}");
    let patch = |cookie: &str, body: Value| req("PATCH", &uri, cookie, Some(body));

    assert_eq!(send(&app, get(&uri, &b)).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, get(&uri, "")).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, get(&uri, &a)).await.status, StatusCode::OK);
    assert_eq!(send(&app, patch(&b, json!({ "name": "x" }))).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, req("DELETE", &uri, &b, None)).await.status, StatusCode::NOT_FOUND);
    assert_eq!(action(&app, &b, id, "publish").await.status, StatusCode::NOT_FOUND);

    let published = action(&app, &a, id, "publish").await;
    assert_eq!(published.body["status"], "public");
    assert!(published.body["publishedAt"].is_string());
    let anonymous = send(&app, get(&uri, "")).await;
    assert_eq!(anonymous.status, StatusCode::OK);
    assert_eq!(anonymous.body["myVote"], Value::Null);
    assert_eq!(send(&app, get(&uri, &b)).await.body["myVote"], Value::Null);
    assert_eq!(send(&app, patch(&b, json!({ "name": "x" }))).await.status, StatusCode::FORBIDDEN);
    assert_eq!(send(&app, req("DELETE", &uri, &b, None)).await.status, StatusCode::FORBIDDEN);
    assert_eq!(action(&app, &b, id, "unpublish").await.status, StatusCode::FORBIDDEN);

    let renamed = send(&app, patch(&a, json!({ "name": " Renamed ", "talents": [2, 2, 2, 2] }))).await;
    assert_eq!(renamed.status, StatusCode::OK);
    assert_eq!(renamed.body["name"], "Renamed");
    assert_eq!(renamed.body["talents"], json!([2, 2, 2, 2]));
    assert_eq!(renamed.body["guide"], "Play **well**");
    assert_eq!(renamed.body["status"], "public");

    assert_eq!(action(&app, &a, id, "unpublish").await.body["status"], "draft");
    assert_eq!(send(&app, get(&uri, &b)).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, req("DELETE", &uri, &a, None)).await.status, StatusCode::NO_CONTENT);
    assert_eq!(send(&app, get(&uri, &a)).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, req("DELETE", &uri, &a, None)).await.status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn deck_limits(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let try_create = |name: &str, cards: Value, talents: Value, guide: String| {
        req("POST", "/api/decks", &a, Some(json!({ "name": name, "cards": cards, "talents": talents, "guide": guide })))
    };
    let ok_cards = json!(SEAT0_CARDS);
    let ok_talents = json!([0, 1, 2, 0]);
    let cases = [
        try_create("", ok_cards.clone(), ok_talents.clone(), String::new()),
        try_create("   ", ok_cards.clone(), ok_talents.clone(), String::new()),
        try_create(&"x".repeat(61), ok_cards.clone(), ok_talents.clone(), String::new()),
        try_create("ok", ok_cards.clone(), ok_talents.clone(), "g".repeat(50_001)),
        try_create("ok", json!(&SEAT0_CARDS[..3]), json!([0, 0, 0]), String::new()),
        try_create(
            "ok",
            json!([SEAT0_CARDS[0], SEAT0_CARDS[0], SEAT0_CARDS[1], SEAT0_CARDS[2]]),
            ok_talents.clone(),
            String::new(),
        ),
        try_create("ok", ok_cards.clone(), json!([0, 1, 2, 3]), String::new()),
        try_create("ok", ok_cards.clone(), json!([0, 1, 2]), String::new()),
    ];
    for (i, r) in cases.into_iter().enumerate() {
        assert_eq!(send(&app, r).await.status, StatusCode::BAD_REQUEST, "case {i}");
    }
    let max = send(&app, try_create(&"é".repeat(60), ok_cards.clone(), ok_talents.clone(), "g".repeat(50_000))).await;
    assert_eq!(max.status, StatusCode::CREATED);
    let id = max.body["id"].as_i64().unwrap();
    let uri = format!("/api/decks/{id}");
    for body in
        [json!({ "name": "x".repeat(61) }), json!({ "guide": "g".repeat(50_001) }), json!({ "talents": [3, 0, 0, 0] })]
    {
        assert_eq!(send(&app, req("PATCH", &uri, &a, Some(body))).await.status, StatusCode::BAD_REQUEST);
    }
    assert_eq!(action(&app, &a, id, "publish").await.status, StatusCode::OK);
    let unknown = json!({ "cards": [UNKNOWN, SEAT0_CARDS[1], SEAT0_CARDS[2], SEAT0_CARDS[3]] });
    assert_eq!(send(&app, req("PATCH", &uri, &a, Some(unknown))).await.status, StatusCode::BAD_REQUEST);
    assert_eq!(send(&app, get(&uri, "")).await.body["cards"], json!(SEAT0_CARDS));
}

#[sqlx::test]
async fn guide_html_is_safe(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let guide = "# Title\n\n<script>alert(1)</script>\n\n[bad](javascript:alert(1)) [ok](https://example.com) \
                 <a href=\"javascript:alert(2)\" onclick=\"x()\">raw</a> ![p](data:image/png;base64,AAAA)";
    let body = json!({ "name": "G", "cards": SEAT0_CARDS, "talents": [0, 0, 0, 0], "guide": guide });
    let res = send(&app, req("POST", "/api/decks", &a, Some(body))).await;
    let html = res.body["guideHtml"].as_str().unwrap();
    assert!(html.contains("<h1>Title</h1>"), "{html}");
    assert!(!html.contains("<script"), "{html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"), "{html}");
    assert!(html.contains("<a rel=\"nofollow ugc noopener\">bad</a>"), "{html}");
    assert!(!html.contains("<a href=\"javascript"), "{html}");
    assert!(html.contains("&lt;a href=\"javascript:alert(2)\" onclick=\"x()\"&gt;raw&lt;/a&gt;"), "{html}");
    assert!(html.contains("<img alt=\"p\">"), "{html}");
    assert!(!html.contains("data:image"), "{html}");
    assert!(html.contains("<a href=\"https://example.com\" rel=\"nofollow ugc noopener\">ok</a>"), "{html}");
    assert_eq!(res.body["guide"], guide);
}

#[sqlx::test]
async fn guide_preview(pool: PgPool) {
    let app = app(pool.clone());
    let (a_id, a) = user(&pool, "76561190000000001").await;
    let preview =
        |cookie: &str, guide: &str| req("POST", "/api/decks/preview", cookie, Some(json!({ "guide": guide })));
    let guide = "# Title\n\n<script>alert(1)</script>\n\n- [ok](https://example.com)";
    assert_eq!(send(&app, preview("", guide)).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(send(&app, preview(&a, &"g".repeat(50_001))).await.status, StatusCode::BAD_REQUEST);
    assert_eq!(send(&app, preview(&a, &"g".repeat(50_000))).await.status, StatusCode::OK);

    let res = send(&app, preview(&a, guide)).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.body);
    let html = res.body["html"].as_str().unwrap();
    assert!(!html.contains("<script"), "{html}");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"), "{html}");
    let body = json!({ "name": "G", "cards": SEAT0_CARDS, "talents": [0, 0, 0, 0], "guide": guide });
    let created = send(&app, req("POST", "/api/decks", &a, Some(body))).await;
    assert_eq!(created.body["guideHtml"], res.body["html"]);

    sqlx::query("UPDATE users SET banned_at = now() WHERE id = $1").bind(a_id).execute(&pool).await.unwrap();
    assert_eq!(send(&app, preview(&a, guide)).await.status, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn list_decks_filters(pool: PgPool) {
    let app = app(pool.clone());
    let (a_id, a) = user(&pool, "76561190000000001").await;
    let (_, b) = user(&pool, "76561190000000002").await;
    let d1 = create(&app, &a, "One", SEAT0_CARDS).await;
    let d2 = create(&app, &a, "Two", SEAT1_CARDS).await;
    let d3 = create(&app, &b, "Three", SEAT1_CARDS).await;
    action(&app, &a, d1, "publish").await;
    action(&app, &b, d3, "publish").await;
    sqlx::query("UPDATE decks SET published_at = now() - make_interval(hours => id::int) WHERE status = 'public'")
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(ids(&send(&app, get("/api/decks?sort=recent", "")).await.body), [d1, d3]);
    assert_eq!(ids(&send(&app, get("/api/decks?author=me&sort=recent", &a)).await.body), [d2, d1]);
    assert_eq!(send(&app, get("/api/decks?author=me", "")).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(ids(&send(&app, get(&format!("/api/decks?author={a_id}"), &a)).await.body), [d1]);
    assert_eq!(send(&app, get("/api/decks?author=x", "")).await.status, StatusCode::BAD_REQUEST);
    assert_eq!(ids(&send(&app, get(&format!("/api/decks?card={}", SEAT1_CARDS[0]), "")).await.body), [d3]);
    let first = send(&app, get("/api/decks?sort=recent&limit=1", "")).await.body;
    assert_eq!(ids(&first), [d1]);
    let before = first[0]["publishedAt"].as_str().unwrap().replace('+', "%2B");
    assert_eq!(ids(&send(&app, get(&format!("/api/decks?sort=recent&before={before}"), "")).await.body), [d3]);
    let list = send(&app, get("/api/decks?sort=recent", "")).await.body;
    assert!(list[0].get("guide").is_none());
    assert_eq!(list[0]["authorName"], "76561190000000001");
}

#[sqlx::test]
async fn list_decks_search(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let fire = create(&app, &a, "Fire Rush", SEAT0_CARDS).await;
    let water = create(&app, &a, "Water rush", SEAT1_CARDS).await;
    let full = create(&app, &a, "100% win", SEAT0_CARDS).await;
    let plain = create(&app, &a, "100 wins", SEAT1_CARDS).await;
    let draft = create(&app, &a, "Rush draft", SEAT0_CARDS).await;
    for id in [fire, water, full, plain] {
        action(&app, &a, id, "publish").await;
    }
    sqlx::query("UPDATE decks SET published_at = now() - make_interval(hours => id::int) WHERE status = 'public'")
        .execute(&pool)
        .await
        .unwrap();
    let search = |q: &str| get(&format!("/api/decks?sort=recent&q={q}"), "");
    assert_eq!(ids(&send(&app, search("RUSH")).await.body), [fire, water]);
    assert_eq!(ids(&send(&app, search("%20%20ush%20")).await.body), [fire, water]);
    assert_eq!(ids(&send(&app, search("0%25")).await.body), [full]);
    assert_eq!(ids(&send(&app, search("0_")).await.body), Vec::<i64>::new());
    assert_eq!(ids(&send(&app, search("rush&limit=1")).await.body), [fire]);
    assert_eq!(ids(&send(&app, search(&format!("rush&card={}", SEAT1_CARDS[0]))).await.body), [water]);
    assert_eq!(ids(&send(&app, get("/api/decks?author=me&sort=recent&q=draft", &a)).await.body), [draft]);
    assert_eq!(ids(&send(&app, search("draft")).await.body), Vec::<i64>::new());
    for bad in ["r", "%20r%20", "", &"r".repeat(65)] {
        assert_eq!(send(&app, search(bad)).await.status, StatusCode::BAD_REQUEST, "{bad}");
    }
    assert_eq!(send(&app, search(&"r".repeat(64))).await.status, StatusCode::OK);
}

#[sqlx::test]
async fn votes(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let (_, b) = user(&pool, "76561190000000002").await;
    let (_, c) = user(&pool, "76561190000000003").await;
    let x = create(&app, &a, "X", SEAT0_CARDS).await;
    let y = create(&app, &a, "Y", SEAT1_CARDS).await;
    let z = create(&app, &a, "Z", SEAT0_CARDS).await;
    let w = create(&app, &a, "W", SEAT1_CARDS).await;
    let draft = create(&app, &a, "Draft", SEAT0_CARDS).await;
    for id in [x, y, z, w] {
        action(&app, &a, id, "publish").await;
    }
    let put = |cookie: &str, id: i64, value: i64| {
        req("PUT", &format!("/api/decks/{id}/vote"), cookie, Some(json!({ "value": value })))
    };
    let delete = |cookie: &str, id: i64| req("DELETE", &format!("/api/decks/{id}/vote"), cookie, None);
    let seen = async |cookie: &str, id: i64| {
        let body = send(&app, get(&format!("/api/decks/{id}"), cookie)).await.body;
        (body["upvotes"].clone(), body["downvotes"].clone(), body["myVote"].clone())
    };

    for _ in 0..2 {
        assert_eq!(send(&app, put(&b, x, 1)).await.status, StatusCode::NO_CONTENT);
    }
    assert_eq!(seen(&b, x).await, (json!(1), json!(0), json!(1)));
    for _ in 0..2 {
        assert_eq!(send(&app, put(&b, x, -1)).await.status, StatusCode::NO_CONTENT);
    }
    assert_eq!(seen(&b, x).await, (json!(0), json!(1), json!(-1)));
    assert_eq!(seen("", x).await, (json!(0), json!(1), Value::Null));
    for value in [0, 2] {
        assert_eq!(send(&app, put(&b, x, value)).await.status, StatusCode::BAD_REQUEST, "{value}");
    }
    assert_eq!(send(&app, put(&a, x, 1)).await.status, StatusCode::FORBIDDEN);
    assert_eq!(send(&app, put(&b, draft, 1)).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, put(&b, 999_999, 1)).await.status, StatusCode::NOT_FOUND);
    for _ in 0..2 {
        assert_eq!(send(&app, delete(&b, x)).await.status, StatusCode::NO_CONTENT);
    }
    assert_eq!(seen(&b, x).await, (json!(0), json!(0), Value::Null));

    for (cookie, id, value) in [(&b, y, 1), (&c, y, 1), (&b, x, 1), (&c, x, -1), (&c, z, -1)] {
        assert_eq!(send(&app, put(cookie, id, value)).await.status, StatusCode::NO_CONTENT);
    }
    let sorted = send(&app, get("/api/decks?sort=votes", "")).await.body;
    assert_eq!(ids(&sorted), [y, x, w, z]);
    assert_eq!((sorted[0]["upvotes"].clone(), sorted[0]["downvotes"].clone()), (json!(2), json!(0)));
    assert_eq!(ids(&send(&app, get("/api/decks", "")).await.body), [y, x, w, z]);
    assert_eq!(ids(&send(&app, get("/api/decks?sort=winrate", "")).await.body), [y, w, x, z]);
    let my_votes = |list: &Value| list.as_array().unwrap().iter().map(|d| d.get("myVote").cloned()).collect::<Vec<_>>();
    assert_eq!(my_votes(&sorted), vec![Some(Value::Null); 4]);
    let mine = send(&app, get("/api/decks?sort=votes", &c)).await.body;
    assert_eq!(ids(&mine), [y, x, w, z]);
    assert_eq!(my_votes(&mine), [Some(json!(1)), Some(json!(-1)), Some(Value::Null), Some(json!(-1))]);
    let page = |q: &str| get(&format!("/api/decks?sort=votes&limit=2{q}"), "");
    assert_eq!(ids(&send(&app, page("")).await.body), [y, x]);
    assert_eq!(ids(&send(&app, page("&offset=2")).await.body), [w, z]);
    assert_eq!(ids(&send(&app, page("&offset=10000")).await.body), Vec::<i64>::new());
    for bad in ["&offset=10001", "&offset=-1"] {
        assert_eq!(send(&app, page(bad)).await.status, StatusCode::BAD_REQUEST, "{bad}");
    }
}

#[sqlx::test]
async fn comments(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let (b_id, b) = user(&pool, "76561190000000002").await;
    let (_, c) = user(&pool, "76561190000000003").await;
    let deck = create(&app, &a, "D", SEAT0_CARDS).await;
    let draft = create(&app, &a, "Draft", SEAT1_CARDS).await;
    action(&app, &a, deck, "publish").await;
    let uri = format!("/api/decks/{deck}/comments");
    let post = |cookie: &str, body: String| req("POST", &uri, cookie, Some(json!({ "body": body })));

    for bad in [String::new(), "   ".into(), "c".repeat(2001)] {
        assert_eq!(send(&app, post(&b, bad)).await.status, StatusCode::BAD_REQUEST);
    }
    let long = send(&app, post(&b, "c".repeat(2000))).await;
    assert_eq!(long.status, StatusCode::CREATED);
    assert_eq!(long.body["authorId"], b_id);
    let draft_post = req("POST", &format!("/api/decks/{draft}/comments"), &b, Some(json!({ "body": "hi" })));
    assert_eq!(send(&app, draft_post).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, get(&format!("/api/decks/{draft}/comments"), "")).await.status, StatusCode::NOT_FOUND);
    let first = send(&app, post(&b, " <b>first</b> ".into())).await.body;
    send(&app, post(&c, "second".into())).await;

    let list = send(&app, get(&uri, "")).await.body;
    let bodies: Vec<_> = list.as_array().unwrap().iter().map(|c| c["body"].as_str().unwrap().len()).collect();
    assert_eq!(bodies, [2000, 12, 6]);
    assert_eq!(list[1]["body"], "<b>first</b>");
    assert_eq!(list[1]["authorName"], "76561190000000002");
    assert!(list[1].get("authorAvatar").is_some());

    let comment_uri = format!("/api/comments/{}", first["id"]);
    let edit = |cookie: &str, body: &str| req("PATCH", &comment_uri, cookie, Some(json!({ "body": body })));
    assert_eq!(send(&app, edit(&c, "hacked")).await.status, StatusCode::FORBIDDEN);
    assert_eq!(send(&app, req("DELETE", &comment_uri, &c, None)).await.status, StatusCode::FORBIDDEN);
    assert_eq!(send(&app, edit(&b, "")).await.status, StatusCode::BAD_REQUEST);
    let edited = send(&app, edit(&b, "edited")).await;
    assert_eq!(edited.status, StatusCode::OK);
    assert_eq!(edited.body["body"], "edited");
    assert_eq!(send(&app, req("DELETE", &comment_uri, &b, None)).await.status, StatusCode::NO_CONTENT);
    assert_eq!(send(&app, req("DELETE", &comment_uri, &b, None)).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, get(&uri, "")).await.body.as_array().unwrap().len(), 2);
    let decks = send(&app, get("/api/decks", "")).await.body;
    assert_eq!((decks[0]["id"].as_i64(), decks[0]["comments"].clone()), (Some(deck), json!(2)));
    assert_eq!(send(&app, get(&format!("/api/decks/{deck}"), "")).await.body["comments"], 2);
}

#[sqlx::test]
async fn comment_rate_limit(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let (_, b) = user(&pool, "76561190000000002").await;
    let deck = create(&app, &a, "D", SEAT0_CARDS).await;
    action(&app, &a, deck, "publish").await;
    let post =
        |cookie: &str| req("POST", &format!("/api/decks/{deck}/comments"), cookie, Some(json!({ "body": "hi" })));
    for _ in 0..10 {
        assert_eq!(send(&app, post(&b)).await.status, StatusCode::CREATED);
    }
    assert_eq!(send(&app, post(&b)).await.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(send(&app, post(&a)).await.status, StatusCode::CREATED);
    assert_eq!(count(&pool, "comments").await, 11);
}

#[sqlx::test]
async fn reports(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let (_, b) = user(&pool, "76561190000000002").await;
    let deck = create(&app, &a, "D", SEAT0_CARDS).await;
    let draft = create(&app, &a, "Draft", SEAT1_CARDS).await;
    action(&app, &a, deck, "publish").await;
    let comment = send(&app, req("POST", &format!("/api/decks/{deck}/comments"), &a, Some(json!({ "body": "x" }))))
        .await
        .body["id"]
        .as_i64()
        .unwrap();
    let report = |kind: &str, id: i64, reason: String| {
        req("POST", "/api/reports", &b, Some(json!({ "targetType": kind, "targetId": id, "reason": reason })))
    };
    for _ in 0..2 {
        assert_eq!(send(&app, report("deck", deck, "spam".into())).await.status, StatusCode::NO_CONTENT);
    }
    assert_eq!(count(&pool, "reports").await, 1);
    assert_eq!(send(&app, report("deck", deck, " ".into())).await.status, StatusCode::BAD_REQUEST);
    assert_eq!(send(&app, report("deck", deck, "r".repeat(501))).await.status, StatusCode::BAD_REQUEST);
    assert_eq!(send(&app, report("deck", draft, "spam".into())).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, report("comment", 999_999, "spam".into())).await.status, StatusCode::NOT_FOUND);
    assert_eq!(send(&app, report("comment", comment, "r".repeat(500))).await.status, StatusCode::NO_CONTENT);
    assert_eq!(send(&app, report("comment", comment, "again".into())).await.status, StatusCode::NO_CONTENT);
    assert_eq!(count(&pool, "reports").await, 2);
}

#[sqlx::test]
async fn concurrent_imports_of_one_deck(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    sqlx::query("UPDATE users SET spam_account_id = $1::uuid").bind(ACCOUNT_A).execute(&pool).await.unwrap();
    for round in 0..5 {
        let lines =
            [line(ACCOUNT_A, &[game_deck(&format!("deck-{round}"), "D", &SEAT0_CARDS, &[0, 0, 0, 0])], &[], &[])];
        let (one, two) = tokio::join!(import(&app, &cookie, &lines), import(&app, &cookie, &lines));
        assert_eq!((one.status, two.status), (StatusCode::OK, StatusCode::OK), "{} {}", one.body, two.body);
        let mut statuses = [results(&one)[0].1.clone(), results(&two)[0].1.clone()];
        statuses.sort();
        assert_eq!(statuses, ["new", "unchanged"]);
        assert_eq!(one.body["decks"][0]["deckId"], two.body["decks"][0]["deckId"]);
    }
    assert_eq!(count(&pool, "decks").await, 5);
}
