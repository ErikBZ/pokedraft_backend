use crate::models::draft::{
    DraftPhase, DraftRules, DraftSession, DraftSessionCreateForm, DraftUser, DraftUserForm,
    DraftUserReturnData,
};
use crate::models::pokemon::{Pokemon, PokemonDraftSet, PokemonType};
use crate::models::{hash_uuid, Record};

use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::State;

use serde::{Deserialize, Serialize};

use surrealdb::engine::remote::ws::Client;
use surrealdb::sql::{Id, Thing};
use surrealdb::Surreal;
use uuid::Uuid;

const DRAFT_USER_RELATION: &str = "players";
const DRAFT_SESSION: &str = "draft_session";
const DRAFT_USER_TB: &str = "draft_user";

fn to_json_err(str: &str) -> String {
    return format!("{{\"message\": \"{}\"}}", str)
}

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

#[get("/draft_set")]
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

#[get("/draft_set/<id>?<detailed>")]
pub async fn get_pokemon_draft_set(
    id: &str,
    detailed: bool,
    db: &State<Surreal<Client>>,
) -> Option<Json<PokemonDraftSet>> {
    let query: String = if !detailed {
        format!("SELECT name,id,array::sort(->contains.out.dex_id, asc) as pokemon.Ids FROM pokemon_draft_set:{id};")
    } else {
        format!("SELECT name,id,array::sort(->contains.out.*, asc) as pokemon.Stats FROM pokemon_draft_set:{id};")
    };

    match run_query(query, db).await {
        Some(p) => Some(Json(p)),
        None => None,
    }
}

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
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            return None;
        }
    };

    let record = if result.len() > 0 {
        format!("{{\"id\": \"{}\"}}", result[0].id.id)
    } else {
        "{\"message\": \"Could not create Draft Rule\"}".into()
    };

    Some(record)
}

#[get("/draft_session/<id>")]
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

#[options("/draft_session/create")]
pub fn option_draft_session<'a>() -> &'a str {
    "Ok"
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
        format!("{{\"id\": \"{}\"}}", result[0].id.id)
    } else {
        "{\"message\": \"Could not create Draft Rule\"}".into()
    };

    Some(record)
}

#[get("/draft_session/<id>/update")]
pub async fn update_draft_session(
    id: &str,
    db: &State<Surreal<Client>>,
) -> Result<Json<UpdateDraftSessionResponse>, NotFound<String>> {
    let query =
        format!("SELECT *,(SELECT * from ->{DRAFT_USER_RELATION}.out ORDER BY order_in_session ASC) as players FROM draft_session:{id};");

    let session: DraftSession = match run_query(query, db).await {
        Some(s) => s,
        None => return Err(NotFound("Session not found".into())),
    };

    let resp = UpdateDraftSessionResponse::from(session);
    Ok(Json(resp))
}

#[options("/draft_session/<id>/create-user")]
pub fn option_create_user<'a>(id: &str) -> &'a str {
    "Ok"
}

#[post(
    "/draft_session/<id>/create-user",
    format = "application/json",
    data = "<user_form>"
)]
pub async fn create_user(
    user_form: Json<DraftUserForm>,
    id: &str,
    db: &State<Surreal<Client>>,
) -> Result<Json<DraftUserReturnData>, NotFound<String>> {
    let new_username = user_form.0.name;
    let query =
        format!("SELECT *,->{DRAFT_USER_RELATION}.out.* as players FROM draft_session:{id};");

    // Guarding Checks
    let session: DraftSession = match run_query(query, db).await {
        Some(s) => s,
        None => return Err(NotFound(to_json_err("Session not found"))),
    };

    if session.is_name_taken(&new_username) {
        return Err(NotFound(to_json_err("Username already in use")));
    }

    if !session.slots_available() {
        return Err(NotFound(to_json_err("No slots available to join")));
    }

    // Create User
    let key = Uuid::new_v4();
    let hash = hash_uuid(&key);

    let new_user = DraftUser::new(new_username.clone(), hash, session.num_of_players());
    let new_records: Vec<Record> = match db.create("draft_user").content(new_user).await {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            return Err(NotFound(to_json_err("Could not create record")));
        }
    };

    relate_objects(
        db,
        &Thing {
            tb: "draft_session".into(),
            id: Id::String(id.into()),
        },
        &new_records[0].id,
        DRAFT_USER_RELATION,
    )
    .await?;

    #[derive(Serialize)]
    struct UpdateData {
        accepting_players: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        current_player: Option<Thing>,
    }
    let mut update_data = UpdateData {
        accepting_players: true,
        current_player: None,
    };

    let user_id = format!("{}", &new_records[0].id.id);
    if session.num_of_players() == 0 {
        // TODO Ugh this looks awful
        update_data.current_player = Some(Thing::from((DRAFT_USER_TB.to_owned(), user_id.clone())));
    };

    // TODO might need to do smarter casting of u16 to u32
    if session.num_of_players() + 1 >= (session.max_num_players as u32) {
        update_data.accepting_players = false;
    }

    let _updated: Option<Record> = db
        .update((DRAFT_SESSION, id))
        .merge(update_data)
        .await
        .map_err(|e| NotFound(e.to_string()))?;

    let return_data = DraftUserReturnData::new(
        new_username.clone(),
        id.into(),
        user_id,
        false,
        format!("{key}"),
    );

    // Only Return Subset of Items
    Ok(Json(return_data))
}

#[options("/draft_session/<id>/select-pokemon")]
pub fn option_select_pokemon<'a>(id: &str) -> &'a str {
    "Ok"
}

#[post(
    "/draft_session/<id>/select-pokemon",
    format = "application/json",
    data = "<select_pokemon_form>"
)]
pub async fn select_pokemon<'a>(
    select_pokemon_form: Json<SelectPokemonRequest>,
    id: &str,
    db: &State<Surreal<Client>>,
) -> Result<Json<SelectPokemonResponse>, NotFound<String>> {
    let select_pokemon = select_pokemon_form.0;

    let query =
        format!("SELECT *,(SELECT * from ->{DRAFT_USER_RELATION}.out ORDER BY order_in_session ASC) as players FROM draft_session:{id};");

    let session: DraftSession = match run_query(query, db).await {
        Some(s) => s,
        None => return Err(NotFound("Session not found".into())),
    };

    let draft_user_id = Thing::from((DRAFT_USER_TB.to_owned(), select_pokemon.user_id.clone()));

    let key_hash = match Uuid::parse_str(&select_pokemon.secret) {
        Ok(k) => hash_uuid(&k),
        Err(_) => return Err(NotFound("Could not parse uuid".into())),
    };

    if !session.draft_has_started() {
        return Err(NotFound(to_json_err("Draft has not yet started")));
    }
    if session.is_pokemon_chosen(&select_pokemon.pokemon_id) {
        return Err(NotFound(
            to_json_err("Pokemon cannot be selected. It's either banned or has already been selected."),
        ));
    }
    if select_pokemon.action != session.current_phase {
        return Err(NotFound(to_json_err("Current action not allowed")));
    }
    if !session.is_current_player(&draft_user_id) {
        return Err(NotFound(to_json_err("It is not your turn")));
    };

    // Get Next Player Thing in session
    let (turn, next_player_id) = session.get_next_player_id();
    let next_player_id = match next_player_id {
        Some(s) => Some(Thing::from((DRAFT_USER_TB.to_owned(), s))),
        None => None
    };

    // Get Next Phase in Session
    let next_phase = session.get_next_phase();

    let (mut pokemon_chosen_in_session, players) = (session.selected_pokemon, session.players);
    let players = match players {
        Some(p) => p,
        None => {
            return Err(NotFound(to_json_err("Nothing")))
        }
    };
    let mut player = match get_current_player(players, &draft_user_id) {
        Some(p) => p,
        None => {
            return Err(NotFound(to_json_err("User not in session.")))
        }
    };

    if !player.check_key_hash(key_hash) {
        return Err(NotFound(to_json_err("Access Denied")));
    };

    if let DraftPhase::Pick = select_pokemon.action {
        player.selected_pokemon.push(select_pokemon.pokemon_id);
    };

    pokemon_chosen_in_session.push(select_pokemon.pokemon_id);

    // update Session
    #[derive(Serialize)]
    struct SessionUpdateData<'a> {
        selected_pokemon: &'a [u32],
        turn_ticker: u32,
        current_player: Option<Thing>,
        current_phase: DraftPhase,
    }
    let update_data = SessionUpdateData {
        selected_pokemon: &pokemon_chosen_in_session[..],
        turn_ticker: turn,
        current_player: next_player_id,
        current_phase: next_phase,
    };
    let _updated: Option<Record> = db
        .update((DRAFT_SESSION, id))
        .merge(update_data)
        .await
        .map_err(|e| NotFound(e.to_string()))?;

    #[derive(Serialize)]
    struct PlayerUpdateData<'a> {
        selected_pokemon: &'a [u32],
    }
    let update_data = PlayerUpdateData {
        selected_pokemon: &player.selected_pokemon[..]
    };
    let _updated: Option<Record> = db
        .update((DRAFT_USER_TB, id))
        .merge(update_data)
        .await
        .map_err(|e| NotFound(e.to_string()))?;

    // TODO selected_pokemon should be set to the updated array of pk_ids
    Ok(Json(SelectPokemonResponse {
        selected_pokemon: player.selected_pokemon,
        banned_pokemon: pokemon_chosen_in_session,
        phase: next_phase,
    }))
}

fn get_current_player(players: Vec<DraftUser>, id: &Thing) -> Option<DraftUser> {
    for player in players {
        if let Some(ref t) = player.id {
            if t == id {
                return Some(player)
            }
        }
    }
    None
}

// TODO actually do something useful with those errors
// use map_error, maybe return result?
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
        Err(e) => {
            println!("{}", e);
            None
        }
    };

    resp
}

async fn relate_objects(
    db: &State<Surreal<Client>>,
    obj_in: &Thing,
    obj_out: &Thing,
    relation: &str,
) -> Result<(), NotFound<String>> {
    let query = format!("RELATE {}->{}->{};", obj_in, relation, obj_out);
    let _ = db.query(query).await.map_err(|e| NotFound(e.to_string()));
    Ok(())
}

// structs
#[derive(Debug, Serialize, Deserialize)]
pub struct SelectPokemonRequest {
    user_id: String,
    pokemon_id: u32,
    action: DraftPhase,
    secret: String,
}

#[derive(Debug, Serialize)]
pub struct SelectPokemonResponse {
    selected_pokemon: Vec<u32>,
    banned_pokemon: Vec<u32>,
    phase: DraftPhase,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDraftSessionResponse {
    current_phase: DraftPhase,
    banned_pokemon: Vec<u32>,
    current_player: Option<String>,
    players: Vec<PlayerData>,
}

impl UpdateDraftSessionResponse {
    fn from(session: DraftSession) -> UpdateDraftSessionResponse {
        let current_player_name = session.get_current_player_name();
        let players: Vec<DraftUser> = match session.players {
            Some(p) => p,
            None => Vec::new(),
        };

        // TODO clone is very expensive, figure out a way to avoid using it
        // TODO get as slice maybe?
        // Oh it gets data from players so probably can't use slices
        let player_data: Vec<PlayerData> = players
            .iter()
            .map(|element| PlayerData {
                name: element.name.clone(),
                pokemon: element.selected_pokemon.clone(),
            })
            .collect();

        UpdateDraftSessionResponse {
            banned_pokemon: session.selected_pokemon,
            current_phase: session.current_phase,
            current_player: current_player_name,
            players: player_data,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerData {
    name: String,
    pokemon: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PokemonSubData<'a> {
    name: &'a str,
    type1: PokemonType,
    type2: PokemonType,
    id: u32,
}
