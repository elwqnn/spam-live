mod common;

use std::path::PathBuf;

use api::catalog::{Catalog, artist_link, parse_version};
use axum::http::StatusCode;
use common::*;
use serde_json::{Value, json};
use sqlx::PgPool;

const CARD: &str = "00000000-0000-4000-8000-0000000000c1";

fn mini(version: &str, card_name: Value) -> Value {
    json!({
        "appVersion": version,
        "languages": ["en", "fr"],
        "cards": [{
            "id": CARD, "name": card_name, "description": null, "element": "fire", "rarity": "rarity-common",
            "hp": 500, "goldIncome": 5, "fireResistance": 1, "waterResistance": 2, "earthResistance": 3,
            "natureResistance": 4, "lightResistance": 5, "shadowResistance": 6, "order": 1,
            "activeAbilities": [{ "id": "hit", "cooldown": 2 }, { "id": "kick", "cooldown": 1 }], "passiveAbilities": ["tox1", "tox2"],
            "talents": ["t1", "t2", "t3"], "isReleased": 1
        }],
        "abilities": [
            { "id": "hit", "name": { "en": "Hit", "fr": "Coup" }, "description": null, "cooldown": 9, "manaCost": 3, "team": "water" },
            { "id": "kick", "name": { "en": "Kick", "fr": null }, "description": null }
        ],
        "talents": [
            { "id": "t1", "name": { "en": "One", "fr": null }, "description": { "en": "First" } },
            { "id": "t2", "name": { "en": "Two" }, "description": null },
            { "id": "t3", "name": null, "description": null }
        ],
        "tox": [
            {
                "id": "tox1", "title": "Burn_R", "rarity": "rarity-rare", "modificatorImage": "tox_burn",
                "name": { "en": "Burn", "fr": "Brûlure" }, "description": null
            },
            { "id": "tox2", "title": "Ghost_C", "modificatorImage": null, "name": null, "description": null }
        ]
    })
}

fn temp_dir(name: &str, files: &[(&str, String)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("spam-catalog-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (file, content) in files {
        std::fs::write(dir.join(file), content).unwrap();
    }
    dir
}

fn lazy_pool() -> PgPool {
    PgPool::connect_lazy("postgres://127.0.0.1:1/none").unwrap()
}

#[test]
fn versions_compare_numerically() {
    assert!(parse_version("1.28.1") > parse_version("1.9.0"));
    assert!(parse_version("2.0") > parse_version("1.99.99"));
    for bad in ["", "1..2", "1.2a", "+1.2", "1.2.", " 1.2"] {
        assert_eq!(parse_version(bad), None, "{bad:?}");
    }
}

#[test]
fn load_errors() {
    let empty = temp_dir("empty", &[]);
    assert!(Catalog::load(&empty).err().unwrap().contains("no catalog file"));
    let invalid = temp_dir("invalid", &[("1.0.0.json", "{".into())]);
    assert!(Catalog::load(&invalid).err().unwrap().contains("invalid catalog file"));
    let mut dangling = mini("1.0.0", json!({ "en": "A" }));
    dangling["cards"][0]["talents"][0] = json!("missing");
    let dangling = temp_dir("dangling", &[("1.0.0.json", dangling.to_string())]);
    assert!(Catalog::load(&dangling).err().unwrap().contains("unknown id"));
    let mut unknown_artist = mini("1.0.0", json!({ "en": "A" }));
    unknown_artist["cards"][0]["artist"] = json!("00000000-0000-4000-8000-0000000000a1");
    let unknown_artist = temp_dir("unknown-artist", &[("1.0.0.json", unknown_artist.to_string())]);
    assert!(Catalog::load(&unknown_artist).err().unwrap().contains("unknown artist"));
    let mut duplicate_artist = mini("1.0.0", json!({ "en": "A" }));
    let artist = json!({ "id": "00000000-0000-4000-8000-0000000000a1", "name": "Ann", "link": "" });
    duplicate_artist["artists"] = json!([artist, artist]);
    let duplicate_artist = temp_dir("duplicate-artist", &[("1.0.0.json", duplicate_artist.to_string())]);
    assert!(Catalog::load(&duplicate_artist).err().unwrap().contains("duplicate artist id"));
    let duplicate = mini("1.0.0", json!({ "en": "A" })).to_string();
    let duplicate = temp_dir("duplicate", &[("a.json", duplicate.clone()), ("b.json", duplicate)]);
    assert!(Catalog::load(&duplicate).err().unwrap().contains("duplicate catalog version"));
    assert!(Catalog::load(&PathBuf::from("/nonexistent/catalog")).is_err());
    for dir in [empty, invalid, dangling, unknown_artist, duplicate_artist, duplicate] {
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn known_cards_use_latest_and_entry_version() {
    let old_card = "00000000-0000-4000-8000-0000000000c9";
    let mut old = mini("1.9.0", json!({ "en": "Old" }));
    old["cards"][0]["id"] = json!(old_card);
    let dir = temp_dir(
        "known",
        &[("1.9.0.json", old.to_string()), ("1.28.1.json", mini("1.28.1", json!({ "en": "A" })).to_string())],
    );
    let catalog = Catalog::load(&dir).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    let id = |s: &str| s.parse::<uuid::Uuid>().unwrap();
    assert!(catalog.knows_card("1.9.0", &id(old_card)));
    assert!(!catalog.knows_card("1.28.1", &id(old_card)));
    assert!(!catalog.knows_card("2.0.0", &id(old_card)));
    assert!(catalog.knows_card("1.9.0", &id(CARD)));
    assert!(catalog.knows_card("2.0.0", &id(CARD)));
    assert!(!catalog.knows_card("1.28.1", &id("00000000-0000-4000-8000-0000000000ca")));
}

#[tokio::test]
async fn versions_and_language_fallback() {
    let dir = temp_dir(
        "fallback",
        &[
            ("1.9.0.json", mini("1.9.0", json!({ "en": "Old" })).to_string()),
            ("1.28.1.json", mini("1.28.1", json!({ "en": "Alpha" })).to_string()),
            ("notes.txt", "ignored".into()),
        ],
    );
    let catalog = Catalog::load(&dir).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    let app = api::router(state_with(lazy_pool(), "https://steamcommunity.com/openid/login", catalog));

    let versions = send(&app, get("/api/catalog/versions", "")).await;
    assert_eq!(versions.headers["cache-control"], "public, max-age=300");
    let names: Vec<_> = versions.body.as_array().unwrap().iter().map(|v| v["version"].clone()).collect();
    assert_eq!(names, ["1.28.1", "1.9.0"]);

    let list = send(&app, get("/api/cards?lang=fr", "")).await;
    assert_eq!(list.status, StatusCode::OK);
    assert_eq!(list.headers["cache-control"], "public, max-age=300");
    assert_eq!(list.body[0]["name"], "Alpha");
    assert_eq!(list.body[0]["resistances"]["shadow"], 6);
    assert_eq!(list.body[0]["abilities"], json!([{ "id": "hit", "cooldown": 2 }, { "id": "kick", "cooldown": 1 }]));
    assert_eq!(list.body[0]["talents"], json!(["t1", "t2", "t3"]));
    let face = json!({
        "abilities": [
            { "name": "Coup", "cooldown": 2, "element": "water" },
            { "name": "Kick", "cooldown": 1, "element": null }
        ],
        "tox": [
            { "id": "tox1", "name": "Brûlure", "rarity": "rarity-rare", "image": true },
            { "id": "tox2", "name": "Ghost_C", "rarity": null, "image": false }
        ]
    });
    assert_eq!(list.body[0]["face"], face);
    assert_eq!(send(&app, get("/api/cards?version=1.9.0", "")).await.body[0]["name"], "Old");

    let detail = send(&app, get(&format!("/api/cards/{CARD}?lang=fr"), "")).await;
    assert_eq!(detail.status, StatusCode::OK);
    let d = detail.body;
    assert_eq!(d["name"], "Alpha");
    assert_eq!(d["face"], face);
    assert_eq!(d["description"], Value::Null);
    assert_eq!(d["abilities"][0]["cooldown"], 2);
    assert_eq!(d["abilities"][0]["ability"]["name"], "Coup");
    assert_eq!(d["abilities"][0]["ability"]["data"]["manaCost"], 3);
    assert_eq!(d["tox"][0]["name"], "Brûlure");
    assert_eq!(d["tox"][0]["data"]["title"], "Burn_R");
    let talents: Vec<_> = d["talents"].as_array().unwrap().iter().map(|t| t["name"].clone()).collect();
    assert_eq!(talents, [json!("One"), json!("Two"), Value::Null]);
    assert_eq!(d["talents"][0]["description"], "First");
    assert_eq!(d.get("artist"), Some(&Value::Null));

    assert_eq!(send(&app, get("/api/cards?lang=de", "")).await.status, StatusCode::BAD_REQUEST);
    assert_eq!(send(&app, get("/api/cards?version=9.9.9", "")).await.status, StatusCode::NOT_FOUND);
    let unknown = send(&app, get("/api/cards/00000000-0000-4000-8000-0000000000c2", "")).await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert!(unknown.headers.get("cache-control").is_none());
}

#[tokio::test]
async fn real_catalog_card() {
    let app = api::router(state_with(lazy_pool(), "https://steamcommunity.com/openid/login", real_catalog()));
    let list = send(&app, get("/api/cards", "")).await;
    assert_eq!(list.body.as_array().unwrap().len(), 151);
    let d = send(&app, get(&format!("/api/cards/{}?lang=fr", SEAT0_CARDS[0]), "")).await.body;
    assert!(d["name"].is_string());
    assert_eq!(d["talents"].as_array().unwrap().len(), 3);
    assert!(!d["abilities"].as_array().unwrap().is_empty());
}

#[test]
fn artist_link_kinds() {
    let kind = |url: &str| serde_json::to_value(artist_link(url).map(|l| l.kind)).unwrap();
    for (url, expected) in [
        ("https://www.instagram.com/a/", "instagram"),
        ("https://x.com/a", "x"),
        ("https://twitter.com/a", "x"),
        ("https://kilianbillet.artstation.com", "artstation"),
        ("https://bsky.app/profile/a", "bluesky"),
        ("https://www.twitch.tv/a", "twitch"),
        ("https://linktr.ee/a", "linktree"),
        ("https://example.com/", "website"),
        ("https://notinstagram.com/", "website"),
        ("https://instagram.com.evil.example/", "website"),
    ] {
        assert_eq!(kind(url), expected, "{url}");
    }
    assert_eq!(artist_link("https://kilianbillet.artstation.com").unwrap().url, "https://kilianbillet.artstation.com");
    for bad in ["http://www.instagram.com/a", "", "javascript:alert(1)", "ftp://a.com", "https://"] {
        assert!(artist_link(bad).is_none(), "{bad:?}");
    }
}

#[tokio::test]
async fn card_artist() {
    let mut file = mini("1.0.0", json!({ "en": "A" }));
    let artists =
        [(" Ann ", "https://www.instagram.com/ann/"), ("Bob", "http://bob.example/"), ("  ", "https://a.example/")];
    let card = file["cards"][0].clone();
    file["cards"] = json!([]);
    file["artists"] = json!([]);
    for (i, (name, link)) in artists.iter().enumerate() {
        let id = format!("00000000-0000-4000-8000-0000000000a{i}");
        file["artists"].as_array_mut().unwrap().push(json!({ "id": id, "name": name, "link": link }));
        let mut c = card.clone();
        c["id"] = json!(format!("00000000-0000-4000-8000-0000000000c{i}"));
        c["artist"] = json!(id);
        file["cards"].as_array_mut().unwrap().push(c);
    }
    let dir = temp_dir("artist", &[("1.0.0.json", file.to_string())]);
    let catalog = Catalog::load(&dir).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    let app = api::router(state_with(lazy_pool(), "https://steamcommunity.com/openid/login", catalog));
    let artist = async |i: usize| {
        send(&app, get(&format!("/api/cards/00000000-0000-4000-8000-0000000000c{i}"), ""))
            .await
            .body
            .get("artist")
            .cloned()
    };
    let ann = json!({ "name": "Ann", "links": [{ "kind": "instagram", "url": "https://www.instagram.com/ann/" }] });
    assert_eq!(artist(0).await, Some(ann));
    assert_eq!(artist(1).await, Some(json!({ "name": "Bob", "links": [] })));
    assert_eq!(artist(2).await, Some(Value::Null));
    let list = send(&app, get("/api/cards", "")).await.body;
    assert!(list[0].get("artist").is_none());
}

#[tokio::test]
async fn real_catalog_artist() {
    let app = api::router(state_with(lazy_pool(), "https://steamcommunity.com/openid/login", real_catalog()));
    let meridia = send(&app, get("/api/cards/5103c8e0-1326-4bc0-9b2f-05f5daf76b4a", "")).await.body;
    let trinity =
        json!({ "name": "Trinity", "links": [{ "kind": "linktree", "url": "https://linktr.ee/trinitydraw" }] });
    assert_eq!(meridia["artist"], trinity);
    let vivy = send(&app, get("/api/cards/4b5c9594-0497-4d5f-9631-e413c3a89483", "")).await.body;
    assert_eq!(vivy["artist"], json!({ "name": "Mr Wolf", "links": [] }));
}

#[tokio::test]
async fn artist_list() {
    let mut file = mini("1.0.0", json!({ "en": "A" }));
    let artist_id = |i: usize| format!("00000000-0000-4000-8000-0000000000a{i}");
    let card_id = |i: usize| format!("00000000-0000-4000-8000-0000000000c{i}");
    let artists = [
        ("bob", "http://bob.example/"),
        ("Ann", "https://www.instagram.com/ann/"),
        ("Carl", ""),
        ("  ", "https://a.example/"),
        ("Unused", "https://u.example/"),
        ("dee", ""),
        (" Dee ", ""),
    ];
    file["artists"] = artists
        .iter()
        .enumerate()
        .map(|(i, (name, link))| json!({ "id": artist_id(i), "name": name, "link": link }))
        .collect();
    let card = file["cards"][0].clone();
    file["cards"] = [
        (Some(0), 5),
        (Some(1), 3),
        (Some(2), 1),
        (Some(0), 2),
        (Some(3), 4),
        (Some(1), 1),
        (Some(0), 4),
        (None, 0),
        (Some(6), 0),
        (Some(5), 0),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (artist, order))| {
        let mut c = card.clone();
        c["id"] = json!(card_id(i));
        c["artist"] = json!(artist.map(artist_id));
        c["order"] = json!(order);
        c
    })
    .collect();
    let dir = temp_dir("artist-list", &[("1.0.0.json", file.to_string())]);
    let catalog = Catalog::load(&dir).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    let app = api::router(state_with(lazy_pool(), "https://steamcommunity.com/openid/login", catalog));
    let res = send(&app, get("/api/artists?lang=fr", "")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    let list = res.body.as_array().unwrap();
    let names: Vec<_> = list.iter().map(|a| a["name"].clone()).collect();
    assert_eq!(names, [json!("Ann"), json!("bob"), json!("Carl"), json!("Dee"), json!("dee")]);
    assert_eq!(list[0]["links"], json!([{ "kind": "instagram", "url": "https://www.instagram.com/ann/" }]));
    assert_eq!((list[1]["links"].clone(), list[2]["links"].clone()), (json!([]), json!([])));
    let ids = |a: &Value| a["cards"].as_array().unwrap().iter().map(|c| c["id"].clone()).collect::<Vec<_>>();
    let card = |i: usize| json!(card_id(i));
    assert_eq!(ids(&list[1]), [card(3), card(6), card(0)]);
    assert_eq!(ids(&list[0]), [card(5), card(1)]);
    assert_eq!(ids(&list[2]), [card(2)]);
    assert_eq!((ids(&list[3]), ids(&list[4])), (vec![card(8)], vec![card(9)]));
    assert_eq!(list[1]["cards"][0]["face"]["abilities"][0]["name"], "Coup");
    assert_eq!(send(&app, get("/api/artists?lang=xx", "")).await.status, StatusCode::BAD_REQUEST);
    assert_eq!(send(&app, get("/api/artists?version=9.9.9", "")).await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn real_catalog_artists() {
    let app = api::router(state_with(lazy_pool(), "https://steamcommunity.com/openid/login", real_catalog()));
    let list = send(&app, get("/api/artists", "")).await.body;
    let list = list.as_array().unwrap();
    assert_eq!(list.len(), 29);
    let mut ids: Vec<_> = list.iter().flat_map(|a| a["cards"].as_array().unwrap()).map(|c| c["id"].clone()).collect();
    assert_eq!(ids.len(), 151);
    ids.sort_by_key(|id| id.to_string());
    ids.dedup();
    assert_eq!(ids.len(), 151);
    let names: Vec<_> = list.iter().map(|a| a["name"].as_str().unwrap().to_lowercase()).collect();
    assert!(names.is_sorted());
    let wolf = list.iter().find(|a| a["name"] == "Mr Wolf").unwrap();
    assert_eq!(wolf["links"], json!([]));
}

#[tokio::test]
async fn glossary_without_lists() {
    let dir = temp_dir("glossary", &[("1.0.0.json", mini("1.0.0", json!({ "en": "A" })).to_string())]);
    let catalog = Catalog::load(&dir).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    let app = api::router(state_with(lazy_pool(), "https://steamcommunity.com/openid/login", catalog));
    let res = send(&app, get("/api/glossary?lang=fr", "")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.headers["cache-control"], "public, max-age=300");
    let g = res.body;
    assert_eq!((g["keywords"].clone(), g["effects"].clone()), (json!([]), json!([])));
    let names = |list: &Value| list.as_array().unwrap().iter().map(|e| e["name"].clone()).collect::<Vec<_>>();
    assert_eq!(names(&g["abilities"]), [json!("Coup"), json!("Kick")]);
    assert_eq!(names(&g["talents"]), [json!("One"), json!("Two")]);
    assert_eq!(names(&g["tox"]), [json!("Brûlure")]);
    assert_eq!(g["abilities"][0]["cards"], json!([CARD]));
    assert_eq!(g["abilities"][0]["data"]["manaCost"], 3);
    assert_eq!(send(&app, get("/api/glossary/keywords", "")).await.body, json!([]));
    assert_eq!(send(&app, get("/api/glossary?lang=xx", "")).await.status, StatusCode::BAD_REQUEST);
    assert_eq!(send(&app, get("/api/glossary/keywords?version=9.9.9", "")).await.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn real_catalog_glossary() {
    let catalog = real_catalog();
    let app = api::router(state_with(lazy_pool(), "https://steamcommunity.com/openid/login", catalog));
    let cards = send(&app, get("/api/cards?version=1.28.1", "")).await.body;
    let order: std::collections::HashMap<_, _> = cards
        .as_array()
        .unwrap()
        .iter()
        .map(|c| (c["id"].as_str().unwrap().to_string(), c["order"].as_i64().unwrap()))
        .collect();
    let g = send(&app, get("/api/glossary?version=1.28.1", "")).await.body;
    let len = |key: &str| g[key].as_array().unwrap().len();
    assert_eq!((len("keywords"), len("effects"), len("abilities"), len("talents")), (26, 16, 227, 86));
    assert!(len("tox") > 0);
    let taunt = g["keywords"].as_array().unwrap().iter().find(|k| k["id"] == "provocation").unwrap();
    assert_eq!(taunt["name"], "Taunt");
    for key in ["abilities", "talents", "tox"] {
        for e in g[key].as_array().unwrap() {
            assert!(e["name"].is_string(), "{key} {}", e["id"]);
            let list: Vec<i64> = e["cards"].as_array().unwrap().iter().map(|c| order[c.as_str().unwrap()]).collect();
            assert!(!list.is_empty() && list.is_sorted(), "{key} {}", e["id"]);
        }
    }
    let card = &cards[0];
    let ability = card["abilities"][0]["id"].as_str().unwrap();
    let found = g["abilities"].as_array().unwrap().iter().find(|a| a["id"] == ability).unwrap();
    assert!(found["cards"].as_array().unwrap().contains(&card["id"]));

    let fr = send(&app, get("/api/glossary?lang=fr", "")).await.body;
    let taunt = fr["keywords"].as_array().unwrap().iter().find(|k| k["id"] == "provocation").unwrap();
    assert_eq!(taunt["name"], "Provocation");
    let keywords = send(&app, get("/api/glossary/keywords?lang=fr", "")).await;
    assert_eq!(keywords.headers["cache-control"], "public, max-age=300");
    assert_eq!(keywords.body, fr["keywords"]);
}

#[tokio::test]
async fn openapi_lists_routes() {
    let app = api::router(state_with(lazy_pool(), "https://steamcommunity.com/openid/login", real_catalog()));
    let doc = send(&app, get("/api/openapi.json", "")).await.body;
    for path in [
        "/api/matches",
        "/api/cards/{id}",
        "/api/artists",
        "/api/glossary",
        "/api/glossary/keywords",
        "/api/device/token",
        "/api/me/matches",
        "/api/me/top-decks",
        "/api/decks/import",
        "/api/decks/preview",
        "/api/decks/{id}/vote",
        "/api/comments/{id}",
        "/api/reports",
        "/api/stats/decks",
        "/api/stats/cards",
        "/api/admin/reports/{id}/resolve",
    ] {
        assert!(doc["paths"].get(path).is_some(), "{path}");
    }
}

fn refs<'a>(v: &'a Value, out: &mut Vec<&'a str>) {
    match v {
        Value::Object(m) => {
            if let Some(Value::String(r)) = m.get("$ref") {
                out.push(r);
            }
            m.values().for_each(|v| refs(v, out));
        }
        Value::Array(a) => a.iter().for_each(|v| refs(v, out)),
        _ => {}
    }
}

#[tokio::test]
async fn openapi_refs_resolve() {
    let app = api::router(state_with(lazy_pool(), "https://steamcommunity.com/openid/login", real_catalog()));
    let doc = send(&app, get("/api/openapi.json", "")).await.body;
    let mut list = Vec::new();
    refs(&doc, &mut list);
    assert!(!list.is_empty());
    for r in list {
        let name = r.strip_prefix("#/components/schemas/").unwrap_or(r);
        assert!(doc["components"]["schemas"].get(name).is_some(), "{r}");
    }
}
