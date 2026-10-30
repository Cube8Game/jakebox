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

    pub fn add_player(&mut self, username: String) -> std::result::Result<(), ()> {
        if self.players.contains_key(&username) {
            return Err(())
        }
        self.players.insert(username.clone(), Player::new(username));
        Ok(())
    }

    pub fn get_player_event(&self, username: &str) -> Option<&Event> {
        Some(&self.players.get(username)?.event)
    }

    pub fn get_player_event_mut(&mut self, username: &str) -> Option<&mut Event> {
        Some(&mut self.players.get_mut(username)?.event)
    }
    
    pub fn set_player_event(&mut self, username: &str, new_value: Event) -> bool {
        match self.players.get_mut(username) {
            Some(player) => {
                player.event = new_value;
            }
            None => {
                return false;
            }
        }
        true
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
                from: value.type_name(), 
                to: "GameState".to_string(), 
                message: Some("Expected a table".to_string()) 
            }
        })?;
        
        let players: HashMap<String, Player> = table.get("players").map_err(|_| {
            LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "GameState".to_string(), 
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
            event: Event::new()
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
                from: value.type_name(), 
                to: "Player".to_string(), 
                message: Some("Expected a table".to_string()) 
            }
        })?;
        
        let username: String = table.get("username").map_err(|_| {
            LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "Player".to_string(), 
                message: Some("Missing or invalid field 'username'".to_string()) 
            }
        })?;
        
        let event: Event = table.get("event").map_err(|_| {
            LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "Player".to_string(), 
                message: Some("Missing or invalid field 'event'".to_string()) 
            }
        })?;

        Ok(Self { username, event })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub ui: Vec<UiElement>,
    pub data: HashMap<String, EventData>
}

impl Event {
    pub fn new() -> Self {
        Self {
            ui: Vec::new(),
            data: HashMap::new()
        }
    }
}

#[cfg(feature = "host")]
impl IntoLua for Event {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table();
        table.set("ui", self.ui)?;
        table.set("data", self.data)?;
        Ok(luaur::Value::Table(table))
    }
}

#[cfg(feature = "host")]
impl FromLua for Event {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        let table = value.as_table().ok_or_else(|| {
            LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "Event".to_string(), 
                message: Some("Expected a table".to_string()) 
            }
        })?;
        
        let ui: Vec<UiElement> = table.get("ui").map_err(|_| {
            LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "Event".to_string(), 
                message: Some("Missing or invalid field 'ui'".to_string()) 
            }
        })?;
        
        let data: HashMap<String, EventData> = table.get("data").map_err(|_| {
            LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "Event".to_string(), 
                message: Some("Missing or invalid field 'data'".to_string()) 
            }
        })?;

        Ok(Self { ui, data })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum UiElement {
    Text(String),
    TextInput {
        key: String,
        placeholder: String,
    }
}

#[cfg(feature = "host")]
impl IntoLua for UiElement {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table();
        match self {
            Self::Text(s) => {
                table.set("type", "text")?;
                table.set("content", s)?;
            }
            Self::TextInput { key, placeholder } => {
                table.set("type", "text_input")?;
                table.set("key", key)?;
                table.set("placeholder", placeholder)?;
            }
        }
        Ok(Value::Table(table))
    }
}

#[cfg(feature = "host")]
impl FromLua for UiElement {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        let table = value.as_table().ok_or_else(|| {
            LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "UiElement".to_string(), 
                message: Some("Expected a table".to_string()) 
            }
        })?;
        let ui_type: String = table.get("type").map_err(|_| {
            LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "UiElement".to_string(), 
                message: Some("Missing or invalid field 'type'".to_string()) 
            }
        })?;

        match ui_type.as_str() {
            "text" => {
                let content: String = table.get("content").map_err(|_| {
                    LuaError::FromLuaConversionError { 
                        from: value.type_name(), 
                        to: "Event".to_string(), 
                        message: Some("Missing or invalid field 'content'".to_string()) 
                    }
                })?;

                Ok(Self::Text(content))
            }
            "text_input" => {
                let key: String = table.get("key").map_err(|_| {
                    LuaError::FromLuaConversionError { 
                        from: value.type_name(), 
                        to: "Event".to_string(), 
                        message: Some("Missing or invalid field 'key'".to_string()) 
                    }
                })?;

                let placeholder: String = table.get("placeholder").map_err(|_| {
                    LuaError::FromLuaConversionError { 
                        from: value.type_name(), 
                        to: "Event".to_string(), 
                        message: Some("Missing or invalid field 'placeholder'".to_string()) 
                    }
                })?;

                Ok(Self::TextInput { key, placeholder })
            }
            _ => Err(LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "Event".to_string(), 
                message: Some(format!("Invalid type '{}'", ui_type)) 
            })
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EventData {
    Text(String)
}

#[cfg(feature = "host")]
impl IntoLua for EventData {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        match self {
            EventData::Text(s) => {
                s.into_lua(lua)
            }
        }
    }
}

#[cfg(feature = "host")]
impl FromLua for EventData {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            Value::String(s) => {
                Ok(Self::Text(s.to_string_lossy()))
            }
            _ => Err(LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "Event".to_string(), 
                message: Some(format!("Cannot convert {} to Event.", value.type_name())) 
            })
        }
    }
}