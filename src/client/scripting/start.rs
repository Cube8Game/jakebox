use luaur::prelude::*;

use crate::client::{connection::http::HttpConnection, host_ui::HostUiElement, scripting::event::{EventManager, EventQueue}, state::State};
use super::{api::*, error::Result};

pub async fn start_game<
    HostUiState: 'static + State<Vec<HostUiElement>>,
    EventQueueState: 'static + State<EventQueue>
>(game: String, connection: HttpConnection, host_ui: HostUiState, event_queue: EventQueueState) -> Result<()> {
    let lua = Lua::new();
    lua.set_app_data(connection);
    lua.set_app_data(host_ui);
    lua.set_app_data(event_queue);
    lua.set_app_data(EventManager::default());
    
    lua.sandbox(true)?;

    // Load Jakebox API to Lua state
    
    let api = lua.create_table();
    
    api.set(
        "get_state",
        lua.create_async_function(lua_get_state).unwrap()
    )?;
    
    api.set(
        "force_state",
        lua.create_async_function(lua_force_state).unwrap()
    )?;
    
    api.set(
        "update_ui",
        lua.create_async_function(lua_update_ui).unwrap()
    )?;
    
    api.set(
        "update_host_ui",
        lua.create_function(lua_update_host_ui::<HostUiState>).unwrap()
    )?;
    
    api.set(
        "on_event",
        lua.create_function(lua_on_event).unwrap()
    )?;
    
    api.set(
        "update_events",
        lua.create_function(lua_update_events::<EventQueueState>).unwrap()
    )?;

    lua.globals().set("jakebox", api).unwrap();
    
    lua.globals().set(
        "sleep",
        lua.create_async_function(lua_sleep).unwrap()
    ).unwrap();
    
    lua.globals().set(
        "rand_num",
        lua.create_function(lua_rand_num).unwrap()
    ).unwrap();
    
    lua.globals().set(
        "rand_int",
        lua.create_function(lua_rand_int).unwrap()
    ).unwrap();
    
    lua.globals().set(
        "print",
        lua.create_function(lua_print).unwrap()
    ).unwrap();

    lua.load(&game).exec_async().await?;

    Ok(())
}