use std::process::ExitCode;

use oneagent_cli::{ClientRequest, ExecutionOutcome, RequestExecutor, run_with_executor};

struct PendingHttpExecutor;

impl RequestExecutor for PendingHttpExecutor {
    fn execute(&mut self, _request: &ClientRequest) -> ExecutionOutcome {
        ExecutionOutcome::TransportFailure
    }
}

fn main() -> ExitCode {
    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();
    let mut executor = PendingHttpExecutor;
    ExitCode::from(run_with_executor(
        std::env::args_os().skip(1),
        &mut stdout,
        &mut stderr,
        &mut executor,
    ))
}
