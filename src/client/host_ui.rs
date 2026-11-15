use luaur::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostUiElement {
    Text(String)
}

impl IntoLua for HostUiElement {
    fn into_lua(self, lua: &Lua) -> Result<Value> {
        let table = lua.create_table();
        match self {
            Self::Text(s) => {
                table.set("type", "text")?;
                table.set("content", s)?;
            }
        }
        Ok(Value::Table(table))
    }
}

impl FromLua for HostUiElement {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {let table = value.as_table().ok_or_else(|| {
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
            _ => Err(LuaError::FromLuaConversionError { 
                from: value.type_name(), 
                to: "Event".to_string(), 
                message: Some(format!("Invalid type '{}'", ui_type)) 
            })
        }
    }
}
