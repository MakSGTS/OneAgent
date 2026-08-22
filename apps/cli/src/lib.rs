//! Supported command boundary for the `OneAgent` Runtime client.

use std::ffi::OsString;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Successful invocation exit code.
pub const EXIT_SUCCESS: u8 = 0;
/// Invalid command-line exit code.
pub const EXIT_USAGE: u8 = 2;
/// Runtime transport failure exit code.
pub const EXIT_TRANSPORT: u8 = 3;
/// Non-success Runtime response exit code.
pub const EXIT_SERVER: u8 = 4;
/// Invalid Runtime protocol response exit code.
pub const EXIT_PROTOCOL: u8 = 5;
/// Command output failure exit code.
pub const EXIT_OUTPUT: u8 = 6;

const DEFAULT_PORT: u16 = 3000;
const USAGE_DIAGNOSTIC: &[u8] =
    b"oneagent-cli: usage_error: invalid command line\nTry 'oneagent-cli --help' for usage.\n";
const TRANSPORT_DIAGNOSTIC: &[u8] =
    b"oneagent-cli: transport_error: failed to communicate with runtime\n";
const PROTOCOL_DIAGNOSTIC: &[u8] = b"oneagent-cli: protocol_error: runtime response is invalid\n";
const OUTPUT_DIAGNOSTIC: &[u8] = b"oneagent-cli: output_error: failed to write command output\n";

const EDGE_KINDS: &[&str] = &[
    "contains",
    "calls",
    "references",
    "reads",
    "writes",
    "grants",
    "includes",
    "extends",
    "depends_on",
    "opens",
    "triggers",
];

const HELP: &str = concat!(
    "OneAgent Runtime client\n\n",
    "Usage:\n",
    "  oneagent-cli --help\n",
    "  oneagent-cli --version\n",
    "  oneagent-cli [--address <IP:PORT>] health <live|ready>\n",
    "  oneagent-cli [--address <IP:PORT>] configurations [--limit <1..100>]\n",
    "  oneagent-cli [--address <IP:PORT>] node --configuration-id <ID> --node-id <ID>\n",
    "  oneagent-cli [--address <IP:PORT>] relations --configuration-id <ID> --node-id <ID> --direction <incoming|outgoing> [--edge-kind <KIND>] [--limit <1..100>]\n",
    "  oneagent-cli [--address <IP:PORT>] traverse --configuration-id <ID> --node-id <ID> --direction <incoming|outgoing> --max-depth <0..4> [--edge-kind <KIND>] [--include-start] [--limit <1..100>]\n\n",
    "Defaults:\n",
    "  --address 127.0.0.1:3000\n",
    "  --limit   50 (Runtime default)\n\n",
    "Edge kinds:\n",
    "  contains, calls, references, reads, writes, grants, includes, extends,\n",
    "  depends_on, opens, triggers\n\n",
    "Exit codes:\n",
    "  0 success  2 usage  3 transport  4 server  5 protocol  6 output\n",
);

/// Runtime health probe selected by the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthProbe {
    /// Runtime liveness.
    Live,
    /// Runtime readiness.
    Ready,
}

/// Graph relation direction selected by the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Follow target-to-source relations.
    Incoming,
    /// Follow source-to-target relations.
    Outgoing,
}

impl Direction {
    /// Returns the accepted Runtime query value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Incoming => "incoming",
            Self::Outgoing => "outgoing",
        }
    }
}

/// One supported Runtime operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Query one health probe.
    Health(HealthProbe),
    /// List published Workspace configurations.
    Configurations {
        /// Optional server result limit.
        limit: Option<usize>,
    },
    /// Look up one exact graph node.
    Node {
        /// Canonical Configuration identifier.
        configuration_id: String,
        /// Canonical node identifier.
        node_id: String,
    },
    /// List direct graph relations.
    Relations {
        /// Canonical Configuration identifier.
        configuration_id: String,
        /// Canonical node identifier.
        node_id: String,
        /// Literal graph direction.
        direction: Direction,
        /// Optional exact edge kind.
        edge_kind: Option<String>,
        /// Optional server result limit.
        limit: Option<usize>,
    },
    /// Traverse the graph with accepted bounds.
    Traverse {
        /// Canonical Configuration identifier.
        configuration_id: String,
        /// Canonical node identifier.
        node_id: String,
        /// Literal graph direction.
        direction: Direction,
        /// Mandatory maximum edge depth.
        max_depth: usize,
        /// Optional exact edge kind.
        edge_kind: Option<String>,
        /// Whether to include the start node.
        include_start: bool,
        /// Optional server result limit.
        limit: Option<usize>,
    },
}

/// One validated client request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientRequest {
    address: SocketAddr,
    command: Command,
}

impl ClientRequest {
    /// Creates a validated request for an injected executor.
    #[must_use]
    pub const fn new(address: SocketAddr, command: Command) -> Self {
        Self { address, command }
    }

    /// Returns the target Runtime address.
    #[must_use]
    pub const fn address(&self) -> SocketAddr {
        self.address
    }

    /// Returns the selected Runtime operation.
    #[must_use]
    pub const fn command(&self) -> &Command {
        &self.command
    }
}

/// Result returned by a client request executor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionOutcome {
    /// Complete `200` Runtime JSON body.
    Success(Vec<u8>),
    /// Complete non-`200` Runtime JSON body.
    ServerFailure(Vec<u8>),
    /// Socket connection or I/O failed.
    TransportFailure,
    /// Runtime response violated the accepted protocol.
    ProtocolFailure,
}

/// Executes one validated Runtime client request.
pub trait RequestExecutor {
    /// Executes the request and returns one closed outcome.
    fn execute(&mut self, request: &ClientRequest) -> ExecutionOutcome;
}

/// Executes one CLI invocation through an injected request executor.
///
/// # Errors
///
/// All failures are represented by the returned stable process exit code and
/// the accepted output written to `stdout` or `stderr`.
pub fn run_with_executor<I, A, O, E, X>(
    args: I,
    stdout: &mut O,
    stderr: &mut E,
    executor: &mut X,
) -> u8
where
    I: IntoIterator<Item = A>,
    A: Into<OsString>,
    O: Write,
    E: Write,
    X: RequestExecutor,
{
    let Ok(args) = unicode_arguments(args) else {
        return diagnostic(stderr, USAGE_DIAGNOSTIC, EXIT_USAGE);
    };

    if matches!(args.as_slice(), [value] if value == "--help" || value == "-h") {
        return output(stdout, stderr, HELP.as_bytes(), EXIT_SUCCESS);
    }
    if matches!(args.as_slice(), [value] if value == "--version" || value == "-V") {
        let version = format!("oneagent-cli {}\n", env!("CARGO_PKG_VERSION"));
        return output(stdout, stderr, version.as_bytes(), EXIT_SUCCESS);
    }

    let Ok(request) = parse_request(&args) else {
        return diagnostic(stderr, USAGE_DIAGNOSTIC, EXIT_USAGE);
    };

    match executor.execute(&request) {
        ExecutionOutcome::Success(body) => output(stdout, stderr, &body, EXIT_SUCCESS),
        ExecutionOutcome::ServerFailure(body) => server_output(stderr, &body),
        ExecutionOutcome::TransportFailure => {
            diagnostic(stderr, TRANSPORT_DIAGNOSTIC, EXIT_TRANSPORT)
        }
        ExecutionOutcome::ProtocolFailure => diagnostic(stderr, PROTOCOL_DIAGNOSTIC, EXIT_PROTOCOL),
    }
}

fn unicode_arguments<I, A>(args: I) -> Result<Vec<String>, ()>
where
    I: IntoIterator<Item = A>,
    A: Into<OsString>,
{
    args.into_iter()
        .map(|value| value.into().into_string().map_err(|_| ()))
        .collect()
}

fn parse_request(args: &[String]) -> Result<ClientRequest, ()> {
    let mut cursor = Cursor::new(args);
    let address = if cursor.peek() == Some("--address") {
        cursor.next();
        cursor.next().ok_or(())?.parse().map_err(|_| ())?
    } else {
        default_address()
    };
    let command = cursor.next().ok_or(())?;
    let command = match command {
        "health" => parse_health(&mut cursor)?,
        "configurations" => parse_configurations(&mut cursor)?,
        "node" => parse_node(&mut cursor)?,
        "relations" => parse_relations(&mut cursor)?,
        "traverse" => parse_traverse(&mut cursor)?,
        _ => return Err(()),
    };
    if cursor.next().is_some() {
        return Err(());
    }
    Ok(ClientRequest::new(address, command))
}

const fn default_address() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), DEFAULT_PORT)
}

fn parse_health(cursor: &mut Cursor<'_>) -> Result<Command, ()> {
    let probe = match cursor.next().ok_or(())? {
        "live" => HealthProbe::Live,
        "ready" => HealthProbe::Ready,
        _ => return Err(()),
    };
    Ok(Command::Health(probe))
}

fn parse_configurations(cursor: &mut Cursor<'_>) -> Result<Command, ()> {
    let mut limit = None;
    while let Some(option) = cursor.peek() {
        match option {
            "--limit" => set_once(&mut limit, parse_limit(cursor.value()?)?)?,
            _ => return Err(()),
        }
    }
    Ok(Command::Configurations { limit })
}

fn parse_node(cursor: &mut Cursor<'_>) -> Result<Command, ()> {
    let mut configuration_id = None;
    let mut node_id = None;
    while let Some(option) = cursor.peek() {
        match option {
            "--configuration-id" => {
                set_once(&mut configuration_id, parse_id(cursor.value()?)?)?;
            }
            "--node-id" => set_once(&mut node_id, parse_id(cursor.value()?)?)?,
            _ => return Err(()),
        }
    }
    Ok(Command::Node {
        configuration_id: configuration_id.ok_or(())?,
        node_id: node_id.ok_or(())?,
    })
}

fn parse_relations(cursor: &mut Cursor<'_>) -> Result<Command, ()> {
    let mut configuration_id = None;
    let mut node_id = None;
    let mut direction = None;
    let mut edge_kind = None;
    let mut limit = None;
    while let Some(option) = cursor.peek() {
        match option {
            "--configuration-id" => {
                set_once(&mut configuration_id, parse_id(cursor.value()?)?)?;
            }
            "--node-id" => set_once(&mut node_id, parse_id(cursor.value()?)?)?,
            "--direction" => {
                set_once(&mut direction, parse_direction(cursor.value()?)?)?;
            }
            "--edge-kind" => {
                set_once(&mut edge_kind, parse_edge_kind(cursor.value()?)?)?;
            }
            "--limit" => set_once(&mut limit, parse_limit(cursor.value()?)?)?,
            _ => return Err(()),
        }
    }
    Ok(Command::Relations {
        configuration_id: configuration_id.ok_or(())?,
        node_id: node_id.ok_or(())?,
        direction: direction.ok_or(())?,
        edge_kind,
        limit,
    })
}

fn parse_traverse(cursor: &mut Cursor<'_>) -> Result<Command, ()> {
    let mut configuration_id = None;
    let mut node_id = None;
    let mut direction = None;
    let mut max_depth = None;
    let mut edge_kind = None;
    let mut include_start = false;
    let mut limit = None;
    while let Some(option) = cursor.peek() {
        match option {
            "--configuration-id" => {
                set_once(&mut configuration_id, parse_id(cursor.value()?)?)?;
            }
            "--node-id" => set_once(&mut node_id, parse_id(cursor.value()?)?)?,
            "--direction" => {
                set_once(&mut direction, parse_direction(cursor.value()?)?)?;
            }
            "--max-depth" => {
                set_once(&mut max_depth, parse_max_depth(cursor.value()?)?)?;
            }
            "--edge-kind" => {
                set_once(&mut edge_kind, parse_edge_kind(cursor.value()?)?)?;
            }
            "--include-start" => {
                cursor.next();
                if include_start {
                    return Err(());
                }
                include_start = true;
            }
            "--limit" => set_once(&mut limit, parse_limit(cursor.value()?)?)?,
            _ => return Err(()),
        }
    }
    Ok(Command::Traverse {
        configuration_id: configuration_id.ok_or(())?,
        node_id: node_id.ok_or(())?,
        direction: direction.ok_or(())?,
        max_depth: max_depth.ok_or(())?,
        edge_kind,
        include_start,
        limit,
    })
}

fn parse_id(value: &str) -> Result<String, ()> {
    if value.is_empty() || value.trim().is_empty() {
        Err(())
    } else {
        Ok(value.to_owned())
    }
}

fn parse_direction(value: &str) -> Result<Direction, ()> {
    match value {
        "incoming" => Ok(Direction::Incoming),
        "outgoing" => Ok(Direction::Outgoing),
        _ => Err(()),
    }
}

fn parse_edge_kind(value: &str) -> Result<String, ()> {
    EDGE_KINDS
        .contains(&value)
        .then(|| value.to_owned())
        .ok_or(())
}

fn parse_limit(value: &str) -> Result<usize, ()> {
    let value = parse_unsigned(value)?;
    (1..=100).contains(&value).then_some(value).ok_or(())
}

fn parse_max_depth(value: &str) -> Result<usize, ()> {
    let value = parse_unsigned(value)?;
    (value <= 4).then_some(value).ok_or(())
}

fn parse_unsigned(value: &str) -> Result<usize, ()> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(());
    }
    value.parse().map_err(|_| ())
}

fn set_once<T>(target: &mut Option<T>, value: T) -> Result<(), ()> {
    if target.is_some() {
        return Err(());
    }
    *target = Some(value);
    Ok(())
}

fn output<O: Write, E: Write>(writer: &mut O, stderr: &mut E, body: &[u8], exit: u8) -> u8 {
    let result = writer.write_all(body).and_then(|()| {
        if body.ends_with(b"\n") {
            Ok(())
        } else {
            writer.write_all(b"\n")
        }
    });
    if result.and_then(|()| writer.flush()).is_err() {
        let _ = stderr.write_all(OUTPUT_DIAGNOSTIC);
        let _ = stderr.flush();
        EXIT_OUTPUT
    } else {
        exit
    }
}

fn server_output<E: Write>(stderr: &mut E, body: &[u8]) -> u8 {
    let result = stderr.write_all(body).and_then(|()| {
        if body.ends_with(b"\n") {
            Ok(())
        } else {
            stderr.write_all(b"\n")
        }
    });
    if result.and_then(|()| stderr.flush()).is_err() {
        let _ = stderr.write_all(OUTPUT_DIAGNOSTIC);
        let _ = stderr.flush();
        EXIT_OUTPUT
    } else {
        EXIT_SERVER
    }
}

fn diagnostic<E: Write>(stderr: &mut E, body: &[u8], exit: u8) -> u8 {
    if stderr
        .write_all(body)
        .and_then(|()| stderr.flush())
        .is_err()
    {
        EXIT_OUTPUT
    } else {
        exit
    }
}

struct Cursor<'a> {
    args: &'a [String],
    index: usize,
}

impl<'a> Cursor<'a> {
    const fn new(args: &'a [String]) -> Self {
        Self { args, index: 0 }
    }

    fn peek(&self) -> Option<&'a str> {
        self.args.get(self.index).map(String::as_str)
    }

    fn next(&mut self) -> Option<&'a str> {
        let value = self.peek()?;
        self.index += 1;
        Some(value)
    }

    fn value(&mut self) -> Result<&'a str, ()> {
        self.next();
        self.next().ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use super::*;

    #[derive(Debug)]
    struct RecordingExecutor {
        outcome: ExecutionOutcome,
        requests: Vec<ClientRequest>,
    }

    impl RecordingExecutor {
        fn new(outcome: ExecutionOutcome) -> Self {
            Self {
                outcome,
                requests: Vec::new(),
            }
        }
    }

    impl RequestExecutor for RecordingExecutor {
        fn execute(&mut self, request: &ClientRequest) -> ExecutionOutcome {
            self.requests.push(request.clone());
            self.outcome.clone()
        }
    }

    #[derive(Debug, Default)]
    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("controlled output failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("controlled output failure"))
        }
    }

    fn run(args: &[&str], outcome: ExecutionOutcome) -> (u8, Vec<u8>, Vec<u8>, Vec<ClientRequest>) {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut executor = RecordingExecutor::new(outcome);
        let exit = run_with_executor(args, &mut stdout, &mut stderr, &mut executor);
        (exit, stdout, stderr, executor.requests)
    }

    fn success(args: &[&str]) -> (Vec<u8>, Vec<ClientRequest>) {
        let (exit, stdout, stderr, requests) =
            run(args, ExecutionOutcome::Success(br#"{"ok":true}"#.to_vec()));
        assert_eq!(exit, EXIT_SUCCESS);
        assert_eq!(stdout, b"{\"ok\":true}\n");
        assert!(stderr.is_empty());
        (stdout, requests)
    }

    #[test]
    fn help_and_version_are_exact_and_do_not_execute_requests() {
        for option in ["--help", "-h"] {
            let (exit, stdout, stderr, requests) =
                run(&[option], ExecutionOutcome::TransportFailure);
            assert_eq!(exit, EXIT_SUCCESS);
            assert_eq!(stdout, HELP.as_bytes());
            assert!(stderr.is_empty());
            assert!(requests.is_empty());
        }

        for option in ["--version", "-V"] {
            let (exit, stdout, stderr, requests) =
                run(&[option], ExecutionOutcome::TransportFailure);
            assert_eq!(exit, EXIT_SUCCESS);
            assert_eq!(stdout, b"oneagent-cli 0.1.0\n");
            assert!(stderr.is_empty());
            assert!(requests.is_empty());
        }
    }

    #[test]
    fn health_commands_use_the_default_or_exact_override_address() {
        let (_, requests) = success(&["health", "live"]);
        assert_eq!(
            requests,
            [ClientRequest::new(
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000),
                Command::Health(HealthProbe::Live)
            )]
        );

        let (_, requests) = success(&["--address", "127.0.0.2:4317", "health", "ready"]);
        assert_eq!(requests[0].address().to_string(), "127.0.0.2:4317");
        assert_eq!(requests[0].command(), &Command::Health(HealthProbe::Ready));
    }

    #[test]
    fn configurations_accepts_omitted_and_boundary_limits() {
        for (args, expected) in [
            (vec!["configurations"], None),
            (vec!["configurations", "--limit", "0001"], Some(1)),
            (vec!["configurations", "--limit", "100"], Some(100)),
        ] {
            let (_, requests) = success(&args);
            assert_eq!(
                requests[0].command(),
                &Command::Configurations { limit: expected }
            );
        }
    }

    #[test]
    fn node_accepts_options_in_either_order_and_preserves_identifiers() {
        for args in [
            vec![
                "node",
                "--configuration-id",
                "configuration with space",
                "--node-id",
                "узел&one",
            ],
            vec![
                "node",
                "--node-id",
                "узел&one",
                "--configuration-id",
                "configuration with space",
            ],
        ] {
            let (_, requests) = success(&args);
            assert_eq!(
                requests[0].command(),
                &Command::Node {
                    configuration_id: "configuration with space".to_owned(),
                    node_id: "узел&one".to_owned(),
                }
            );
        }
    }

    #[test]
    fn relations_accepts_complete_options_in_arbitrary_order() {
        let (_, requests) = success(&[
            "relations",
            "--limit",
            "50",
            "--edge-kind",
            "depends_on",
            "--direction",
            "incoming",
            "--node-id",
            "node",
            "--configuration-id",
            "configuration",
        ]);
        assert_eq!(
            requests[0].command(),
            &Command::Relations {
                configuration_id: "configuration".to_owned(),
                node_id: "node".to_owned(),
                direction: Direction::Incoming,
                edge_kind: Some("depends_on".to_owned()),
                limit: Some(50),
            }
        );
    }

    #[test]
    fn traverse_accepts_depth_boundaries_flag_and_optional_values() {
        for (depth, include_start) in [("0", false), ("04", true)] {
            let mut args = vec![
                "traverse",
                "--configuration-id",
                "configuration",
                "--node-id",
                "node",
                "--direction",
                "outgoing",
                "--max-depth",
                depth,
                "--edge-kind",
                "contains",
                "--limit",
                "1",
            ];
            if include_start {
                args.push("--include-start");
            }
            let (_, requests) = success(&args);
            assert_eq!(
                requests[0].command(),
                &Command::Traverse {
                    configuration_id: "configuration".to_owned(),
                    node_id: "node".to_owned(),
                    direction: Direction::Outgoing,
                    max_depth: usize::from(include_start) * 4,
                    edge_kind: Some("contains".to_owned()),
                    include_start,
                    limit: Some(1),
                }
            );
        }
    }

    #[test]
    fn invalid_structural_forms_do_not_execute_requests() {
        let cases: &[&[&str]] = &[
            &[],
            &["--help", "node"],
            &["help"],
            &["--address"],
            &["--address", "localhost:3000", "health", "live"],
            &["health"],
            &["health", "unknown"],
            &["health", "live", "extra"],
            &["configurations", "--limit"],
            &["configurations", "--limit=1"],
            &["configurations", "--limit", "1", "--limit", "2"],
            &["node", "--configuration-id", "configuration"],
            &["node", "configuration", "node"],
            &[
                "relations",
                "--configuration-id",
                "configuration",
                "--node-id",
                "node",
                "--direction",
                "outgoing",
                "--include-start",
            ],
            &[
                "traverse",
                "--configuration-id",
                "configuration",
                "--node-id",
                "node",
                "--direction",
                "outgoing",
                "--max-depth",
                "1",
                "--include-start",
                "--include-start",
            ],
        ];
        for args in cases {
            let (exit, stdout, stderr, requests) = run(args, ExecutionOutcome::Success(Vec::new()));
            assert_eq!(exit, EXIT_USAGE, "args: {args:?}");
            assert!(stdout.is_empty());
            assert_eq!(stderr, USAGE_DIAGNOSTIC);
            assert!(requests.is_empty());
        }
    }

    #[test]
    fn invalid_values_do_not_execute_requests() {
        let cases: &[&[&str]] = &[
            &["configurations", "--limit", "0"],
            &["configurations", "--limit", "101"],
            &["configurations", "--limit", "+1"],
            &["configurations", "--limit", "184467440737095516161"],
            &["node", "--configuration-id", " ", "--node-id", "node"],
            &[
                "relations",
                "--configuration-id",
                "configuration",
                "--node-id",
                "node",
                "--direction",
                "OUTGOING",
            ],
            &[
                "relations",
                "--configuration-id",
                "configuration",
                "--node-id",
                "node",
                "--direction",
                "outgoing",
                "--edge-kind",
                "unknown",
            ],
            &[
                "traverse",
                "--configuration-id",
                "configuration",
                "--node-id",
                "node",
                "--direction",
                "outgoing",
                "--max-depth",
                "5",
            ],
        ];
        for args in cases {
            let (exit, stdout, stderr, requests) = run(args, ExecutionOutcome::Success(Vec::new()));
            assert_eq!(exit, EXIT_USAGE, "args: {args:?}");
            assert!(stdout.is_empty());
            assert_eq!(stderr, USAGE_DIAGNOSTIC);
            assert!(requests.is_empty());
        }
    }

    #[cfg(unix)]
    #[test]
    fn non_unicode_argument_is_a_usage_error_without_execution() {
        use std::os::unix::ffi::OsStringExt;

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut executor = RecordingExecutor::new(ExecutionOutcome::Success(Vec::new()));
        let exit = run_with_executor(
            [OsString::from_vec(vec![0xff])],
            &mut stdout,
            &mut stderr,
            &mut executor,
        );
        assert_eq!(exit, EXIT_USAGE);
        assert!(stdout.is_empty());
        assert_eq!(stderr, USAGE_DIAGNOSTIC);
        assert!(executor.requests.is_empty());
    }

    #[test]
    fn execution_outcomes_use_exact_streams_newlines_and_exit_codes() {
        for (outcome, expected_exit, expected_stdout, expected_stderr) in [
            (
                ExecutionOutcome::Success(b"success".to_vec()),
                EXIT_SUCCESS,
                b"success\n".as_slice(),
                b"".as_slice(),
            ),
            (
                ExecutionOutcome::Success(b"success\n".to_vec()),
                EXIT_SUCCESS,
                b"success\n".as_slice(),
                b"".as_slice(),
            ),
            (
                ExecutionOutcome::ServerFailure(b"server".to_vec()),
                EXIT_SERVER,
                b"".as_slice(),
                b"server\n".as_slice(),
            ),
            (
                ExecutionOutcome::TransportFailure,
                EXIT_TRANSPORT,
                b"".as_slice(),
                TRANSPORT_DIAGNOSTIC,
            ),
            (
                ExecutionOutcome::ProtocolFailure,
                EXIT_PROTOCOL,
                b"".as_slice(),
                PROTOCOL_DIAGNOSTIC,
            ),
        ] {
            let (exit, stdout, stderr, requests) = run(&["health", "live"], outcome);
            assert_eq!(exit, expected_exit);
            assert_eq!(stdout, expected_stdout);
            assert_eq!(stderr, expected_stderr);
            assert_eq!(requests.len(), 1);
        }
    }

    #[test]
    fn output_failure_returns_six_and_reports_best_effort_diagnostic() {
        let mut stdout = FailingWriter;
        let mut stderr = Vec::new();
        let mut executor = RecordingExecutor::new(ExecutionOutcome::Success(b"success".to_vec()));
        let exit = run_with_executor(["health", "live"], &mut stdout, &mut stderr, &mut executor);
        assert_eq!(exit, EXIT_OUTPUT);
        assert_eq!(stderr, OUTPUT_DIAGNOSTIC);
        assert_eq!(executor.requests.len(), 1);

        let mut stdout = Vec::new();
        let mut stderr = FailingWriter;
        let mut executor = RecordingExecutor::new(ExecutionOutcome::TransportFailure);
        let exit = run_with_executor(["health", "live"], &mut stdout, &mut stderr, &mut executor);
        assert_eq!(exit, EXIT_OUTPUT);
    }

    #[test]
    fn fresh_invocations_are_equal_and_do_not_share_state() {
        let first = run(
            &["configurations", "--limit", "2"],
            ExecutionOutcome::Success(b"result".to_vec()),
        );
        let second = run(
            &["configurations", "--limit", "2"],
            ExecutionOutcome::Success(b"result".to_vec()),
        );
        assert_eq!(first, second);
    }
}
