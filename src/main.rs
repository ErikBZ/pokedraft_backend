#[macro_use] extern crate rocket;
extern crate surrealdb;
use api::CORS;
use surrealdb::engine::remote::ws::{Ws, Client};
use surrealdb::opt::auth::{Root, Database};
use surrealdb::Surreal;
mod models;
mod api;
use api::endpoints;
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
        .mount("/api/v1", routes![endpoints::get_pokemon])
        .mount("/api/v1", routes![endpoints::list_pokemon])
        .mount("/api/v1", routes![endpoints::get_pokemon_draft_set])
        .mount("/api/v1", routes![endpoints::list_pokemon_draft_set])
        .mount("/api/v1", routes![endpoints::get_draft_rules])
        .mount("/api/v1", routes![endpoints::list_draft_rules])
        .mount("/api/v1", routes![endpoints::create_draft_rules])
        .mount("/api/v1", routes![endpoints::get_draft_session])
        .mount("/api/v1", routes![endpoints::create_draft_session])
        .mount("/api/v1", routes![endpoints::option_draft_session])
        .mount("/api/v1", routes![endpoints::update_draft_session])
        .mount("/api/v1", routes![endpoints::create_user])
        .mount("/api/v1", routes![endpoints::option_create_user])
        .mount("/api/v1", routes![endpoints::select_pokemon])
        .mount("/api/v1", routes![endpoints::option_select_pokemon])
        .attach(CORS)
}
