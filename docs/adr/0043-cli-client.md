# ADR-0043: CLI Client

## Status

Accepted

## Context

Sprint 21 must replace the `oneagent-cli` placeholder with the first supported
client for accepted Runtime health, Workspace configuration listing, exact node
lookup, direct relations, and bounded traversal.

The [Sprint 21 investigation](../architecture/cli-client-investigation.md)
confirms that the CLI has no existing contract or dependency, while ADR-0038
and ADR-0040 already define the complete server-side HTTP/1.1 surface. The
Runtime library and tracked mixed EDT/Designer fixture provide deterministic
public client/server oracles. Standard-library argument, socket, I/O, and exit
primitives are sufficient; no new production dependency or external data is
required.

This ADR fixes the client authority, grammar, endpoint, request, response,
output, failure, exit, resource, compatibility, dependency, evidence, and
deferred-scope contracts before implementation.

## Decision

### Authority and dependency direction

`oneagent-cli` owns only command-line parsing, client request construction,
blocking HTTP/1.1 communication, response presentation, and process exit
classification. It is a consumer of accepted Runtime wires.

ADR-0038 remains the authority for health behavior. ADR-0040 remains the
authority for Graph Query routes, parameters, values, schemas, errors, bounds,
ordering, readiness gating, and per-request snapshot consistency. Runtime,
Workspace, `SemanticGraph`, File Watching, Persistent Cache, and source adapters
retain their existing ownership. CLI code does not deserialize responses into
canonical graph or protocol-domain types and creates no second semantic or wire
authority.

The dependency direction is:

```text
oneagent-cli command model
    -> oneagent-cli HTTP client
        -> accepted oneagent-runtime HTTP/1.1 wires
```

Runtime and lower-level crates do not depend on the CLI. `oneagent-protocol`
remains a placeholder and is not activated. The first client is implemented
with the Rust standard library and adds no production dependency.

### Library and executable boundary

The package gains a reusable library boundary plus the existing binary.

The library owns:

- closed command and request types;
- parsing and local validation;
- a request-executor abstraction used by focused tests;
- production blocking HTTP execution;
- output routing and exit classification;
- one top-level invocation function that accepts an argument iterator and
  caller-owned stdout/stderr writers.

Exact Rust type and function names may follow local conventions. Public APIs
must document errors and accepted behavior. The binary does only this:

1. collect `args_os()` excluding the executable name;
2. pass them and locked stdout/stderr handles to the library invocation;
3. return the accepted `ExitCode`.

The binary constructs no Runtime service, Tokio runtime, background task,
global state, or hidden dependency. Focused tests may inject an executor;
production selects the standard-library HTTP executor explicitly.

### Exact command grammar

The command name shown in help and diagnostics is `oneagent-cli`.

Global forms are exactly:

```text
oneagent-cli --help
oneagent-cli -h
oneagent-cli --version
oneagent-cli -V
oneagent-cli [--address <IP:PORT>] <command> [command options]
```

`--help`, `-h`, `--version`, and `-V` are accepted only as the sole argument.
They are not command-local options and do not override other tokens. There is
no `help` command. No combined short options or lowercase `-v` alias exists.

`--address <IP:PORT>` is optional and may appear exactly once only before the
command. It uses Rust `SocketAddr` text, so it accepts IPv4 or bracketed IPv6
numeric addresses with a port and rejects hostnames, schemes, paths, missing
ports, and zone identifiers. The default is `127.0.0.1:3000`, matching
`RuntimeConfig::default()`.

The exact commands are:

```text
oneagent-cli [--address <IP:PORT>] health <live|ready>

oneagent-cli [--address <IP:PORT>] configurations
    [--limit <1..100>]

oneagent-cli [--address <IP:PORT>] node
    --configuration-id <ID>
    --node-id <ID>

oneagent-cli [--address <IP:PORT>] relations
    --configuration-id <ID>
    --node-id <ID>
    --direction <incoming|outgoing>
    [--edge-kind <KIND>]
    [--limit <1..100>]

oneagent-cli [--address <IP:PORT>] traverse
    --configuration-id <ID>
    --node-id <ID>
    --direction <incoming|outgoing>
    --max-depth <0..4>
    [--edge-kind <KIND>]
    [--include-start]
    [--limit <1..100>]
```

Command options may appear in any order after the command. Every value option
uses two tokens, `--name <value>`; `--name=<value>` is unsupported. Every option
may appear at most once. `--include-start` is a valueless flag and maps to
`include_start=true`; omission delegates the accepted server default `false` by
omitting the query parameter.

There are no other short aliases, positional IDs, option abbreviations, command
aliases, case folding, or `--` terminator. Unknown commands/options, extra
positional tokens, duplicates, missing option values, and options invalid for a
command are usage errors.

All arguments must be valid Unicode. A non-Unicode argument is a usage error.
Configuration and node IDs must be non-empty and not whitespace-only, then are
preserved exactly. Direction and edge kind use the exact case-sensitive
ADR-0040 vocabularies. `limit` and `max-depth` use non-empty ASCII decimal
digits only, leading zeroes are accepted, and ranges match ADR-0040 exactly.
Local validation prevents unsupported requests but does not redefine the
server's accepted set.

Parsing is left-to-right. Structural grammar is validated before value
vocabularies or ranges. The CLI returns one closed usage diagnostic rather than
making individual parser prose a compatibility surface.

### Help and version

Help is a stable UTF-8 usage document that lists the exact forms, commands,
options, defaults, bounds, direction values, edge-kind values, and exit codes
accepted by this ADR. Its exact committed bytes are tested. It is written to
stdout with one terminal newline and exits `0` without constructing or invoking
an executor.

Version output is exactly:

```text
oneagent-cli 0.1.0
```

where the version is sourced at compile time from `CARGO_PKG_VERSION`. It is
written to stdout with one terminal newline and exits `0` without an executor.

No command-specific help, localization, color, pager, shell completion, or
dynamic server capability discovery is accepted.

### Command-to-request mapping

Every valid command produces exactly one GET request:

| CLI command | Exact path and query order |
| --- | --- |
| `health live` | `/health/live` |
| `health ready` | `/health/ready` |
| `configurations` | `/api/v1/configurations` then optional `limit` |
| `node` | `/api/v1/graph/node?configuration_id=...&node_id=...` |
| `relations` | `/api/v1/graph/relations?configuration_id=...&node_id=...&direction=...`, then optional `edge_kind`, then optional `limit` |
| `traverse` | `/api/v1/graph/traverse?configuration_id=...&node_id=...&direction=...&max_depth=...`, then optional `edge_kind`, optional `include_start=true`, then optional `limit` |

Omitted CLI options are omitted from the query so ADR-0040 server defaults stay
authoritative. Numeric values are emitted in canonical decimal form after local
parsing. IDs and other string values are percent-encoded from UTF-8 bytes using
RFC 3986 unreserved bytes `ALPHA / DIGIT / "-" / "." / "_" / "~"`; every
other byte is uppercase `%HH`. Spaces are `%20`, never `+`. Names and separators
are fixed ASCII and are never user-controlled.

The exact request is:

```text
GET <target> HTTP/1.1\r\n
Host: <SocketAddr>\r\n
Accept: application/json\r\n
Connection: close\r\n
\r\n
```

`SocketAddr::to_string()` supplies the Host value, including brackets for IPv6.
No user-controlled byte can create a header or request-target delimiter.

### Connection ownership and time bounds

One invocation owns at most one blocking `TcpStream`. It uses
`TcpStream::connect_timeout` with a fixed five-second timeout and sets fixed
five-second read and write timeouts. These constants bound an interactive local
client; they are compatibility behavior but not performance guarantees.

The client performs one connect, one complete request write, one bounded
response read, and then drops the stream on every success or failure. It sends
`Connection: close`, never retries, pools, redirects, upgrades, reconnects,
spawns, or detaches work. There is no cancellation protocol beyond process
termination and socket drop.

Tests do not wait for these timeout durations as acceptance evidence. Controlled
listeners, released ports, EOF, and bounded external test guards prove terminal
paths deterministically.

### HTTP/1.1 response contract

The client accepts exactly one final HTTP/1.0 or HTTP/1.1 response delimited by
connection close. It reads at most 64 KiB through and including the header
terminator and at most 16 MiB of body. Exceeding either bound is a protocol
failure. These are client resource bounds, not server schema or security claims.

The response head must:

- be ASCII;
- contain one `HTTP/1.0` or `HTTP/1.1` status line;
- contain exactly one three-digit status code;
- contain `\r\n\r\n` within the head bound;
- contain syntactically non-empty ASCII header names followed by `:`;
- contain no obsolete folded/continuation header line;
- have no `transfer-encoding` header;
- have zero or one valid decimal `content-length` header;
- contain exactly one `content-type` header whose trimmed ASCII value is
  `application/json` case-insensitively.

Header names are compared case-insensitively. Incidental headers are ignored
after syntax and bounds validation. Duplicate `content-type`, duplicate
`content-length`, invalid length, conflicting framing, NUL/control bytes, or an
unsupported HTTP version is a protocol failure.

The client reads until EOF because it requested connection close. If
`content-length` exists, it must exactly equal the received body length. Without
it, EOF is the accepted delimiter. A premature read error, timeout, missing EOF,
truncated declared body, or bytes beyond the declared length is a transport or
protocol failure according to whether the I/O operation or completed bytes
expose the fault.

The body must be non-empty valid UTF-8. The CLI does not parse or reserialize
JSON: the exact Runtime `application/json` body remains the wire authority. A
media-type-valid UTF-8 body is passed through. Semantic JSON/schema validation
would require a protocol model and dependency and remains deferred.

The only successful status is `200`. Any other syntactically valid response
with the accepted media type/body is a server response failure, including the
accepted `400`, `404`, and `503` domain rows. The CLI preserves its exact body;
it does not reinterpret error codes or messages. Unexpected statuses do not
become success or local parser errors.

### Output and newline contract

For `200`, the exact body bytes are written to stdout. For every non-`200`
server response, the exact body bytes are written to stderr. After the body, the
CLI adds one `\n` only when the body does not already end in `\n`; it never
removes or otherwise changes body bytes.

Help and version follow their exact stdout contracts. Local diagnostics go to
stderr. No logs, banners, request echo, status prose, headers, source paths,
Rust error names, OS error strings, `Debug` output, or backtraces are written by
the supported boundary.

Output write or flush failure is a local I/O failure. The library returns its
exit classification; the binary makes a best-effort attempt to write the static
output failure diagnostic to stderr and does not panic if stderr also fails.

### Closed diagnostics and exit codes

The exact local diagnostic bodies are:

Usage:

```text
oneagent-cli: usage_error: invalid command line
Try 'oneagent-cli --help' for usage.
```

Transport:

```text
oneagent-cli: transport_error: failed to communicate with runtime
```

Protocol:

```text
oneagent-cli: protocol_error: runtime response is invalid
```

Output:

```text
oneagent-cli: output_error: failed to write command output
```

Each body ends with exactly one newline. They intentionally omit unstable OS,
parser, socket, path, or source-chain prose.

The exact process exit map is:

| Exit | Meaning |
| ---: | --- |
| `0` | Help, version, or one `200` Runtime response was written successfully. |
| `2` | Local command-line usage error. |
| `3` | Connect, request-write, response-read, or socket timeout/transport failure. |
| `4` | One complete syntactically valid non-`200` Runtime response was written to stderr. |
| `5` | Completed response bytes violate the accepted HTTP/media/body contract. |
| `6` | Required stdout/stderr output could not be completed. |

Usage errors are detected before an executor exists or any connection attempt.
Transport failures precede response interpretation. A complete response is
framing/media/body validated before status classification or output. Output
failure supersedes the otherwise selected success/server exit because the
accepted observable result was not delivered.

### Public evidence contract

Focused command tests must cover:

- exact help/version bytes and zero executor calls;
- every command and optional/default combination;
- options in different accepted orders;
- every unknown, duplicate, missing, extra, misplaced, invalid-Unicode,
  invalid-ID, invalid-vocabulary, and range boundary case;
- exact typed requests, output streams, newline behavior, and exits;
- executor success/server/transport/protocol results and output failure;
- deterministic repeated invocation without shared state.

Focused controlled-server tests must cover:

- exact request bytes and query ordering;
- reserved/unicode ID percent encoding and IPv4/IPv6 Host syntax;
- accepted HTTP/1.0/1.1, optional exact content length, EOF delimiting, case-
  insensitive header names/media value, and incidental headers;
- success and non-success body passthrough;
- connect/write/read failure where deterministic;
- missing terminator, invalid status/version/header, duplicate framing/media,
  transfer encoding, invalid length/media/UTF-8, empty/truncated/extra/oversized
  body, cleanup, and repeated connections.

Public client/server evidence must use the real CLI invocation boundary, real
query-enabled Runtime, port zero, lifecycle/address observations, and the
tracked mixed EDT/Designer fixture. It must prove:

- liveness and ready commands;
- canonical two-configuration listing;
- exact node results from both formats;
- relation and traversal operations with required and optional inputs;
- limits/defaults, missing configuration/node, not-ready or unavailable server
  state where deterministically observable, and unreachable address;
- exact stdout/stderr/exit behavior;
- graceful Runtime shutdown, listener release, connection completion, and equal
  repeated/fresh runs.

Tests use watches/oneshots or equivalent event synchronization and bounded hang
guards. They use no arbitrary sleep, fixed port, real signal, external service,
ignored corpus, Unix-only process behavior, or host-global state. CI remains
macOS/Windows compatible. Managed loopback restrictions require only bounded
local-network permission.

### Compatibility

The accepted command names, option names/forms, global option placement,
defaults, value vocabularies, local diagnostics, stdout/stderr assignment,
newline behavior, request mapping/order/encoding, resource bounds, timeout
constants, and exit codes are the Sprint 21 supported CLI compatibility surface.

Within the current major CLI surface:

- adding a command or optional flag requires compatibility review and complete
  client/server evidence;
- removing/renaming a command or option, changing a default, accepted value,
  output stream, diagnostic, exit, request mapping, bound, or timeout is
  breaking and requires an explicit migration decision;
- an additive ADR-0040 response field remains opaque and passes through without
  a CLI change;
- a breaking `/api/v1` change requires version migration before this client can
  follow it;
- future protocol-crate migration must preserve these observed command/wire
  results or explicitly version the client surface.

The package version remains `0.1.0`; this ADR does not claim semantic versioning
stability, packaging, or installation support beyond repository-built binaries.

## First production slice

Sprint 21 implements only:

1. one reusable dependency-free CLI library boundary and thin executable;
2. exact help/version, address, health, configurations, node, relations, and
   traverse grammar;
3. exact local validation, typed requests, diagnostics, streams, and exits;
4. one blocking bounded HTTP/1.1 connection per invocation;
5. exact accepted request mapping/encoding and close-delimited response
   validation/passthrough;
6. focused command/client and public CLI-to-production-Runtime evidence;
7. truthful current-state documentation.

## Rejected alternatives

### Start or supervise Runtime from the CLI

Rejected. No process configuration, discovery, signal, restart, daemon, or
cross-platform supervision contract exists. Sprint 21 is a client of an already
running accepted endpoint.

### Add Clap and Reqwest because they are convenient

Rejected for the first slice. Standard-library implementation is complete and
testable; neither dependency is approved or locked as a direct CLI dependency.

### Reuse Hyper transitively from Axum

Rejected. A lockfile transitive is not a public direct dependency contract, and
it would still require manifest/API/runtime decisions.

### Activate `oneagent-protocol` and deserialize all JSON

Rejected. The crate has no accepted schema, and Runtime owns the v1 wire.
Opaque validated passthrough avoids duplicate protocol and semantic authority.

### Accept arbitrary URLs or hostnames

Rejected. Runtime configuration exposes `SocketAddr`, not URL, DNS, proxy, TLS,
or base-path behavior. Exact numeric addresses are sufficient for the first
local client.

### Support every HTTP/1.1 framing feature

Rejected. Production evidence uses connection close and fixed Axum JSON. Chunked
transfer, interim responses, trailers, compression, upgrades, persistence, and
proxy behavior are not required by the accepted server surface.

### Print human-readable tables or reinterpret server errors

Rejected. It would introduce a second evolving schema and unstable prose.
Exact JSON passthrough is deterministic, scriptable, and compatibility-safe.

### Delegate all invalid values to Runtime

Rejected. The closed CLI grammar must reject unsupported command lines before a
connection. Local validation exactly mirrors accepted vocabularies and bounds;
server errors remain authoritative after a valid request is sent.

### Retry failed requests

Rejected. Query idempotence does not establish retry timing, shutdown,
replacement-snapshot, user-expectation, or remote policy. One invocation makes
one request.

## Deferred scope

- Runtime process start/stop/supervision, endpoint discovery, environment or
  configuration files, Workspace open/mutation, watch/cache controls, streaming,
  subscriptions, progress, batch, pagination, and arbitrary queries;
- human/table/color/pager/localized output, shell completion, interactive UI,
  JSON interpretation, protocol schema migration, and alternate transports;
- URLs, DNS, proxies, redirects, authentication, authorization, TLS, HTTP/2,
  transfer encoding, compression, retries, connection pooling, configurable
  timeouts, cancellation, remote-server policy, and general version negotiation;
- Git/network Workspaces, MCP, LSP, IDE, AI/context, packaging/installers,
  releases, telemetry, metrics, tracing, benchmarks, performance targets,
  denial-of-service claims, and security certification.

## Implementation prerequisites

1. Implement the closed command/request/output/exit boundary with an injected
   executor and exhaustive focused tests; perform no network I/O in this task.
2. Implement the bounded standard-library HTTP client and connect it to the
   command executor; prove exact requests, response rules, failure categories,
   cleanup, and repetition with controlled listeners.
3. Add public real-CLI-to-real-Runtime evidence over the tracked mixed fixture,
   lifecycle/address observations, shutdown, and fresh repetition.
4. Synchronize current-state docs only after production and public evidence
   exist. Do not change semantic or source Coverage.
5. Run focused affected tests and the canonical complete workspace gate for
   every Rust, public API, manifest, or client/server compatibility change.

## Coverage Registry impact

None. Sprint 21 consumes already accepted Runtime and semantic behavior. It adds
no graph fact, query semantic, source parser, adapter capability, or Coverage
transition.

## Consequences

- OneAgent gains a small deterministic local CLI without changing Runtime or
  semantic authority.
- JSON output stays byte-compatible with accepted server responses and remains
  useful to scripts without a duplicated schema.
- Numeric endpoint input and bounded one-shot HTTP keep the first slice narrow,
  dependency-free, and cross-platform.
- Stable grammar, diagnostics, streams, limits, timeouts, and exits create an
  explicit compatibility surface future client work must version carefully.
- Process management, discovery, protocol migration, richer output, remote
  security, packaging, and alternate clients remain visible later decisions.
