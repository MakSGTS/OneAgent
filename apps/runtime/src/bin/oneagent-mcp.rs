use std::process::ExitCode;

use oneagent_runtime::{
    McpStdioOutcome, McpStdioTransport, WorkspaceSnapshotBuilder, semantic_server,
};

#[tokio::main]
async fn main() -> ExitCode {
    let Ok(root) = std::env::current_dir() else {
        return fail("workspace root unavailable");
    };
    let Ok(snapshot) = WorkspaceSnapshotBuilder::new().build(&root) else {
        return fail("workspace build failure");
    };
    let Ok(server) = semantic_server(snapshot) else {
        return fail("semantic server construction failure");
    };
    let transport = McpStdioTransport::new(server);
    let mut input = tokio::io::stdin();
    let mut output = tokio::io::stdout();

    match transport
        .run(&mut input, &mut output, tokio::signal::ctrl_c())
        .await
    {
        Ok(McpStdioOutcome::EndOfInput | McpStdioOutcome::Cancelled) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("oneagent-mcp: {}", error.kind().as_str());
            ExitCode::FAILURE
        }
    }
}

fn fail(category: &str) -> ExitCode {
    eprintln!("oneagent-mcp: {category}");
    ExitCode::FAILURE
}
