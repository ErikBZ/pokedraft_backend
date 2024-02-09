use crate::models::draft::{DraftRules, DraftSession, DraftSessionCreateForm};
use crate::models::pokemon::{Pokemon, PokemonDraftSet};
use crate::models::Record;
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
            return None;
        }
    };

    match pokemon {
        Some(p) => Some(Json(p)),
        None => None,
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
        format!("SELECT name,id,->contains.out.dex_id as pokemon.Ids FROM pokemon_draft_set:{id};")
    } else {
        format!("SELECT name,id,->contains.out.* as pokemon.Stats FROM pokemon_draft_set:{id};")
    };

    match run_query(query, db).await {
        Some(p) => Some(Json(p)),
        None => None,
    }
}

#[get("/draft_rules/get/<id>")]
pub async fn get_draft_rules(id: &str, db: &State<Surreal<Client>>) -> Option<Json<DraftRules>> {
    let rules: Option<DraftRules> = match db.select(("draft_rules", id)).await {
        Ok(p) => p,
        Err(e) => {
            println!("{}", e);
            None
        }
    };

    match rules {
        Some(r) => Some(Json(r)),
        None => None,
    }
}

#[get("/draft_rules/get")]
pub async fn list_draft_rules(db: &State<Surreal<Client>>) -> Json<Vec<DraftRules>> {
    let draft_sets = match db.select("draft_rules").await {
        Ok(p) => p,
        Err(e) => {
            println!("{}", e);
            Vec::new()
        }
    };

    Json(draft_sets)
}

#[post("/draft_rules/create", format = "application/json", data = "<dr_form>")]
pub async fn create_draft_rules(
    dr_form: Json<DraftRules>,
    db: &State<Surreal<Client>>,
) -> Option<String> {
    // should you even do this?
    let draft_rules: DraftRules = dr_form.0;

    let result: Vec<Record> = match db.create("draft_rules").content(draft_rules).await {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            return None;
        }
    };

    let record = if result.len() > 0 {
        format!("{{\"id\": \"{}\"}}", result[0].id)
    } else {
        "{\"message\": \"Could not create Draft Rule\"}".into()
    };

    Some(record)
}

#[get("/draft_session/get/<id>")]
pub async fn get_draft_session(
    id: &str,
    db: &State<Surreal<Client>>,
) -> Option<Json<DraftSession>> {
    let session: Option<DraftSession> = match db.select(("draft_session", id)).await {
        Ok(p) => p,
        Err(e) => {
            println!("{}", e);
            None
        }
    };

    match session {
        Some(ds) => Some(Json(ds)),
        None => None,
    }
}

#[post(
    "/draft_session/create",
    format = "application/json",
    data = "<session_form>"
)]
pub async fn create_draft_session(
    session_form: Json<DraftSessionCreateForm>,
    db: &State<Surreal<Client>>,
) -> Option<String> {
    // should you even do this?
    let session_form: DraftSessionCreateForm = session_form.0;

    let rules: DraftRules = match db.select(("draft_rules", &session_form.draft_rules)).await {
        Ok(p) => match p {
            Some(r) => r,
            None => return None,
        },
        Err(e) => {
            println!("{}", e);
            return None;
        }
    };

    let draft_session = DraftSession::from(session_form, rules);
    let result: Vec<Record> = match db.create("draft_session").content(draft_session).await {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            return None;
        }
    };

    let record = if result.len() > 0 {
        format!("{{\"id\": \"{}\"}}", result[0].id)
    } else {
        "{\"message\": \"Could not create Draft Rule\"}".into()
    };

    Some(record)
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
            }
        },
        Err(_) => None,
    };

    resp
}
