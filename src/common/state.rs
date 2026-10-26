use std::collections::HashMap;

#[cfg(feature = "host")]
use luaur::prelude::*;
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

#[cfg(feature = "host")]
impl IntoLua for GameState {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table();
        table.set("players", self.players)?;
        Ok(luaur::Value::Table(table))
    }
}

#[cfg(feature = "host")]
impl FromLua for GameState {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        let table = value.as_table().ok_or_else(|| {
            LuaError::FromLuaConversionError { 
                from: "GameState", 
                to: "table".to_string(), 
                message: Some("Expected a table".to_string()) 
            }
        })?;
        
        let players: HashMap<String, Player> = table.get("players").map_err(|_| {
            LuaError::FromLuaConversionError { 
                from: "GameState", 
                to: "table".to_string(), 
                message: Some("Missing or invalid field 'players'".to_string()) 
            }
        })?;

        let mut output = Self::new(String::new());
        output.update_player_map(players.values().cloned().collect());

        Ok(output)
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

#[cfg(feature = "host")]
impl IntoLua for Player {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table();
        table.set("username", self.username)?;
        table.set("event", self.event)?;
        Ok(luaur::Value::Table(table))
    }
}

#[cfg(feature = "host")]
impl FromLua for Player {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        let table = value.as_table().ok_or_else(|| {
            LuaError::FromLuaConversionError { 
                from: "Player", 
                to: "table".to_string(), 
                message: Some("Expected a table".to_string()) 
            }
        })?;
        
        let username: String = table.get("username").map_err(|_| {
            LuaError::FromLuaConversionError { 
                from: "Player", 
                to: "table".to_string(), 
                message: Some("Missing or invalid field 'username'".to_string()) 
            }
        })?;
        
        let event: Event = table.get("event").map_err(|_| {
            LuaError::FromLuaConversionError { 
                from: "Player", 
                to: "table".to_string(), 
                message: Some("Missing or invalid field 'event'".to_string()) 
            }
        })?;

        Ok(Self { username, event })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    Waiting,
    TextPrompt { prompt: String },
    TextPromptComplete { answer: String },
    TextDisplay { text: String }
}

#[cfg(feature = "host")]
impl IntoLua for Event {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table();
        match self {
            Self::Waiting => {
                table.set("status", "waiting")?;
            }
            Self::TextPrompt { prompt } => {
                table.set("status", "text_prompt")?;
                table.set("prompt", prompt)?;
            }
            Self::TextPromptComplete { answer } => {
                table.set("status", "text_prompt_complete")?;
                table.set("answer", answer)?;
            }
            Self::TextDisplay { text } => {
                table.set("status", "text_display")?;
                table.set("text", text)?;
            }
        }
        Ok(luaur::Value::Table(table))
    }
}

#[cfg(feature = "host")]
impl FromLua for Event {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        let table = value.as_table().ok_or_else(|| {
            LuaError::FromLuaConversionError { 
                from: "Event", 
                to: "table".to_string(), 
                message: Some("Expected a table".to_string()) 
            }
        })?;
        
        let status: String = table.get("status").map_err(|_| {
            LuaError::FromLuaConversionError { 
                from: "Event", 
                to: "table".to_string(), 
                message: Some("Missing or invalid field 'status'".to_string()) 
            }
        })?;

        match status.as_str() {
            "waiting" => {
                Ok(Self::Waiting)
            }
            "text_prompt" => {
                let prompt: String = table.get("prompt").map_err(|_| {
                    LuaError::FromLuaConversionError { 
                        from: "Event", 
                        to: "table".to_string(), 
                        message: Some("Missing or invalid field 'prompt'".to_string()) 
                    }
                })?;

                Ok(Self::TextPrompt { prompt })
            }
            "text_prompt_complete" => {
                let answer: String = table.get("answer").map_err(|_| {
                    LuaError::FromLuaConversionError { 
                        from: "Event", 
                        to: "table".to_string(), 
                        message: Some("Missing or invalid field 'answer'".to_string()) 
                    }
                })?;

                Ok(Self::TextPromptComplete { answer })
            }
            "text_display" => {
                let text: String = table.get("text").map_err(|_| {
                    LuaError::FromLuaConversionError { 
                        from: "Event", 
                        to: "table".to_string(), 
                        message: Some("Missing or invalid field 'answer'".to_string()) 
                    }
                })?;

                Ok(Self::TextDisplay { text })
            }
            _ => {
                Err(LuaError::FromLuaConversionError { 
                    from: "Event", 
                    to: "table".to_string(), 
                    message: Some(format!("Invalid status '{}'", status)) 
                })
            }
        }
    }
}