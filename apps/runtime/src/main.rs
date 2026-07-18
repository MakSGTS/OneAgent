//! `OneAgent Runtime` executable.

mod app;
mod config;
mod error;
mod state;

use app::App;
use config::DefaultConfigurationProvider;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    App::builder()
        .configure(&DefaultConfigurationProvider)?
        .build()?
        .run()?;

    Ok(())
}
