use std::time::Duration;

use luaur::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use tokio::time::sleep;
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::sleep;

use crate::{client::connection::{common::Connection, http::HttpConnection}, common::{connection::{GameAction, HostGameAction}, state::GameState}};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Lua error: {0}")]
    LuaError(#[from] luaur::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error)
}

type Result<T> = std::result::Result<T, Error>;

async fn lua_get_state(lua: Lua, _: ()) -> LuaResult<GameState> {
    let connection = lua.app_data_ref::<HttpConnection>().expect("Connection not provided by Lua state.");
    connection.get_state().await.map_err(luaur::Error::external)
}

async fn lua_set_state(lua: Lua, new_state: GameState) -> LuaResult<()> {
    let connection = lua.app_data_ref::<HttpConnection>().expect("Connection not provided by Lua state.");
    connection.send_action(GameAction::Host(HostGameAction::UpdatePlayerMap {
        new_map: new_state.get_players().values().cloned().collect()
    })).await.map_err(luaur::Error::external)?;
    Ok(())
}

async fn lua_sleep(_lua: Lua, secs: f32) -> LuaResult<()> {
    sleep(Duration::from_secs_f32(secs)).await;
    Ok(())
}

fn lua_print(_lua: &Lua, text: String) -> LuaResult<()> {
    println!("{}", text);
    Ok(())
}

pub async fn start_game(game: String, connection: HttpConnection) -> Result<()> {
    let lua = Lua::new();
    lua.set_app_data(connection);
    
    lua.sandbox(true)?;

    // Load Jakebox API to Lua state
    
    let api = lua.create_table();
    
    api.set(
        "get_state",
        lua.create_async_function(lua_get_state).unwrap()
    )?;
    
    api.set(
        "set_state",
        lua.create_async_function(lua_set_state).unwrap()
    )?;

    lua.globals().set("jakebox", api).unwrap();
    
    lua.globals().set(
        "sleep",
        lua.create_async_function(lua_sleep).unwrap()
    ).unwrap();
    
    lua.globals().set(
        "print",
        lua.create_function(lua_print).unwrap()
    ).unwrap();

    lua.load(&game).exec_async().await?;

    Ok(())
}