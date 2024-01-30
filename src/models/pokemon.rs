use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
}

// probably a better way than to make these all public
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Pokemon{
    pub dex_id: u8,
    pub name: String,
    pub type1: PokemonType,
    pub type2: PokemonType,
    pub evolves_from: String,
    pub gen: u8,
    pub is_legendary: bool,
    pub is_mythic: bool,
}

impl Pokemon {
    pub fn from_id(id: i8) -> Pokemon{
        Pokemon {
            dex_id: 1,
            name: "Bulbasaur".into(),
            type1: PokemonType::GRASS,
            type2: PokemonType::POISON,
            evolves_from: "0".into(),
            gen: 1,
            is_legendary: false,
            is_mythic: false
        }
    }
}
