use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use super::pokemon::PokemonResponse;

#[derive(Debug, Serialize, Deserialize)]
#[warn(dead_code)]
pub enum DraftPhase {
    Pick,
    Ban,
}

#[derive(Debug, Serialize, Deserialize)]
#[warn(dead_code)]
pub enum TurnType {
    RoundRobin,
    Snake,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DraftRules {
    id: Option<Thing>,
    name: String,
    picks_per_round: u16,
    bans_per_round: u16,
    max_pokemon: u16,
    starting_phase: DraftPhase,
    turn_type: TurnType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DraftSession {
    id: Option<Thing>,
    name: String,
    min_num_players: u16,
    max_num_players: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    banned_pokemon: Option<PokemonResponse>,
    draft_rules: Option<String>,
    draft_set: Option<String>,
    current_player: Option<u32>,
    turn_ticker: u32,
    accepting_players: bool,
    current_phase: DraftPhase,
}

impl DraftSession {
    pub fn from(form: DraftSessionCreateForm, rules: DraftRules) -> DraftSession {
        DraftSession {
            id: None,
            name: form.name,
            min_num_players: form.min_num_players,
            max_num_players: form.max_num_players,
            banned_pokemon: None,
            draft_rules: Some(form.draft_rules),
            draft_set: Some(form.draft_set),
            current_player: None,
            turn_ticker: 0,
            accepting_players: true,
            current_phase: rules.starting_phase,
        }
    }
}

// Maybe just change this into a regular form?
#[derive(Debug, Serialize, Deserialize)]
pub struct DraftSessionCreateForm {
    pub name: String,
    draft_set: String,
    pub draft_rules: String,
    min_num_players: u16,
    max_num_players: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DraftUser {
    id: Option<Thing>,
    name: String,
    session: Option<Thing>,
    pokemon_selected: Vec<u32>,
    // find some crypto hash
    key_hash: Option<String>,
    order_in_session: u16,
}
