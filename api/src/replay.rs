use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
};

use serde_json::Value;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::matches::{gunzip, split_replay};

pub const DECODER_VERSION: i32 = 1;

const NEW_TURN: u16 = 2003;
const SHOP_BUY: u16 = 2020;
const TRUNCATED: &str = "replay is truncated";

pub struct ShopBuy {
    pub seat: i16,
    pub idx: i32,
    pub turn: i32,
    pub card: Uuid,
    pub tox: Uuid,
    pub equipped: String,
}

pub struct TalentPick {
    pub seat: i16,
    pub card: Uuid,
    pub talent: i16,
}

pub struct CardDamage {
    pub seat: i16,
    pub card: Uuid,
    pub done: i32,
    pub taken: i32,
}

pub struct HpTurn {
    pub seat: i16,
    pub turn: i32,
    pub hp: i32,
}

pub struct Decoded {
    pub uploader_seat: i16,
    pub shop_buys: Vec<ShopBuy>,
    pub talents: Vec<TalentPick>,
    pub damage: Vec<CardDamage>,
    pub hp: Vec<HpTurn>,
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let (head, rest) = self.0.split_at_checked(n)?;
        self.0 = rest;
        Some(head)
    }

    fn u32(&mut self) -> Option<u32> {
        self.take(4)?.try_into().ok().map(u32::from_le_bytes)
    }

    fn i32(&mut self) -> Option<i32> {
        self.take(4)?.try_into().ok().map(i32::from_le_bytes)
    }

    fn uv(&mut self) -> Option<u64> {
        let mut n = 0;
        for shift in (0..64).step_by(7) {
            let b = self.take(1)?[0];
            n |= u64::from(b & 0x7f) << shift;
            if b & 0x80 == 0 {
                return Some(n);
            }
        }
        None
    }

    fn zz(&mut self) -> Option<i64> {
        let v = self.uv()?;
        Some((v >> 1) as i64 ^ -((v & 1) as i64))
    }

    fn str8(&mut self) -> Option<Option<&'a str>> {
        let n = self.uv()?;
        if n == 0 {
            return Some(None);
        }
        let len = usize::try_from(n / 2).ok()?.checked_sub(1)?;
        std::str::from_utf8(self.take(len)?).ok().map(Some)
    }

    fn str16(&mut self) -> Option<String> {
        let n = usize::try_from(self.u32()?).ok()?;
        let units: Vec<u16> =
            self.take(n.checked_mul(2)?)?.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        String::from_utf16(&units).ok()
    }
}

struct CardState<'a> {
    card: Uuid,
    uid: Option<&'a str>,
    player: i64,
    talent: i64,
    damage: i64,
    hp: i64,
    done: i64,
    taken: i64,
}

fn card_at(p: &[u8], j: usize) -> Option<CardState<'_>> {
    let mut lengths = Reader(p.get(j.checked_sub(8)?..j)?);
    let len = lengths.u32()?;
    if lengths.u32()? != len {
        return None;
    }
    let mut r = Reader(p.get(j..j.checked_add(usize::try_from(len).ok()?)?)?);
    r.take(2)?;
    let id = r.str8()?.filter(|id| id.len() == 36)?;
    let card = Uuid::parse_str(id).ok()?;
    let uid = r.str8()?;
    let player = r.zz()?;
    r.str8()?;
    let talent = r.zz()?;
    for _ in 0..3 {
        r.zz()?;
    }
    r.take(1)?;
    let damage = r.zz()?;
    let hp = r.zz()?;
    for _ in 0..5 {
        r.zz()?;
    }
    let mut stats = [0; 12];
    for s in &mut stats {
        *s = r.zz()?;
    }
    Some(CardState { card, uid, player, talent, damage, hp, done: stats[0], taken: stats[1] })
}

fn cards(p: &[u8]) -> impl Iterator<Item = CardState<'_>> {
    (8..p.len()).filter(|&j| p[j..].starts_with(&[0xC5, 1])).filter_map(move |j| card_at(p, j))
}

fn shop_buy(p: &[u8]) -> Option<(i16, String, String, String)> {
    let mut r = Reader(p.get(2..)?);
    let seat = match r.i32()? {
        0 => 0,
        1 => 1,
        _ => return None,
    };
    let (uid, tox, equipped) = (r.str16()?, r.str16()?, r.str16()?);
    r.str16()?;
    r.0.is_empty().then_some((seat, uid, tox, equipped))
}

fn int(v: i64) -> i32 {
    v.clamp(i32::MIN.into(), i32::MAX.into()) as i32
}

pub fn decode(gz: &[u8], seat0_cards: &[Uuid], seat1_cards: &[Uuid]) -> Result<Decoded, String> {
    let raw = gunzip(gz)?;
    let (header, body) = split_replay(&raw)?;
    let header: Value = serde_json::from_slice(header).map_err(|_| "invalid replay header")?;
    let uploader_seat: i16 = match header.get("Seat").and_then(Value::as_i64) {
        Some(0) => 0,
        Some(1) => 1,
        _ => return Err("replay Seat must be 0 or 1".into()),
    };
    let mut r = Reader(body);
    let count = r.u32().ok_or(TRUNCATED)?;
    if header.get("MessageCount").and_then(Value::as_u64).is_some_and(|n| n != u64::from(count)) {
        return Err("message count does not match the header".into());
    }
    let mut messages = Vec::new();
    for _ in 0..count {
        r.u32().ok_or(TRUNCATED)?;
        let len = r.u32().ok_or(TRUNCATED)?;
        let payload = r.take(len as usize).ok_or(TRUNCATED)?;
        let kind = payload.get(..2).ok_or("message has no type")?;
        messages.push((u16::from_le_bytes([kind[0], kind[1]]), payload));
    }
    if !r.0.is_empty() {
        return Err("data after the last message".into());
    }

    let decks = [seat0_cards, seat1_cards];
    let mut uids: HashMap<&str, Uuid> = HashMap::new();
    let mut last: HashMap<(i64, Uuid), CardState> = HashMap::new();
    let mut hp: BTreeMap<i32, [i64; 2]> = BTreeMap::new();
    let mut buys = Vec::new();
    let mut turn = 0;
    let snapshot = |last: &HashMap<(i64, Uuid), CardState>| {
        [0, 1].map(|s| {
            decks[s]
                .iter()
                .filter_map(|c| last.get(&(s as i64, *c)))
                .fold(0i64, |sum, c| sum.saturating_add(c.hp.saturating_sub(c.damage).max(0)))
        })
    };
    for (kind, payload) in &messages {
        match *kind {
            NEW_TURN => {
                let n = Reader(payload.get(10..).unwrap_or_default()).i32().ok_or("invalid NewTurn message")?;
                if n != turn {
                    if turn > 0 {
                        hp.insert(turn, snapshot(&last));
                    }
                    turn = n;
                }
            }
            SHOP_BUY => {
                let (seat, uid, tox, equipped) = shop_buy(payload).ok_or("invalid shop buy message")?;
                buys.push((seat, turn, uid, tox, equipped));
            }
            _ => {}
        }
        for c in cards(payload) {
            if let Some(uid) = c.uid {
                uids.entry(uid).or_insert(c.card);
            }
            last.insert((c.player, c.card), c);
        }
    }
    if turn > 0 {
        hp.insert(turn, snapshot(&last));
    }

    let mut idx = [0; 2];
    let shop_buys = buys
        .into_iter()
        .filter_map(|(seat, turn, uid, tox, equipped)| {
            let card = *uids.get(uid.as_str())?;
            let tox = Uuid::parse_str(&tox).ok()?;
            let i = &mut idx[seat as usize];
            *i += 1;
            Some(ShopBuy { seat, idx: *i - 1, turn, card, tox, equipped })
        })
        .collect();
    let last = &last;
    let talents = decks[uploader_seat as usize]
        .iter()
        .filter_map(|&card| {
            let talent = i16::try_from(last.get(&(uploader_seat.into(), card))?.talent).ok().filter(|t| *t >= 1)?;
            Some(TalentPick { seat: uploader_seat, card, talent })
        })
        .collect();
    let damage = (0..2)
        .flat_map(|s| {
            decks[s].iter().filter_map(move |&card| {
                let c = last.get(&(s as i64, card))?;
                Some(CardDamage { seat: s as i16, card, done: int(c.done), taken: int(c.taken) })
            })
        })
        .collect();
    let hp = hp
        .into_iter()
        .flat_map(|(turn, hp)| (0..2).map(move |s| HpTurn { seat: s as i16, turn, hp: int(hp[s]) }))
        .collect();
    Ok(Decoded { uploader_seat, shop_buys, talents, damage, hp })
}

#[derive(FromRow)]
struct Stored {
    user_id: i64,
    seat: i16,
    data: Vec<u8>,
    decoder_version: Option<i32>,
    decode_error: Option<String>,
}

impl Stored {
    fn failed(&self) -> bool {
        self.decoder_version == Some(DECODER_VERSION) && self.decode_error.is_some()
    }
}

pub async fn decode_match(db: &PgPool, game_uid: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let Some((seat0, seat1)): Option<(Vec<Uuid>, Vec<Uuid>)> =
        sqlx::query_as("SELECT seat0_cards, seat1_cards FROM matches WHERE game_uid = $1")
            .bind(game_uid)
            .fetch_optional(db)
            .await?
    else {
        return Ok(());
    };
    let replays: Vec<Stored> = sqlx::query_as(
        "SELECT r.user_id, u.seat, r.data, r.decoder_version, r.decode_error
         FROM replays r JOIN uploads u USING (game_uid, user_id)
         WHERE r.game_uid = $1 ORDER BY r.uploaded_at, r.user_id",
    )
    .bind(game_uid)
    .fetch_all(db)
    .await?;
    let sources: Vec<(i16, bool, i32)> =
        sqlx::query_as("SELECT seat, own, decoder_version FROM match_event_sources WHERE game_uid = $1")
            .bind(game_uid)
            .fetch_all(db)
            .await?;
    let mut results: Vec<Option<Result<Decoded, String>>> = replays.iter().map(|_| None).collect();
    for seat in [0, 1] {
        let usable_own = replays.iter().any(|r| r.seat == seat && !r.failed());
        let done =
            sources.iter().any(|&(s, own, version)| s == seat && version == DECODER_VERSION && (own || !usable_own));
        if done {
            continue;
        }
        let order = (0..replays.len()).filter(|&i| replays[i].seat == seat);
        for i in order.chain((0..replays.len()).filter(|&i| replays[i].seat != seat)) {
            let r = &replays[i];
            if r.failed() {
                continue;
            }
            if results[i].is_none() {
                let (data, decks) = (r.data.clone(), [seat0.clone(), seat1.clone()]);
                let mut result = tokio::task::spawn_blocking(move || decode(&data, &decks[0], &decks[1])).await?;
                if result.as_ref().is_ok_and(|d| d.uploader_seat != r.seat) {
                    result = Err("replay Seat does not match the index entry".into());
                }
                if let Err(reason) = &result {
                    tracing::warn!(game_uid, reason = %reason, "replay decode failed");
                }
                sqlx::query(
                    "UPDATE replays SET decoder_version = $3, decode_error = $4 WHERE game_uid = $1 AND user_id = $2",
                )
                .bind(game_uid)
                .bind(r.user_id)
                .bind(DECODER_VERSION)
                .bind(result.as_ref().err())
                .execute(db)
                .await?;
                results[i] = Some(result);
            }
            if let Some(Ok(d)) = &results[i] {
                write(db, game_uid, seat, r.user_id, r.seat == seat, d).await?;
                break;
            }
        }
    }
    Ok(())
}

pub async fn write(
    db: &PgPool,
    game_uid: &str,
    seat: i16,
    user_id: i64,
    own: bool,
    d: &Decoded,
) -> Result<bool, sqlx::Error> {
    let mut tx = db.begin().await?;
    sqlx::query("SELECT 1 FROM matches WHERE game_uid = $1 FOR UPDATE").bind(game_uid).execute(&mut *tx).await?;
    let current: Option<bool> = sqlx::query_scalar(
        "SELECT own FROM match_event_sources WHERE game_uid = $1 AND seat = $2 AND decoder_version = $3",
    )
    .bind(game_uid)
    .bind(seat)
    .bind(DECODER_VERSION)
    .fetch_optional(&mut *tx)
    .await?;
    if current.is_some_and(|current| current || !own) {
        return Ok(false);
    }
    sqlx::query(
        "WITH a AS (DELETE FROM shop_buys WHERE game_uid = $1 AND seat = $2),
             b AS (DELETE FROM talent_picks WHERE game_uid = $1 AND seat = $2),
             c AS (DELETE FROM card_damage WHERE game_uid = $1 AND seat = $2)
         DELETE FROM hp_turns WHERE game_uid = $1 AND seat = $2",
    )
    .bind(game_uid)
    .bind(seat)
    .execute(&mut *tx)
    .await?;
    let buys: Vec<&ShopBuy> = d.shop_buys.iter().filter(|b| b.seat == seat).collect();
    sqlx::query(
        "INSERT INTO shop_buys (game_uid, seat, idx, turn, card, tox, decoder_version)
         SELECT $1, $2, idx, turn, card, tox, $7 FROM unnest($3::int[], $4::int[], $5::uuid[], $6::uuid[])
             AS t (idx, turn, card, tox)
         ON CONFLICT DO NOTHING",
    )
    .bind(game_uid)
    .bind(seat)
    .bind(buys.iter().map(|b| b.idx).collect::<Vec<_>>())
    .bind(buys.iter().map(|b| b.turn).collect::<Vec<_>>())
    .bind(buys.iter().map(|b| b.card).collect::<Vec<_>>())
    .bind(buys.iter().map(|b| b.tox).collect::<Vec<_>>())
    .bind(DECODER_VERSION)
    .execute(&mut *tx)
    .await?;
    let talents: Vec<&TalentPick> = d.talents.iter().filter(|t| own && t.seat == seat).collect();
    sqlx::query(
        "INSERT INTO talent_picks (game_uid, seat, card, talent, decoder_version)
         SELECT $1, $2, card, talent, $5 FROM unnest($3::uuid[], $4::smallint[]) AS t (card, talent)
         ON CONFLICT DO NOTHING",
    )
    .bind(game_uid)
    .bind(seat)
    .bind(talents.iter().map(|t| t.card).collect::<Vec<_>>())
    .bind(talents.iter().map(|t| t.talent).collect::<Vec<_>>())
    .bind(DECODER_VERSION)
    .execute(&mut *tx)
    .await?;
    let damage: Vec<&CardDamage> = d.damage.iter().filter(|c| c.seat == seat).collect();
    sqlx::query(
        "INSERT INTO card_damage (game_uid, seat, card, done, taken, decoder_version)
         SELECT $1, $2, card, done, taken, $6 FROM unnest($3::uuid[], $4::int[], $5::int[]) AS t (card, done, taken)
         ON CONFLICT DO NOTHING",
    )
    .bind(game_uid)
    .bind(seat)
    .bind(damage.iter().map(|c| c.card).collect::<Vec<_>>())
    .bind(damage.iter().map(|c| c.done).collect::<Vec<_>>())
    .bind(damage.iter().map(|c| c.taken).collect::<Vec<_>>())
    .bind(DECODER_VERSION)
    .execute(&mut *tx)
    .await?;
    let hp: Vec<&HpTurn> = d.hp.iter().filter(|h| h.seat == seat).collect();
    sqlx::query(
        "INSERT INTO hp_turns (game_uid, seat, turn, hp, decoder_version)
         SELECT $1, $2, turn, hp, $5 FROM unnest($3::int[], $4::int[]) AS t (turn, hp)
         ON CONFLICT DO NOTHING",
    )
    .bind(game_uid)
    .bind(seat)
    .bind(hp.iter().map(|h| h.turn).collect::<Vec<_>>())
    .bind(hp.iter().map(|h| h.hp).collect::<Vec<_>>())
    .bind(DECODER_VERSION)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO match_event_sources (game_uid, seat, user_id, own, decoder_version) VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (game_uid, seat) DO UPDATE SET user_id = excluded.user_id, own = excluded.own,
             decoder_version = excluded.decoder_version, decoded_at = now()",
    )
    .bind(game_uid)
    .bind(seat)
    .bind(user_id)
    .bind(own)
    .bind(DECODER_VERSION)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
}

pub async fn decode_stored_replays(db: &PgPool) -> Result<usize, Box<dyn Error + Send + Sync>> {
    let uids: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT r.game_uid FROM replays r JOIN uploads u USING (game_uid, user_id)
         WHERE (r.decoder_version IS NULL OR r.decoder_version < $1)
             AND (NOT EXISTS (SELECT 1 FROM match_event_sources s WHERE s.game_uid = r.game_uid
                     AND s.seat = u.seat AND s.decoder_version = $1 AND s.own)
                 OR NOT EXISTS (SELECT 1 FROM match_event_sources s WHERE s.game_uid = r.game_uid
                     AND s.seat = 1 - u.seat AND s.decoder_version = $1))",
    )
    .bind(DECODER_VERSION)
    .fetch_all(db)
    .await?;
    for uid in &uids {
        if let Err(e) = decode_match(db, uid).await {
            tracing::error!(game_uid = uid, error = %e, "replay decode failed");
        }
    }
    Ok(uids.len())
}
