use std::sync::Arc;

use gloo_timers::future::TimeoutFuture;
use leptos::{html, logging::debug_log, prelude::*};
use wasm_bindgen_futures::spawn_local;

use crate::{client::{connection::{common::Connection, http::HttpConnection}, game_search::Game, scripting}, common::{connection::{ClientGameAction, GameAction}, state::{Event, EventData, UiElement}}};

#[derive(Clone, Copy)]
enum PlayMode {
    Join,
    Host
}

#[component]
pub fn App() -> impl IntoView {
    let server_ip = LocalResource::new(
        || { HttpConnection::find_server() }
    );

    let (joined, set_joined) = signal(false);
    let (hosting, set_hosting) = signal(false);

    let (connection, set_connection) = signal(None::<Arc<HttpConnection>>);

    let (play_mode, set_play_mode) = signal(PlayMode::Join);

    let (custom_script, set_custom_script) = signal(false);
    let (script, set_script) = signal(None::<String>);
    let (games, set_games) = signal(None::<Vec<Game>>);
    let (current_game, set_current_game) = signal(None::<Game>);

    Effect::new(move || {
        spawn_local(async move {
            loop {
                debug_log!("Getting games list");
                let games = Game::get_all().await;
                if games.is_some() {
                    set_games.set(games);
                    break;
                }
                TimeoutFuture::new(2000).await;
            }
        });
    });

    let on_submit_join = move |game_id: String, username: String| {
        let server_ip = match server_ip.get() {
            Some(ip) => match ip {
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

    let on_submit_host = move |username: String| {
        let server_ip = match server_ip.get() {
            Some(ip) => match ip {
                Some(ip) => ip,
                None => {
                    return;
                }
            },
            None => {
                return;
            }
        };
        let script = match script.get() {
            Some(s) => s,
            None => {
                return;
            }
        };
        spawn_local(async move {
            if let Ok(connection) = HttpConnection::host(
                server_ip.to_string(),
                username
            ).await {
                set_connection.set(Some(Arc::new(connection.clone())));
                set_hosting.set(true);
                spawn_local(async move {
                    scripting::start_game(script, connection).await.unwrap();
                })
            };
        });
    };

    let join_code_input: NodeRef<html::Input> = NodeRef::new();
    let username_input: NodeRef<html::Input> = NodeRef::new();

    view! {
        <h1>"Jakebox"</h1>
        <Show
            when=move || {!(joined.get() || hosting.get())}
            fallback=move || if joined.get() {
                view! { <Game connection=connection/> }.into_any()
            } else {
                view! { <HostScreen connection=connection/> }.into_any()
            }
        >
            {match play_mode.get() {
                PlayMode::Join => view! {
                    <button on:click=move |_| {
                        set_play_mode.set(PlayMode::Host)
                    }>"Host Game"</button>
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
                    <p>{if joined.get() {"Joining..."} else {""}}</p>
                }.into_any(),
                PlayMode::Host => view! {
                    <button on:click=move |_| {
                        set_play_mode.set(PlayMode::Join)
                    }>"Join Game"</button>
                    <form on:submit=move |event| {
                        event.prevent_default();
                        let username = username_input
                            .get()
                            .expect("Input should be mounted")
                            .value();

                        on_submit_host(username);
                    }>
                        <input
                            node_ref=username_input
                            type="text"
                            placeholder="Username"
                        />

                        <button type="submit">{ ["Host".to_string(), match current_game.get() {
                            Some(game) => {
                                [" ", &game.name].concat()
                            },
                            None => {String::new()}
                        }].concat() }</button>
                    </form>
                    <Show
                        when=move || {!custom_script.get()}
                        fallback=view! {
                            <p>
                                <button on:click=move |_| {
                                    set_custom_script.set(false);
                                }>"Find game"</button>
                            </p>
                            <p>
                                <textarea
                                    placeholder="Lua script..."
                                    prop:value=move || script.get()
                                    on:input=move |event| {
                                        set_script.set(Some(event_target_value(&event)))
                                    }
                                />
                            </p>
                        }
                    >
                        <p>
                            <button on:click=move |_| {
                                set_custom_script.set(true);
                                set_current_game.set(None);
                            }>"Use custom script"</button>
                        </p>
                        <p>
                            <For
                                each=move || {games.get().unwrap_or_default()}
                                key=move |game| game.clone()
                                children=move |game| view! {
                                    <h2>{game.name.clone()}</h2>
                                    <p>{game.description.clone()}</p>
                                    <button on:click=move |_| {
                                        set_current_game.set(Some(game.clone()));
                                        set_script.set(Some(game.script.clone()))
                                    }>{format!("Run {}", game.name)}</button>
                                }
                            />
                        </p>
                    </Show>
                    <p>{if joined.get() {"Joining..."} else {""}}</p>
                }.into_any()
            }}
            <p>"Server IP: " {
                move || match server_ip.get() {
                    None => "Searching...".to_string(),
                    Some(None) => "No server found".to_string(),
                    Some(Some(ip)) => ip,
                }
            }</p>
        </Show>
    } 
}

#[component]
fn Game(connection: ReadSignal<Option<Arc<HttpConnection>>>) -> impl IntoView {
    let (event, set_event) = signal(Event::new());

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

    view! {
        <h1>"Game"</h1>
        <For
            each=move || {event.get().ui}
            key=|element| element.clone()
            children=move |element| {
                match element {
                    UiElement::Text(s) => view! {
                        {s}
                    }.into_any(),
                    UiElement::TextInput { key, placeholder } => {
                        let (response, set_response) = signal(String::new());

                        view! {
                            <form on:submit=move |event| {
                                event.prevent_default();
                                let key = key.clone();
                                let data = EventData::Text(response.get());
                                spawn_local(async move {connection.get().unwrap().send_action(
                                    GameAction::Client(ClientGameAction::SubmitEventData { key, data })
                                ).await.unwrap();});
                            }>
                                <input
                                    type="text"
                                    placeholder={placeholder}
                                    prop:value=move || response.get()
                                    on:input=move |event| {
                                        set_response.set(event_target_value(&event))
                                    }
                                />
                            </form>
                        }.into_any()
                    }
                }
            }
        />
    }
}

#[component]
fn HostScreen(connection: ReadSignal<Option<Arc<HttpConnection>>>) -> impl IntoView {
    view! {
        <p>"This tab is currently hosting the example game. Go to another tab and join with code " {connection.get().unwrap().game_id.clone()} "."</p>
    }
}
