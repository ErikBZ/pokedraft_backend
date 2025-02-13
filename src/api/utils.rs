use rocket::State;
use rocket::response::status::NotFound;

use serde::Deserialize;

use surrealdb::{Surreal, RecordId};
use surrealdb::engine::remote::ws::Client;

// TODO: Do someting useful with these errors
pub async fn run_query<T>(query: String, db: &State<Surreal<Client>>) -> Option<T>
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
        Err(e) => {
            println!("{}", e);
            None
        }
    };

    resp
}

pub async fn relate_objects(
    db: &State<Surreal<Client>>,
    obj_in: &RecordId,
    obj_out: &RecordId,
    relation: &str,
) -> Result<(), NotFound<String>> {
    let query = format!("RELATE {}->{}->{};", obj_in, relation, obj_out);
    let _ = db.query(query).await.map_err(|e| NotFound(e.to_string()));
    Ok(())
}


