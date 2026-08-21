//! `OneAgent Runtime` executable.

use oneagent_runtime::{App, DefaultConfigurationProvider};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    App::builder()
        .configure(&DefaultConfigurationProvider)?
        .build()?
        .run()?;

    Ok(())
}
