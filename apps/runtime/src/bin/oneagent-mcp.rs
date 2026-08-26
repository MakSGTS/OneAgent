use std::process::ExitCode;

use oneagent_runtime::{McpStdioOutcome, McpStdioTransport};

#[tokio::main]
async fn main() -> ExitCode {
    let transport = McpStdioTransport::default();
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
