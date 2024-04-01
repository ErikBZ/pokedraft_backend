use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[warn(dead_code)]
pub enum DraftPhase {
    Pick,
    Ban,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
    pub min_num_players: u16,
    pub max_num_players: u16,
    pub selected_pokemon: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<Vec<DraftUser>>,
    draft_rules: DraftRules,
    draft_set: Option<String>,
    pub current_player: Option<Thing>,
    turn_ticker: u32,
    // TODO: Use enum here DraftState::{ACCEPTING_PLAYER, MIN_JOINED, MAX_JOINED, ONGOING, DONE}
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
            current_phase: rules.starting_phase,
            draft_rules: rules,
            draft_set: Some(form.draft_set),
            current_player: None,
            turn_ticker: 0,
            accepting_players: true,
        }
    }

    pub fn is_name_taken(&self, name: &str) -> bool {
        let players: &Vec<DraftUser> = match &self.players {
            Some(s) => s,
            None => return false,
        };
        let players_with_name = players.iter().filter(|&x| x.name == name).collect::<Vec<_>>();

        players_with_name.len() != 0
    }

    pub fn get_next_player_id(&self) -> (u32, Option<String>) {
        if let Some(players) = &self.players {
            let num_of_players = players.len() as u32;
            let x = (self.turn_ticker + 1) % num_of_players;
            let round = (self.turn_ticker + 1) / num_of_players;

            let next_player_i = if round % 2 == 0 || self.draft_rules.turn_type == TurnType::RoundRobin {
                x
            } else {
                num_of_players - (x + 1)
            } as usize;

            if let Some(player) = players.get(next_player_i) {
                if let Some(t) = &player.id {
                    return (self.turn_ticker + 1, Some(format!("{}", t.id)));
                }
            }
        } 

        (self.turn_ticker, None)
    }

    // TODO: Used enum Pick(u32) and Ban(u32) to track how long have for the round
    pub fn get_next_phase(&self) -> DraftPhase {
        let picks_per_round = self.draft_rules.picks_per_round as u32; 
        let bans_per_round = self.draft_rules.bans_per_round as u32; 
        let num_of_players = self.num_of_players();
        let round = (self.turn_ticker + 1) / num_of_players;
        let full_cycle = bans_per_round + picks_per_round;
        let normalized_round = round % full_cycle;

        println!("PPR: {picks_per_round}, BPR: {bans_per_round}, NoP: {num_of_players}, Round: {round}, NR: {normalized_round}, FC: {full_cycle}");

        match self.draft_rules.starting_phase {
            DraftPhase::Ban => {
                let some_bool = (normalized_round + bans_per_round) < full_cycle;
                println!("{some_bool}");
                if (normalized_round + bans_per_round) < full_cycle {
                    DraftPhase::Ban
                } else {
                    DraftPhase::Pick
                }
            },
            DraftPhase::Pick => {
                if (normalized_round + picks_per_round) < full_cycle {
                    DraftPhase::Pick
                } else {
                    DraftPhase::Ban
                }
            }
        }
    }

    pub fn is_current_player(&self, id: &Thing) -> bool {
        if let Some(ref t) = self.current_player {
            return t == id
        }
        false
    }

    pub fn get_current_player_name(&self) -> Option<String> {
        todo!()
    }

    pub fn draft_has_started(&self) -> bool {
        !self.accepting_players
    }

    pub fn slots_available(&self) -> bool {
        match &self.players {
            Some(p) => (p.len() as u16) < self.max_num_players && self.accepting_players,
            None => false,
        }
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
    pub id: Option<Thing>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session: Option<Thing>,
    pub selected_pokemon: Vec<u32>,
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
            selected_pokemon: Vec::new(),
            key_hash: key,
            order_in_session: order,
        }
    }

    pub fn check_key_hash(&self, key: i64) -> bool {
        self.key_hash == key
    }

    pub fn select_pokemon(&mut self, pk: u32) {
        self.selected_pokemon.push(pk);
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
