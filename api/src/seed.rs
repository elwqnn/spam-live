use std::error::Error;

use chrono::{DateTime, TimeDelta, Timelike, Utc};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgConnectOptions};
use uuid::{Builder, Uuid};

use crate::{
    analytics::utc_hour,
    catalog::{Catalog, parse_version},
    matches::{check_account, parse_entry, store},
};

const STEAM_PREFIX: &str = "765611900001";
const UID_PREFIX: &str = "match-00";
const SEED: u64 = 0x5EED_2026_1104_0001;
const PLAYERS: usize = 50;
const FLOOR: usize = 36;
const EXTRA: f64 = 700.0;
const RANDOM: usize = 120;
const EXTRA_ARCHETYPES: usize = 6;
const PUBLIC_DECKS: usize = 20;
const HOURS: i32 = 30 * 24;
const GAP: std::ops::Range<i32> = 240..243;
const DEVICES: [(Option<&str>, i64); 6] = [
    (Some("0.1.0"), 1),
    (Some("0.1.0"), 3),
    (Some("0.1.0"), 30),
    (Some("0.0.9"), 50),
    (None, 100),
    (Some("0.0.9"), 400),
];

const NAMES: [&str; 60] = [
    "Ashen Wolf",
    "pixelmancer",
    "KoboldKing",
    "MossyRock",
    "Emberly",
    "quietstorm",
    "TideCaller",
    "Rune_Smith",
    "VoidPotato",
    "sunny.d",
    "Grimbold",
    "lazy_lich",
    "Nettlewick",
    "frostbyte",
    "Captain Crumb",
    "OakenShield42",
    "Mira the Bold",
    "deckwright",
    "Zephyrine",
    "toxic_tofu",
    "Brambleback",
    "nyx",
    "SaltyShaman",
    "Lumen",
    "Gearhead Gus",
    "spam_enjoyer",
    "Cinderpaw",
    "Wobblefoot",
    "Hexa",
    "SirTopDeck",
    "MarshMallow",
    "glimmer",
    "Ironquill",
    "Dusk Rider",
    "pebble",
    "ManaLeak",
    "Thornberry",
    "Sable",
    "Quokka Lord",
    "Driftwood",
    "MidnightOil",
    "Fennec",
    "BigTurnTim",
    "Starling",
    "Rook & Roll",
    "Mossling",
    "Vesper",
    "Crit Happens",
    "Tinker Tess",
    "Owlbear Hugs",
    "NoDrawNoFun",
    "Kestrel",
    "flintlock",
    "Gloomcap",
    "Ripple",
    "Copperkettle",
    "Wildfire_99",
    "Pip",
    "Obsidia",
    "Lantern Jack",
];

const DECK_NAMES: [&str; 30] = [
    "Burn Them All",
    "Tidal Lock",
    "Shadow Tempo",
    "Root and Rot",
    "Glass Cannon",
    "Slow and Steady",
    "Light Rush",
    "Divine Control",
    "Budget Aggro",
    "Stone Wall",
    "The Long Game",
    "Ember Swarm",
    "Poison Pills",
    "Mist Walkers",
    "Top 100 Climb",
    "Midrange Mash",
    "Sunrise Combo",
    "Nightfall",
    "Thorns Everywhere",
    "Gold Rush",
    "Cheap Tricks",
    "Tox Stack",
    "My First Deck",
    "Comeback Kings",
    "x",
    "Fire + Water = Steam",
    "Untitled",
    "Ranked grind deck v3",
    "Please Nerf This",
    "A deck with a very long name that stays below the limit ok",
];

const GUIDES: [&str; 8] = [
    "",
    "## Plan\n\nPlay the cheap cards first. Keep gold for the last turn.\n\n- Mulligan for the fire card\n- Do not trade early",
    "Simple deck. **Attack** every turn.",
    "## Matchups\n\n| Opponent | Plan |\n| --- | --- |\n| Aggro | Defend |\n| Control | Push damage |\n\nThe deck is weak \
     against *shadow*.",
    "Put the talents on the second card. ~~The old version used water.~~",
    "",
    "1. Keep the healer back.\n2. Stack Tox on the biggest card.\n3. Win.",
    "",
];

const COMMENTS: [&str; 16] = [
    "Nice deck!",
    "Works well in ranked. I went from 20 to 35 stars with it.",
    "Why not the fire card in slot 3?",
    "This deck is too strong, please nerf.",
    "Lost 5 games in a row with it. Not for me.",
    "Good guide, thanks.",
    "The middle talent on card 2 is better.",
    "gg",
    "Tried it against shadow decks: about 50/50.",
    "The updated version is better.",
    "How do you play the mirror?",
    "Still good after the patch.",
    "+1",
    "The first turns are the key. Good write-up.",
    "I played this deck for a full week. It is very good against slow decks because the first two cards put a lot \
     of pressure on the opponent before the gold income starts to matter. Against fast decks it is weaker: keep the \
     healer back and do not spend gold on the first turn. With these changes I won most of my games.",
    "Does it still work at high stars?",
];

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let z = (self.0 ^ (self.0 >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        let z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn chance(&mut self, p: f64) -> bool {
        self.unit() < p
    }

    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            items.swap(i, self.below(i + 1));
        }
    }

    fn between(&mut self, from: DateTime<Utc>, to: DateTime<Utc>) -> DateTime<Utc> {
        from + TimeDelta::milliseconds(((to - from).num_milliseconds() as f64 * self.unit()) as i64)
    }

    fn uuid(&mut self) -> Uuid {
        Builder::from_random_bytes(((u128::from(self.next()) << 64) | u128::from(self.next())).to_le_bytes())
            .into_uuid()
    }

    fn fill(&mut self, mut deck: Vec<usize>, cards: usize) -> Vec<usize> {
        while deck.len() < 4 {
            let c = self.below(cards);
            if !deck.contains(&c) {
                deck.push(c);
            }
        }
        deck
    }
}

struct Deck {
    cards: Vec<usize>,
    bonus: f64,
}

impl Deck {
    fn key(&self) -> Vec<usize> {
        let mut key = self.cards.clone();
        key.sort();
        key
    }
}

struct Game<'a> {
    version: &'a str,
    mode: i32,
    ai: bool,
    players: [usize; 2],
    decks: [Vec<Uuid>; 2],
    winner: Option<usize>,
    end: DateTime<Utc>,
    duration: TimeDelta,
    turns: i32,
    stars: [i32; 2],
    uploads: [bool; 2],
}

fn entry(g: &Game, uid: &str, seat: usize, accounts: &[Uuid]) -> Value {
    let me = accounts[g.players[seat]];
    let opponent = (!g.ai).then(|| accounts[g.players[1 - seat]]);
    let winner = match g.winner {
        Some(w) if w == seat => Some(me),
        Some(_) => opponent,
        None => None,
    };
    json!({
        "FormatVersion": 2,
        "AppVersion": g.version,
        "GameUid": uid,
        "AccountId": me,
        "Seat": seat,
        "GameMode": g.mode,
        "OpponentId": opponent,
        "OpponentIsAi": g.ai,
        "WinnerId": winner,
        "IsDraw": g.winner.is_none(),
        "MyCards": g.decks[seat],
        "OpponentCards": g.decks[1 - seat],
        "StartUtc": g.end - g.duration,
        "EndUtc": g.end,
        "DurationMs": g.duration.num_milliseconds(),
        "TurnCount": g.turns,
        "Loading": { "me": { "stars": g.stars[seat] } },
    })
}

fn older(catalog: &Catalog, latest: &str) -> Result<String, Box<dyn Error>> {
    if let Some(v) = catalog.previous(latest) {
        return Ok(v.into());
    }
    let mut parts = parse_version(latest).ok_or("the latest catalog version is invalid")?;
    let i = parts.iter().rposition(|&p| p > 0).ok_or("the latest catalog version has no older version")?;
    parts[i] -= 1;
    Ok(parts.iter().map(u64::to_string).collect::<Vec<_>>().join("."))
}

pub fn check_host(url: &str) -> Result<PgConnectOptions, String> {
    let options: PgConnectOptions = url.parse().map_err(|e: sqlx::Error| e.to_string())?;
    if !matches!(options.get_host(), "127.0.0.1" | "localhost") {
        return Err(format!("--seed refuses database host {:?}: use 127.0.0.1 or localhost", options.get_host()));
    }
    Ok(options)
}

pub async fn run(db: &PgPool, catalog: &Catalog) -> Result<(), Box<dyn Error>> {
    let cards = catalog.card_ids();
    if cards.len() < 8 {
        return Err("the latest catalog needs at least 8 cards".into());
    }
    let v_new = catalog.latest();
    let v_old = older(catalog, v_new)?;
    let now = Utc::now();
    let day = TimeDelta::days(1);
    let hour = TimeDelta::hours(1);
    let mut rng = Rng(SEED);

    let accounts: Vec<Uuid> = (0..PLAYERS).map(|_| rng.uuid()).collect();
    let base_stars: Vec<i32> = (0..PLAYERS).map(|_| rng.below(61) as i32).collect();
    let mut order: Vec<usize> = (0..cards.len()).collect();
    rng.shuffle(&mut order);
    let mut decks: Vec<Deck> = order
        .chunks(4)
        .map(<[usize]>::to_vec)
        .chain(std::iter::repeat_with(Vec::new).take(EXTRA_ARCHETYPES))
        .map(|d| Deck { cards: rng.fill(d, cards.len()), bonus: (rng.unit() - 0.5) * 0.12 })
        .collect();
    let archetypes = decks.len();
    let mut strength: Vec<f64> = (0..cards.len()).map(|_| (rng.unit() - 0.5) * 0.16).collect();

    let windows = [
        (v_new, now - day * 7 + hour, now - hour),
        (v_new, now - day * 14 + hour, now - day * 7),
        (v_old.as_str(), now - day * 28, now - day * 14),
    ];
    let mut games = Vec::new();
    for (version, from, to) in windows {
        for s in &mut strength {
            *s += (rng.unit() - 0.5) * 0.05;
        }
        let weights: Vec<f64> = (0..archetypes).map(|a| (0.7 + 0.6 * rng.unit()) / (1.0 + a as f64 * 0.25)).collect();
        let total: f64 = weights.iter().sum();
        let mut slots = Vec::new();
        for (a, w) in weights.iter().enumerate() {
            slots.extend(std::iter::repeat_n(a, FLOOR + (EXTRA * w / total).round() as usize));
        }
        let target = slots.len() + RANDOM + (slots.len() + RANDOM) % 2;
        while slots.len() < target {
            decks.push(Deck { cards: rng.fill(Vec::new(), cards.len()), bonus: 0.0 });
            slots.push(decks.len() - 1);
        }
        rng.shuffle(&mut slots);
        while let Some(i) = (0..slots.len() / 2).find(|i| decks[slots[2 * i]].key() == decks[slots[2 * i + 1]].key()) {
            let j = rng.below(slots.len());
            slots.swap(2 * i + 1, j);
        }
        let pairs = slots.len() / 2;
        for i in 0..pairs + pairs / 9 {
            let counted = i < pairs;
            let pair =
                if counted { [slots[2 * i], slots[2 * i + 1]] } else { [rng.below(archetypes), rng.below(archetypes)] };
            let ai = !counted && rng.chance(0.5);
            let s = pair.map(|d| decks[d].cards.iter().map(|&c| strength[c]).sum::<f64>() / 4.0 + decks[d].bonus);
            let winner = if counted && rng.chance(0.02) {
                None
            } else if rng.chance((0.5 + s[0] - s[1]).clamp(0.2, 0.8)) {
                Some(0)
            } else {
                Some(1)
            };
            let p0 = rng.below(PLAYERS);
            let p1 = (p0 + 1 + rng.below(PLAYERS - 1)) % PLAYERS;
            games.push(Game {
                version,
                mode: if counted || ai { 10 } else { 40 },
                ai,
                players: [p0, p1],
                decks: pair.map(|d| decks[d].cards.iter().map(|&c| cards[c]).collect()),
                winner,
                end: rng.between(from, to),
                duration: TimeDelta::milliseconds(240_000 + rng.below(960_000) as i64),
                turns: 6 + rng.below(25) as i32,
                stars: [p0, p1].map(|p| (base_stars[p] + rng.below(7) as i32 - 3).clamp(0, 60)),
                uploads: [true, !ai && rng.chance(0.5)],
            });
        }
    }

    sqlx::query("DELETE FROM matches WHERE game_uid LIKE $1").bind(format!("{UID_PREFIX}%")).execute(db).await?;
    sqlx::query("DELETE FROM users WHERE steam_id LIKE $1").bind(format!("{STEAM_PREFIX}%")).execute(db).await?;
    let mut users = Vec::with_capacity(NAMES.len());
    for (i, name) in NAMES.iter().enumerate() {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO users (steam_id, display_name, created_at) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(format!("{STEAM_PREFIX}{i:05}"))
        .bind(name)
        .bind(rng.between(now - day * 90, now - day * 30))
        .fetch_one(db)
        .await?;
        users.push(id);
    }

    let mut linked = vec![None; PLAYERS];
    for (n, g) in games.iter().enumerate() {
        let uid = format!("{UID_PREFIX}{:08}", n + 1);
        for seat in [0, 1].into_iter().filter(|&s| g.uploads[s]) {
            let p = g.players[seat];
            let m = parse_entry(entry(g, &uid, seat, &accounts), now)?;
            check_account(db, users[p], &mut linked[p], accounts[p]).await.map_err(|e| e.1)??;
            store(db, catalog, users[p], &m, if (n + seat) % 3 == 0 { "web" } else { "desktop" }).await?;
        }
    }

    sqlx::query("UPDATE matches SET created_at = end_utc + interval '2 minutes' WHERE game_uid LIKE $1")
        .bind(format!("{UID_PREFIX}%"))
        .execute(db)
        .await?;
    for (i, (version, hours)) in DEVICES.iter().enumerate() {
        sqlx::query(
            "INSERT INTO devices (user_id, token_hash, created_at, last_used_at, app_version)
             VALUES ($1, sha256(convert_to('seed-device-' || $1, 'UTF8')), $2, $3, $4)",
        )
        .bind(users[i])
        .bind(now - day * 20)
        .bind(now - hour * *hours as i32)
        .bind(version)
        .execute(db)
        .await?;
    }

    let mut picks: Vec<usize> = (0..archetypes).collect();
    rng.shuffle(&mut picks);
    for (i, name) in DECK_NAMES.iter().enumerate() {
        let public = i < PUBLIC_DECKS;
        let deck = if public || i % 2 == 0 {
            decks[picks[i % archetypes]].cards.clone()
        } else {
            rng.fill(Vec::new(), cards.len())
        };
        let deck: Vec<Uuid> = deck.into_iter().map(|c| cards[c]).collect();
        let talents: Vec<i16> = (0..4).map(|_| rng.below(3) as i16).collect();
        let author = rng.below(users.len());
        let game_deck_id = rng.chance(0.35).then(|| format!("{:016x}", rng.next()));
        let guide = GUIDES[rng.below(GUIDES.len())];
        let created = rng.between(now - day * 14, now - hour * 2);
        let published = public.then(|| rng.between(created, now - hour));
        let updated = published.unwrap_or_else(|| rng.between(created, now));
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO decks (author_id, name, cards, talents, guide, status, app_version, game_deck_id,
                 created_at, updated_at, published_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id",
        )
        .bind(users[author])
        .bind(name)
        .bind(&deck)
        .bind(&talents)
        .bind(guide)
        .bind(if public { "public" } else { "draft" })
        .bind(if i % 8 == 3 { v_old.as_str() } else { v_new })
        .bind(game_deck_id)
        .bind(created)
        .bind(updated)
        .bind(published)
        .fetch_one(db)
        .await?;
        let Some(published) = published else { continue };
        let mut voters: Vec<usize> = (0..users.len()).filter(|&u| u != author).collect();
        rng.shuffle(&mut voters);
        voters.truncate(rng.below(26));
        for v in voters {
            sqlx::query("INSERT INTO votes (deck_id, user_id, created_at) VALUES ($1, $2, $3)")
                .bind(id)
                .bind(users[v])
                .bind(rng.between(published, now))
                .execute(db)
                .await?;
        }
        let mut times: Vec<DateTime<Utc>> = (0..rng.below(7)).map(|_| rng.between(published, now)).collect();
        times.sort();
        for t in times {
            sqlx::query(
                "INSERT INTO comments (deck_id, author_id, body, created_at, updated_at) VALUES ($1, $2, $3, $4, $4)",
            )
            .bind(id)
            .bind(users[rng.below(users.len())])
            .bind(COMMENTS[rng.below(COMMENTS.len())])
            .bind(t)
            .execute(db)
            .await?;
        }
    }
    analytics(db, &mut rng, now).await
}

async fn analytics(db: &PgPool, rng: &mut Rng, now: DateTime<Utc>) -> Result<(), Box<dyn Error>> {
    let end = utc_hour(now);
    let start = end - TimeDelta::hours(i64::from(HOURS));
    let (mut hours, mut web_peaks, mut app_peaks, mut web_ids, mut app_ids) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for i in (0..HOURS).filter(|i| !GAP.contains(i)) {
        let hour = start + TimeDelta::hours(i64::from(i));
        let day = 0.5 + 0.5 * (std::f64::consts::TAU * (f64::from(hour.hour()) - 20.0) / 24.0).cos();
        let growth = 0.6 + 0.4 * f64::from(i) / f64::from(HOURS);
        let web = (2.0 + 14.0 * day * growth + 3.0 * rng.unit()).round() as i32;
        let app = (f64::from(web) * (0.3 + 0.2 * rng.unit())).round() as i32;
        hours.push(hour);
        web_peaks.push(web);
        app_peaks.push(app);
        web_ids.push(web * (2 + rng.below(3) as i32) + rng.below(4) as i32);
        app_ids.push(app + rng.below(3) as i32);
    }
    sqlx::query("DELETE FROM analytics_hourly WHERE hour >= $1 AND hour < $2")
        .bind(start)
        .bind(end)
        .execute(db)
        .await?;
    sqlx::query(
        "WITH m AS MATERIALIZED (
             SELECT created_at, game_uid IN (SELECT game_uid FROM deck_results) AS ranked FROM matches
         )
         INSERT INTO analytics_hourly (hour, web_peak, app_peak, web_distinct, app_distinct, matches, ranked_matches,
             users, linked_accounts, public_decks, devices)
         SELECT t.hour, t.web_peak, t.app_peak, t.web_distinct, t.app_distinct,
             (SELECT count(*) FROM m WHERE m.created_at < t.hour + interval '1 hour'),
             (SELECT count(*) FROM m WHERE m.ranked AND m.created_at < t.hour + interval '1 hour'),
             (SELECT count(*) FROM users u WHERE u.created_at < t.hour + interval '1 hour'),
             (SELECT count(*) FROM users u WHERE u.spam_account_id IS NOT NULL
                 AND u.created_at < t.hour + interval '1 hour'),
             (SELECT count(*) FROM decks d WHERE d.status = 'public' AND d.published_at < t.hour + interval '1 hour'),
             (SELECT count(*) FROM devices d WHERE d.created_at < t.hour + interval '1 hour')
         FROM unnest($1::timestamptz[], $2::int[], $3::int[], $4::int[], $5::int[])
             AS t (hour, web_peak, app_peak, web_distinct, app_distinct)",
    )
    .bind(&hours)
    .bind(&web_peaks)
    .bind(&app_peaks)
    .bind(&web_ids)
    .bind(&app_ids)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn counts(db: &PgPool) -> Result<[(&'static str, i64); 6], sqlx::Error> {
    let (users, matches, uploads, decks, votes, comments): (i64, i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM users WHERE steam_id LIKE $1),
             (SELECT count(*) FROM matches WHERE game_uid LIKE $2),
             (SELECT count(*) FROM uploads WHERE game_uid LIKE $2),
             (SELECT count(*) FROM decks d JOIN users u ON u.id = d.author_id WHERE u.steam_id LIKE $1),
             (SELECT count(*) FROM votes v JOIN users u ON u.id = v.user_id WHERE u.steam_id LIKE $1),
             (SELECT count(*) FROM comments c JOIN users u ON u.id = c.author_id WHERE u.steam_id LIKE $1)",
    )
    .bind(format!("{STEAM_PREFIX}%"))
    .bind(format!("{UID_PREFIX}%"))
    .fetch_one(db)
    .await?;
    Ok([
        ("users", users),
        ("matches", matches),
        ("uploads", uploads),
        ("decks", decks),
        ("votes", votes),
        ("comments", comments),
    ])
}
