# Sprint 21 CLI Client Integration Review

## Decision

`pass`

Sprint 21 satisfies ADR-0043 and the Roadmap completion gate. No blocking or
non-blocking findings and no missing acceptance evidence remain. The v0.4
release integration review is the unique next gate.

## Reviewed baseline

- Planning parent: `45c4473365a026f2acb83b4a6e9db0d8b2dbe2fb`
- Reviewed range: `677dbaaa^..2f055f43`
- Committed Task 5 head: `2f055f4338e81b6ab2ddec396da3f19d06329276`
- Review date: 2026-08-22

| Commit | Subject | Owned paths |
| --- | --- | --- |
| `677dbaaa3d64fe9fda352efc6925e9c067a52af7` | `Plan Sprint 21 CLI Client` | `docs/Roadmap.md`; the seven files in `docs/codex/prompts/sprint-21-cli-client/` |
| `fd94a696a216476d8b809643118ddc6ad16565cb` | `Investigate Sprint 21 CLI Client` | `docs/architecture/cli-client-investigation.md` |
| `17ce52ce96a08c7bd079bf77bcf70cceb000694e` | `Define Sprint 21 CLI Client contract` | `docs/adr/0043-cli-client.md` |
| `6d71c95a902767ebcfd9ec34005eb755b4f69f23` | `Implement Sprint 21 CLI command boundary` | `apps/cli/src/lib.rs`, `apps/cli/src/main.rs` |
| `23e20cb8fac2c51677ee2e31209450cfef951d8f` | `Implement Sprint 21 Runtime HTTP client` | `apps/cli/src/http.rs`, `apps/cli/src/lib.rs`, `apps/cli/src/main.rs` |
| `2f055f4338e81b6ab2ddec396da3f19d06329276` | `Complete Sprint 21 CLI Client evidence` | `Cargo.lock`, `README.md`, `apps/cli/Cargo.toml`, `apps/cli/tests/runtime_client.rs`, `docs/Architecture.md`, `docs/architecture/semantic-model-2.md` |

The range changes only the CLI client, its public production evidence,
development-only test dependencies, ADR/investigation/current-state documents,
Roadmap planning, and the Sprint 21 prompt suite. `oneagent-cli` still has no
normal production dependency. The range changes no Runtime route, semantic
schema, Workspace, watcher, cache, adapter, protocol, or later-sprint behavior.

## Acceptance evidence matrix

| Criterion | Evidence | Result |
| --- | --- | --- |
| Planning readiness | The committed plan starts from the completed Sprint 20 baseline, orders six prerequisite-gated tasks, and preserves the exact Sprint 20 retirement gate. | pass |
| Investigation | The repository-backed investigation inventories the placeholder CLI, Runtime routes and lifecycle, dependency feasibility, fixtures, platform constraints, consumers, and deterministic seams before selecting the contract. | pass |
| Accepted architecture | ADR-0043 fixes command ownership, grammar, endpoint, request encoding, response bounds, output, failure, exit, resource, compatibility, and deferred-scope contracts. | pass |
| Authority | Runtime remains authoritative for lifecycle, query semantics, JSON, status codes, and ordering; the CLI performs no semantic interpretation and `oneagent-protocol` remains inactive. | pass |
| Command grammar | Exact health, configurations, node, relations, and traverse commands accept only the closed option grammar and locally validate required values and bounds. | pass |
| Help and version | Sole-argument help and version paths are exact, deterministic, side-effect free, and covered through both injected unit seams and the real executable. | pass |
| Endpoint ownership | One optional numeric `SocketAddr` override precedes the command, the default is stable, IPv4 and IPv6 host headers remain exact, and no DNS or ambient configuration is introduced. | pass |
| Request mapping | Every command maps to one exact GET route; query fields retain the accepted order and RFC 3986 percent encoding with uppercase hexadecimal digits. | pass |
| HTTP framing and bounds | One blocking `TcpStream` owns fixed connect/read/write timeouts, bounded response head and body, connection-close framing, accepted HTTP versions, and exact JSON media validation. | pass |
| Output preservation | Successful Runtime JSON is passed through opaquely to stdout with one terminal newline; server JSON is preserved on stderr without client-side reserialization. | pass |
| Failures and exits | Usage, transport, server, protocol, and output failures have stable local diagnostics, streams, and exit codes, with no retry, redirect, pooling, TLS, or background work. | pass |
| Resource ownership | Every invocation owns one connection and terminates it on every success or failure path; repeated fresh invocations share no client state. | pass |
| Focused unit evidence | Eighteen CLI unit tests cover parsing, validation, exact requests, percent encoding, response parsing, limits, failures, streams, exits, and repeated execution. | pass |
| Public executable evidence | The two-test public target invokes `CARGO_BIN_EXE_oneagent-cli` against a real query-enabled Runtime and a bounded malformed-response server. | pass |
| Lifecycle compatibility | Liveness succeeds during initialization while readiness and queries preserve Runtime `503`; readiness and every query succeed only after complete startup. | pass |
| Production formats | The tracked mixed fixture proves canonical configuration listing and exact node responses for both Designer XML and EDT inputs. | pass |
| Graph operations | Exact node, direct relation, and bounded traversal commands preserve Runtime order, filtering, limits, truncation, and JSON. | pass |
| Domain and protocol errors | Missing configurations, unreachable endpoints, and malformed HTTP responses retain their accepted server, transport, and protocol classifications. | pass |
| Cleanup and repetition | Runtime shutdown releases the listener, child processes and connections terminate, and two fresh mixed-fixture runs return equal observations. | pass |
| Platform behavior | The client uses portable standard-library networking and byte handling; public tests use port-zero loopback, disposable roots, process timeouts only as hang guards, and repository CI retains macOS and Windows targets. | pass |
| Dependency approval | `cargo tree -p oneagent-cli --edges normal` contains only `oneagent-cli`; added Tokio, tempfile, Runtime, and serde_json entries are development-only public-test support. | pass |
| Compatibility | Existing Runtime, Workspace, Graph Query, File Watching, Persistent Cache, health, semantic, and adapter tests all pass unchanged. | pass |
| Documentation truth | README, Architecture, Semantic Model, Roadmap, investigation, ADR-0043, help text, and public evidence agree on the implemented first slice. | pass |
| Scope containment | The range adds no server mutation, authentication, TLS, DNS, configuration files, environment policy, streaming, retry, interactive mode, shell completion, protocol activation, or new production dependency. | pass |

## Findings

No blocking or non-blocking findings.

## Missing evidence

None.

The review counted only non-zero focused targets: 18 CLI unit tests, 2 public
CLI-to-Runtime tests, 4 public Runtime health tests, and 3 public Runtime Graph
Query tests.

## Validation

The review reran the focused client and compatibility matrix:

- `cargo test -p oneagent-cli --test runtime_client` — 2 passed.
- `cargo test -p oneagent-cli` — 18 unit and 2 public integration tests passed;
  zero-test binary and doctest harnesses also completed successfully.
- `cargo test -p oneagent-runtime --test http_health` — 4 passed.
- `cargo test -p oneagent-runtime --test graph_query_api` — 3 passed.
- `cargo tree -p oneagent-cli --edges normal` — only `oneagent-cli`.

The canonical complete gate also passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
- `git diff --check`

The managed sandbox denies loopback bind without additional local permission;
targets containing local TCP evidence ran with bounded loopback permission. No
external network or service was used.

## Deferred scope

Runtime mutation, Workspace management, interactive or batch mode, shell
completion, configuration files, environment-variable policy, aliases, color,
progress, streaming, pagination beyond the accepted bounded slice, retries,
redirects, pooling, DNS, TLS, authentication, protocol-crate activation,
machine-specific endpoint discovery, packaging, and release automation remain
deferred.

## Risk assessment

Residual risk is bounded to the accepted local first slice. The close-delimited
blocking client favors a small dependency-free contract over general HTTP
interoperability, supports only numeric socket addresses, and intentionally
passes JSON through without client-side validation. Public evidence exercises
real processes and Runtime routes on macOS; portable standard-library APIs,
unit coverage, and repository CI guard Windows behavior. These accepted limits
do not block ADR-0043.

## Previous-suite retirement

After the `pass` decision, `git ls-files` and the filesystem both contained
exactly the eight planned Sprint 20 prompt files and no additional or untracked
file. Repository search found no retained Markdown link dependency on an
individual deleted prompt. The exact suite is retired atomically with this
review; the complete Sprint 21 suite,
`docs/codex/prompts/run-next-sprint.md`, non-adjacent suites, and `.codex/`
remain unchanged.
