# Sprint 21 CLI Client Investigation

## Investigation status

Investigation complete at committed Sprint 21 planning head
`677dbaaa3d64fe9fda352efc6925e9c067a52af7`.

The [Sprint 20 review](../reviews/sprint-20-persistent-cache.md) records `pass`,
the Roadmap makes Sprint 21 CLI Client the unique `next` target, and the working
tree was clean before this document was created. This investigation changes no
Rust, Cargo manifest, fixture, Runtime contract, or sprint state.

## Objective

Establish whether the repository contains enough production contracts and
deterministic test evidence to accept and implement the first supported
`oneagent-cli` client, and enumerate every decision ADR-0043 must make before
production work starts.

## Evidence classification

- **Confirmed repository evidence** is observed directly in current code,
  manifests, fixtures, tests, Git history, or successful commands.
- **Accepted constraints** are fixed by ADR-0037 through ADR-0042.
- **Unresolved decisions** have complete repository-owned oracles but no
  accepted Sprint 21 contract.
- **Unsupported behavior** has neither an accepted contract nor sufficient
  evidence and cannot enter the first slice.

## Confirmed CLI baseline

`apps/cli` is one workspace package named `oneagent-cli`, version `0.1.0`, with
one binary source file and no library target, dependencies, dev-dependencies,
features, tests, configuration inputs, command model, error type, or reusable
execution seam. `main` prints exactly `Hello, world!` and always returns the
default successful process status.

`cargo tree -p oneagent-cli` lists only the package itself. The placeholder
therefore establishes no supported syntax, endpoint, output, exit, transport,
serialization, process-management, or compatibility contract.

Repository search finds no production consumer of `oneagent-cli`. README,
Architecture, Semantic Model, and the Roadmap consistently describe it as a
placeholder owned by Sprint 21. Git history shows no previous supported client
implementation to preserve.

The Rust standard library provides all primitives needed for a bounded first
slice without a new production dependency:

- `std::env::args_os` for exact process arguments;
- `std::net::SocketAddr` and `TcpStream` for a typed endpoint and blocking
  HTTP/1.1 connection;
- read/write timeouts and `Read`/`Write` for bounded termination;
- `std::io::Take` or explicit length accounting for response containment;
- `ExitCode` or an equivalent `Termination` boundary for stable process status;
- injected readers/writers or byte buffers for deterministic stream evidence.

This proves feasibility, not an accepted choice. ADR-0043 must still select the
library/main boundary, command grammar, endpoint syntax, timeouts, request and
response rules, body limit, diagnostics, output streams, and exit codes.

## Accepted Runtime server inputs

### Ownership and lifecycle

ADR-0037 makes Runtime the owner of registered service tasks, cancellation,
reverse cleanup, and canonical lifecycle. ADR-0038 makes one `HttpService` the
sole listener owner and canonical lifecycle the sole readiness authority.
ADR-0039 publishes one complete immutable Workspace snapshot containing
separate configurations. ADR-0040 adds a transport-neutral Graph Query service
and exact HTTP adapter. ADR-0041 and ADR-0042 replace complete snapshots after
source changes and restore validated cache hits without changing successful
query wires.

A CLI client can consume these public contracts but cannot start, stop,
configure, supervise, rebuild, mutate, or become an authority for any of them
without a new accepted server contract.

### Endpoint configuration

`RuntimeConfig` owns one typed `SocketAddr`. Its default is
`127.0.0.1:3000`; port zero is accepted for tests. The production
`DefaultConfigurationProvider` returns this fixed default. Runtime has no
environment, file, URL, hostname, DNS, CLI, service-discovery, proxy, TLS, or
hot-configuration provider.

The CLI therefore has one evidence-backed candidate endpoint vocabulary:
`SocketAddr`, with an accepted default equal to the Runtime default and an
explicit per-invocation override. URI schemes, paths, hostnames, IPv6 zone
identifiers, environment variables, configuration files, project-local state,
and automatic Runtime discovery remain unresolved or unsupported.

### Stable operations

The complete current public GET surface relevant to Sprint 21 is:

| Capability | Exact request path | Accepted result |
| --- | --- | --- |
| Liveness | `/health/live` | `200` and `{"status":"alive"}` while reachable |
| Readiness | `/health/ready` | `200`/`{"status":"ready"}` while Running; otherwise `503`/`{"status":"not_ready"}` |
| Workspace configurations | `/api/v1/configurations?limit=<limit>` | deterministic bounded configuration list |
| Exact node | `/api/v1/graph/node?configuration_id=<id>&node_id=<id>` | one owned node projection |
| Direct relations | `/api/v1/graph/relations?...` | deterministic incoming/outgoing bounded relation list |
| Bounded traversal | `/api/v1/graph/traverse?...` | deterministic breadth-first bounded node list |

ADR-0040 fixes parameter names, required/optional status, values, defaults,
bounds, percent-decoding, response schemas, enum vocabularies, status codes,
error codes/messages, and ordering. The client must not rename parameters,
invent aliases, alter defaults, reinterpret direction, merge configurations,
or deserialize graph facts into a second semantic model.

The relevant input rules include:

- `limit` defaults to `50` and accepts ASCII decimal `1..=100`;
- `max_depth` is required for traversal and accepts `0..=4`;
- `include_start` is exact `true` or `false`, defaulting to `false`;
- direction is exact `incoming` or `outgoing`;
- edge kind is one optional exact closed ADR-0040 string;
- configuration and node IDs are arbitrary decoded non-empty,
  non-whitespace-only UTF-8 strings and are not UUID-only;
- unknown, duplicate, malformed, missing, or unsupported query values have
  exact Runtime-owned JSON error rows.

ADR-0043 must decide which of these validations occur locally for predictable
CLI usage and which are deliberately delegated to Runtime. Local validation
must not change the accepted value set or obscure a server response after a
request is made.

### HTTP compatibility and observed framing

ADR-0038/0040 accept HTTP/1.1 raw-loopback behavior. Requests use exact GET
paths. Success and domain errors are JSON; object-member order and incidental
transport headers are not compatibility. Health and Graph Query public tests
send:

```text
GET <target> HTTP/1.1\r\n
Host: localhost\r\n
Connection: close\r\n
\r\n
```

They read until EOF, split at the first `\r\n\r\n`, parse the numeric status and
case-insensitive headers, and compare the remaining bytes as the body. All
accepted Axum JSON responses expose `content-type: application/json` in those
tests. The tests intentionally make transport headers such as content length or
transfer encoding non-authoritative.

Consequently, `Connection: close` plus bounded read-to-EOF is the only complete
repository-proven response-termination seam for the first client. Existing
evidence does not require a persistent connection or prove a general proxy,
redirect, interim-response, trailer, compression, upgrade, HTTP/2, or arbitrary
chunked-server compatibility contract. ADR-0043 must choose whether to accept
only the framing the production Runtime emits under connection close or add
controlled-server evidence for any broader HTTP/1.1 framing it accepts.

A supported client also needs an explicit maximum response-head/body size,
header parsing policy, status-line version/status rules, UTF-8/body policy,
media-type rule, malformed/truncated response classification, and read/write
timeouts. None is currently accepted. Unbounded `read_to_end`, panicking parse
helpers, or printing partial bytes as success would be unsafe production
behavior even though they are adequate inside bounded test helpers.

## Output, errors, and process behavior

No repository document currently decides CLI-visible text or exit codes.
Accepted Runtime JSON is already the complete stable machine-readable result.
Opaque byte-preserving passthrough is feasible without Serde and avoids a
second protocol/schema authority, but ADR-0043 must decide:

- whether successful JSON goes to stdout and Runtime domain-error JSON to
  stderr or whether all server JSON stays on stdout;
- whether exactly one terminal newline is added while preserving body bytes;
- whether help/version are stdout and local diagnostics are stderr;
- exact stable local diagnostic strings and validation precedence;
- distinct exit codes for success, usage, transport, malformed protocol, and
  non-success Runtime responses;
- whether `503` readiness responses are ordinary server-domain failures;
- whether an empty, non-JSON, oversized, truncated, or unexpected-status body is
  protocol failure rather than a Runtime domain error;
- how broken output pipes and other local I/O failures are classified.

The CLI must not print Rust `Debug`, OS error prose as a stable schema, source
paths, backtraces, partial server bytes, or adapter diagnostics unless ADR-0043
explicitly accepts a bounded diagnostic contract.

## Candidate command grammar

Repository capabilities support exactly the following command families; this
table is a decision input, not an accepted syntax:

| Candidate command | Server operation | Required values | Optional values |
| --- | --- | --- | --- |
| `health live` | liveness | none | endpoint |
| `health ready` | readiness | none | endpoint |
| `configurations` | list configurations | none | endpoint, limit |
| `node` | exact node | configuration ID, node ID | endpoint |
| `relations` | direct relations | configuration ID, node ID, direction | endpoint, edge kind, limit |
| `traverse` | bounded traversal | configuration ID, node ID, direction, max depth | endpoint, edge kind, include start, limit |

ADR-0043 must define exact spelling and placement, `--option value` versus
`--option=value`, option ordering, global versus command-local endpoint,
duplicate handling, `--` handling, non-Unicode arguments, missing values,
unknown commands/options, help/version precedence, short aliases, and whether
server bounds are checked locally.

There is no evidence for a command that opens a Workspace, starts Runtime,
controls watching/cache, reads configuration files, streams changes, mutates
state, or performs an arbitrary graph query.

## Dependency and protocol inventory

`oneagent-cli` has no direct dependency. `Cargo.lock` contains Hyper through
Axum, but a transitive lock entry is not authorization or a usable direct API.
Adding Hyper, Tokio, Reqwest, Serde, Serde JSON, Clap, a URL crate, or another
production dependency would require a manifest change and explicit approval.

`oneagent-protocol` contains only `component_name()` and no schema or dependency.
ADR-0040 intentionally keeps accepted wire projections in Runtime and defers
protocol migration. A raw JSON passthrough client does not require protocol
activation. Moving or duplicating schema types into the protocol crate would be
a separate compatibility and dependency decision, not incidental CLI work.

Repository-local dev-dependencies may be needed for public client/server tests,
but Task 5 must justify each exact dependency and keep production dependency
count unchanged unless separately approved.

## Consumers and compatibility constraints

| Owner or consumer | Constraint on Sprint 21 |
| --- | --- |
| Runtime lifecycle and HTTP health | Client cannot redefine readiness, service startup, shutdown, listener ownership, or health JSON. |
| Graph Query HTTP API | Client must preserve exact v1 routes, parameters, values, schemas, errors, bounds, and order. |
| Workspace Service | Configuration selection remains explicit by canonical ID; no merge, open, or mutation operation exists. |
| File Watching | Client observes later query results only; no watch-control or subscription route exists. |
| Persistent Cache | Cache stays private and has no management/status CLI surface. |
| Graph and source adapters | Client is a consumer, never a fact, validation, parsing, or resolution authority. |
| `oneagent-protocol` | Placeholder status remains unless a separately accepted compatibility task activates it. |
| Future MCP/LSP/IDE/AI clients | They cannot infer shared protocol authority from a CLI-private implementation. |

## Repository-owned fixtures and deterministic oracles

The tracked `apps/runtime/tests/fixtures/workspace_service/` root is the public
production oracle. Its README records exact provenance and SHA-256 inventory.
Production discovery yields, in canonical ID order:

| Format | Configuration ID | Name | Graph observation |
| --- | --- | --- | --- |
| Designer XML | `408a41e7-907a-4fb3-8999-83d1e8b6e093` | `DNSWorldEdition` | 4 nodes, 3 edges |
| EDT | `50000000-0000-0000-0000-000000000000` | `WritesFixture` | 13 nodes, 14 edges, diagnostics and one request |

Existing public Runtime tests prove real loopback behavior without arbitrary
sleeps or fixed ports:

- `cargo test -p oneagent-runtime --test graph_query_api -- --list` lists 3
  non-zero production/wire/lifecycle tests;
- `cargo test -p oneagent-runtime --test http_health -- --list` lists 4 non-zero
  health/lifecycle/failure/repetition tests.

They use port-zero loopback listeners, bound-address watches, lifecycle watches,
oneshot synchronization, timeout hang guards, temporary fixture copies, exact
raw HTTP requests, graceful shutdown, listener rebind, and repeated fresh apps.
CI runs on `macos-14` and `windows-latest`.

The following deterministic Sprint 21 oracles require no external data:

| Concern | Oracle |
| --- | --- |
| Grammar and precedence | Inject argument vectors and a recording executor; assert typed request or exact local outcome and zero executor calls on local failures. |
| Streams and exits | Inject byte writers; assert exact stdout, stderr, newline, and exit classification. |
| Exact requests | Controlled loopback server records complete request bytes and returns one bounded response. |
| Encoding | IDs containing spaces and reserved query bytes are compared with the exact recorded target and Runtime decoding behavior. |
| Framing and bounds | Controlled responses cover accepted close framing plus every ADR-accepted malformed, truncated, oversized, wrong-media, and status case. |
| Production operations | Real query-enabled Runtime plus the tracked mixed fixture proves health, configurations, both exact nodes, relations, and traversal. |
| Runtime domain failures | Real Runtime proves not-ready, missing configuration/node, invalid bounds/value rows as selected by the ADR. |
| Transport failure | An explicitly released port or controlled closed listener yields deterministic connect failure without an external service. |
| Cleanup | Runtime listener rebind and controlled-server join prove both sides release sockets; connection EOF proves client completion. |
| Repetition | Equal command results across repeated calls and two fresh Runtime apps prove independence and ordering. |

Public evidence may test a reusable CLI library boundary in-process and must
also prove the real executable/main exit boundary when the toolchain exposes a
deterministic binary path. ADR-0043 must select a cross-platform method rather
than assume Unix process signals or shell behavior.

Managed sandboxes may deny loopback bind. Such validation needs bounded local-
network permission, exactly as current Runtime reviews record; external network
access is never required.

## Confirmed unsupported or deferred behavior

- Runtime start/stop/supervision, daemonization, PID files, signal forwarding,
  auto-start, endpoint discovery, environment/config files, and Workspace open;
- mutation, edit transactions, watch/cache management, streaming,
  subscriptions, progress, pagination, batch or arbitrary query commands;
- JSON semantic deserialization, alternate human/table output, color, pager,
  interactive UI, shell completion, localization, and stable shell scripting
  beyond the exact accepted JSON/exit contract;
- hostnames/DNS, URL schemes/paths, proxies, redirects, authentication,
  authorization, TLS, HTTP/2, compression, retries, connection pooling, and
  remote/server security policy;
- protocol-crate migration, MCP, LSP, IDE, AI/context, Git/network Workspace,
  packaging/installers/releases, telemetry, benchmarks, performance, and
  security certification.

## ADR-0043 decision matrix

ADR-0043 must close every row before implementation:

| Area | Required decision |
| --- | --- |
| Authority | CLI/library/main owner, Runtime/wire authority, dependency direction, and no-second-semantic/protocol-authority rule. |
| Grammar | Exact commands, subcommands, option spellings/forms/order, required/optional/default values, duplicates, unknowns, `--`, and non-Unicode behavior. |
| Help/version | Exact triggers, precedence, stdout/stderr, content stability, version source, exit, and whether a connection is prohibited. |
| Endpoint | Exact type, default, override position, address scope, parsing/validation, and unsupported discovery/URL behavior. |
| Request mapping | One exact route per command, query parameter order, percent encoding, Host/Connection headers, bounds, and local-versus-server validation. |
| Response | Accepted HTTP version/status/header syntax, close/content framing, maximum head/body sizes, media type, UTF-8/JSON boundary, empty/truncated/oversized behavior. |
| Presentation | Byte preservation, terminal newline, stdout/stderr ownership, broken-pipe/local I/O behavior, and absence of semantic reformatting. |
| Failures and exits | Stable local diagnostics, validation precedence, server JSON, transport/protocol/local-I/O categories, exact exit codes, and unexpected status handling. |
| Resources | Blocking owner, connect/read/write termination, timeouts, connection close, no retries/pool/tasks/globals, and repeated invocation behavior. |
| Dependencies | Standard library implementation versus any explicitly approved production addition; dev-only public-test dependencies and protocol-crate status. |
| Compatibility | Relationship to ADR-0038/0040 wires, additive CLI evolution, breaking grammar/output/exit changes, and future protocol migration. |
| Evidence | Non-zero focused grammar/client tests and public executable-to-production-Runtime mixed-fixture matrix on macOS/Windows seams. |
| Deferred scope | Process management, configuration/discovery, mutation/watch/cache control, alternate output/transports, security, packaging, performance, and later integrations. |

## Decision readiness

The repository contains enough stable Runtime wire behavior, typed endpoint
configuration, public library construction, tracked production fixtures,
loopback/lifecycle synchronization, dependency-free client primitives,
controlled failure seams, and cross-platform CI to accept and test a bounded
first CLI Client without external data or a new production dependency.

The unresolved items are architecture choices with observable local or
repository-owned oracles. ADR-0043 is ready only if it closes every matrix row
without broadening the accepted Runtime API or inventing discovery, transport,
protocol, packaging, security, or performance claims. If it cannot, Task 3 must
stop rather than guess.
