//! `OneAgent Runtime` executable.

use oneagent_runtime::{App, DefaultConfigurationProvider, HttpService, WorkspaceService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let workspace = WorkspaceService::new();
    let _workspace_snapshot = workspace.snapshot_observer();

    App::builder()
        .configure(&DefaultConfigurationProvider)?
        .register_service("http", HttpService::new())?
        .register_service("workspace", workspace)?
        .build()?
        .run(tokio::signal::ctrl_c())
        .await?;

    Ok(())
}
