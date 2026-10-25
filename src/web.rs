use std::sync::Arc;

use leptos::{html, prelude::*};
use wasm_bindgen_futures::spawn_local;

use crate::{client::connection::{common::Connection, http::HttpConnection}, common::state::Event};

#[component]
pub fn App() -> impl IntoView {
    let (event, set_event) = signal(Event::Waiting);
    let (join_code, set_join_code) = signal(None::<String>);
    let (username, set_username) = signal(None::<String>);
    let server_ip = LocalResource::new(
        || { HttpConnection::find_server() }
    );

    let (connection, set_connection) = signal(None::<Arc<HttpConnection>>);

    Effect::new(move |_| {
        let connection = Arc::clone(match &connection.get() {
            Some(connection) => connection,
            None => {
                return;
            }
        });
        let set_event = set_event;
        
        spawn_local(async move {
            loop {
                if let Ok(event) = connection.get_event().await {
                    set_event.set(event);
                }
            }
        });
    });

    let on_submit_join = move || {
        let server_ip = match server_ip.get() {
            Some(game_id) => match game_id {
                Some(game_id) => game_id,
                None => {
                    return;
                }
            },
            None => {
                return;
            }
        };
        let game_id = match join_code.get() {
            Some(game_id) => game_id,
            None => {
                return;
            }
        };
        let username = match username.get() {
            Some(username) => username,
            None => {
                return;
            }
        };
        spawn_local(async move {
            if let Ok(connection) = HttpConnection::join(
                server_ip.to_string(), 
                game_id, 
                username
            ).await {
                set_connection.set(Some(Arc::new(connection)));
            };
        });
    };

    let join_code_input: NodeRef<html::Input> = NodeRef::new();
    let username_input: NodeRef<html::Input> = NodeRef::new();

    view! {
        <h1>"Jakebox"</h1>
        <form on:submit=move |event| {
            event.prevent_default();
            let new_username = username_input
                .get()
                .expect("Input should be mounted")
                .value();
            
            set_join_code.set(Some(new_username));
            
            let new_join_code = join_code_input
                .get()
                .expect("Input should be mounted")
                .value();
            
            set_join_code.set(Some(new_join_code));

            on_submit_join()
        }>
            <input
                node_ref=username_input
                type="text"
                placeholder="Username"
            />
            <input
                node_ref=join_code_input
                type="text"
                placeholder="Join Code"
            />

            <button type="submit">"Join"</button>
        </form>
        <p>"Server IP: " {
            move || match server_ip.get() {
                None => "Searching...".to_string(),
                Some(None) => "No server found".to_string(),
                Some(Some(ip)) => ip,
            }
        }</p>
    }
}