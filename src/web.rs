use std::sync::Arc;

use gloo_timers::future::TimeoutFuture;
use leptos::{html, logging::debug_log, prelude::*};
use wasm_bindgen_futures::spawn_local;

use crate::{client::connection::{common::Connection, http::HttpConnection}, common::{connection::{ClientGameAction, GameAction}, state::Event::{self, TextPromptComplete}}};

#[component]
pub fn App() -> impl IntoView {
    let server_ip = LocalResource::new(
        || { HttpConnection::find_server() }
    );

    let (joined, set_joined) = signal(false);

    let (connection, set_connection) = signal(None::<Arc<HttpConnection>>);

    let on_submit_join = move |game_id: String, username: String| {
        let server_ip = match server_ip.get() {
            Some(game_id) => match game_id {
                Some(ip) => ip,
                None => {
                    return;
                }
            },
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
                set_joined.set(true);
            };
        });
    };

    let join_code_input: NodeRef<html::Input> = NodeRef::new();
    let username_input: NodeRef<html::Input> = NodeRef::new();

    view! {
        <h1>"Jakebox"</h1>
        <Show
            when=move || !joined.get()
            fallback=move || view! { <Game connection=connection/> }
        >
            <form on:submit=move |event| {
                event.prevent_default();
                let username = username_input
                    .get()
                    .expect("Input should be mounted")
                    .value();
                
                let join_code = join_code_input
                    .get()
                    .expect("Input should be mounted")
                    .value();

                on_submit_join(join_code, username);
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
            <p>{if joined.get() {"Joining..."} else {""}}</p>
        </Show>
    }
}

#[component]
fn Game(connection: ReadSignal<Option<Arc<HttpConnection>>>) -> impl IntoView {
    let (event, set_event) = signal(Event::Waiting);
    let (disable_prompt_input, set_disable_prompt_input) = signal(false);

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
                debug_log!("Getting event...");
                match connection.get_event().await {
                    Ok(event) => {
                        debug_log!("{:?}", &event);
                        set_event.set(event);
                    }
                    Err(error) => {
                        debug_log!("{}", error);
                    },
                }
                TimeoutFuture::new(2000).await;
            }
        });
    });

    let prompt_input: NodeRef<html::Input> = NodeRef::new();

    view! {
        <h1>"Game"</h1>
        {move || match event.get() {
            Event::Waiting => view! {
                <p>"Waiting for host to start event..."</p>
            }.into_any(),
            Event::TextPrompt { prompt } => view! {
                <form on:submit=move |event| {
                    event.prevent_default();
                    let response = prompt_input
                        .get()
                        .expect("Input should be mounted")
                        .value();

                    set_disable_prompt_input.set(true);
                    
                    spawn_local(async move {connection.get().unwrap().send_action(
                        GameAction::Client(ClientGameAction::SubmitText { text: response })
                    ).await.unwrap();});
                }>
                    <p>{ prompt }</p>
                    <input
                        node_ref=prompt_input
                        type="text"
                        placeholder="Type your response..."
                        disabled={ disable_prompt_input.get() }
                    />
                    <button type="submit">"Submit"</button>
                </form>
            }.into_any(),
            TextPromptComplete { answer } => {
                set_disable_prompt_input.set(false);
                view! {
                    <input
                        node_ref=prompt_input
                        type="text"
                        placeholder="Type your response..."
                        disabled=true
                        value={ answer }
                    />
                    <button type="submit" disabled=true>"Submit"</button>
                }.into_any()
            },
            Event::TextDisplay { text } => view! {
                <p>{text}</p>
            }.into_any()
        }}
    }
}