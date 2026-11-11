use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

use axum::{
    Router,
    extract::{Path as UrlPath, Query, State},
    http::StatusCode,
    response::Response,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{ApiError, AppState, public};

type Text = Option<HashMap<String, Option<String>>>;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct File {
    app_version: String,
    languages: Vec<String>,
    cards: Vec<RawCard>,
    abilities: Vec<RawEntry>,
    talents: Vec<RawEntry>,
    tox: Vec<RawEntry>,
    #[serde(default)]
    keywords: Vec<RawEntry>,
    #[serde(default)]
    effects: Vec<RawEntry>,
    #[serde(default)]
    artists: Vec<RawArtist>,
    #[serde(default)]
    ranks: Vec<RawRank>,
    #[serde(default)]
    elements: Vec<RawOrdered>,
    #[serde(default)]
    rarities: Vec<RawOrdered>,
    #[serde(default)]
    game_types: Vec<RawGameType>,
    #[serde(default)]
    enums: Enums,
}

#[derive(Deserialize)]
struct RawRank {
    id: String,
    threshold: i64,
    name: Text,
}

#[derive(Deserialize)]
struct RawOrdered {
    id: String,
    order: i64,
    name: Text,
}

#[derive(Deserialize)]
struct RawGameType {
    id: String,
    value: i64,
    name: Text,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawCard {
    id: Uuid,
    name: Text,
    description: Text,
    element: String,
    rarity: String,
    hp: i64,
    gold_income: i64,
    fire_resistance: i64,
    water_resistance: i64,
    earth_resistance: i64,
    nature_resistance: i64,
    light_resistance: i64,
    shadow_resistance: i64,
    order: i64,
    active_abilities: Vec<AbilityRef>,
    passive_abilities: Vec<String>,
    talents: Vec<String>,
    #[serde(default)]
    artist: Option<Uuid>,
}

#[derive(Deserialize)]
struct RawArtist {
    id: Uuid,
    name: String,
    #[serde(default)]
    link: Option<String>,
}

#[derive(Deserialize)]
struct RawEntry {
    id: String,
    name: Text,
    description: Text,
    #[serde(flatten)]
    data: Map<String, Value>,
}

struct Version {
    name: String,
    key: Vec<u64>,
    languages: Vec<String>,
    cards: Vec<RawCard>,
    abilities: HashMap<String, RawEntry>,
    talents: HashMap<String, RawEntry>,
    tox: HashMap<String, RawEntry>,
    keywords: Vec<RawEntry>,
    effects: Vec<RawEntry>,
    artists: HashMap<Uuid, RawArtist>,
    ranks: Vec<RawRank>,
    elements: Vec<RawOrdered>,
    rarities: Vec<RawOrdered>,
    game_types: Vec<RawGameType>,
    enums: Enums,
}

pub struct Catalog {
    versions: Vec<Version>,
}

pub fn parse_version(s: &str) -> Option<Vec<u64>> {
    if s.len() > 32 {
        return None;
    }
    s.split('.')
        .map(|p| (!p.is_empty() && p.bytes().all(|b| b.is_ascii_digit())).then(|| p.parse().ok()).flatten())
        .collect()
}

fn index(list: Vec<RawEntry>, kind: &str) -> Result<HashMap<String, RawEntry>, String> {
    let len = list.len();
    let map: HashMap<_, _> = list.into_iter().map(|e| (e.id.clone(), e)).collect();
    if map.len() != len {
        return Err(format!("duplicate {kind} id"));
    }
    Ok(map)
}

impl Version {
    fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let file: File = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let key =
            parse_version(&file.app_version).ok_or_else(|| format!("invalid appVersion {:?}", file.app_version))?;
        if !file.languages.iter().any(|l| l == "en") {
            return Err("languages does not contain en".into());
        }
        let abilities = index(file.abilities, "ability")?;
        let talents = index(file.talents, "talent")?;
        let tox = index(file.tox, "tox")?;
        let len = file.artists.len();
        let artists: HashMap<_, _> = file.artists.into_iter().map(|a| (a.id, a)).collect();
        if artists.len() != len {
            return Err("duplicate artist id".into());
        }
        for card in &file.cards {
            let missing = card
                .active_abilities
                .iter()
                .map(|a| &a.id)
                .find(|id| !abilities.contains_key(*id))
                .or_else(|| card.passive_abilities.iter().find(|id| !tox.contains_key(*id)))
                .or_else(|| card.talents.iter().find(|id| !talents.contains_key(*id)));
            if let Some(id) = missing {
                return Err(format!("card {} refers to unknown id {id:?}", card.id));
            }
            if let Some(id) = card.artist.filter(|id| !artists.contains_key(id)) {
                return Err(format!("card {} refers to unknown artist {id}", card.id));
            }
        }
        Ok(Self {
            name: file.app_version,
            key,
            languages: file.languages,
            cards: file.cards,
            abilities,
            talents,
            tox,
            keywords: file.keywords,
            effects: file.effects,
            artists,
            ranks: file.ranks,
            elements: file.elements,
            rarities: file.rarities,
            game_types: file.game_types,
            enums: file.enums,
        })
    }
}

impl Catalog {
    pub fn load(dir: &Path) -> Result<Self, String> {
        let read =
            std::fs::read_dir(dir).map_err(|e| format!("cannot read catalog directory {}: {e}", dir.display()))?;
        let mut versions = Vec::new();
        for entry in read {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.extension().is_some_and(|e| e == "json") {
                versions
                    .push(Version::load(&path).map_err(|e| format!("invalid catalog file {}: {e}", path.display()))?);
            }
        }
        if versions.is_empty() {
            return Err(format!("no catalog file in {}", dir.display()));
        }
        versions.sort_by(|a, b| b.key.cmp(&a.key));
        if let Some(w) = versions.windows(2).find(|w| w[0].key == w[1].key) {
            return Err(format!("duplicate catalog version {}", w[0].name));
        }
        Ok(Self { versions })
    }

    pub fn version_names(&self) -> impl Iterator<Item = &str> {
        self.versions.iter().map(|v| v.name.as_str())
    }

    pub fn latest(&self) -> &str {
        &self.versions[0].name
    }

    #[cfg(feature = "seed")]
    pub fn card_ids(&self) -> Vec<Uuid> {
        self.versions[0].cards.iter().map(|c| c.id).collect()
    }

    #[cfg(feature = "seed")]
    pub fn tox_ids(&self) -> Vec<Uuid> {
        self.versions[0].tox.keys().filter_map(|id| id.parse().ok()).collect()
    }

    pub fn previous(&self, version: &str) -> Option<&str> {
        let key = &self.versions.iter().find(|v| v.name == version)?.key;
        self.versions.iter().find(|v| v.key < *key).map(|v| v.name.as_str())
    }

    pub fn element(&self, id: &Uuid) -> Option<&str> {
        self.versions.iter().find_map(|v| v.cards.iter().find(|c| c.id == *id)).map(|c| c.element.as_str())
    }

    pub fn knows_card(&self, app_version: &str, id: &Uuid) -> bool {
        let has = |v: &Version| v.cards.iter().any(|c| c.id == *id);
        has(&self.versions[0]) || self.versions.iter().find(|v| v.name == app_version).is_some_and(has)
    }

    pub fn talent_slots(&self, card: &Uuid) -> (Vec<String>, Vec<i16>, Vec<String>) {
        let mut out = (Vec::new(), Vec::new(), Vec::new());
        for v in &self.versions {
            for c in v.cards.iter().filter(|c| c.id == *card) {
                for (n, id) in (1..).zip(&c.talents) {
                    out.0.push(v.name.clone());
                    out.1.push(n);
                    out.2.push(id.clone());
                }
            }
        }
        out
    }

    pub fn talent_name(&self, id: &str, lang: &str) -> Option<String> {
        self.versions.iter().find_map(|v| v.talents.get(id)).and_then(|e| tr(&e.name, lang))
    }

    pub fn tox(&self, id: &str, lang: &str) -> Option<FaceTox> {
        self.versions.iter().find_map(|v| v.tox.get(id)).map(|e| face_tox(e, lang))
    }

    fn select<'a>(&'a self, q: &'a CardQuery) -> Result<(&'a Version, &'a str), ApiError> {
        let version = match &q.version {
            None => &self.versions[0],
            Some(name) => self
                .versions
                .iter()
                .find(|v| &v.name == name)
                .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "unknown catalog version"))?,
        };
        let lang = q.lang.as_deref().unwrap_or("en");
        if !version.languages.iter().any(|l| l == lang) {
            return Err(ApiError::new(StatusCode::BAD_REQUEST, "unknown language"));
        }
        Ok((version, lang))
    }
}

fn tr(text: &Text, lang: &str) -> Option<String> {
    let text = text.as_ref()?;
    let get = |l: &str| text.get(l).cloned().flatten().filter(|t| !t.is_empty());
    get(lang).or_else(|| get("en"))
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CardQuery {
    pub version: Option<String>,
    pub lang: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct CatalogVersion {
    pub version: String,
    pub languages: Vec<String>,
}

#[derive(Clone, Default, Deserialize, Serialize, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct Enums {
    pub trigger: BTreeMap<String, String>,
    pub target: BTreeMap<String, String>,
    pub display_type: BTreeMap<String, String>,
    pub effect: BTreeMap<String, String>,
}

#[derive(Serialize, ToSchema)]
pub struct Rank {
    pub id: String,
    pub threshold: i64,
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct Ordered {
    pub id: String,
    pub order: i64,
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct GameType {
    pub id: String,
    pub value: i64,
    pub name: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CatalogNames {
    pub version: String,
    pub ranks: Vec<Rank>,
    pub elements: Vec<Ordered>,
    pub rarities: Vec<Ordered>,
    pub game_types: Vec<GameType>,
    pub enums: Enums,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct AbilityRef {
    pub id: String,
    pub cooldown: i64,
}

#[derive(Serialize, ToSchema)]
pub struct Resistances {
    pub fire: i64,
    pub water: i64,
    pub earth: i64,
    pub nature: i64,
    pub light: i64,
    pub shadow: i64,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardBase {
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub element: String,
    pub rarity: String,
    pub hp: i64,
    pub gold_income: i64,
    pub resistances: Resistances,
    pub order: i64,
    pub face: CardFace,
}

#[derive(Serialize, ToSchema)]
pub struct CardFace {
    pub abilities: Vec<FaceAbility>,
    pub tox: Vec<FaceTox>,
}

#[derive(Serialize, ToSchema)]
pub struct FaceAbility {
    pub name: String,
    pub cooldown: i64,
    pub element: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct FaceTox {
    pub id: String,
    pub name: String,
    pub rarity: Option<String>,
    pub image: bool,
}

#[derive(Serialize, ToSchema)]
pub struct CardSummary {
    #[serde(flatten)]
    pub card: CardBase,
    pub abilities: Vec<AbilityRef>,
    pub tox: Vec<String>,
    pub talents: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct Entry {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    #[schema(value_type = Object)]
    pub data: Map<String, Value>,
}

#[derive(Serialize, ToSchema)]
pub struct CardAbility {
    pub cooldown: i64,
    pub ability: Entry,
}

#[derive(Serialize, ToSchema)]
pub struct CardDetail {
    #[serde(flatten)]
    pub card: CardBase,
    pub abilities: Vec<CardAbility>,
    pub tox: Vec<Entry>,
    pub talents: Vec<Entry>,
    pub artist: Option<Artist>,
}

#[derive(Serialize, ToSchema)]
pub struct Artist {
    pub name: String,
    pub links: Vec<ArtistLink>,
}

#[derive(Serialize, ToSchema)]
pub struct ArtistCards {
    #[serde(flatten)]
    pub artist: Artist,
    pub cards: Vec<CardBase>,
}

#[derive(Serialize, ToSchema)]
pub struct ArtistLink {
    pub kind: LinkKind,
    pub url: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum LinkKind {
    Instagram,
    X,
    Bluesky,
    Twitch,
    Artstation,
    Linktree,
    Website,
}

pub fn artist_link(url: &str) -> Option<ArtistLink> {
    if !url.starts_with("https://") {
        return None;
    }
    let parsed = url::Url::parse(url).ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    let kind = [
        ("instagram.com", LinkKind::Instagram),
        ("x.com", LinkKind::X),
        ("twitter.com", LinkKind::X),
        ("bsky.app", LinkKind::Bluesky),
        ("twitch.tv", LinkKind::Twitch),
        ("artstation.com", LinkKind::Artstation),
        ("linktr.ee", LinkKind::Linktree),
    ]
    .into_iter()
    .find(|(domain, _)| host.strip_suffix(domain).is_some_and(|rest| rest.is_empty() || rest.ends_with('.')))
    .map_or(LinkKind::Website, |(_, kind)| kind);
    Some(ArtistLink { kind, url: url.into() })
}

fn artist(a: &RawArtist) -> Option<Artist> {
    let name = a.name.trim();
    (!name.is_empty())
        .then(|| Artist { name: name.into(), links: a.link.as_deref().and_then(artist_link).into_iter().collect() })
}

fn field(e: &RawEntry, key: &str) -> Option<String> {
    e.data.get(key).and_then(Value::as_str).map(String::from)
}

fn base(v: &Version, c: &RawCard, lang: &str) -> CardBase {
    CardBase {
        id: c.id,
        name: tr(&c.name, lang),
        description: tr(&c.description, lang),
        element: c.element.clone(),
        rarity: c.rarity.clone(),
        hp: c.hp,
        gold_income: c.gold_income,
        resistances: Resistances {
            fire: c.fire_resistance,
            water: c.water_resistance,
            earth: c.earth_resistance,
            nature: c.nature_resistance,
            light: c.light_resistance,
            shadow: c.shadow_resistance,
        },
        order: c.order,
        face: CardFace {
            abilities: c
                .active_abilities
                .iter()
                .map(|a| {
                    let e = &v.abilities[&a.id];
                    FaceAbility {
                        name: tr(&e.name, lang).unwrap_or_else(|| e.id.clone()),
                        cooldown: a.cooldown,
                        element: field(e, "team"),
                    }
                })
                .collect(),
            tox: c.passive_abilities.iter().map(|id| face_tox(&v.tox[id], lang)).collect(),
        },
    }
}

fn face_tox(e: &RawEntry, lang: &str) -> FaceTox {
    FaceTox {
        id: e.id.clone(),
        name: tr(&e.name, lang).or_else(|| field(e, "title")).unwrap_or_else(|| e.id.clone()),
        rarity: field(e, "rarity"),
        image: field(e, "modificatorImage").is_some_and(|s| !s.is_empty()),
    }
}

fn entry(e: &RawEntry, lang: &str) -> Entry {
    Entry { id: e.id.clone(), name: tr(&e.name, lang), description: tr(&e.description, lang), data: e.data.clone() }
}

#[derive(Serialize, ToSchema)]
pub struct Term {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct GlossaryEntry {
    #[serde(flatten)]
    pub entry: Entry,
    pub cards: Vec<Uuid>,
}

#[derive(Serialize, ToSchema)]
pub struct Glossary {
    pub keywords: Vec<Term>,
    pub effects: Vec<Term>,
    pub abilities: Vec<GlossaryEntry>,
    pub talents: Vec<GlossaryEntry>,
    pub tox: Vec<GlossaryEntry>,
}

fn terms(list: &[RawEntry], lang: &str) -> Vec<Term> {
    list.iter()
        .filter_map(|e| {
            Some(Term { id: e.id.clone(), name: tr(&e.name, lang)?, description: tr(&e.description, lang) })
        })
        .collect()
}

fn used(
    list: &HashMap<String, RawEntry>,
    cards: &[&RawCard],
    lang: &str,
    has: impl Fn(&RawCard, &str) -> bool,
) -> Vec<GlossaryEntry> {
    let mut out: Vec<_> = list
        .values()
        .filter_map(|e| {
            let ids: Vec<Uuid> = cards.iter().filter(|c| has(c, &e.id)).map(|c| c.id).collect();
            let entry = entry(e, lang);
            (!ids.is_empty() && entry.name.is_some()).then_some(GlossaryEntry { entry, cards: ids })
        })
        .collect();
    out.sort_by(|a, b| a.entry.id.cmp(&b.entry.id));
    out
}

#[utoipa::path(get, path = "/api/glossary", tag = "catalog", params(CardQuery),
    responses((status = 200, body = Glossary, description = "Keywords, effects, and the abilities, talents and Tox \
        that cards use. cards is sorted by card order. Entries without a name are left out."),
        (status = 400, body = crate::ErrorBody), (status = 404, body = crate::ErrorBody)))]
pub async fn glossary(State(s): State<AppState>, Query(q): Query<CardQuery>) -> Result<Response, ApiError> {
    let (v, lang) = s.catalog.select(&q)?;
    let mut cards: Vec<&RawCard> = v.cards.iter().collect();
    cards.sort_by_key(|c| c.order);
    Ok(public(Glossary {
        keywords: terms(&v.keywords, lang),
        effects: terms(&v.effects, lang),
        abilities: used(&v.abilities, &cards, lang, |c, id| c.active_abilities.iter().any(|a| a.id == id)),
        talents: used(&v.talents, &cards, lang, |c, id| c.talents.iter().any(|t| t == id)),
        tox: used(&v.tox, &cards, lang, |c, id| c.passive_abilities.iter().any(|t| t == id)),
    }))
}

#[utoipa::path(get, path = "/api/glossary/keywords", tag = "catalog", params(CardQuery),
    responses((status = 200, body = Vec<Term>), (status = 400, body = crate::ErrorBody), (status = 404, body = crate::ErrorBody)))]
pub async fn glossary_keywords(State(s): State<AppState>, Query(q): Query<CardQuery>) -> Result<Response, ApiError> {
    let (v, lang) = s.catalog.select(&q)?;
    Ok(public(terms(&v.keywords, lang)))
}

#[utoipa::path(get, path = "/api/catalog/versions", tag = "catalog",
    responses((status = 200, body = Vec<CatalogVersion>, description = "Newest version first")))]
pub async fn versions(State(s): State<AppState>) -> Response {
    let list: Vec<_> = s
        .catalog
        .versions
        .iter()
        .map(|v| CatalogVersion { version: v.name.clone(), languages: v.languages.clone() })
        .collect();
    public(list)
}

#[utoipa::path(get, path = "/api/catalog", tag = "catalog", params(CardQuery),
    responses((status = 200, body = CatalogNames, description = "Names of the ranks (sorted by threshold), elements and \
        rarities (sorted by order) and game types (sorted by value). enums maps each integer of the ability fields \
        trigger, target and displayType and of the Tox field effect to its game name. An older catalog version gives \
        empty lists and maps."),
        (status = 400, body = crate::ErrorBody), (status = 404, body = crate::ErrorBody)))]
pub async fn names(State(s): State<AppState>, Query(q): Query<CardQuery>) -> Result<Response, ApiError> {
    let (v, lang) = s.catalog.select(&q)?;
    let name = |id: &String, text: &Text| tr(text, lang).unwrap_or_else(|| id.clone());
    let ordered = |list: &[RawOrdered]| {
        list.iter().map(|e| Ordered { id: e.id.clone(), order: e.order, name: name(&e.id, &e.name) }).collect()
    };
    Ok(public(CatalogNames {
        version: v.name.clone(),
        ranks: v
            .ranks
            .iter()
            .map(|r| Rank { id: r.id.clone(), threshold: r.threshold, name: name(&r.id, &r.name) })
            .collect(),
        elements: ordered(&v.elements),
        rarities: ordered(&v.rarities),
        game_types: v
            .game_types
            .iter()
            .map(|g| GameType { id: g.id.clone(), value: g.value, name: name(&g.id, &g.name) })
            .collect(),
        enums: v.enums.clone(),
    }))
}

#[utoipa::path(get, path = "/api/cards", tag = "catalog", params(CardQuery),
    responses((status = 200, body = Vec<CardSummary>), (status = 400, body = crate::ErrorBody), (status = 404, body = crate::ErrorBody)))]
pub async fn list_cards(State(s): State<AppState>, Query(q): Query<CardQuery>) -> Result<Response, ApiError> {
    let (v, lang) = s.catalog.select(&q)?;
    let cards: Vec<_> = v
        .cards
        .iter()
        .map(|c| CardSummary {
            card: base(v, c, lang),
            abilities: c
                .active_abilities
                .iter()
                .map(|a| AbilityRef { id: a.id.clone(), cooldown: a.cooldown })
                .collect(),
            tox: c.passive_abilities.clone(),
            talents: c.talents.clone(),
        })
        .collect();
    Ok(public(cards))
}

#[utoipa::path(get, path = "/api/cards/{id}", tag = "catalog",
    params(("id" = Uuid, Path, description = "Card UUID"), CardQuery),
    responses((status = 200, body = CardDetail), (status = 400, body = crate::ErrorBody), (status = 404, body = crate::ErrorBody)))]
pub async fn get_card(
    State(s): State<AppState>,
    UrlPath(id): UrlPath<Uuid>,
    Query(q): Query<CardQuery>,
) -> Result<Response, ApiError> {
    let (v, lang) = s.catalog.select(&q)?;
    let c = v.cards.iter().find(|c| c.id == id).ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "unknown card"))?;
    Ok(public(CardDetail {
        card: base(v, c, lang),
        abilities: c
            .active_abilities
            .iter()
            .map(|a| CardAbility { cooldown: a.cooldown, ability: entry(&v.abilities[&a.id], lang) })
            .collect(),
        tox: c.passive_abilities.iter().map(|id| entry(&v.tox[id], lang)).collect(),
        talents: c.talents.iter().map(|id| entry(&v.talents[id], lang)).collect(),
        artist: c.artist.and_then(|id| artist(&v.artists[&id])),
    }))
}

#[utoipa::path(get, path = "/api/artists", tag = "catalog", params(CardQuery),
    responses((status = 200, body = Vec<ArtistCards>, description = "Artists with 1 or more cards, sorted by name \
        without case. cards is sorted by card order."),
        (status = 400, body = crate::ErrorBody), (status = 404, body = crate::ErrorBody)))]
pub async fn artists(State(s): State<AppState>, Query(q): Query<CardQuery>) -> Result<Response, ApiError> {
    let (v, lang) = s.catalog.select(&q)?;
    let mut cards: Vec<&RawCard> = v.cards.iter().collect();
    cards.sort_by_key(|c| c.order);
    let mut list: Vec<_> = v
        .artists
        .values()
        .filter_map(|a| {
            let artist = artist(a)?;
            let cards: Vec<_> = cards.iter().filter(|c| c.artist == Some(a.id)).map(|c| base(v, c, lang)).collect();
            (!cards.is_empty()).then_some(ArtistCards { artist, cards })
        })
        .collect();
    list.sort_by_cached_key(|a| (a.artist.name.to_lowercase(), a.cards[0].id));
    Ok(public(list))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/catalog", get(names))
        .route("/api/catalog/versions", get(versions))
        .route("/api/cards", get(list_cards))
        .route("/api/cards/{id}", get(get_card))
        .route("/api/artists", get(artists))
        .route("/api/glossary", get(glossary))
        .route("/api/glossary/keywords", get(glossary_keywords))
}
