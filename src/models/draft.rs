use serde::{ser::SerializeStruct, Deserialize, Serialize};
use surrealdb::{sql::Id, RecordId};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum DraftState {
    Open,               // Starting value. Allows players to join
    Ready,              // All players have listed themselves as ready
    InProgress,         // No more players may join, Pick/Bans in progress
    Ended               // Pick/Bans are done. All pokemon have been chosen
}

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
    id: Option<RecordId>,
    name: String,
    picks_per_round: u16,
    bans_per_round: u16,
    max_pokemon: u16,
    starting_phase: DraftPhase,
    turn_type: TurnType,
}

impl Default for DraftRules {
    fn default() -> DraftRules {
        DraftRules{
            id: None,
            name: "".to_string(),
            picks_per_round: 1,
            bans_per_round: 1,
            max_pokemon: 1,
            starting_phase: DraftPhase::Ban,
            turn_type: TurnType::Snake,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DraftSession {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    name: String,
    pub min_num_players: u16,
    pub max_num_players: u16,
    pub selected_pokemon: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<Vec<DraftUser>>,
    draft_rules: DraftRules,
    draft_set: Option<String>,
    pub current_player: Option<RecordId>,
    turn_ticker: u32,
    // TODO: Use enum here DraftState::{ACCEPTING_PLAYER, MIN_JOINED, MAX_JOINED, ONGOING, DONE}
    accepting_players: bool,
    pub draft_state: DraftState,
    pub current_phase: DraftPhase,
}

// TODO: Impl Serialize

// TODO: Impl Deserialize

impl Default for DraftSession {
    fn default() -> DraftSession {
        DraftSession {
            id: None,
            name: "".to_string(),
            min_num_players: 4,
            max_num_players: 4,
            selected_pokemon: vec![],
            players: None,
            draft_rules: DraftRules {
                ..Default::default()
            },
            draft_set: None,
            current_player: None,
            turn_ticker: 0,
            accepting_players: false,
            draft_state: DraftState::Open,
            current_phase: DraftPhase::Ban,
        }
    }
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
            draft_state: DraftState::Open,
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

    pub fn get_next_player_id(&self) -> (u32, Option<RecordId>) {
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
                return (self.turn_ticker + 1, player.id.clone());
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

        match self.draft_rules.starting_phase {
            DraftPhase::Ban => {
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

    pub fn check_if_session_is_over(&self) -> bool {
        self.calculate_pk_num_floor() >= (self.draft_rules.max_pokemon as u32)
    }

    fn calculate_pk_num_floor(&self) -> u32 {
        let picks_per_round = self.draft_rules.picks_per_round as u32;
        let bans_per_round = self.draft_rules.bans_per_round as u32;
        let num_of_players = self.num_of_players();
        let turns_per_round = num_of_players * (picks_per_round + bans_per_round);
        let num_of_rounds = (self.turn_ticker + 1) / turns_per_round;
        let remaining_turns = (self.turn_ticker + 1) % turns_per_round;

        // Getting the minimum number of pokemon that all players have
        // once ALL players have at least draft_rules.max_pokemon then the sesion has ended
        let pokemon_selected = if self.draft_rules.starting_phase == DraftPhase::Pick {
            if remaining_turns > (picks_per_round * num_of_players) {
                picks_per_round
            } else {
                remaining_turns / (picks_per_round * num_of_players)
            }
        } else {
            if remaining_turns > (bans_per_round * num_of_players) {
                (remaining_turns - (bans_per_round * num_of_players)) / (picks_per_round * num_of_players)
            } else {
                0
            }
        };

        pokemon_selected + (num_of_rounds * picks_per_round)
    }

    pub fn is_current_player(&self, id: &RecordId) -> bool {
        if let Some(ref t) = self.current_player {
            return t == id
        }
        false
    }

    pub fn get_current_player_name(&self) -> Option<String> {
        if let Some(players) = &self.players {
            for player in players {
                if player.id == self.current_player {
                    return Some(player.name.clone());
                }
            }
        }
        None
    }

    pub fn draft_has_started(&self) -> bool {
        self.draft_state == DraftState::InProgress
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
    pub id: Option<RecordId>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session: Option<RecordId>,
    pub selected_pokemon: Vec<u32>,
    // find some crypto hash
    key_hash: i64,
    pub order_in_session: u32,
    pub ready: bool,
}

impl Default for DraftUser {
    fn default() -> DraftUser {
        DraftUser {
            id: None,
            name: "".to_string(),
            session: None,
            selected_pokemon: vec![],
            key_hash: 0,
            order_in_session: 0,
            ready: false
        }
    }
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
            ready: false
        }
    }

    pub fn check_key_hash(&self, key: i64) -> bool {
        self.key_hash == key
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DraftUserReturnData {
    name: String,
    session_id: String,
    user_id: RecordId,
    current_turn: bool,
    key: String,
}

impl DraftUserReturnData {
    pub fn new(
        name: String,
        session_id: String,
        user_id: RecordId,
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

#[cfg(test)]
mod test {
    use super::*;

    fn generate_players(size: u32) -> Vec<DraftUser>{
        let mut players: Vec<DraftUser> = vec![];
        for _i in 0..size {
            players.push(DraftUser {
                ..Default::default()
            })
        }
        players
    }

    #[test]
    fn test_pk_floor_base_rules_four() {
        let session = DraftSession {
            turn_ticker: 8,
            players: Some(generate_players(4)),
            draft_rules: DraftRules {
                picks_per_round: 1,
                bans_per_round: 1,
                max_pokemon: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(session.calculate_pk_num_floor(), 1)
    }

    #[test]
    fn test_pk_floor_base_rules_three() {
        let mut session = DraftSession {
            turn_ticker: 7,
            players: Some(generate_players(3)),
            draft_rules: DraftRules {
                picks_per_round: 2,
                bans_per_round: 2,
                max_pokemon: 5,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(session.calculate_pk_num_floor(), 0);
        session.turn_ticker = 25;
        assert_eq!(session.calculate_pk_num_floor(), 4)
    }

    #[test]
    fn test_pk_floor_base_rules_two() {
        let mut session = DraftSession {
            turn_ticker: 0,
            players: Some(generate_players(2)),
            draft_rules: DraftRules {
                picks_per_round: 1,
                bans_per_round: 1,
                max_pokemon: 3,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(session.calculate_pk_num_floor(), 0);
        session.turn_ticker = 9;
        assert_eq!(session.calculate_pk_num_floor(), 2);
        session.draft_rules.starting_phase = DraftPhase::Pick;
        assert_eq!(session.calculate_pk_num_floor(), 3);
        session.turn_ticker = 10;
        assert_eq!(session.calculate_pk_num_floor(), 3);
    }
}

