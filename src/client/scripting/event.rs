use luaur::prelude::*;
use std::{collections::HashMap};

use crate::common::state::{EventData, Player};

#[derive(Default, Clone)]
pub struct EventQueue {
    events: Vec<EventEmit>
}

impl EventQueue {
    pub fn pop(&mut self) -> Option<EventEmit> {
        self.events.pop()
    }

    pub fn push(&mut self, value: EventEmit) {
        self.events.push(value);
    }
}

#[derive(Default)]
pub struct EventManager {
    events: HashMap<EventType, Vec<Function>>
}

impl EventManager {
    pub fn subscribe(&mut self, event_type: EventType, function: Function) {
        if let Some(functions) = self.events.get_mut(&event_type) {
            functions.push(function);
        } else {
            self.events.insert(event_type, vec![function]);
        };
    }

    pub fn emit(&self, emit: EventEmit) -> LuaResult<()> {
        let functions = self.events.get(&emit.clone().into()).cloned().unwrap_or_default();

        match &emit {
            EventEmit::PlayerJoin(player) => {
                for function in functions {
                    function.call::<Player>(player.clone())?;
                }
            }
            EventEmit::PlayerSubmit { player, key, value } => {
                for function in functions {
                    function.call::<(Player, String, EventData)>((player.clone(), key.clone(), value.clone()))?;
                }
            }
        }
        Ok(())
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum EventType {
    PlayerJoin,
    PlayerSubmit
}

impl IntoLua for EventType {
    fn into_lua(self, lua: &Lua) -> Result<Value> {
        match self {
            Self::PlayerJoin => "player_join",
            Self::PlayerSubmit => "player_submit",
        }.into_lua(lua)
    }
}

impl FromLua for EventType {
    fn from_lua(value: Value, _lua: &Lua) -> Result<Self> {
        let s = value.as_string().ok_or_else(|| {
            LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "EventType".to_string(), 
                message: Some("Expected a string".to_string()) 
            }
        })?;

        match s.to_str()?.as_str() {
            "player_join" => Ok(Self::PlayerJoin),
            "player_submit" => Ok(Self::PlayerSubmit),
            s => Err(LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "EventType".to_string(), 
                message: Some(format!("Invalid type '{}'", s)) 
            })
        }
    }
}

#[derive(Clone)]
pub enum EventEmit {
    PlayerJoin(Player),
    PlayerSubmit {
        player: Player,
        key: String,
        value: EventData
    }
}

impl EventEmit {
    fn event_type(&self) -> EventType {
        match self {
            Self::PlayerJoin(_) => EventType::PlayerJoin,
            Self::PlayerSubmit { player: _, key: _, value: _ } => EventType::PlayerSubmit
        }
    }
}

impl IntoLua for EventEmit {
    fn into_lua(self, lua: &Lua) -> Result<Value> {
        let table = lua.create_table();
        table.set("type", self.event_type().into_lua(lua)?)?;
        match self {
            Self::PlayerJoin(s) => {
                table.set("player", s)?;
            }
            Self::PlayerSubmit { player, key, value } => {
                table.set("player", player)?;
                table.set("key", key)?;
                table.set("value", value)?;
            }
        }
        table.into_lua(lua)
    }
}

impl FromLua for EventEmit {
    fn from_lua(value: Value, _lua: &Lua) -> Result<Self> {
        let table = value.as_table().ok_or_else(|| {
            LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "EventEmit".to_string(), 
                message: Some("Expected a table".to_string()) 
            }
        })?;
        let event_type: EventType = table.get("type").map_err(|_| {
            LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "EventEmit".to_string(), 
                message: Some("Missing or invalid field 'type'".to_string()) 
            }
        })?;

        match event_type {
            EventType::PlayerJoin => {
                let player: Player = table.get("player").map_err(|_| {
                    LuaError::FromLuaConversionError { 
                        from: value.type_name(), 
                        to: "Event".to_string(), 
                        message: Some("Missing or invalid field 'player'".to_string()) 
                    }
                })?;

                Ok(Self::PlayerJoin(player))
            }
            EventType::PlayerSubmit => {
                let player: Player = table.get("player").map_err(|_| {
                    LuaError::FromLuaConversionError { 
                        from: value.type_name(), 
                        to: "Event".to_string(), 
                        message: Some("Missing or invalid field 'player'".to_string()) 
                    }
                })?;

                let key: String = table.get("key").map_err(|_| {
                    LuaError::FromLuaConversionError { 
                        from: value.type_name(), 
                        to: "Event".to_string(), 
                        message: Some("Missing or invalid field 'key'".to_string()) 
                    }
                })?;

                let value: EventData = table.get("value").map_err(|_| {
                    LuaError::FromLuaConversionError { 
                        from: value.type_name(), 
                        to: "Event".to_string(), 
                        message: Some("Missing or invalid field 'value'".to_string()) 
                    }
                })?;

                Ok(Self::PlayerSubmit { player, key, value })
            }
        }
    }
}

impl Into<EventType> for EventEmit {
    fn into(self) -> EventType {
        self.event_type()
    }
}

