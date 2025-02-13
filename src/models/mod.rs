use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use uuid::Uuid;

pub mod draft;
pub mod pokemon;

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub id: RecordId,
}

pub fn hash_uuid(uuid: &Uuid) -> i64 {
    let mut hasher = DefaultHasher::new();
    uuid.hash(&mut hasher);
    hasher.finish() as i64
}
