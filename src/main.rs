use anyhow::Result;
#[cfg(feature = "cli")]
use jakebox::cli;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "cli")]
    cli::start_cli().await
}
