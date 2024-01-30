#[macro_use] extern crate rocket;
use std::string;

use rocket::response::status::Created;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{self, Ws, Client};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::{Error, Surreal};
use rocket::{State};
mod models;
use models::pokemon::{Pokemon, PokemonType};
mod api;
use api::endpoints;

#[derive(Debug, Deserialize)]
struct Record {
    id: Thing
}

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

    let created: Result<Vec<Record>, Error> = db.create("pokemon").content( Pokemon {
        dex_id: 1,
        name: "Bulbasaur".into(),
        type1: PokemonType::GRASS,
        type2: PokemonType::POISON,
        evolves_from: "0".into(),
        gen: 1,
        is_legendary: false,
        is_mythic: false
    }).await;

    match created {
        Ok(p) => {
            for x in &p {
                println!("{}", x.id)
            }
        },
        Err(_) => {},
    }

    // Connect to the server
    rocket::build()
        .manage(db)
        .mount("/api/v1", routes![endpoints::get_pokemon])
        .mount("/api/v1", routes![endpoints::list_pokemon])
}
