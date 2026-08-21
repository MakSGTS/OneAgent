//! `OneAgent Runtime` executable.

use oneagent_runtime::{
    App, DefaultConfigurationProvider, GraphQueryService, HttpService, WorkspaceService,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let workspace = WorkspaceService::new();
    let graph_query = GraphQueryService::new(workspace.snapshot_observer());

    App::builder()
        .configure(&DefaultConfigurationProvider)?
        .register_service("http", HttpService::with_graph_query(graph_query))?
        .register_service("workspace", workspace)?
        .build()?
        .run(tokio::signal::ctrl_c())
        .await?;

    Ok(())
}
