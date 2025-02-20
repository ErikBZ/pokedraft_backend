#[macro_use] extern crate rocket;
extern crate surrealdb;

mod api;
mod models;
use api::{draft_session, pokemon, draft_set, draft_rules, CORS};

use surrealdb::Surreal;
use surrealdb::opt::auth::{Root, Database};
use surrealdb::engine::remote::ws::{Ws, Client};

use serde::Deserialize;

#[derive(Deserialize)]
struct DBConfig {
    surreal_addr: String,
    surreal_username: String,
    surreal_password: String,
    surreal_namespace: String,
    surreal_db_name: String,
    surreal_db_user_type: DBUserType
}

#[derive(Deserialize)]
enum DBUserType {
    Root,
    Database
}

async fn init_db(conf: DBConfig) -> Surreal<Client> {
    let db = match Surreal::new::<Ws>(conf.surreal_addr).await {
        Ok(f) => f,
        Err(e) => panic!("Unable to start connection to DB: {e}"),
    };

    match conf.surreal_db_user_type {
        DBUserType::Root => {
            db.signin(Root { username: &conf.surreal_username, password: &conf.surreal_password }).await.expect("Unable to log in with Root User");
            db.use_ns(conf.surreal_namespace).use_db(conf.surreal_db_name).await.expect("Unable to start namespace or database connection");
        },
        DBUserType::Database => {
            db.signin(Database {
                username: &conf.surreal_username,
                password: &conf.surreal_password,
                namespace: &conf.surreal_namespace,
                database: &conf.surreal_db_name
            }).await.expect("Unable to log in with Database User");
        }
    }

    db
}

#[launch]
async fn rocket() -> _ {
    let rocket = rocket::build();
    let figment = rocket.figment();

    let config: DBConfig = figment.extract().expect("Unable to read surreal db configuration");
    let db = init_db(config).await;

    rocket.manage(db)
        .mount("/api/v1", routes![pokemon::get])
        .mount("/api/v1", routes![pokemon::list])
        .mount("/api/v1", routes![draft_set::get_pokemon_draft_set])
        .mount("/api/v1", routes![draft_set::list_pokemon_draft_set])
        .mount("/api/v1", routes![draft_rules::get_draft_rules])
        .mount("/api/v1", routes![draft_rules::list_draft_rules])
        .mount("/api/v1", routes![draft_rules::create_draft_rules])
        .mount("/api/v1", routes![draft_session::get_draft_session])
        .mount("/api/v1", routes![draft_session::create_draft_session])
        .mount("/api/v1", routes![draft_session::option_draft_session])
        .mount("/api/v1", routes![draft_session::update_draft_session])
        .mount("/api/v1", routes![draft_session::create_user])
        .mount("/api/v1", routes![draft_session::option_create_user])
        .mount("/api/v1", routes![draft_session::select_pokemon])
        .mount("/api/v1", routes![draft_session::option_select_pokemon])
        .mount("/api/v1", routes![draft_session::toggle_ready])
        .mount("/api/v1", routes![draft_session::option_toggle_ready])
        .mount("/api/v1", routes![draft_session::start])
        .mount("/api/v1", routes![draft_session::option_start])
        .attach(CORS)
}
