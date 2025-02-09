use crate::models::Record;
use crate::models::draft::DraftRules;

use rocket::State;
use rocket::serde::json::Json;

use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Client;

// TODO: Move these out and get draft rules based on name too?
#[get("/draft_rules/<id>")]
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

#[get("/draft_rules")]
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
        Ok(Some(r)) => r,
        Ok(None) => {
            return None;
        }
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


