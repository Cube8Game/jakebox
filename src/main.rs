#[cfg(feature = "cli")]
use jakebox::cli;

#[tokio::main]
async fn main() {
    #[cfg(feature = "cli")]
    cli::start_cli().await;
}
