//! `OneAgent Runtime` executable.

use oneagent_runtime::{App, DefaultConfigurationProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    App::builder()
        .configure(&DefaultConfigurationProvider)?
        .build()?
        .run(tokio::signal::ctrl_c())
        .await?;

    Ok(())
}
