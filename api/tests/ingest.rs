mod common;

use std::path::Path;

use axum::http::StatusCode;
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;

const OPP: &str = "00000000-0000-4000-8000-0000000000ff";

fn replay(e: &Value) -> (String, Vec<u8>) {
    (format!("{}.srpl", e["GameUid"].as_str().unwrap()), srpl(e))
}

#[sqlx::test]
async fn win_is_stored_without_personal_data(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let e = entry(1, 0, ACCOUNT_A, OPP, Some(ACCOUNT_A));
    let res = upload(&app, &cookie, std::slice::from_ref(&e), &[replay(&e)]).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(statuses(&res.body, "entries"), ["new"]);
    assert_eq!(statuses(&res.body, "replays"), ["new"]);
    let h = history(&app, &cookie).await;
    assert_eq!(h.len(), 1);
    assert_eq!(h[0]["result"], "win");
    assert_eq!(h[0]["mySeat"], 0);
    assert_eq!(h[0]["myCards"].as_array().unwrap().len(), 4);
    let (row, stars): (String, Option<i32>) =
        sqlx::query_as("SELECT row_to_json(m)::text, seat0_stars FROM matches m").fetch_one(&pool).await.unwrap();
    assert_eq!(stars, Some(20));
    for secret in [OPP, "FakeOpponent", "fake-ticket", "FakeMe"] {
        assert!(!row.contains(secret));
    }
    let linked: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT spam_account_id FROM users").fetch_one(&pool).await.unwrap();
    assert_eq!(linked.unwrap().to_string(), ACCOUNT_A);
}

#[sqlx::test]
async fn loss(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let res = upload(&app, &cookie, &[entry(1, 1, ACCOUNT_A, OPP, Some(OPP))], &[]).await;
    assert_eq!(statuses(&res.body, "entries"), ["new"]);
    let h = history(&app, &cookie).await;
    assert_eq!(h[0]["result"], "loss");
    assert_eq!(h[0]["mySeat"], 1);
    assert_eq!(h[0]["myCards"], json!(SEAT1_CARDS));
    assert_eq!(h[0]["opponentCards"], json!(SEAT0_CARDS));
    let winner: Option<i16> = sqlx::query_scalar("SELECT winner_seat FROM matches").fetch_one(&pool).await.unwrap();
    assert_eq!(winner, Some(0));
}

#[sqlx::test]
async fn draw(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let res = upload(&app, &cookie, &[entry(1, 0, ACCOUNT_A, OPP, None)], &[]).await;
    assert_eq!(statuses(&res.body, "entries"), ["new"]);
    assert_eq!(history(&app, &cookie).await[0]["result"], "draw");
    let (winner, is_draw): (Option<i16>, bool) =
        sqlx::query_as("SELECT winner_seat, is_draw FROM matches").fetch_one(&pool).await.unwrap();
    assert_eq!((winner, is_draw), (None, true));
}

#[sqlx::test]
async fn ai_matches_and_mode_40(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let mut ai_win = entry(1, 0, ACCOUNT_A, OPP, Some(ACCOUNT_A));
    ai_win["OpponentIsAi"] = json!(true);
    let mut ai_loss_null = entry(2, 1, ACCOUNT_A, OPP, None);
    ai_loss_null["OpponentIsAi"] = json!(true);
    ai_loss_null["IsDraw"] = json!(false);
    ai_loss_null["OpponentId"] = Value::Null;
    let mut ai_loss_absent = entry(3, 0, ACCOUNT_A, OPP, Some("ai-bot"));
    ai_loss_absent["OpponentIsAi"] = json!(true);
    ai_loss_absent.as_object_mut().unwrap().remove("OpponentId");
    let mut duel = entry(4, 0, ACCOUNT_A, OPP, Some(OPP));
    duel["GameMode"] = json!(40);
    let mut human_null = entry(5, 0, ACCOUNT_A, OPP, None);
    human_null["IsDraw"] = json!(false);
    human_null["OpponentId"] = Value::Null;
    let res = upload(&app, &cookie, &[ai_win, ai_loss_null, ai_loss_absent, duel, human_null], &[]).await;
    assert_eq!(statuses(&res.body, "entries"), ["new", "new", "new", "new", "rejected"]);
    assert_eq!(reason(&res.body, "entries", 4), "WinnerId is not a player of the match");
    let h = history(&app, &cookie).await;
    let find = |uid: &str| h.iter().find(|m| m["gameUid"] == uid).unwrap().clone();
    assert_eq!((find("match-1")["isAi"].clone(), find("match-1")["result"].clone()), (json!(true), json!("win")));
    assert_eq!((find("match-2")["isAi"].clone(), find("match-2")["result"].clone()), (json!(true), json!("loss")));
    assert_eq!(find("match-3")["result"], "loss");
    assert_eq!((find("match-4")["gameMode"].clone(), find("match-4")["isAi"].clone()), (json!(40), json!(false)));
    let winners: Vec<Option<i16>> =
        sqlx::query_scalar("SELECT winner_seat FROM matches ORDER BY game_uid").fetch_all(&pool).await.unwrap();
    assert_eq!(winners, [Some(0), Some(0), Some(1), Some(1)]);
}

#[sqlx::test]
async fn same_upload_twice(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let entries = [entry(1, 0, ACCOUNT_A, OPP, Some(ACCOUNT_A)), entry(2, 1, ACCOUNT_A, OPP, None)];
    let replays: Vec<_> = entries.iter().map(replay).collect();
    let first = upload(&app, &cookie, &entries, &replays).await;
    assert_eq!(statuses(&first.body, "entries"), ["new", "new"]);
    let second = upload(&app, &cookie, &entries, &replays).await;
    assert_eq!(second.status, StatusCode::OK);
    assert_eq!(statuses(&second.body, "entries"), ["known", "known"]);
    assert_eq!(statuses(&second.body, "replays"), ["known", "known"]);
    assert_eq!(count(&pool, "matches").await, 2);
    assert_eq!(count(&pool, "uploads").await, 2);
    assert_eq!(count(&pool, "replays").await, 2);
    let disputed: i64 =
        sqlx::query_scalar("SELECT count(*) FROM matches WHERE disputed").fetch_one(&pool).await.unwrap();
    assert_eq!(disputed, 0);
}

#[sqlx::test]
async fn both_seats_of_one_match(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let (_, b) = user(&pool, "76561190000000002").await;
    let ra = upload(&app, &a, &[entry(1, 0, ACCOUNT_A, ACCOUNT_B, Some(ACCOUNT_B))], &[]).await;
    let rb = upload(&app, &b, &[entry(1, 1, ACCOUNT_B, ACCOUNT_A, Some(ACCOUNT_B))], &[]).await;
    assert_eq!(statuses(&ra.body, "entries"), ["new"]);
    assert_eq!(statuses(&rb.body, "entries"), ["new"]);
    assert_eq!(count(&pool, "matches").await, 1);
    assert_eq!(count(&pool, "uploads").await, 2);
    let (disputed, s0, s1): (bool, Option<i32>, Option<i32>) =
        sqlx::query_as("SELECT disputed, seat0_stars, seat1_stars FROM matches").fetch_one(&pool).await.unwrap();
    assert_eq!((disputed, s0, s1), (false, Some(20), Some(21)));
    assert_eq!(history(&app, &a).await[0]["result"], "loss");
    assert_eq!(history(&app, &b).await[0]["result"], "win");
}

async fn disputed_after(pool: PgPool, second: Value) -> bool {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let (_, b) = user(&pool, "76561190000000002").await;
    upload(&app, &a, &[entry(1, 0, ACCOUNT_A, ACCOUNT_B, Some(ACCOUNT_A))], &[]).await;
    let res = upload(&app, &b, &[second], &[]).await;
    assert_eq!(statuses(&res.body, "entries"), ["new"]);
    assert_eq!(count(&pool, "uploads").await, 2);
    let disputed = sqlx::query_scalar("SELECT disputed FROM matches").fetch_one(&pool).await.unwrap();
    assert_eq!(history(&app, &b).await[0]["disputed"], disputed);
    disputed
}

#[sqlx::test]
async fn disputed_on_winner(pool: PgPool) {
    assert!(disputed_after(pool, entry(1, 1, ACCOUNT_B, ACCOUNT_A, Some(ACCOUNT_B))).await);
}

#[sqlx::test]
async fn disputed_on_cards(pool: PgPool) {
    let mut e = entry(1, 1, ACCOUNT_B, ACCOUNT_A, Some(ACCOUNT_A));
    e["MyCards"][0] = json!("00000000-0000-4000-8000-0000000000c1");
    assert!(disputed_after(pool, e).await);
}

#[sqlx::test]
async fn disputed_on_same_seat(pool: PgPool) {
    assert!(disputed_after(pool, entry(1, 0, ACCOUNT_B, ACCOUNT_A, Some(ACCOUNT_B))).await);
}

#[sqlx::test]
async fn consistent_second_upload_is_not_disputed(pool: PgPool) {
    assert!(!disputed_after(pool, entry(1, 1, ACCOUNT_B, ACCOUNT_A, Some(ACCOUNT_A))).await);
}

#[sqlx::test]
async fn card_order_is_kept_and_not_disputed(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let (_, b) = user(&pool, "76561190000000002").await;
    upload(&app, &a, &[entry(1, 0, ACCOUNT_A, ACCOUNT_B, Some(ACCOUNT_A))], &[]).await;
    let mut other_view = entry(1, 1, ACCOUNT_B, ACCOUNT_A, Some(ACCOUNT_A));
    let reversed = |cards: [&str; 4]| json!(cards.iter().rev().collect::<Vec<_>>());
    other_view["MyCards"] = reversed(SEAT1_CARDS);
    other_view["OpponentCards"] = reversed(SEAT0_CARDS);
    let res = upload(&app, &b, &[other_view], &[]).await;
    assert_eq!(statuses(&res.body, "entries"), ["new"]);
    let (disputed, seat0, seat1): (bool, Vec<uuid::Uuid>, Vec<uuid::Uuid>) =
        sqlx::query_as("SELECT disputed, seat0_cards, seat1_cards FROM matches").fetch_one(&pool).await.unwrap();
    assert!(!disputed);
    let ids = |cards: [&str; 4]| cards.map(|c| c.parse::<uuid::Uuid>().unwrap()).to_vec();
    assert_eq!(seat0, ids(SEAT0_CARDS));
    assert_eq!(seat1, ids(SEAT1_CARDS));
    let h = history(&app, &a).await;
    assert_eq!(h[0]["myCards"], json!(SEAT0_CARDS));
    assert_eq!(h[0]["opponentCards"], json!(SEAT1_CARDS));
}

#[sqlx::test]
async fn other_account_id_for_linked_user(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    upload(&app, &cookie, &[entry(1, 0, ACCOUNT_A, OPP, None)], &[]).await;
    let res = upload(&app, &cookie, &[entry(2, 0, ACCOUNT_B, OPP, None)], &[]).await;
    assert_eq!(statuses(&res.body, "entries"), ["rejected"]);
    assert!(reason(&res.body, "entries", 0).contains("linked to your Steam account"));
    assert_eq!(count(&pool, "matches").await, 1);
}

#[sqlx::test]
async fn account_id_linked_to_another_user(pool: PgPool) {
    let app = app(pool.clone());
    let (_, a) = user(&pool, "76561190000000001").await;
    let (b_id, b) = user(&pool, "76561190000000002").await;
    upload(&app, &a, &[entry(1, 0, ACCOUNT_A, OPP, None)], &[]).await;
    let res = upload(&app, &b, &[entry(2, 0, ACCOUNT_A, OPP, None)], &[]).await;
    assert_eq!(statuses(&res.body, "entries"), ["rejected"]);
    assert!(reason(&res.body, "entries", 0).contains("another Steam account"));
    let linked: Option<uuid::Uuid> = sqlx::query_scalar("SELECT spam_account_id FROM users WHERE id = $1")
        .bind(b_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(linked, None);
}

#[sqlx::test]
async fn unknown_card_is_accepted(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let unknown = "00000000-0000-4000-8000-0000000000c1";
    let mut e = entry(1, 0, ACCOUNT_A, ACCOUNT_B, None);
    e["OpponentCards"][2] = json!(unknown);
    let res = upload(&app, &cookie, std::slice::from_ref(&e), &[]).await;
    assert_eq!(statuses(&res.body, "entries"), ["new"]);
    let seen = || async {
        sqlx::query_as::<_, (String, i64, bool)>(
            "SELECT card_id::text, seen_count, first_seen <= last_seen FROM unknown_cards",
        )
        .fetch_all(&pool)
        .await
        .unwrap()
    };
    assert_eq!(seen().await, [(unknown.to_string(), 1, true)]);

    let (_, b) = user(&pool, "76561190000000002").await;
    let mut b_view = entry(1, 1, ACCOUNT_B, ACCOUNT_A, None);
    b_view["MyCards"][2] = json!(unknown);
    assert_eq!(statuses(&upload(&app, &b, &[b_view], &[]).await.body, "entries"), ["new"]);
    upload(&app, &cookie, &[e], &[]).await;
    assert_eq!(seen().await, [(unknown.to_string(), 1, true)]);

    let mut mirror = entry(2, 0, ACCOUNT_A, ACCOUNT_B, None);
    mirror["MyCards"][0] = json!(unknown);
    mirror["OpponentCards"][2] = json!(unknown);
    assert_eq!(statuses(&upload(&app, &cookie, &[mirror], &[]).await.body, "entries"), ["new"]);
    assert_eq!(seen().await, [(unknown.to_string(), 2, true)]);
}

#[sqlx::test]
async fn format_version_3_is_rejected(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let mut e = entry(1, 0, ACCOUNT_A, OPP, None);
    e["FormatVersion"] = json!(3);
    let res = upload(&app, &cookie, &[e], &[]).await;
    assert_eq!(statuses(&res.body, "entries"), ["rejected"]);
    assert_eq!(reason(&res.body, "entries", 0), "unsupported FormatVersion 3");
    assert_eq!(res.body["entries"][0]["gameUid"], "match-1");
    assert_eq!(count(&pool, "matches").await, 0);
}

#[sqlx::test]
async fn winner_not_a_player_is_rejected(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let res = upload(&app, &cookie, &[entry(1, 0, ACCOUNT_A, OPP, Some(ACCOUNT_B))], &[]).await;
    assert_eq!(statuses(&res.body, "entries"), ["rejected"]);
    assert!(reason(&res.body, "entries", 0).contains("WinnerId"));
}

#[sqlx::test]
async fn entry_validation(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let base = entry(1, 0, ACCOUNT_A, OPP, None);
    let future = json!(chrono::Utc::now() + chrono::TimeDelta::minutes(10));
    let future_end = json!(chrono::Utc::now() + chrono::TimeDelta::minutes(20));
    let cases: Vec<(Vec<(&str, Value)>, &str)> = vec![
        (vec![("GameUid", json!("match-1x"))], "invalid GameUid"),
        (vec![("GameUid", json!("../match-1"))], "invalid GameUid"),
        (vec![("MyCards", json!(&SEAT0_CARDS[..3]))], "MyCards must contain 4 distinct card UUIDs"),
        (
            vec![("MyCards", json!([SEAT0_CARDS[0], SEAT0_CARDS[0], SEAT0_CARDS[1], SEAT0_CARDS[2]]))],
            "MyCards must contain 4 distinct card UUIDs",
        ),
        (
            vec![("OpponentCards", json!(["x", SEAT1_CARDS[1], SEAT1_CARDS[2], SEAT1_CARDS[3]]))],
            "OpponentCards must contain 4 distinct card UUIDs",
        ),
        (vec![("EndUtc", json!("2026-01-01T09:00:00Z"))], "EndUtc must be after StartUtc"),
        (
            vec![("StartUtc", future), ("EndUtc", future_end), ("DurationMs", json!(600_000))],
            "StartUtc is in the future",
        ),
        (vec![("DurationMs", json!(605_200))], "DurationMs does not match EndUtc - StartUtc"),
        (vec![("TurnCount", json!(0))], "TurnCount must be at least 1"),
        (vec![("Seat", json!(2))], "Seat must be 0 or 1"),
        (vec![("AccountId", json!("not-a-uuid"))], "invalid AccountId"),
        (vec![("AppVersion", json!("1.28.1-beta"))], "invalid AppVersion"),
        (vec![("IsDraw", json!(false))], "WinnerId is not a player of the match"),
        (vec![("TurnCount", json!("many"))], "invalid entry"),
    ];
    let entries: Vec<Value> = cases
        .iter()
        .map(|(patch, _)| {
            let mut e = base.clone();
            for (field, value) in patch {
                e[*field] = value.clone();
            }
            e
        })
        .collect();
    let res = upload(&app, &cookie, &entries, &[]).await;
    assert_eq!(statuses(&res.body, "entries"), vec!["rejected"; cases.len()]);
    for (i, (_, expected)) in cases.iter().enumerate() {
        assert!(reason(&res.body, "entries", i).starts_with(expected), "case {i}");
    }
    assert_eq!(count(&pool, "matches").await, 0);
}

#[sqlx::test]
async fn replay_with_wrong_game_uid(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let e = entry(1, 0, ACCOUNT_A, OPP, None);
    let other = entry(999, 0, ACCOUNT_A, OPP, None);
    let res = upload(&app, &cookie, &[e], &[("match-1.srpl".into(), srpl(&other))]).await;
    assert_eq!(statuses(&res.body, "entries"), ["new"]);
    assert_eq!(statuses(&res.body, "replays"), ["rejected"]);
    assert_eq!(reason(&res.body, "replays", 0), "GameUid is not in index.json");
    assert_eq!(count(&pool, "replays").await, 0);
}

#[sqlx::test]
async fn replay_of_rejected_entry_and_invalid_replays(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let mut bad = entry(1, 0, ACCOUNT_A, OPP, None);
    bad["TurnCount"] = json!(0);
    let mut wrong_magic = srpl_raw(&bad, 0);
    wrong_magic[0] = b'X';
    let mut wrong_version = srpl_raw(&bad, 0);
    wrong_version[4] = 3;
    let replays = vec![
        replay(&bad),
        ("a.srpl".into(), b"not gzip".to_vec()),
        ("b.srpl".into(), gzip(&wrong_magic)),
        ("c.srpl".into(), gzip(&wrong_version)),
    ];
    let res = upload(&app, &cookie, &[bad], &replays).await;
    assert_eq!(statuses(&res.body, "replays"), ["rejected"; 4]);
    assert_eq!(reason(&res.body, "replays", 0), "index entry is rejected");
    assert_eq!(reason(&res.body, "replays", 1), "replay is not valid gzip");
    assert_eq!(reason(&res.body, "replays", 2), "replay magic is not SRPL");
    assert_eq!(reason(&res.body, "replays", 3), "unsupported replay version");
}

#[sqlx::test]
async fn gzip_bomb_is_rejected(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let e = entry(1, 0, ACCOUNT_A, OPP, None);
    let bomb = gzip(&srpl_raw(&e, 6 << 20));
    assert!(bomb.len() < 512 << 10);
    let res = upload(&app, &cookie, std::slice::from_ref(&e), &[("match-1.srpl".into(), bomb)]).await;
    assert_eq!(statuses(&res.body, "entries"), ["new"]);
    assert_eq!(statuses(&res.body, "replays"), ["rejected"]);
    assert_eq!(reason(&res.body, "replays", 0), "replay is larger than 5 MB after gunzip");
    assert_eq!(count(&pool, "replays").await, 0);
}

fn noise(len: usize) -> Vec<u8> {
    let mut x = 0x9E37_79B9_7F4A_7C15u64;
    (0..len)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x as u8
        })
        .collect()
}

#[sqlx::test]
async fn size_limits(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;

    let e = entry(1, 0, ACCOUNT_A, OPP, None);
    let mut raw = srpl_raw(&e, 0);
    raw.extend(noise(600 << 10));
    let res = upload(&app, &cookie, std::slice::from_ref(&e), &[("match-1.srpl".into(), gzip(&raw))]).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(statuses(&res.body, "replays"), ["rejected"]);
    assert_eq!(reason(&res.body, "replays", 0), "replay is larger than 512 KB");

    let mut big = serde_json::to_vec(&[&e]).unwrap();
    big.pop();
    big.extend(format!(",\"{}\"]", "x".repeat(1 << 20)).bytes());
    let res = send(&app, upload_req(&cookie, &big, &[])).await;
    assert_eq!(res.status, StatusCode::PAYLOAD_TOO_LARGE);

    let many: Vec<Value> = (0..51).map(|n| entry(n, 0, ACCOUNT_A, OPP, None)).collect();
    let res = upload(&app, &cookie, &many, &[]).await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);

    let replays: Vec<_> = (0..51).map(|n| (format!("{n}.srpl"), vec![0u8])).collect();
    let res = upload(&app, &cookie, &[], &replays).await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);

    let huge = vec![0u8; 30 << 20];
    let res = send(&app, upload_req(&cookie, b"[]", &[("x.srpl".into(), huge)])).await;
    assert_eq!(res.status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[sqlx::test]
async fn banned_user_cannot_upload(pool: PgPool) {
    let app = app(pool.clone());
    let (id, cookie) = user(&pool, "76561190000000001").await;
    sqlx::query("UPDATE users SET banned_at = now() WHERE id = $1").bind(id).execute(&pool).await.unwrap();
    let res = upload(&app, &cookie, &[entry(1, 0, ACCOUNT_A, OPP, None)], &[]).await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
    assert_eq!(count(&pool, "matches").await, 0);
    assert_eq!(send(&app, get("/api/me/matches", &cookie)).await.status, StatusCode::OK);
}

#[sqlx::test]
async fn upload_requires_login(pool: PgPool) {
    let app = app(pool.clone());
    let res = upload(&app, "session=nope", &[entry(1, 0, ACCOUNT_A, OPP, None)], &[]).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn rate_limit(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    for _ in 0..30 {
        assert_eq!(upload(&app, &cookie, &[], &[]).await.status, StatusCode::OK);
    }
    assert_eq!(upload(&app, &cookie, &[], &[]).await.status, StatusCode::TOO_MANY_REQUESTS);
    let (_, other) = user(&pool, "76561190000000002").await;
    assert_eq!(upload(&app, &other, &[], &[]).await.status, StatusCode::OK);
}

#[sqlx::test]
async fn history_paging(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let entries: Vec<Value> = (1..=3)
        .map(|n| {
            let mut e = entry(n, 0, ACCOUNT_A, OPP, None);
            e["StartUtc"] = json!(format!("2026-01-0{n}T10:00:00Z"));
            e["EndUtc"] = json!(format!("2026-01-0{n}T10:10:00Z"));
            e["DurationMs"] = json!(600_000);
            e
        })
        .collect();
    upload(&app, &cookie, &entries, &[]).await;
    let page = send(&app, get("/api/me/matches?limit=2", &cookie)).await.body;
    let uids: Vec<_> = page.as_array().unwrap().iter().map(|m| m["gameUid"].clone()).collect();
    assert_eq!(uids, ["match-3", "match-2"]);
    let before = page[1]["startUtc"].as_str().unwrap();
    let rest = send(&app, get(&format!("/api/me/matches?before={before}"), &cookie)).await.body;
    assert_eq!(rest.as_array().unwrap().len(), 1);
    assert_eq!(rest[0]["gameUid"], "match-1");
}

#[sqlx::test]
async fn cleanup_deletes_expired_replays(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let entries = [entry(1, 0, ACCOUNT_A, OPP, None), entry(2, 0, ACCOUNT_A, OPP, None)];
    let replays: Vec<_> = entries.iter().map(replay).collect();
    upload(&app, &cookie, &entries, &replays).await;
    let expires_ok: bool = sqlx::query_scalar(
        "SELECT bool_and(expires_at BETWEEN now() + interval '364 days' AND now() + interval '367 days') FROM replays",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(expires_ok);
    sqlx::query("UPDATE replays SET expires_at = now() - interval '1 second' WHERE game_uid = 'match-1'")
        .execute(&pool)
        .await
        .unwrap();
    api::cleanup(&pool).await.unwrap();
    let left: Vec<String> = sqlx::query_scalar("SELECT game_uid FROM replays").fetch_all(&pool).await.unwrap();
    assert_eq!(left, ["match-2"]);
    assert_eq!(count(&pool, "matches").await, 2);
}

const ALLY: &str = "FakeAllyé";
const SECRETS: [&str; 4] = ["FakeMe", "FakeOpponent", ALLY, "fake-ticket"];
const PERSONAL: [&str; 5] = [
    "/OpponentName",
    "/Loading/ticket",
    "/Loading/me/username",
    "/Loading/opponent/username",
    "/Loading/ally/username",
];

fn leb128(mut n: usize) -> Vec<u8> {
    let mut out = Vec::new();
    while n >= 0x80 {
        out.push((n & 0x7f) as u8 | 0x80);
        n >>= 7;
    }
    out.push(n as u8);
    out
}

fn utf16(s: &str) -> Vec<u8> {
    s.encode_utf16().flat_map(u16::to_le_bytes).collect()
}

fn contains(data: &[u8], s: &str) -> bool {
    [s.as_bytes().to_vec(), utf16(s)].iter().any(|n| data.windows(n.len()).any(|w| w == n))
}

fn gunzip(gz: &[u8]) -> Vec<u8> {
    use std::io::Read;
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(gz).read_to_end(&mut out).unwrap();
    out
}

fn take<'a>(rest: &mut &'a [u8], n: usize) -> &'a [u8] {
    let (head, tail) = rest.split_at(n);
    *rest = tail;
    head
}

fn u32_le(rest: &mut &[u8]) -> u32 {
    u32::from_le_bytes(take(rest, 4).try_into().unwrap())
}

fn parse_srpl(raw: &[u8]) -> (Value, Vec<(u32, Vec<u8>)>) {
    let mut rest = raw.strip_prefix(b"SRPL").expect("magic");
    assert!(u32_le(&mut rest) == 2, "version");
    let (mut len, mut shift) = (0, 0);
    loop {
        let b = take(&mut rest, 1)[0];
        len |= usize::from(b & 0x7f) << shift;
        shift += 7;
        if b & 0x80 == 0 {
            break;
        }
    }
    let header = serde_json::from_slice(take(&mut rest, len)).expect("header");
    let count = u32_le(&mut rest);
    let messages = (0..count)
        .map(|_| {
            let time = u32_le(&mut rest);
            let len = u32_le(&mut rest) as usize;
            (time, take(&mut rest, len).to_vec())
        })
        .collect();
    assert!(rest.is_empty(), "bytes after the last message");
    (header, messages)
}

fn personal_entry(n: u64) -> Value {
    let mut e = entry(n, 0, ACCOUNT_A, OPP, Some(ACCOUNT_A));
    e["Loading"]["ally"] = json!({ "username": ALLY, "stars": 3 });
    e
}

fn messages(clean: bool) -> Vec<(u32, Vec<u8>)> {
    let name = |s: &str, units: usize| if clean { "_".repeat(units) } else { s.to_string() };
    let u8n = |s: &str| {
        let s = name(s, s.len());
        [leb128(2 * (s.len() + 1)), s.into_bytes()].concat()
    };
    let u16n = |s: &str| {
        let s = name(s, s.encode_utf16().count());
        [(s.encode_utf16().count() as u32).to_le_bytes().to_vec(), utf16(&s)].concat()
    };
    let msg = |kind: u16, parts: Vec<Vec<u8>>| [kind.to_le_bytes().to_vec(), parts.concat()].concat();
    [
        msg(2000, vec![vec![7, 0, 0, 0], u8n("FakeMe"), u8n("FakeOpponent")]),
        msg(2001, vec![vec![1], u16n("FakeMe"), vec![2], u16n("FakeOpponent"), u16n(ALLY)]),
        msg(2004, vec![u8n("FakeOpponent"), vec![0xff; 3]]),
        msg(2010, vec![u8n(ALLY), u8n("FakeMe")]),
        msg(2015, vec![u8n("FakeMe")]),
        msg(2204, vec![vec![5], u8n("FakeOpponent"), vec![6]]),
        msg(3000, vec![b"\x01FakeOpp\x0cFakeM\x02FakeMe\x03".to_vec()]),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, p)| (i as u32 * 250, p))
    .collect()
}

fn personal_srpl(header: &Value) -> Vec<u8> {
    let mut raw = srpl_raw(header, 0);
    let messages = messages(false);
    raw.extend((messages.len() as u32).to_le_bytes());
    for (time, payload) in messages {
        raw.extend(time.to_le_bytes());
        raw.extend((payload.len() as u32).to_le_bytes());
        raw.extend(payload);
    }
    gzip(&raw)
}

fn assert_clean(gz: &[u8], original: &Value) {
    let raw = gunzip(gz);
    let (header, stored) = parse_srpl(&raw);
    let before_last = &raw[..raw.len() - stored.last().unwrap().1.len()];
    for secret in SECRETS {
        assert!(!contains(before_last, secret), "{secret} is in the replay");
    }
    let mut expected = original.clone();
    for path in PERSONAL {
        let (parent, key) = path.rsplit_once('/').unwrap();
        expected.pointer_mut(parent).unwrap().as_object_mut().unwrap().remove(key).unwrap();
        assert!(header.pointer(path).is_none(), "{path}");
    }
    assert_eq!(header, expected);
    let dirty = messages(false);
    assert_eq!(stored.len(), dirty.len());
    for ((_, a), (_, b)) in stored.iter().zip(&dirty) {
        assert_eq!(a.len(), b.len());
    }
    assert_eq!(stored, messages(true));
}

#[sqlx::test]
async fn upload_removes_personal_data_from_replay(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let e = personal_entry(1);
    let res = upload(&app, &cookie, std::slice::from_ref(&e), &[("match-1.srpl".into(), personal_srpl(&e))]).await;
    assert_eq!(statuses(&res.body, "replays"), ["new"]);
    let (data, clean): (Vec<u8>, bool) =
        sqlx::query_as("SELECT data, clean FROM replays").fetch_one(&pool).await.unwrap();
    assert!(clean);
    assert_clean(&data, &e);
}

#[sqlx::test]
async fn replay_without_loading(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let mut absent = entry(1, 0, ACCOUNT_A, OPP, Some(ACCOUNT_A));
    absent.as_object_mut().unwrap().remove("Loading");
    let mut null = entry(2, 0, ACCOUNT_A, OPP, Some(ACCOUNT_A));
    null["Loading"] = Value::Null;
    let entries = [absent, null];
    let replays: Vec<_> = entries.iter().map(|e| (replay(e).0, personal_srpl(e))).collect();
    let res = upload(&app, &cookie, &entries, &replays).await;
    assert_eq!(statuses(&res.body, "entries"), ["new", "new"]);
    assert_eq!(statuses(&res.body, "replays"), ["new", "new"]);
    let stored: Vec<Vec<u8>> =
        sqlx::query_scalar("SELECT data FROM replays ORDER BY game_uid").fetch_all(&pool).await.unwrap();
    assert_eq!(stored.len(), 2);
    for (data, e) in stored.iter().zip(&entries) {
        let raw = gunzip(data);
        let (header, _) = parse_srpl(&raw);
        assert!(header.get("OpponentName").is_none());
        assert_eq!(header.get("Loading"), e.get("Loading"));
        assert!(!contains(&raw, "FakeOpponent"));
    }
}

#[sqlx::test]
async fn stored_replays_are_cleaned_once(pool: PgPool) {
    let app = app(pool.clone());
    let (user_id, cookie) = user(&pool, "76561190000000001").await;
    let entries = [personal_entry(1), personal_entry(2)];
    upload(&app, &cookie, &entries, &[]).await;
    for (uid, data) in [("match-1", personal_srpl(&entries[0])), ("match-2", b"not gzip".to_vec())] {
        sqlx::query(
            "INSERT INTO replays (game_uid, user_id, data, clean, expires_at)
             VALUES ($1, $2, $3, false, now() + interval '1 day')",
        )
        .bind(uid)
        .bind(user_id)
        .bind(data)
        .execute(&pool)
        .await
        .unwrap();
    }
    assert_eq!(api::matches::clean_stored_replays(&pool).await.unwrap(), (1, 1));
    let rows: Vec<(String, Vec<u8>, bool)> =
        sqlx::query_as("SELECT game_uid, data, clean FROM replays").fetch_all(&pool).await.unwrap();
    assert_eq!(rows.len(), 1);
    let (uid, data, clean) = &rows[0];
    assert_eq!((uid.as_str(), *clean), ("match-1", true));
    assert_clean(data, &entries[0]);
    assert_eq!(count(&pool, "uploads").await, 2);
    assert_eq!(api::matches::clean_stored_replays(&pool).await.unwrap(), (0, 0));
}

#[sqlx::test]
async fn min_version(pool: PgPool) {
    let res = send(&app(pool), get("/api/desktop/min-version", "")).await;
    assert_eq!(res.body["minVersion"], "1.2.3");
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
}

#[sqlx::test]
#[ignore]
async fn real_sample_folder(pool: PgPool) {
    let root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../samples/save/replays/api.crocodyle.fr"));
    let Some(dir) = std::fs::read_dir(root).ok().and_then(|mut d| d.next()).map(|d| d.unwrap().path()) else {
        eprintln!("sample folder is absent, skip");
        return;
    };
    let index = std::fs::read(dir.join("index.json")).unwrap();
    let mut replays = Vec::new();
    for f in std::fs::read_dir(&dir).unwrap() {
        let path = f.unwrap().path();
        if path.extension().is_some_and(|e| e == "srpl") {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            replays.push((name, std::fs::read(&path).unwrap()));
        }
    }
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let first = send(&app, upload_req(&cookie, &index, &replays)).await;
    assert_eq!(first.status, StatusCode::OK);
    assert_eq!(statuses(&first.body, "entries"), ["new"; 10]);
    assert_eq!(statuses(&first.body, "replays"), ["new"; 10]);
    let second = send(&app, upload_req(&cookie, &index, &replays)).await;
    assert_eq!(statuses(&second.body, "entries"), ["known"; 10]);
    assert_eq!(statuses(&second.body, "replays"), ["known"; 10]);
    let stored: Vec<(String, Vec<u8>, bool)> =
        sqlx::query_as("SELECT game_uid, data, clean FROM replays").fetch_all(&pool).await.unwrap();
    assert_eq!(stored.len(), 10);
    for (game_uid, data, clean) in stored {
        assert!(clean, "{game_uid}");
        let raw = gunzip(&data);
        let (header, messages) = parse_srpl(&raw);
        assert!(header["MessageCount"].as_u64() == Some(messages.len() as u64), "count in {game_uid}");
        let (original, original_messages) =
            parse_srpl(&gunzip(&std::fs::read(dir.join(format!("{game_uid}.srpl"))).unwrap()));
        let same_framing = original_messages.len() == messages.len()
            && original_messages.iter().zip(&messages).all(|((t1, p1), (t2, p2))| t1 == t2 && p1.len() == p2.len());
        assert!(same_framing, "framing in {game_uid}");
        for path in PERSONAL {
            assert!(header.pointer(path).is_none(), "{path} in {game_uid}");
            if let Some(secret) = original.pointer(path).and_then(Value::as_str).filter(|s| !s.is_empty()) {
                let found = contains(&raw, secret);
                assert!(!found, "name in {game_uid}");
            }
        }
    }
    let h = history(&app, &cookie).await;
    assert_eq!(h.len(), 10);
    let ranked: Vec<_> = h.iter().filter(|m| m["gameMode"] == 10 && m["isAi"] == false).collect();
    assert_eq!(ranked.len(), 7);
    let n = |r: &str| ranked.iter().filter(|m| m["result"] == r).count();
    assert_eq!((n("win"), n("loss"), n("draw")), (4, 3, 0));

    let stats = send(&app, get("/api/stats/decks?version=1.28.1", "")).await.body;
    let keys = stats.as_array().unwrap();
    assert_eq!(keys.len(), 10);
    assert_eq!(keys.iter().map(|k| k["games"].as_i64().unwrap()).sum::<i64>(), 14);
    let card = json!("0c8c383b-2207-4d18-b378-4f2a3795a58e");
    let with_card: Vec<_> = keys.iter().filter(|k| k["cards"].as_array().unwrap().contains(&card)).collect();
    let games: Vec<_> = with_card.iter().map(|k| k["games"].as_i64().unwrap()).collect();
    assert_eq!(games, [5, 1]);
    let mut main = SEAT0_CARDS;
    main.sort();
    assert_eq!(with_card[0]["cards"], json!(main));

    let save = root.parent().unwrap().parent().unwrap();
    let mut lines = Vec::new();
    for log in ["Player-prev.log", "Player.log"] {
        let text = String::from_utf8_lossy(&std::fs::read(save.join(log)).unwrap()).into_owned();
        lines.extend(text.lines().filter(|l| l.contains("Sending decks to server: ")).map(str::to_owned));
    }
    let res = send(&app, text_req("/api/decks/import", &cookie, lines.join("\n"))).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.body["error"]);
    let decks = res.body["decks"].as_array().unwrap();
    assert_eq!(decks.len(), 2);
    for (prefix, talents) in [("6519f5b9", json!([0, 1, 0, 2])), ("5274500e", json!([0, 0, 0, 0]))] {
        let d = decks.iter().find(|d| d["gameDeckId"].as_str().unwrap().starts_with(prefix)).unwrap();
        assert_eq!(d["status"], "new");
        let deck = send(&app, get(&format!("/api/decks/{}", d["deckId"]), &cookie)).await.body;
        assert_eq!(deck["talents"], talents, "{prefix}");
        assert_eq!(deck["status"], "draft");
    }
}
