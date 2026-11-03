use std::{collections::HashMap, path::Path};

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
        }
        Ok(Self { name: file.app_version, key, languages: file.languages, cards: file.cards, abilities, talents, tox })
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

    pub fn latest(&self) -> &str {
        &self.versions[0].name
    }

    pub fn knows_card(&self, app_version: &str, id: &Uuid) -> bool {
        let has = |v: &Version| v.cards.iter().any(|c| c.id == *id);
        has(&self.versions[0]) || self.versions.iter().find(|v| v.name == app_version).is_some_and(has)
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
    let get = |l: &str| text.get(l).cloned().flatten();
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
}

fn base(c: &RawCard, lang: &str) -> CardBase {
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
    }
}

fn entry(e: &RawEntry, lang: &str) -> Entry {
    Entry { id: e.id.clone(), name: tr(&e.name, lang), description: tr(&e.description, lang), data: e.data.clone() }
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

#[utoipa::path(get, path = "/api/cards", tag = "catalog", params(CardQuery),
    responses((status = 200, body = Vec<CardSummary>), (status = 400, body = crate::ErrorBody), (status = 404, body = crate::ErrorBody)))]
pub async fn list_cards(State(s): State<AppState>, Query(q): Query<CardQuery>) -> Result<Response, ApiError> {
    let (v, lang) = s.catalog.select(&q)?;
    let cards: Vec<_> = v
        .cards
        .iter()
        .map(|c| CardSummary {
            card: base(c, lang),
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
        card: base(c, lang),
        abilities: c
            .active_abilities
            .iter()
            .map(|a| CardAbility { cooldown: a.cooldown, ability: entry(&v.abilities[&a.id], lang) })
            .collect(),
        tox: c.passive_abilities.iter().map(|id| entry(&v.tox[id], lang)).collect(),
        talents: c.talents.iter().map(|id| entry(&v.talents[id], lang)).collect(),
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/catalog/versions", get(versions))
        .route("/api/cards", get(list_cards))
        .route("/api/cards/{id}", get(get_card))
}
