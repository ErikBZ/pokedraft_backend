use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize)]
#[warn(dead_code)]
pub enum PokemonType {
    NORMAL,
    FIRE,
    WATER,
    ELECTRIC,
    GRASS,
    ICE,
    FIGHTING,
    POISON,
    GROUND,
    FLYING,
    PSYCHIC,
    BUG,
    ROCK,
    GHOST,
    DRAGON,
    DARK,
    STEEL,
    FAIRY,
    NONE,
}

// probably a better way than to make these all public
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Pokemon {
    pub dex_id: u16,
    pub name: String,
    pub type1: PokemonType,
    pub type2: Option<PokemonType>,
    pub evolves_from: u16,
    pub gen: u8,
    pub is_legendary: bool,
    pub is_mythic: bool,
}

// TODO: Option<Thing> is ugly, need to figure out a better way
// to return this as json
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PokemonDraftSet {
    id: Option<Thing>,
    name: String,
    pokemon: Option<PokemonResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub enum PokemonResponse {
    Ids(Vec<u32>),
    Stats(Vec<Pokemon>),
}
