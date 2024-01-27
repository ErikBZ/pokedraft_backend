#[macro_use] extern crate rocket;
use std::string;

use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{self, Ws, Client};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::{Error, Surreal};
use rocket::{State};

#[derive(Debug, Serialize)]
enum PokemonType {
    NORMAL,
    FIRE,
    WATER,
    ELECTRIC,
    GRASS,
    ICE,
    FIGHTING,
    POISON,
    GROUND,
    FLYING,
    PSYCHIC,
    BUG,
    ROCK,
    GHOST,
    DRAGON,
    DARK,
    STEEL,
    FAIRY,
}

#[derive(Debug, Serialize)]
struct Pokemon <'a>{
    name: &'a str,
    type1: PokemonType,
    type2: PokemonType,
    evolves_from: &'a str,
    gen: u8,
    is_legendary: bool,
    is_mythic: bool,
}

#[derive(Debug, Deserialize)]
struct Record {
    id: Thing
}


#[get("/")]
async fn index(db: &State<Surreal<Client>>) -> &'static str {
    let created: Result<Vec<Record>, Error> = db.create("pokemon").content( Pokemon {
        name: "Bulbasaur",
        type1: PokemonType::GRASS,
        type2: PokemonType::POISON,
        evolves_from: "0",
        gen: 1,
        is_legendary: false,
        is_mythic: false
    }).await;

    "Hello, world!"
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

    // Connect to the server
    rocket::build()
        .manage(db)
        .mount("/", routes![index])
}
