# Sprint 15 Runtime Service Container Integration Review

## Decision

`pass`

Sprint 15 satisfies ADR-0037 and the Roadmap completion gate. No blocking or
non-blocking findings and no missing acceptance evidence remain. Sprint 16 HTTP
API and Health may become the unique `next` target.

## Reviewed baseline

- Planning parent: `bac838be07bbf9b9686e60419397e91e702adec1`
- Reviewed range: `9c2beeaa^..f5c3f5e5`
- Task 5 head: `f5c3f5e5b8eee3280a0c20b16b66b4db4acba514`
- Review date: 2026-08-21

| Commit | Outcome |
| --- | --- |
| `9c2beeaa7b8968f9279722607d8e5da138f03bb9` | Sprint 15 execution plan and ordered prompt suite |
| `5ac95d4cf1b27149a4df2d3ba0cfc667e3c00ea8` | Live Runtime ownership, dependency, failure, and testability investigation |
| `866782f1d7da0595ddee923b5ce184b4d130e448` | Accepted ADR-0037 contract |
| `b0d0ae11a7d5a31a810bd10fee6ebef23f20ae45` | Public service-container, cancellation, ownership, errors, and focused unit evidence |
| `84317cbfd13d76ff3fca11be73d62a64439a16ab` | Async App lifecycle, injected shutdown, and production `ctrl_c` integration |
| `f5c3f5e5b8eee3280a0c20b16b66b4db4acba514` | Public integration matrix and current-state documentation |

The range changes Runtime code/tests and the bounded architecture, Roadmap, and
current-state documents. It adds no dependency and changes no semantic graph,
source-adapter, protocol, persistence, CLI, or later-sprint implementation.

## Acceptance evidence matrix

| Criterion | Evidence | Result |
| --- | --- | --- |
| Composition ownership | `AppBuilder` owns configuration and ordered registration; built `App` owns immutable state, lifecycle, and one container; `main.rs` supplies only defaults and `ctrl_c`. | pass |
| Service identity and registration | Explicit owned non-empty names, synchronous duplicate rejection, stable name access in Runtime errors, and ordered registry tests. | pass |
| Startup and rollback | Sequential asynchronous acknowledgements, concurrent observation of already-started tasks, named start failure, reverse cancellation/join rollback, and public partial-start evidence. | pass |
| Task/resource ownership | Startup futures have owned join handles; acknowledged tasks live in one container-owned `JoinSet`; every App terminal path completes cleanup before return. | pass |
| Cancellation and shutdown | Receiver-only per-service cancellation sources stay container-owned; reverse one-at-a-time cancellation and join are covered by unit and public probes. | pass |
| Running behavior | The public App remains pending after startup and before injected shutdown; empty production composition waits on cross-platform `ctrl_c`. | pass |
| Failure propagation | Startup error, unexpected `Ok`, service error, task panic/join error, shutdown-source error, and secondary cleanup context have distinct typed Runtime classifications. | pass |
| Lifecycle | `Initializing -> Running -> Stopping -> Stopped` is observable; startup failure may enter `Stopping`; controlled probes prove `Stopping` precedes task release and `Stopped` follows join. | pass |
| Determinism and cleanup | Channels provide startup/cancellation/stop acknowledgements; channel closure proves probe tasks do not survive `App::run`; two fresh builds/runs remain independent. | pass |
| Cross-platform boundary | Tests use in-memory Tokio channels and timeout guards only, with no real signals, sleeps, sockets, filesystem fixtures, or Unix-only behavior. CI remains macOS/Windows compatible. | pass |
| Scope containment | No HTTP route, health/readiness schema, workspace/graph service, watcher, persistence, CLI behavior, global state, DI framework, or new dependency was added. | pass |
| Documentation and state | README, Architecture, Roadmap, Semantic Model ownership text, ADR-0037, and investigation evidence agree on implemented versus deferred behavior. | pass |

## Findings

No blocking or non-blocking findings.

## Missing evidence

None.

Filtered zero-test reports from focused commands were not counted as evidence:
the `service::` and `app::` filters each matched eight library unit tests but
zero binary/public-integration tests, while the exact public integration target
independently matched six tests.

## Validation

The review reran every focused command required by Tasks 3-5:

- `cargo test -p oneagent-runtime service:: -- --list` — 8 library tests;
  accompanying binary and integration targets matched zero and were excluded.
- `cargo test -p oneagent-runtime service::` — 8 passed.
- `cargo test -p oneagent-runtime app:: -- --list` — 8 library tests;
  accompanying binary and integration targets matched zero and were excluded.
- `cargo test -p oneagent-runtime app::` — 8 passed.
- `cargo test -p oneagent-runtime --test service_container -- --list` — 6 tests.
- `cargo test -p oneagent-runtime --test service_container` — 6 passed.
- `cargo test -p oneagent-runtime --no-fail-fast` — 17 unit and 6 public
  integration tests passed; binary and doctest targets contained zero tests.

The complete gate also passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
- `git diff --check`

## Deferred scope

The accepted cooperative service contract has no bounded startup or graceful
shutdown timeout. HTTP listener and public health/readiness behavior remain
Sprint 16; workspace lifecycle, graph queries, file watching, persistence, and
supported CLI behavior remain Sprints 17-21. These are explicit boundaries, not
missing Sprint 15 evidence.

## Previous-suite retirement

Before review outputs, both `git ls-files` and the filesystem contained exactly
the nine planned Sprint 14 prompt files and no additional or untracked file.
Repository search found no retained Markdown link dependency on an individual
file. The exact suite is retired atomically with this review; the Sprint 15
suite and `docs/codex/prompts/run-next-sprint.md` remain tracked.
