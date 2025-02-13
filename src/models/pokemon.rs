use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

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
// TODO: Serializing theses fields looks a little gross in the frontend
// I should implement the serialiers myself to make em nicer
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Pokemon {
    pub dex_id: u32,
    pub id: Option<RecordId>,
    pub name: String,
    pub type1: PokemonType,
    pub type2: Option<PokemonType>,
    pub evolves_from: u16,
    pub gen: u8,
    pub is_legendary: bool,
    pub is_mythic: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PokemonDraftSet {
    id: Option<RecordId>,
    name: String,
    pokemon: Option<PokemonResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub enum PokemonResponse {
    Ids(Vec<u32>),
    Stats(Vec<Pokemon>),
}
