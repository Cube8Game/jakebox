use std::sync::Arc;

use axum::{Json, Router, extract::{Path, State}, http::{Method, StatusCode}, routing::{get, post}};
use dashmap::DashMap;
use rand::RngExt;
use tower_http::cors::{Any, CorsLayer};

use crate::common::{connection::{ClientGameAction, GameAction, GameRequest, HostGameAction}, state::{Event, GameState}};

type Games = Arc<DashMap<String, GameState>>;

#[derive(Clone)]
struct AppState {
    games: Games,
    capacity: usize,
}

impl AppState {
    fn from_capacity(capacity: usize) -> Self {
        assert!(capacity <= MAX_CAPACITY);
        Self {
            games: Arc::new(DashMap::new()),
            capacity
        }
    }
}

const JOIN_CODE_LENGTH: usize = 4;
const MAX_CAPACITY: usize = 26usize.pow(JOIN_CODE_LENGTH as u32);

pub async fn start_server(address: &str, game_capacity: usize) -> std::io::Result<()> {
    let app_state = AppState::from_capacity(game_capacity);

    let games_router = Router::new()
        .route("/", post(post_game))
        .route("/{id}/join", post(join))
        .route("/{id}/event", get(get_event))
        .route("/{id}/state", get(get_state))
        .route("/{id}/action", post(action))
        .with_state(app_state);

    let api_router = Router::new()
        .route("/status", get(status))
        .nest("/games", games_router);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::OPTIONS
        ]);

    let app = Router::new()
        .nest("/api/unstable", api_router)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(address).await?;

    axum::serve(listener, app).await
}


async fn status() -> StatusCode {
    StatusCode::OK
}


fn generate_join_code() -> String {
    let mut rng = rand::rng();
    (0..JOIN_CODE_LENGTH).map(|_| {
        let letter = rng.random_range(0..26);
        char::from(b'A' + letter)
    }).collect()
}

async fn join(State(state): State<AppState>, Path(id): Path<String>, Json(request): Json<GameRequest>) -> Result<(), StatusCode> {
    if let Some(mut pair) = state.games.get_mut(&id) {
        let game = pair.value_mut();
        game.add_player(request.username);
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn post_game(State(state): State<AppState>, Json(request): Json<GameRequest>) -> Result<String, StatusCode> {
    if state.games.len() >= state.capacity {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    loop {
        let join_code = generate_join_code();
        if !state.games.contains_key(&join_code) {
            state.games.insert(join_code.clone(), GameState::new(request.username));
            return Ok(join_code);
        }
    }
}

async fn get_event(State(state): State<AppState>, Path(id): Path<String>, Json(request): Json<GameRequest>) -> Result<Json<Event>, StatusCode> {
    if let Some(game) = state.games.get(&id) {
        Ok(Json(game.value().get_player_event(&request.username)))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_state(State(state): State<AppState>, Path(id): Path<String>, Json(request): Json<GameRequest>) -> Result<Json<GameState>, StatusCode> {
    if let Some(game) = state.games.get(&id) {
        if request.username != game.host() {
            return Err(StatusCode::FORBIDDEN)
        }
        Ok(Json(game.value().clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn action(State(state): State<AppState>, Path(id): Path<String>, Json(request): Json<GameRequest>) -> Result<(), StatusCode> {
    if let Some(mut pair) = state.games.get_mut(&id) {
        let game = pair.value_mut();
        if let Some(action) = request.action {
            match action {
                GameAction::Client(client_action) => {
                    match client_action {
                        ClientGameAction::SubmitText { text } => {
                            if let Event::TextPrompt { prompt: _ } = game.get_player_event(&request.username) {
                                game.set_player_event(&request.username, Event::TextPromptComplete { answer: text });
                            } else {
                                return Err(StatusCode::CONFLICT);
                            }
                        }
                    }
                }
                GameAction::Host(host_action) => {
                    if request.username != game.host() {
                        return Err(StatusCode::FORBIDDEN);
                    }
                    match host_action {
                        HostGameAction::UpdatePlayerMap { new_map } => {
                            game.update_player_map(new_map);
                        }
                    }
                }
            }
        }
        Ok(())
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}