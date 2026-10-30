use serde::{Deserialize, Serialize};

use super::state::{EventData, Player};

#[derive(Serialize, Deserialize)]
pub struct GameRequest {
    pub username: String,
    pub action: Option<GameAction>
}

impl GameRequest {
    pub fn new(username: String, action: Option<GameAction>) -> Self {
        Self { username, action }
    }
}

#[derive(Serialize, Deserialize)]
pub enum GameAction {
    Client(ClientGameAction),
    Host(HostGameAction)
}

#[derive(Serialize, Deserialize)]
pub enum ClientGameAction {
    SubmitEventData { key: String, data: EventData }
}

#[derive(Serialize, Deserialize)]
pub enum HostGameAction {
    UpdatePlayerMap { new_map: Vec<Player> }
}