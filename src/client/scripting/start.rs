use luaur::prelude::*;

use crate::client::{connection::http::HttpConnection, host_ui::HostUiElement, state::State};
use super::{api::*, error::Result};

pub async fn start_game<T: 'static + State<Vec<HostUiElement>>>(game: String, connection: HttpConnection, host_ui: T) -> Result<()> {
    let lua = Lua::new();
    lua.set_app_data(connection);
    lua.set_app_data(host_ui);
    
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
        lua.create_function(lua_update_host_ui::<T>).unwrap()
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