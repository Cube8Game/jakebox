#[cfg(feature = "client")]
use std::{io::stdin, thread::sleep, time::Duration};
#[cfg(feature = "host")]
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
#[cfg(feature = "client")]
use crate::{common::{connection::{ClientGameAction, GameAction}, state::Event}};
#[cfg(feature = "host")]
use crate::common::connection::HostGameAction;

#[cfg(feature = "server")]
use crate::server;
#[cfg(feature = "client")]
use crate::client::{connection::{common::Connection, http::HttpConnection}};
// #[cfg(feature = "host")]
// use crate::client::scripting;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(clap::Subcommand)]
enum Commands {
    #[cfg(feature = "server")]
    Server {
        #[arg(long)]
        url: String
    },
    #[cfg(feature = "client")]
    Client {
        #[command(subcommand)]
        command: ClientCommands
    }
}

#[derive(clap::Subcommand)]
enum ClientCommands {
    Join {
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        join_code: String,
        #[arg(long)]
        username: String
    },
    #[cfg(feature = "host")]
    Host {
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        username: String,
        #[command(subcommand)]
        command: Option<HostCommands>,
        #[arg(long)]
        script: Option<PathBuf>
    }
}

#[cfg(feature = "host")]
#[derive(clap::Subcommand)]
enum HostCommands {
    StartPrompt {
        #[arg(long)]
        prompt: String,
        #[arg(long)]
        join_code: String
    }
}

pub async fn start_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        #[cfg(feature = "server")]
        Some(Commands::Server { url }) => {
            println!("{:?}", server::start_server(&url, 100).await);
            Ok(())
        }
        #[cfg(feature = "client")]
        Some(Commands::Client { command }) => {
            match command {
                ClientCommands::Join { url, join_code, username } => {
                    let url = match url {
                        Some(url) => url,
                        None => HttpConnection::find_server().await.ok_or_else(
                            || anyhow::anyhow!("No default servers are up")
                        )?.to_string()
                    };
                    let connection = HttpConnection::join(url, join_code.clone(), username).await?;
                    println!("Connection Successful.");
                    loop {
                        let event = connection.get_event().await?;
                        match event {
                            Event::Waiting => {}
                            Event::TextPrompt { prompt } => {
                                println!("{}", prompt);
                                let mut answer = String::new();
                                stdin().read_line(&mut answer)?;
                                connection.send_action(GameAction::Client(ClientGameAction::SubmitText { text: answer })).await?;
                            }
                            Event::TextPromptComplete { answer: _ } => {}
                            Event::TextDisplay { text } => {
                                println!("{}", text);
                            }
                        }
                        sleep(Duration::from_secs(2));
                    }
                }
                #[cfg(feature = "host")]
                ClientCommands::Host { url, username, command, script: _ } => {
                    match command {
                        None => {
                            let url = match url {
                                Some(url) => url,
                                None => HttpConnection::find_server().await.ok_or_else(
                                    || anyhow::anyhow!("No default servers are up")
                                )?.to_string()
                            };
                            let connection = HttpConnection::host(url, username).await?;
                            println!("Connection Successful. Join code is {}", connection.game_id);
                            // TODO: Re-enable CLI scripts
                            // if let Some(script) = script {
                            //     scripting::start_game(script, connection).await
                            //         .map_err(|e| anyhow::anyhow!(e.to_string()))?
                            // }
                            Ok(())
                        }
                        Some(HostCommands::StartPrompt { prompt, join_code }) => {
                            let url = match url {
                                Some(url) => url,
                                None => HttpConnection::find_server().await.ok_or_else(
                                    || anyhow::anyhow!("No default servers are up")
                                )?.to_string()
                            };
                            let connection = HttpConnection::new(url, join_code, username);
                            let state = connection.get_state().await?;
                            println!("{:?}", state);
                            let mut new_map = Vec::new();
                            for player in state.get_players().values() {
                                new_map.push(player.with_event(Event::TextPrompt { prompt: prompt.clone() }));
                            }
                            connection.send_action(GameAction::Host(HostGameAction::UpdatePlayerMap { new_map })).await?;
                            println!("Successfully sent player map update.");
                            Ok(())
                        }
                    }
                }
            }
        }
        None => Ok(())
    }
}