#[macro_use] extern crate rocket;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{self, Ws, Client};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::{Error, Surreal};

#[get("/")]
fn index() -> &'static str {
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
