use crate::models::pokemon::Pokemon;

use rocket::State;
use rocket::serde::json::Json;

use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;

#[get("/pokemon/get/<id>")]
// Why couldn't this be a u64?
pub async fn get(id: &str, db: &State<Surreal<Client>>) -> Option<Json<Pokemon>> {
    let pokemon: Option<Pokemon> = match db.select(("pokemon", id)).await {
        Ok(p) => p,
        Err(e) => {
            println!("{}", e);
            return None;
        }
    };

    match pokemon {
        Some(p) => Some(Json(p)),
        None => None,
    }
}

#[get("/pokemon/get")]
pub async fn list(db: &State<Surreal<Client>>) -> Json<Vec<Pokemon>> {
    let pokemon: Vec<Pokemon> = match db.select("pokemon").await {
        Ok(p) => p,
        Err(e) => {
            println!("{}", e);
            Vec::new()
        }
    };

    Json(pokemon)
}
