use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameState {
    host_username: String,
    players: HashMap<String, Player>
}

impl GameState {
    pub fn new(host: String) -> Self {
        Self {
            host_username: host,
            players: HashMap::new()
        }
    }

    pub fn add_player(&mut self, username: String) {
        assert!(!self.players.contains_key(&username));
        self.players.insert(username.clone(), Player::new(username));
    }

    pub fn get_player_event(&self, username: &str) -> Event {
        assert!(self.players.contains_key(username));
        self.players.get(username).unwrap().event.clone()
    }
    
    pub fn set_player_event(&mut self, username: &str, new_value: Event) {
        assert!(self.players.contains_key(username));
        self.players.get_mut(username).unwrap().event = new_value;
    }

    pub fn update_player_map(&mut self, players: Vec<Player>) {
        self.players = HashMap::new();
        for player in players {
            self.players.insert(player.username.clone(), player);
        }
    }

    pub fn get_players(&self) -> &HashMap<String, Player> {
        &self.players
    }

    pub fn host(&self) -> &str {
        &self.host_username
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    username: String,
    event: Event
}

impl Player {
    pub fn new(username: String) -> Self {
        Self {
            username,
            event: Event::Waiting
        }
    }

    pub fn with_event(&self, event: Event) -> Self {
        Self {
            username: self.username.clone(),
            event
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    Waiting,
    TextPrompt { prompt: String },
    TextPromptComplete { answer: String }
}