use crate::models::pokemon::{Pokemon, PokemonDraftSet};
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

#[get("/pokemon/get/<id>")]
pub async fn get_pokemon(id: u64, db: &State<Surreal<Client>>) -> Option<Json<Pokemon>> {
    let pokemon: Option<Pokemon> = match db.select(("pokemon", id)).await {
        Ok(p) => p,
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

#[get("/pokemon_draft_set/get")]
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

#[get("/pokemon_draft_set/get/<id>?<detailed>")]
pub async fn get_pokemon_draft_set(
    id: &str,
    detailed: bool,
    db: &State<Surreal<Client>>,
) -> Option<Json<PokemonDraftSet>> {
    let query: String = if !detailed {
        format!("SELECT name,id,->contains.out.dex_id as pokemon_ids FROM pokemon_draft_set:{id};")
    } else {
        format!("SELECT name,id,->contains.out.* as pokemon FROM pokemon_draft_set:{id};")
    };

    match run_query(query, db).await {
        Some(p) => Some(Json(p)),
        None => None,
    }
}

// TODO actually do something useful with those errors
async fn run_query<T>(query: String, db: &State<Surreal<Client>>) -> Option<T>
where
    for<'a> T: Deserialize<'a>,
{
    let resp: Option<T> = match db.query(query).await {
        Ok(mut r) => match r.take(0) {
            Ok(p) => p,
            Err(e) => {
                println!("{}", e);
                None
            },
        },
        Err(_) => None,
    };

    resp
}
