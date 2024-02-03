use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[warn(dead_code)]
pub enum DraftPhase {
    PICK,
    BAN,
}

#[derive(Debug, Serialize, Deserialize)]
#[warn(dead_code)]
pub enum TurnType {
    ROUNDROBIN,
    SNAKE,
}
