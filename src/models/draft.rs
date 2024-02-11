use std::io::Cursor;

use rocket::{http::ContentType, response::Responder, Response};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use rocket::serde::json::serde_json;

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
    pub id: Option<Thing>,
    name: String,
    min_num_players: u16,
    max_num_players: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    banned_pokemon: Option<PokemonResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    players: Option<Vec<DraftUser>>,
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
            players: None,
            draft_rules: Some(form.draft_rules),
            draft_set: Some(form.draft_set),
            current_player: None,
            turn_ticker: 0,
            accepting_players: true,
            current_phase: rules.starting_phase,
        }
    }

    pub fn get_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        let players: &Vec<DraftUser> = match &self.players {
            Some(s) => s,
            None => {
                return names
            }
        };

        for user in players {
            // clonse for now, but maybe I shouldn't use String?
            names.push(user.name.clone());
        }

        names
    }

    pub fn slots_available(&self) -> bool {
        match &self.players {
            Some(p) => (p.len() as u16) < self.max_num_players && self.accepting_players,
            None => false
        }
    }

    pub fn num_of_players(&self) -> u16 {
        match &self.players {
            None => 0,
            Some(p) => p.len() as u16
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
    #[serde(skip_serializing_if = "Option::is_none")]
    session: Option<Thing>,
    pokemon_selected: Vec<u32>,
    // find some crypto hash
    key_hash: u64,
    order_in_session: u16,
}

impl DraftUser {
    pub fn new(name: String, key: u64, order: u16) -> DraftUser {
        DraftUser {
            id: None,
            name: name,
            session: None,
            pokemon_selected: Vec::new(),
            key_hash: key,
            order_in_session: order,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DraftUserReturnData {
    name: String,
    session_id: String,
    user_id: String,
    current_turn: bool,
    key: String,
}

impl DraftUserReturnData {
    pub fn new(name: String,
        session_id: String,
        user_id: String,
        current_turn: bool,
        key: String
    ) -> DraftUserReturnData {
        DraftUserReturnData {
            name: name,
            session_id: session_id,
            user_id: user_id,
            current_turn: current_turn,
            key: key,
        }
    }
}

// TODO take a look at this and understand it better
impl<'r> Responder<'r, 'r> for DraftUserReturnData {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        let response = serde_json::to_string(&self).unwrap();
        Response::build()
            .header(ContentType::JsonApi)
            .sized_body(response.len(), Cursor::new(response))
            .ok()

    }
}

// is there a way to not do this?
#[derive(Debug, Serialize, Deserialize)]
pub struct DraftUserForm {
    pub name: String,
}
