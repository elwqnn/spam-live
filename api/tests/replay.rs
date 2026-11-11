mod common;

use std::{collections::HashMap, path::Path};

use api::replay::{DECODER_VERSION, decode};
use axum::http::StatusCode;
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

fn decks() -> [Vec<Uuid>; 2] {
    [SEAT0_CARDS, SEAT1_CARDS].map(|d| d.iter().map(|c| c.parse().unwrap()).collect())
}

fn header(seat: usize) -> Value {
    entry(1, seat, ACCOUNT_A, ACCOUNT_B, Some(ACCOUNT_A))
}

fn run(raw: &[u8]) -> Result<api::replay::Decoded, String> {
    let [a, b] = decks();
    decode(&gzip(raw), &a, &b)
}

fn uuid(s: &str) -> Uuid {
    s.parse().unwrap()
}

#[test]
fn decodes_a_synthetic_match() {
    let d = run(&srpl_messages(&header(1), &match_messages(1, true))).unwrap();
    assert_eq!(d.uploader_seat, 1);
    let buys: Vec<_> = d.shop_buys.iter().map(|b| (b.seat, b.idx, b.turn, b.card, b.tox)).collect();
    assert_eq!(
        buys,
        [
            (0, 0, 1, uuid(SEAT0_CARDS[0]), uuid(TOX[0])),
            (1, 0, 1, uuid(SEAT1_CARDS[1]), uuid(TOX[1])),
            (0, 1, 2, uuid(SEAT0_CARDS[0]), uuid(TOX[1])),
        ]
    );
    let talents: Vec<_> = d.talents.iter().map(|t| (t.seat, t.card, t.talent)).collect();
    assert_eq!(talents, [(1, uuid(SEAT1_CARDS[1]), 1), (1, uuid(SEAT1_CARDS[2]), 2)]);
    let damage: Vec<_> = d.damage.iter().map(|c| (c.seat, c.card, c.done, c.taken)).collect();
    let expected: Vec<_> = (0..2)
        .flat_map(|s| {
            [SEAT0_CARDS, SEAT1_CARDS][s].iter().map(move |c| {
                let (done, taken) = if s == 0 { (500, 300) } else { (150, 1000) };
                (s as i16, uuid(c), done, taken)
            })
        })
        .collect();
    assert_eq!(damage, expected);
    let hp: Vec<_> = d.hp.iter().map(|h| (h.seat, h.turn, h.hp)).collect();
    assert_eq!(hp, [(0, 1, 3200), (1, 1, 3600), (0, 2, 2800), (1, 2, 0)]);
}

#[test]
fn match_without_buys_or_talents() {
    let d = run(&srpl_messages(&header(1), &match_messages(0, false))).unwrap();
    assert!(d.shop_buys.is_empty() && d.talents.is_empty());
    assert_eq!(d.damage.len(), 8);
    assert_eq!(d.hp.len(), 4);
    let d = run(&srpl_messages(&header(0), &[states(&[])])).unwrap();
    assert!(d.shop_buys.is_empty() && d.talents.is_empty() && d.damage.is_empty() && d.hp.is_empty());
}

#[test]
fn errors() {
    let raw = srpl_messages(&header(0), &match_messages(0, true));
    let err = |raw: &[u8]| run(raw).err().unwrap_or_default();
    assert_eq!(err(&raw[..raw.len() - 3]), "replay is truncated");
    assert_eq!(err(&[raw.as_slice(), &[0]].concat()), "data after the last message");
    let mut wrong = header(0);
    wrong["MessageCount"] = json!(3);
    let messages = match_messages(0, true);
    let mut raw2 = srpl_raw(&wrong, 0);
    raw2.extend((messages.len() as u32).to_le_bytes());
    for p in &messages {
        raw2.extend([0; 4]);
        raw2.extend((p.len() as u32).to_le_bytes());
        raw2.extend(p);
    }
    assert_eq!(err(&raw2), "message count does not match the header");
    let mut v3 = raw.clone();
    v3[4] = 3;
    assert_eq!(err(&v3), "unsupported replay version");
    assert_eq!(err(b"XXXX"), "replay magic is not SRPL");
    let [a, b] = decks();
    assert_eq!(decode(b"not gzip", &a, &b).err().unwrap(), "replay is not valid gzip");
    let mut bad_buy = match_messages(0, true);
    bad_buy[3].pop();
    assert_eq!(err(&srpl_messages(&header(0), &bad_buy)), "invalid shop buy message");
    let mut bad_seat = header(0);
    bad_seat["Seat"] = json!(2);
    assert_eq!(err(&srpl_messages(&bad_seat, &[])), "replay Seat must be 0 or 1");
}

#[test]
fn no_panic_on_damaged_data() {
    let raw = srpl_messages(&header(0), &match_messages(0, true));
    for n in 0..raw.len() {
        assert!(run(&raw[..n]).is_err(), "prefix {n}");
        for flip in [0x80, 0xff] {
            let mut bad = raw.clone();
            bad[n] ^= flip;
            let _ = run(&bad);
        }
    }
}

async fn event_rows(pool: &PgPool) -> i64 {
    sqlx::query_scalar(
        "SELECT (SELECT count(*) FROM shop_buys) + (SELECT count(*) FROM talent_picks)
             + (SELECT count(*) FROM card_damage) + (SELECT count(*) FROM hp_turns)
             + (SELECT count(*) FROM match_event_sources)",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test]
async fn decode_error_keeps_the_upload(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let e = header(0);
    let res = upload(&app, &cookie, std::slice::from_ref(&e), &[("match-1.srpl".into(), srpl(&e))]).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(statuses(&res.body, "replays"), ["new"]);
    let (version, error): (Option<i32>, Option<String>) =
        sqlx::query_as("SELECT decoder_version, decode_error FROM replays").fetch_one(&pool).await.unwrap();
    assert_eq!((version, error.as_deref()), (Some(DECODER_VERSION), Some("message count does not match the header")));
    assert_eq!(event_rows(&pool).await, 0);
    assert_eq!(history(&app, &cookie).await[0]["hp"], json!([]));
    assert_eq!(api::replay::decode_stored_replays(&pool).await.unwrap(), 0);
}

#[sqlx::test]
async fn backfill_decodes_old_and_new_blobs(pool: PgPool) {
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let e = header(0);
    upload(&app, &cookie, std::slice::from_ref(&e), &[("match-1.srpl".into(), match_srpl(&e))]).await;
    let full = event_rows(&pool).await;
    assert_eq!(full, 3 + 2 + 8 + 4 + 2);
    let hp = &history(&app, &cookie).await[0]["hp"];
    assert_eq!(*hp, json!([{ "turn": 1, "mine": 3200, "opponent": 3600 }, { "turn": 2, "mine": 2800, "opponent": 0 }]));
    assert_eq!(api::replay::decode_stored_replays(&pool).await.unwrap(), 0);
    sqlx::query("DELETE FROM shop_buys").execute(&pool).await.unwrap();
    sqlx::query("UPDATE match_event_sources SET decoder_version = 0").execute(&pool).await.unwrap();
    assert_eq!(api::replay::decode_stored_replays(&pool).await.unwrap(), 0);
    sqlx::query("UPDATE replays SET decoder_version = 0").execute(&pool).await.unwrap();
    assert_eq!(api::replay::decode_stored_replays(&pool).await.unwrap(), 1);
    assert_eq!(event_rows(&pool).await, full);
    sqlx::query("UPDATE replays SET decoder_version = NULL").execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM match_event_sources").execute(&pool).await.unwrap();
    assert_eq!(api::replay::decode_stored_replays(&pool).await.unwrap(), 1);
    assert_eq!(event_rows(&pool).await, full);
    let versions: Vec<i32> = sqlx::query_scalar(
        "SELECT decoder_version FROM replays UNION ALL SELECT decoder_version FROM match_event_sources
         UNION ALL SELECT decoder_version FROM shop_buys UNION ALL SELECT decoder_version FROM hp_turns",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(versions.iter().all(|&v| v == DECODER_VERSION));
}

async fn sources(pool: &PgPool) -> Vec<(i16, Option<i64>, bool)> {
    sqlx::query_as("SELECT seat, user_id, own FROM match_event_sources ORDER BY seat").fetch_all(pool).await.unwrap()
}

async fn talents(pool: &PgPool, seat: i16) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM talent_picks WHERE seat = $1").bind(seat).fetch_one(pool).await.unwrap()
}

#[sqlx::test]
async fn stale_write_keeps_the_own_source(pool: PgPool) {
    let app = app(pool.clone());
    let (ua, a) = user(&pool, "76561190000000001").await;
    let (ub, b) = user(&pool, "76561190000000002").await;
    let seat0 = header(0);
    let seat1 = entry(1, 1, ACCOUNT_B, ACCOUNT_A, Some(ACCOUNT_A));
    upload(&app, &a, std::slice::from_ref(&seat0), &[]).await;
    upload(&app, &b, std::slice::from_ref(&seat1), &[]).await;
    let da = run(&srpl_messages(&seat0, &match_messages(0, true))).unwrap();
    let db = run(&srpl_messages(&seat1, &match_messages(1, true))).unwrap();
    let write = |seat, user, own, d| api::replay::write(&pool, "match-1", seat, user, own, d);
    assert!(write(0, ua, true, &da).await.unwrap());
    assert!(!write(0, ub, false, &db).await.unwrap());
    assert!(!write(0, ua, true, &da).await.unwrap());
    assert!(write(1, ua, false, &da).await.unwrap());
    assert!(!write(1, ua, false, &da).await.unwrap());
    assert!(write(1, ub, true, &db).await.unwrap());
    assert!(!write(1, ua, false, &da).await.unwrap());
    assert_eq!(sources(&pool).await, [(0, Some(ua), true), (1, Some(ub), true)]);
    assert_eq!((talents(&pool, 0).await, talents(&pool, 1).await), (2, 2));
    assert_eq!(event_rows(&pool).await, 3 + 4 + 8 + 4 + 2);
}

#[sqlx::test]
async fn backfill_skips_a_replay_that_adds_nothing(pool: PgPool) {
    let app = app(pool.clone());
    let (ua, a) = user(&pool, "76561190000000001").await;
    let (_, c) = user(&pool, "76561190000000003").await;
    let first = header(0);
    let account_c = "00000000-0000-4000-8000-00000000000c";
    let second = entry(1, 0, account_c, ACCOUNT_B, Some(account_c));
    upload(&app, &a, std::slice::from_ref(&first), &[("match-1.srpl".into(), match_srpl(&first))]).await;
    let res = upload(&app, &c, std::slice::from_ref(&second), &[("match-1.srpl".into(), match_srpl(&second))]).await;
    assert_eq!(statuses(&res.body, "replays"), ["new"]);
    let untried: i64 = sqlx::query_scalar("SELECT count(*) FROM replays WHERE decoder_version IS NULL")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(untried, 1);
    assert_eq!(sources(&pool).await, [(0, Some(ua), true), (1, Some(ua), false)]);
    assert_eq!(api::replay::decode_stored_replays(&pool).await.unwrap(), 0);
}

fn log_buys(save: &Path) -> HashMap<String, Vec<(Uuid, String)>> {
    let value = |line: &str, key: &str| {
        let start = line.find(key)? + key.len();
        Some(line[start..].split([' ', ',']).next().unwrap_or_default().to_string())
    };
    let mut out: HashMap<String, Vec<(Uuid, String)>> = HashMap::new();
    let mut current = String::new();
    for log in ["Player-prev.log", "Player.log"] {
        let text = String::from_utf8_lossy(&std::fs::read(save.join(log)).unwrap()).into_owned();
        for line in text.lines() {
            if let Some(start) = line.find("ConnectToGame uid=match-") {
                let digits: String = line[start + 24..].chars().take_while(char::is_ascii_digit).collect();
                current = format!("match-{digits}");
            } else if line.contains("OnInGameShopBuyItemResponse") {
                let card = value(line, "card=").and_then(|c| c.parse().ok()).expect("card id in a shop line");
                let equipped = value(line, "modificatorUID=").expect("modificatorUID in a shop line");
                out.entry(current.clone()).or_default().push((card, equipped));
            }
        }
    }
    out
}

#[sqlx::test]
#[ignore]
async fn real_samples(pool: PgPool) {
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
            replays.push((path.file_name().unwrap().to_string_lossy().into_owned(), std::fs::read(&path).unwrap()));
        }
    }
    let app = app(pool.clone());
    let (_, cookie) = user(&pool, "76561190000000001").await;
    let res = send(&app, upload_req(&cookie, &index, &replays)).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(statuses(&res.body, "replays"), ["new"; 10]);

    let (decoded, failed): (i64, i64) = sqlx::query_as(
        "SELECT count(*) FILTER (WHERE decoder_version = $1 AND decode_error IS NULL),
             count(*) FILTER (WHERE decode_error IS NOT NULL) FROM replays",
    )
    .bind(DECODER_VERSION)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((decoded, failed), (10, 0));
    let counts: (i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM shop_buys), (SELECT count(*) FROM talent_picks),
             (SELECT count(*) FROM card_damage), (SELECT count(*) FROM hp_turns),
             (SELECT count(*) FROM match_event_sources)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    eprintln!("shop_buys, talent_picks, card_damage, hp_turns, match_event_sources: {counts:?}");
    assert_eq!(counts, (85, 16, 80, 252, 20));
    let sums: (i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT sum(hp) FROM hp_turns), (SELECT sum(done) FROM card_damage),
             (SELECT sum(taken) FROM card_damage), (SELECT sum(talent) FROM talent_picks),
             (SELECT sum(turn) FROM shop_buys)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(sums, (535_358, 47_285, 47_285, 19, 686));
    let ends: Vec<(String, i32, i32)> = sqlx::query_as(
        "SELECT m.game_uid, m.turn_count, max(h.turn) FROM matches m JOIN hp_turns h USING (game_uid)
         GROUP BY m.game_uid, m.turn_count",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(ends.len(), 10);
    for (uid, turns, last) in &ends {
        assert_eq!(turns, last, "last HP turn of {uid}");
    }

    let save = root.parent().unwrap().parent().unwrap();
    let log = log_buys(save);
    type Stored = (String, Vec<u8>, Vec<Uuid>, Vec<Uuid>);
    let stored: Vec<Stored> = sqlx::query_as(
        "SELECT r.game_uid, r.data, m.seat0_cards, m.seat1_cards FROM replays r JOIN matches m USING (game_uid)",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let (mut lines, mut equal) = (0, 0);
    for (uid, data, seat0, seat1) in stored {
        let d = decode(&data, &seat0, &seat1).unwrap();
        let buys: Vec<(Uuid, String)> = d.shop_buys.iter().map(|b| (b.card, b.equipped.clone())).collect();
        let expected = log.get(&uid).cloned().unwrap_or_default();
        lines += expected.len();
        equal += buys.iter().zip(&expected).filter(|(a, b)| a == b).count();
        assert!(buys == expected, "{uid}: {} decoded buys, {} log buys", buys.len(), expected.len());
    }
    eprintln!("log buys: {equal} of {lines} equal");
    assert_eq!((lines, equal), (85, 85));
}
