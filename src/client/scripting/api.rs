use std::time::Duration;

use luaur::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use tokio::time::sleep;
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::sleep;

use crate::{client::{connection::{common::Connection as _, http::HttpConnection}, host_ui::HostUiElement, state::State}, common::{connection::{GameAction, HostGameAction}, state::{GameState, UiElement}}};

pub async fn lua_get_state(lua: Lua, _: ()) -> LuaResult<GameState> {
    let connection = lua.app_data_ref::<HttpConnection>().expect("Connection not provided by Lua state.");
    connection.get_state().await.map_err(luaur::Error::external)
}

pub async fn lua_force_state(lua: Lua, new_state: GameState) -> LuaResult<()> {
    let connection = lua.app_data_ref::<HttpConnection>().expect("Connection not provided by Lua state.");
    connection.send_action(GameAction::Host(HostGameAction::ForcePlayerMap {
        new_map: new_state.get_players().values().cloned().collect()
    })).await.map_err(luaur::Error::external)?;
    Ok(())
}

pub async fn lua_update_ui(lua: Lua, (username, new_ui): (String, Vec<UiElement>)) -> LuaResult<()> {
    let connection = lua.app_data_ref::<HttpConnection>().expect("Connection not provided by Lua state.");
    connection.send_action(GameAction::Host(HostGameAction::UpdatePlayerUi { username, new_ui })).await.map_err(luaur::Error::external)?;
    Ok(())
}

pub fn lua_update_host_ui<T: 'static + State<Vec<HostUiElement>>>(lua: &Lua, new_ui: Vec<HostUiElement>) -> LuaResult<()> {
    let host_ui = lua.app_data_ref::<T>().expect("Host UI state not provided by Lua state.");
    host_ui.with_mut(|ui: &mut Vec<HostUiElement>| {*ui = new_ui;});
    Ok(())
}

pub async fn lua_sleep(_lua: Lua, secs: f32) -> LuaResult<()> {
    sleep(Duration::from_secs_f32(secs)).await;
    Ok(())
}

pub fn lua_rand_num(_lua: &Lua, (min, max): (f32, f32)) -> LuaResult<f32> {
    Ok(fastrand::f32() * (max-min) + min)
}

pub fn lua_rand_int(_lua: &Lua, (min, max): (i64, i64)) -> LuaResult<i64> {
    Ok(fastrand::i64(min..max))
}

pub fn lua_print(_lua: &Lua, text: String) -> LuaResult<()> {
    println!("{}", text);
    Ok(())
}