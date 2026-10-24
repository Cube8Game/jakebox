#[cfg(feature = "client")]
use std::{io::stdin, thread::sleep, time::Duration};
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
#[cfg(feature = "client")]
use crate::{common::{connection::{ClientGameAction, GameAction, HostGameAction}, state::Event}};

#[cfg(feature = "server")]
use crate::server;
#[cfg(feature = "client")]
use crate::client::{connection::{common::Connection, http::HttpConnection}, scripting};

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
                        }
                        sleep(Duration::from_secs(2));
                    }
                }
                ClientCommands::Host { url, username, command, script } => {
                    match command {
                        None => {
                            let connection = HttpConnection::host(url, username).await?;
                            println!("Connection Successful. Join code is {}", connection.game_id);
                            if let Some(script) = script {
                                scripting::start_game(script, connection).await
                                    .map_err(|e| anyhow::anyhow!(e.to_string()))?
                            }
                            Ok(())
                        }
                        Some(HostCommands::StartPrompt { prompt, join_code }) => {
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