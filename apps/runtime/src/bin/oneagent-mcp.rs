use std::path::PathBuf;
use std::process::ExitCode;

use oneagent_runtime::{
    App, BoxError, ConfigurationProvider, McpStdioOutcome, McpStdioTransport, RuntimeConfig,
    WorkspaceService, semantic_server_observer,
};
use tokio::sync::oneshot;

#[tokio::main]
async fn main() -> ExitCode {
    let Ok(root) = std::env::current_dir() else {
        return fail("workspace root unavailable");
    };
    let workspace = WorkspaceService::new();
    let observer = workspace.snapshot_observer();
    let mut snapshots = observer.subscribe();
    let Ok(app) = App::builder()
        .configure(&McpConfigurationProvider {
            workspace_root: root,
        })
        .and_then(|builder| {
            builder
                .register_service("workspace", workspace)
                .map_err(|error| Box::new(error) as Box<dyn std::error::Error + Send + Sync>)
        })
        .and_then(|builder| {
            builder
                .build()
                .map_err(|error| Box::new(error) as Box<dyn std::error::Error + Send + Sync>)
        })
    else {
        return fail("workspace service construction failure");
    };
    let (runtime_shutdown, shutdown) = oneshot::channel::<()>();
    let mut runtime = tokio::spawn(app.run_without_banner(shutdown));

    loop {
        if snapshots.borrow_and_update().is_some() {
            break;
        }
        tokio::select! {
            biased;
            result = &mut runtime => {
                return match result {
                    Ok(Err(_)) | Err(_) => fail("workspace build failure"),
                    Ok(Ok(())) => fail("workspace service stopped"),
                };
            }
            changed = snapshots.changed() => {
                if changed.is_err() {
                    let _ = runtime.await;
                    return fail("workspace build failure");
                }
            }
        }
    }

    let Ok(server) = semantic_server_observer(observer) else {
        let _ = runtime_shutdown.send(());
        let _ = runtime.await;
        return fail("semantic server construction failure");
    };
    let transport = McpStdioTransport::new(server);
    let mut input = tokio::io::stdin();
    let mut output = tokio::io::stdout();

    let transport_result = tokio::select! {
        biased;
        result = &mut runtime => {
            return match result {
                Ok(Err(_)) | Err(_) => fail("workspace service failure"),
                Ok(Ok(())) => fail("workspace service stopped"),
            };
        }
        result = transport.run(&mut input, &mut output, tokio::signal::ctrl_c()) => result,
    };
    let _ = runtime_shutdown.send(());
    match runtime.await {
        Ok(Ok(())) => {}
        Ok(Err(_)) | Err(_) => return fail("workspace shutdown failure"),
    }

    match transport_result {
        Ok(McpStdioOutcome::EndOfInput | McpStdioOutcome::Cancelled) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("oneagent-mcp: {}", error.kind().as_str());
            ExitCode::FAILURE
        }
    }
}

struct McpConfigurationProvider {
    workspace_root: PathBuf,
}

impl ConfigurationProvider for McpConfigurationProvider {
    fn load(&self) -> Result<RuntimeConfig, BoxError> {
        Ok(RuntimeConfig::new("OneAgent MCP", "production")
            .with_workspace_root(self.workspace_root.clone()))
    }
}

fn fail(category: &str) -> ExitCode {
    eprintln!("oneagent-mcp: {category}");
    ExitCode::FAILURE
}
