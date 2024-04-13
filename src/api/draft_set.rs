use crate::api::utils::run_query;
use crate::models::pokemon::PokemonDraftSet;

use rocket::State;
use rocket::serde::json::Json;

use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;

#[get("/draft_set")]
pub async fn list_pokemon_draft_set(db: &State<Surreal<Client>>) -> Json<Vec<PokemonDraftSet>> {
    let draft_sets = match db.select("pokemon_draft_set").await {
        Ok(p) => p,
        Err(e) => {
            println!("{}", e);
            Vec::new()
        }
    };

    Json(draft_sets)
}

#[get("/draft_set/<id>?<detailed>")]
pub async fn get_pokemon_draft_set(
    id: &str,
    detailed: bool,
    db: &State<Surreal<Client>>,
) -> Option<Json<PokemonDraftSet>> {
    let query: String = if !detailed {
        format!("SELECT name,id,array::sort(->contains.out.dex_id, asc) as pokemon.Ids FROM pokemon_draft_set:{id};")
    } else {
        format!("SELECT name,id,array::sort(->contains.out.*, asc) as pokemon.Stats FROM pokemon_draft_set:{id};")
    };

    match run_query(query, db).await {
        Some(p) => Some(Json(p)),
        None => None,
    }
}


