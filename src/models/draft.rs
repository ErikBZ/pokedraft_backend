use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
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
    pub max_num_players: u16,
    pub selected_pokemon: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<Vec<DraftUser>>,
    draft_rules: Option<String>,
    draft_set: Option<String>,
    pub current_player: u32,
    turn_ticker: u32,
    accepting_players: bool,
    pub current_phase: DraftPhase,
}

impl DraftSession {
    pub fn from(form: DraftSessionCreateForm, rules: DraftRules) -> DraftSession {
        DraftSession {
            id: None,
            name: form.name,
            min_num_players: form.min_num_players,
            max_num_players: form.max_num_players,
            selected_pokemon: Vec::new(),
            players: None,
            draft_rules: Some(form.draft_rules),
            draft_set: Some(form.draft_set),
            current_player: 0,
            turn_ticker: 0,
            accepting_players: true,
            current_phase: rules.starting_phase,
        }
    }

    pub fn get_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        let players: &Vec<DraftUser> = match &self.players {
            Some(s) => s,
            None => return names,
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
            None => false,
        }
    }

    pub fn draft_has_started(&self) -> bool {
        !self.accepting_players
    }

    pub fn is_current_player(&self, order: u32) -> bool {
        self.current_player == order
    }

    pub fn num_of_players(&self) -> u32 {
        match &self.players {
            None => 0,
            Some(p) => p.len() as u32,
        }
    }

    pub fn is_pokemon_chosen(&self, pk: &u32) -> bool {
        self.selected_pokemon.contains(pk)
    }

    pub fn choose_pokemon(&mut self, pk: u32) {
        self.selected_pokemon.push(pk)
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
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session: Option<Thing>,
    pub pokemon_selected: Vec<u32>,
    // find some crypto hash
    key_hash: i64,
    pub order_in_session: u32,
}

impl DraftUser {
    pub fn new(name: String, key: i64, order: u32) -> DraftUser {
        DraftUser {
            id: None,
            name: name,
            session: None,
            pokemon_selected: Vec::new(),
            key_hash: key,
            order_in_session: order,
        }
    }

    pub fn check_key_hash(&self, key: i64) -> bool {
        self.key_hash == key
    }

    pub fn select_pokemon(&mut self, pk: u32) {
        self.pokemon_selected.push(pk);
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
    pub fn new(
        name: String,
        session_id: String,
        user_id: String,
        current_turn: bool,
        key: String,
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

// is there a way to not do this?
#[derive(Debug, Serialize, Deserialize)]
pub struct DraftUserForm {
    pub name: String,
}
