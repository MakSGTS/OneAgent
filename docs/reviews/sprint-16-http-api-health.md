# Sprint 16 HTTP API and Health Integration Review

## Decision

`pass`

Sprint 16 satisfies ADR-0038 and the Roadmap completion gate. No blocking or
non-blocking findings and no missing acceptance evidence remain. Sprint 17
Workspace Service may become the unique `next` target.

## Reviewed baseline

- Planning parent: `8ca1c0ce3c83dae8bb76fa52a40423bead693f40`
- Reviewed range: `7a6e8a58^..472eea8f`
- Task 5 head: `472eea8f2a0a14bf76901c4614c3c1ad54cb2b03`
- Review date: 2026-08-21

| Commit | Outcome |
| --- | --- |
| `7a6e8a5859364b066fe4ffed56acc5365f07bef3` | Sprint 16 execution plan and ordered prompt suite |
| `fd680fa9ce1fe3fe69d9b51c9915f2c14322da93` | Live Runtime, dependency, HTTP boundary, ownership, and testability investigation |
| `f751aa6a0e22e96a6160d24325251ac5b232612c` | Accepted ADR-0038 contract |
| `014ebbcdf0a50667503b7bcfca162b2caa62c11f` | Transport-neutral lifecycle-derived Runtime health state |
| `c9d79ee360bb64aada1d73dfcadda3e64a371f9a` | Runtime-owned HTTP service, probe routes, startup failure, and graceful shutdown |
| `472eea8f2a0a14bf76901c4614c3c1ad54cb2b03` | Public loopback evidence matrix and current-state documentation |

The range changes Runtime code and tests plus the bounded architecture,
Roadmap, prompt, and current-state documents. It adds no dependency and changes
no workspace, semantic graph, source-adapter, protocol, persistence, supported
CLI, or later-sprint implementation.

## Acceptance evidence matrix

| Criterion | Evidence | Result |
| --- | --- | --- |
| Listener ownership | `HttpService` binds before startup acknowledgement, transfers the listener into one Runtime-owned Axum task, and publishes only the observed bound address. | pass |
| Configuration | The default is typed `127.0.0.1:3000`; `AppBuilder::with_http_bind_address` supports explicit addresses and port zero without a string-parsing boundary. | pass |
| Health authority | `RuntimeHealth` is a transport-neutral projection of the canonical lifecycle watch; no separately mutable readiness label exists. | pass |
| Liveness and readiness | Live returns `200` with `{"status":"alive"}` while reachable; ready returns `200` only in `Running` and `503` in observable `Initializing` and `Stopping`. | pass |
| Exact wire contract | Public raw-TCP requests prove exact status, JSON media type, closed bodies, GET-only routes, `405` plus `Allow: GET`, and empty exact-path `404` responses. | pass |
| Startup failure | An occupied loopback address produces named `ServiceStartFailed` for `http`, publishes no address, and reaches terminal `Stopped`. | pass |
| Cancellation and cleanup | Runtime cancellation drives Axum graceful shutdown; reverse cleanup retains and joins the HTTP task before listener release. | pass |
| Determinism and repetition | Channel acknowledgements define lifecycle assertions; separately built port-zero applications repeat equal responses and independently release their listeners. | pass |
| Public boundary | `apps/runtime/tests/http_health.rs` imports only the public library API and exercises real loopback client/server behavior without arbitrary sleeps. | pass |
| Existing service contract | The Sprint 15 public service-container matrix remains green, preserving startup, rollback, failure, shutdown, and no-detached-task behavior. | pass |
| Cross-platform scope | The implementation uses Tokio/Axum and loopback TCP only, with no Unix-only signal or socket assumption in the public evidence. | pass |
| Scope containment | No workspace service, graph-query endpoint, watcher, cache, supported CLI behavior, auth, TLS, metrics, global registry, forced shutdown, or new dependency was added. | pass |
| Documentation and state | README, Architecture, Roadmap, Semantic Model, ADR-0038, and investigation evidence agree on implemented and deferred behavior. | pass |

## Findings

No blocking or non-blocking findings.

## Missing evidence

None.

Filtered zero-test reports were not counted as evidence. The focused `health`,
`lifecycle`, and `http` filters matched their relevant library or public tests;
the exact public integration targets independently established their own test
inventories.

## Validation

The review reran the focused commands required by Tasks 3-5:

- `cargo test -p oneagent-runtime health` — 4 relevant library tests and 3
  matching public HTTP tests passed; zero-match targets were excluded.
- `cargo test -p oneagent-runtime lifecycle` — 6 relevant library tests passed;
  zero-match public targets were excluded.
- `cargo test -p oneagent-runtime http` — 4 library and 4 public HTTP tests
  passed.
- `cargo test -p oneagent-runtime --test http_health -- --list` — exactly 4
  tests.
- `cargo test -p oneagent-runtime --test http_health` — 4 passed.
- `cargo test -p oneagent-runtime --test service_container` — 6 passed.

The complete gate also passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
- `git diff --check`

## Deferred scope

Workspace lifecycle and semantic-build orchestration remain Sprint 17; graph
and semantic query APIs, file watching, persistent cache, and supported CLI
behavior remain Sprints 18-21. TLS, authentication, authorization, CORS,
compression, metrics, tracing export, OpenAPI, streaming, request bodies,
domain error mapping, retries, forced termination, and a newly selected
shutdown timeout also remain outside the accepted first slice. These are
explicit boundaries, not missing Sprint 16 evidence.

## Previous-suite retirement

Before review outputs, both `git ls-files` and the filesystem contained exactly
the seven planned Sprint 15 prompt files and no additional or untracked file.
Repository search found no retained Markdown link dependency on an individual
file. The exact suite is retired atomically with this review; the Sprint 16
suite and `docs/codex/prompts/run-next-sprint.md` remain tracked.
