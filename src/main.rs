use std::{io::stdin, path::PathBuf, thread::sleep, time::Duration};

use clap::Parser;
use jakebox::{client::{connection::{common::Connection, http::HttpConnection}, scripting}, common::{connection::{ClientGameAction, GameAction, HostGameAction}, state::Event}, server};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(clap::Subcommand)]
enum Commands {
    Server {
        #[arg(long)]
        url: String
    },
    Client {
        #[command(subcommand)]
        command: ClientCommands
    }
}

#[derive(clap::Subcommand)]
enum ClientCommands {
    Join {
        #[arg(long)]
        url: String,
        #[arg(long)]
        join_code: String,
        #[arg(long)]
        username: String
    },
    Host {
        #[arg(long)]
        url: String,
        #[arg(long)]
        username: String,
        #[command(subcommand)]
        command: Option<HostCommands>,
        #[arg(long)]
        script: Option<PathBuf>
    }
}

#[derive(clap::Subcommand)]
enum HostCommands {
    StartPrompt {
        #[arg(long)]
        prompt: String,
        #[arg(long)]
        join_code: String
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Server { url }) => {
            println!("{:?}", server::start_server(&url, 100).await);
        }
        Some(Commands::Client { command }) => {
            match command {
                ClientCommands::Join { url, join_code, username } => {
                    let connection_result = HttpConnection::join(url, join_code.clone(), username).await;
                    match connection_result {
                        Ok(connection) => {
                            println!("Connection Successful.");
                            loop {
                                match connection.get_event().await {
                                    Ok(event) => {
                                        match event {
                                            Event::Waiting => {}
                                            Event::TextPrompt { prompt } => {
                                                println!("{}", prompt);
                                                let mut answer = String::new();
                                                stdin().read_line(&mut answer).unwrap();
                                                connection.send_action(GameAction::Client(ClientGameAction::SubmitText { text: answer })).await.expect("Connection failed");
                                            }
                                            Event::TextPromptComplete { answer: _ } => {}
                                        }
                                    }
                                    Err(error) => {
                                        eprintln!("Error: {}", error);
                                        break;
                                    }
                                }
                                sleep(Duration::from_secs(2));
                            }
                        }
                        Err(error) => {
                            eprintln!("Error connecting to {}: {}", join_code, error);
                        }
                    }
                }
                ClientCommands::Host { url, username, command, script } => {
                    match command {
                        None => {
                            let connection_result = HttpConnection::host(url, username).await;
                            match connection_result {
                                Ok(connection) => {
                                    println!("Connection Successful. Join code is {}", connection.game_id);
                                    if let Some(script) = script {
                                        if let Err(error) = scripting::start_game(script, connection).await {
                                            eprintln!("Error running script: {}", error)
                                        }
                                    }
                                }
                                Err(error) => {
                                    eprintln!("Error hosting game: {}", error);
                                }
                            }
                        }
                        Some(HostCommands::StartPrompt { prompt, join_code }) => {
                            let connection = HttpConnection::new(url, join_code, username);
                            match connection.get_state().await {
                                Ok(state) => {
                                    println!("{:?}", state);
                                    let mut new_map = Vec::new();
                                    for player in state.get_players().values() {
                                        new_map.push(player.with_event(Event::TextPrompt { prompt: prompt.clone() }));
                                    }
                                    connection.send_action(GameAction::Host(HostGameAction::UpdatePlayerMap { new_map })).await.expect("Error when trying to update player map");
                                    println!("Successfully sent player map update.");
                                }
                                Err(error) => {
                                    eprintln!("Could not start prompt: {}", error);
                                }
                            }
                        }
                    }
                }
            }
        }
        None => {}
    }
}
