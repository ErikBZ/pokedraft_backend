use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

pub mod draft;
pub mod pokemon;

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub id: Thing,
}
