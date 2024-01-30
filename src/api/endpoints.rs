use rocket::{State};
use rocket::serde::json::Json;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use crate::models::pokemon::Pokemon;

// TODO Refactor to work with `select` function
#[get("/pokemon/get/<id>")]
pub async fn get_pokemon(id: i8, db: &State<Surreal<Client>>) -> Option<Json<Pokemon>> {
    let pokemon: Option<Pokemon> = match db
            .query("Select * FROM pokemon WHERE dex_id = $dex_id")
            .bind(("dex_id", id)).await {
        Ok(mut r) => {
            match r.take(0) {
                Ok(p) => p,
                Err(e) => {
                    println!("{}", e);
                    return None
                }
            }
        }
        Err(e) => {
            println!("{}", e);
            return None
        }
    };

    match pokemon {
        Some(p) => Some(Json(p)),
        None => None
    }
}

#[get("/pokemon/get")]
pub async fn list_pokemon(db: &State<Surreal<Client>>) -> Json<Vec<Pokemon>> {
    let pokemon: Vec<Pokemon> = match db.select("pokemon").await {
        Ok(p) => p,
        Err(e) => {
            println!("{}", e);
            Vec::new()
        }
    };

    Json(pokemon)
}
