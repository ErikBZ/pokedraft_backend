use crate::api::utils::{relate_objects, run_query};
use crate::models::draft::{
    DraftPhase, DraftRules, DraftSession, DraftSessionCreateForm, DraftState, DraftUser, DraftUserForm, DraftUserReturnData
};
use crate::models::pokemon::PokemonType;
use crate::models::{hash_uuid, Record};

use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::State;

use serde::{Deserialize, Serialize};

use surrealdb::{RecordId, Surreal};
use surrealdb::engine::remote::ws::Client;

use uuid::Uuid;

const DRAFT_USER_RELATION: &str = "players";
const DRAFT_SESSION: &str = "draft_session";
const DRAFT_USER_TB: &str = "draft_user";

fn to_json_msg(str: &str) -> String {
    format!("{{\"message\": \"{}\"}}", str)
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
) -> Option<Json<DraftSession>> {
    // should you even do this?
    let session_form: DraftSessionCreateForm = session_form.0;

    let rules: DraftRules = match db.select(("draft_rules", &session_form.draft_rules)).await {
        Ok(p) => match p {
            Some(r) => r,
            None => return None,
        },
        Err(e) => {
            println!("BLAGH: {}", e);
            return None;
        }
    };

    let draft_session = DraftSession::from(session_form, rules);
    let result: DraftSession = match db.create("draft_session").content(draft_session).await {
        Ok(Some(r)) => r,
        Ok(None) => return None,
        Err(e) => {
            println!("HELLO: {}", e);
            return None;
        }
    };


    Some(Json(result))
}

#[post(
    "/draft_session/<id>/ready",
    format = "application/json",
    data = "<user_form>"
)]
pub async fn toggle_ready(
    id: &str,
    user_form: Json<ReadyDraftUserForm>,
    db: &State<Surreal<Client>>,
) -> Result<String, NotFound<String>> {
    let new_username = user_form.0.user_id;
    let query =
        format!("SELECT *,(SELECT * from ->{DRAFT_USER_RELATION}.out ORDER BY order_in_session ASC) as players FROM draft_session:{id};");

    let session: DraftSession = match run_query(query, db).await {
        Some(s) => s,
        None => return Err(NotFound("Session not found".into())),
    };

    if session.draft_state == DraftState::InProgress || session.draft_state == DraftState::Ended {
        // TODO: Set error message
        return Err(NotFound(to_json_msg(
            "Can't ready when the draft is in progress.",
        )));
    }

    // Get the user
    let user_id = RecordId::from_table_key("draft_user", new_username);
    let players = match session.players {
        Some(p) => p,
        None => {
            return Err(NotFound(to_json_msg(
                "Can't ready when the draft is in progress.",
            )))
        }
    };

    let mut all_players_ready = true;
    for player in players.iter() {
        if let Some(p_id) = &player.id {
            if p_id != &user_id {
                all_players_ready = player.ready & all_players_ready
            }
        }
    }

    let user = match get_current_player(players, &user_id) {
        Some(u) => u,
        None => {
            return Err(NotFound(to_json_msg(
                "Can't ready when the draft is in progress.",
            )))
        }
    };

    let ready = !user.ready;
    all_players_ready = all_players_ready && ready;
    let new_draft_state = if all_players_ready {
        DraftState::Ready
    } else {
        DraftState::Open
    };

    #[derive(Serialize)]
    struct UpdateData {
        ready: bool,
    }
    let update = UpdateData { ready };
    let _updated: Option<Record> = db
        .update(user_id)
        .merge(update)
        .await
        .map_err(|e| NotFound(e.to_string()))?;

    #[derive(Serialize)]
    struct SessionUpdateData {
        draft_state: DraftState,
    }
    let update = SessionUpdateData {
        draft_state: new_draft_state,
    };
    // Maybe use set to session_id so that it's easier to tell what the id is for
    let _updated: Option<Record> = db
        .update((DRAFT_SESSION, id))
        .merge(update)
        .await
        .map_err(|e| NotFound(e.to_string()))?;

    Ok(to_json_msg("All good"))
}

#[post(
    "/draft_session/<id>/start",
    format = "application/json",
    data = "<user_form>"
)]
pub async fn start(
    id: &str,
    user_form: Json<ReadyDraftUserForm>,
    db: &State<Surreal<Client>>,
) -> Result<String, NotFound<String>> {
    #[derive(Serialize)]
    struct SessionUpdateData {
        draft_state: DraftState,
    }
    let update = SessionUpdateData {
        draft_state: DraftState::InProgress,
    };

    let _updated: Option<Record> = db
        .update((DRAFT_SESSION, id))
        .merge(update)
        .await
        .map_err(|e| NotFound(e.to_string()))?;
    Ok(to_json_msg("All Good"))
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

// TODO: Change this to `join`
#[options("/draft_session/<id>/create-user")]
pub fn option_create_user<'a>(id: &str) -> &'a str {
    let _id = id;
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
        None => return Err(NotFound(to_json_msg("Session not found"))),
    };

    if session.num_of_players() >= (session.max_num_players as u32)
        && (session.draft_state != DraftState::InProgress
            || session.draft_state != DraftState::Ended)
    {
        return Err(NotFound(to_json_msg(
            "Draft is no longer accepting players.",
        )));
    }

    if session.is_name_taken(&new_username) {
        return Err(NotFound(to_json_msg("Username already in use")));
    }

    if !session.slots_available() {
        return Err(NotFound(to_json_msg("No slots available to join")));
    }

    // Create User
    let key = Uuid::new_v4();
    let hash = hash_uuid(&key);

    let new_user = DraftUser::new(new_username.clone(), hash, session.num_of_players());
    let new_record: DraftUser = match db.create("draft_user").content(new_user).await {
        Ok(Some(r)) => r,
        Ok(None) => return Err(NotFound(to_json_msg("Could not create record"))),
        Err(e) => {
            println!("{}", e);
            return Err(NotFound(to_json_msg("Could not create record")));
        }
    };

    let record_id = RecordId::from_table_key("draft_session", id);
    let new_user_id = new_record.id.unwrap();

    relate_objects(
        db,
        &record_id,
        &new_user_id,
        DRAFT_USER_RELATION,
    )
    .await?;

    #[derive(Serialize)]
    struct UpdateData {
        accepting_players: bool,
        draft_state: DraftState,
        current_player: Option<RecordId>,
    }
    let mut update_data = UpdateData {
        accepting_players: true,
        current_player: None,
        draft_state: DraftState::Open,
    };

    let user_id = format!("{}", new_user_id);
    if session.num_of_players() == 0 {
        // TODO Ugh this looks awful
        update_data.current_player = Some(RecordId::from_table_key(DRAFT_USER_TB.to_owned(), user_id.clone()));
    };

    // TODO might need to do smarter casting of u16 to u32
    if session.num_of_players() + 1 >= (session.max_num_players as u32) {
        update_data.accepting_players = false;
    };

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
    let _id = id;
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

    let draft_user_id = RecordId::from_table_key(DRAFT_USER_TB.to_owned(), select_pokemon.user_id.clone());

    let key_hash = match Uuid::parse_str(&select_pokemon.secret) {
        Ok(k) => hash_uuid(&k),
        Err(_) => return Err(NotFound("Could not parse uuid".into())),
    };

    if session.draft_state != DraftState::InProgress {
        return Err(NotFound(to_json_msg("Draft has not yet started")));
    }
    if session.is_pokemon_chosen(&select_pokemon.pokemon_id) {
        return Err(NotFound(to_json_msg(
            "Pokemon cannot be selected. It's either banned or has already been selected.",
        )));
    }
    if select_pokemon.action != session.current_phase {
        return Err(NotFound(to_json_msg("Current action not allowed")));
    }
    if !session.is_current_player(&draft_user_id) {
        return Err(NotFound(to_json_msg("It is not your turn")));
    };

    // Get Next Player ID in session
    let (turn, next_player_id) = session.get_next_player_id();
    let next_player_id = match next_player_id {
        Some(s) => Some(RecordId::from_table_key(DRAFT_USER_TB.to_owned(), s)),
        None => None
    };

    // Get Next Phase in Session
    let next_phase = session.get_next_phase();
    // Check if Session has ended
    let draft_state = if session.check_if_session_is_over() {
        DraftState::Ended
    } else {
        session.draft_state
    };

    let (mut pokemon_chosen_in_session, players) = (session.selected_pokemon, session.players);
    let players = match players {
        Some(p) => p,
        None => return Err(NotFound(to_json_msg("Nothing"))),
    };
    let mut player = match get_current_player(players, &draft_user_id) {
        Some(p) => p,
        None => return Err(NotFound(to_json_msg("User not in session."))),
    };

    if !player.check_key_hash(key_hash) {
        return Err(NotFound(to_json_msg("Access Denied")));
    };

    if let DraftPhase::Pick = select_pokemon.action {
        player.selected_pokemon.push(select_pokemon.pokemon_id);
    };

    pokemon_chosen_in_session.push(select_pokemon.pokemon_id);

    // update Session
    #[derive(Serialize)]
    struct SessionUpdateData {
        selected_pokemon: Vec<u32>,
        turn_ticker: u32,
        current_player: Option<RecordId>,
        current_phase: DraftPhase,
        draft_state: DraftState,
    }
    let update_data = SessionUpdateData {
        selected_pokemon: pokemon_chosen_in_session.clone(),
        turn_ticker: turn,
        current_player: next_player_id,
        current_phase: next_phase,
        draft_state,
    };
    let _updated: Option<Record> = db
        .update((DRAFT_SESSION, id))
        .merge(update_data)
        .await
        .map_err(|e| NotFound(e.to_string()))?;

    #[derive(Serialize)]
    struct PlayerUpdateData {
        selected_pokemon: Vec<u32>,
    }
    let update_data = PlayerUpdateData {
        selected_pokemon: player.selected_pokemon.clone(),
    };
    let _updated: Option<Record> = db
        .update(draft_user_id)
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


fn get_current_player(players: Vec<DraftUser>, id: &RecordId) -> Option<DraftUser> {
    for player in players {
        if let Some(ref t) = player.id {
            if t == id {
                return Some(player);
            }
        }
    }
    None
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
    state: DraftState,
}

impl UpdateDraftSessionResponse {
    fn from(session: DraftSession) -> UpdateDraftSessionResponse {
        let current_player_name = session.get_current_player_name();
        let (selected_pokemon, current_phase, players) = (
            session.selected_pokemon,
            session.current_phase,
            session.players,
        );
        let players: Vec<DraftUser> = match players {
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
                ready: element.ready,
            })
            .collect();

        UpdateDraftSessionResponse {
            banned_pokemon: selected_pokemon,
            current_phase: current_phase,
            current_player: current_player_name,
            players: player_data,
            state: session.draft_state,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerData {
    name: String,
    pokemon: Vec<u32>,
    ready: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PokemonSubData<'a> {
    name: &'a str,
    type1: PokemonType,
    type2: PokemonType,
    id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadyDraftUserForm {
    user_id: String,
}

