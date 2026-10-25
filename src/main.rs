use anyhow::Result;
#[cfg(feature = "cli")]
use jakebox::cli;
#[cfg(feature = "web")]
use jakebox::web;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "cli")]
    cli::start_cli().await?;
    #[cfg(feature = "web")]
    leptos::mount::mount_to_body(web::App); 
    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    #[cfg(feature = "cli")]
    cli::start_cli().await?;
    #[cfg(feature = "web")]
    leptos::mount::mount_to_body(web::App); 
    Ok(())
}
