#[cfg(feature = "host")]
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[cfg(feature = "server")]
use crate::server;

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
        None => Ok(())
    }
}