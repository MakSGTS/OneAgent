//! `OneAgent Runtime` executable.

use oneagent_runtime::{App, DefaultConfigurationProvider, HttpService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    App::builder()
        .configure(&DefaultConfigurationProvider)?
        .register_service("http", HttpService::new())?
        .build()?
        .run(tokio::signal::ctrl_c())
        .await?;

    Ok(())
}
