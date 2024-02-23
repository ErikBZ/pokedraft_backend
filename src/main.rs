#[macro_use] extern crate rocket;
extern crate surrealdb;
use api::CORS;
use surrealdb::engine::remote::ws::{Ws, Client};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
mod models;
mod api;
use api::endpoints;

async fn init_db() -> Surreal<Client> {
    let db = match Surreal::new::<Ws>("127.0.0.1:8000").await {
        Err(_) => panic!("Unable to start connection to DB"),
        Ok(f) => f,
    };

    match db.signin(Root {username: "root", password: "root"}).await {
        Err(_) => panic!("Unable to login to DB"),
        Ok(_) => (),
    }

    match db.use_ns("test").use_db("test").await {
        Err(_) => panic!("Unable to start namespace or database"),
        Ok(_) => (),
    }

    db
}

#[launch]
async fn rocket() -> _ {
    let db = init_db().await;

    rocket::build()
        .manage(db)
        .mount("/api/v1", routes![endpoints::get_pokemon])
        .mount("/api/v1", routes![endpoints::list_pokemon])
        .mount("/api/v1", routes![endpoints::get_pokemon_draft_set])
        .mount("/api/v1", routes![endpoints::list_pokemon_draft_set])
        .mount("/api/v1", routes![endpoints::get_draft_rules])
        .mount("/api/v1", routes![endpoints::list_draft_rules])
        .mount("/api/v1", routes![endpoints::create_draft_rules])
        .mount("/api/v1", routes![endpoints::get_draft_session])
        .mount("/api/v1", routes![endpoints::create_draft_session])
        .mount("/api/v1", routes![endpoints::create_user])
        .mount("/api/v1", routes![endpoints::select_pokemon])
        .attach(CORS)
}
