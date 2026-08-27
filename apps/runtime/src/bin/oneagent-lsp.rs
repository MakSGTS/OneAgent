use std::process::ExitCode;

use oneagent_protocol::LspExitStatus;
use oneagent_runtime::{LspStdioOutcome, LspStdioTransport, WorkspaceSnapshotBuilder, lsp_server};

#[tokio::main]
async fn main() -> ExitCode {
    let Ok(root) = std::env::current_dir() else {
        return fail("workspace root unavailable");
    };
    let Ok(snapshot) = WorkspaceSnapshotBuilder::new().build(&root) else {
        return fail("workspace build failure");
    };
    let Ok(server) = lsp_server(snapshot) else {
        return fail("LSP server construction failure");
    };
    let mut transport = LspStdioTransport::new(server);
    let mut input = tokio::io::stdin();
    let mut output = tokio::io::stdout();

    match transport
        .run(&mut input, &mut output, tokio::signal::ctrl_c())
        .await
    {
        Ok(LspStdioOutcome::Exited(LspExitStatus::Success)) => ExitCode::SUCCESS,
        Ok(LspStdioOutcome::Exited(LspExitStatus::Failure)) => fail("lifecycle failure"),
        Ok(LspStdioOutcome::Cancelled) => fail("cancelled"),
        Err(error) => fail(error.kind().as_str()),
    }
}

fn fail(category: &str) -> ExitCode {
    eprintln!("oneagent-lsp: {category}");
    ExitCode::FAILURE
}
