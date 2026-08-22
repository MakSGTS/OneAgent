use std::process::ExitCode;

use oneagent_cli::{HttpExecutor, run_with_executor};

fn main() -> ExitCode {
    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();
    let mut executor = HttpExecutor;
    ExitCode::from(run_with_executor(
        std::env::args_os().skip(1),
        &mut stdout,
        &mut stderr,
        &mut executor,
    ))
}
