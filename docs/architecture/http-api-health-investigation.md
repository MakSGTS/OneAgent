# HTTP API and Health Investigation

## Purpose

This document records the live repository and locked-dependency evidence needed
to decide ADR-0038 and implement the smallest safe Sprint 16 HTTP liveness and
readiness slice. It does not accept endpoint semantics, add an HTTP service, or
change production behavior.

## Confirmed repository evidence

### Runtime composition and lifecycle

- `apps/runtime/src/lib.rs` exposes the reusable `App`, `AppBuilder`,
  `AppState`, `LifecycleState`, service-container, cancellation, configuration,
  and error boundaries. There is no public HTTP type or route.
- `apps/runtime/src/main.rs` is the only process entry point. It configures and
  builds an empty service registry, then awaits `App::run` with
  `tokio::signal::ctrl_c()`; no production service is registered.
- `apps/runtime/src/app/builder.rs::AppBuilder` is the accepted construction
  path. It owns configuration and ordered service registrations, creates one
  immutable `Arc<AppState>`, builds the service container around that same
  state, and transfers both to `App`.
- `apps/runtime/src/service/definition.rs::ServiceContext` gives a starting
  service an `Arc<AppState>` and receiver-only `Cancellation`. A service start
  future acknowledges startup by returning its owned `ServiceTask`.
- `apps/runtime/src/service/container.rs` calls service starts sequentially,
  treats a start error as named `RuntimeError::ServiceStartFailed`, owns each
  returned task in one `JoinSet`, requests cancellation in reverse registration
  order, and joins every acknowledged task before returning.
- `apps/runtime/src/app/runtime.rs::App::run` starts all registered services
  while lifecycle is `Initializing`, transitions to `Running` only after every
  startup acknowledgement, then awaits the injected shutdown source or a
  service terminal outcome. It transitions to `Stopping` before cleanup and to
  `Stopped` after cleanup.
- `apps/runtime/src/app/lifecycle.rs::Lifecycle` owns the canonical
  `tokio::sync::watch::Sender<LifecycleState>`. Public
  `App::subscribe_lifecycle` returns a receiver, but `AppState` and
  `ServiceContext` currently expose no lifecycle observation. An HTTP handler
  therefore cannot derive readiness from canonical lifecycle evidence without
  a bounded Runtime API change.
- `apps/runtime/src/state/mod.rs::AppState` owns only immutable
  `RuntimeConfig`. Adding a receiver or transport-neutral snapshot derived from
  the existing lifecycle watch can preserve immutable shared-state ownership;
  adding an independently writable readiness flag would create a second state
  authority.

### Current configuration and errors

- `apps/runtime/src/config/mod.rs::RuntimeConfig` contains only
  `application_name` and `environment`. `RuntimeConfig::new` takes those two
  strings, and `Default` supplies `OneAgent Runtime` and `development`.
- `apps/runtime/src/config/provider.rs::DefaultConfigurationProvider` returns
  `RuntimeConfig::default`; no environment, file, or command-line HTTP address
  is currently parsed.
- `apps/runtime/src/error/mod.rs::RuntimeError` already separates invalid
  service identity, duplicate identity, named startup failure, named execution
  failure, unexpected success, task-join failure, and shutdown-source failure.
  A listener bind error returned by HTTP service startup fits the existing named
  `ServiceStartFailed` path without a new Runtime error kind. A server future
  error after acknowledgement fits named `ServiceFailed`.
- Existing `Display` strings are diagnostics and ADR-0037 explicitly excludes
  them from wire compatibility. Sprint 16 must define HTTP bodies independently
  rather than serializing `RuntimeError` or its message.

### Locked HTTP and asynchronous APIs

`apps/runtime/Cargo.toml` and `cargo tree -p oneagent-runtime --depth 1` confirm
the direct locked surface: Axum 0.8.9, Tokio 1.53.0, Serde 1.0.228, Serde JSON
1.0.150, tracing, and tracing-subscriber. No dependency addition is required.

- Axum 0.8.9 `src/routing/mod.rs::Router::new` has a default 404 fallback and
  composes method routes without an external service framework.
- Axum 0.8.9 `src/json.rs::Json<T>` serializes response values through Serde and
  establishes the JSON media type. A closed derived response struct can define
  an exact small wire schema.
- Axum 0.8.9 `src/serve/mod.rs::serve` accepts a Tokio listener and a completed
  `Router<()>`. `Serve::local_addr` and
  `WithGracefulShutdown::local_addr` expose the bound address.
- Axum 0.8.9 `Serve::with_graceful_shutdown` accepts a `Send + 'static` future.
  Its implementation stops accepting after the signal, drops the listener, and
  waits for connection tasks to finish before completion.
- The same Axum source documents that `serve` and graceful serve return
  `io::Result<()>` but do not ordinarily return listener-loop errors; accepted
  architecture must not claim an executable runtime serve-error case that this
  concrete API cannot produce. Bind error remains executable.
- Tokio 1.53.0 `tokio::net::TcpListener::bind` acquires a listener asynchronously,
  and `TcpListener::local_addr` exposes the actual loopback address selected for
  port zero. Binding before returning the `ServiceTask` makes acquisition part
  of service startup acknowledgement.
- Tokio 1.53.0 `tokio::net::TcpStream::connect` plus
  `tokio::io::{AsyncReadExt, AsyncWriteExt}` can send and read raw HTTP/1.1 over
  loopback. Tests therefore need no HTTP client or `tower` dependency.
- Existing Tokio `oneshot`, `mpsc`, and `watch` channels and bounded
  `tokio::time::timeout` provide deterministic coordination and hang guards.

### Consumers, compatibility, and CI

- Repository search finds no `axum::`, `Router`, public Runtime health,
  readiness, or liveness implementation. The Axum dependency is present but
  unused.
- Current Runtime consumers are `apps/runtime/src/main.rs`, Runtime unit tests,
  and `apps/runtime/tests/service_container.rs`. Other workspace crates do not
  import the Runtime library.
- `apps/runtime/tests/service_container.rs` is a public six-test target using
  only in-memory services. It proves startup, rollback, failures, requested
  shutdown, reverse cleanup, and repeated fresh runs, but no socket or wire
  behavior.
- `cargo test -p oneagent-runtime -- --list` at planning commit `7a6e8a585936`
  lists 17 library tests, zero binary tests, six public service-container tests,
  and zero doc tests.
- `.github/workflows/ci.yml` runs format, workspace check, workspace tests, and
  workspace Clippy on macOS 14 and Windows. HTTP evidence must use TCP loopback,
  not Unix sockets, signals, descriptor assumptions, or fixed-port availability.
- `README.md`, `docs/Architecture.md`, and
  `docs/architecture/semantic-model-2.md` all state that HTTP health/readiness is
  not implemented and belongs to Sprint 16. That wording is
  compatibility-sensitive until public production evidence exists.

## Accepted constraints from existing architecture

- ADR-0002 keeps dependency construction and lifecycle in Runtime, `main.rs`
  thin, `AppBuilder` responsible for construction, and `AppState` immutable.
- ADR-0037 requires explicit service identity, startup acknowledgement,
  Runtime-owned top-level task handles, receiver-only cancellation, reverse
  cleanup, complete joining, and no detached work.
- ADR-0037 permits lifecycle observation but explicitly leaves HTTP paths,
  status codes, schemas, liveness, readiness, and wire compatibility to Sprint
  16. It forbids reporting `Running` before every service acknowledges startup.
- `docs/codex/workflows/runtime-service.md` requires process liveness to be
  distinguished from ability to serve accepted work, readiness to be derived
  from owned lifecycle evidence, and public client/server entry-point tests for
  a supported transport.
- Sprint 16 must not change graph meaning, source ingestion, workspace build
  orchestration, or later Runtime services.

## Ownership and lifecycle decision surface

| Concern | Confirmed current owner | Decision required from ADR-0038 |
| --- | --- | --- |
| Bind configuration | `RuntimeConfig` has no HTTP field | Typed address representation, default, validation, and public accessor |
| HTTP construction | No owner exists | Concrete HTTP service type constructed by Runtime composition |
| Listener before acknowledgement | Service start future may acquire resources | Require bind before returning the service task |
| Listener and server future | Running service task can own both | Exact Axum serve/graceful-shutdown wiring |
| Shared request state | `Arc<AppState>` is already supplied | Minimal lifecycle-derived health observation in shared state or an equivalent immutable view |
| Lifecycle mutation | `App` through `Lifecycle` | HTTP reads only; it must never drive lifecycle |
| Cancellation request | Running container | HTTP service observes only its receiver and completes graceful shutdown |
| Bound-address observation | No public seam exists | Minimal production-safe handle/watch needed for port-zero integration tests, or another exact public seam |
| Active connections | Axum graceful server internals | Whether accepted first-slice shutdown waits without a new timeout |
| Terminal failure | Runtime named service taxonomy | Bind as startup failure; executable post-start failure boundary and limitations |

`LifecycleState::Running` is the only current state that proves every required
service acknowledged startup and shutdown has not begun. It is therefore the
only repository-backed readiness candidate. `Initializing`, `Stopping`, and
`Stopped` cannot be ready. `Created` and `Building` are not observable by a
running HTTP handler under the current construction order, but a
transport-neutral mapping can still be total and directly testable.

Process liveness is not equivalent to readiness. A handler that has accepted a
request proves the HTTP service task and listener are alive enough to respond;
its liveness response need not claim that other Runtime services are ready.
ADR-0038 must decide whether liveness is an unconditional successful probe while
the HTTP handler is reachable or a lifecycle projection with another meaning.

## Public wire decision matrix

No route or schema is implemented or previously accepted. ADR-0038 must decide
every row before implementation:

| Case | Repository constraint | Bounded candidate for decision |
| --- | --- | --- |
| Liveness path | Must not collide with later workspace/graph APIs | `GET /health/live` |
| Readiness path | Must be lifecycle-derived | `GET /health/ready` |
| Liveness success | Distinct from ability to serve work | `200 OK` while handler is reachable |
| Ready response | Only canonical `Running` qualifies | `200 OK` |
| Not-ready response | Must remain a valid health response, not a transport failure | `503 Service Unavailable` |
| Body | Must not expose diagnostic error prose | Closed JSON object with stable status vocabulary |
| Media type | Serde JSON is already direct | `application/json` |
| Wrong method | Must be explicit and testable | Axum route-level `405 Method Not Allowed` |
| Unknown path | Router default is deterministic | `404 Not Found` |
| `HEAD` | Axum `get` can add HEAD behavior | Explicitly accept or reject; do not inherit accidentally |
| Redirect/trailing slash | No requirement exists | Exact paths only, no redirect |
| Versioning | No other product endpoints exist | Decide whether unversioned probes are the intentionally narrow stable exception |

The status field names and values remain unresolved. Candidate shapes such as
`{"status":"ok"}` and `{"status":"not_ready"}` are not accepted evidence.
ADR-0038 must select an exact schema, decide whether lifecycle is included, and
define key ordering only if raw-body equality is part of compatibility.

## Failure and negative-case inventory

| Case | Executable evidence | Required architectural boundary |
| --- | --- | --- |
| Invalid bind configuration | Construct/parse a rejected address | Decide configuration-time error type and phase |
| Address already in use | Hold one loopback listener, start Runtime HTTP on the same address | Named `ServiceStartFailed` with no acknowledged HTTP task |
| Port zero | Bind `127.0.0.1:0`, observe actual address | Public test seam must not mutate production state |
| Wrong method | Raw request to each accepted path | Exact 405 and body/header expectations |
| Unknown route | Raw request to an unregistered path | Exact 404 expectations |
| Initializing request | Listener exists before all services finish startup | Coordinate a later gated service to hold `Initializing` |
| Running request | Wait for lifecycle `Running` | Exact ready response |
| Stopping request | HTTP is cancelled in reverse registration order | Registration order must make a request during `Stopping` observable if the contract requires it; otherwise document non-observability |
| Requested shutdown | Release injected shutdown | HTTP task completes only after graceful server completion |
| Open connection during shutdown | Raw client can hold or close a connection | Decide whether the first slice tests a controlled connection drain without selecting a timeout |
| Listener release | Connect/rebind after `App::run` returns | Prove no bound resource survives |
| Repeated fresh runs | Build two apps with port zero | Equal wire behavior and independent addresses/resources |
| Serve-loop error | Axum API documents no ordinary error | Do not invent a deterministic test; retain type-level propagation only if useful |

## Deterministic public loopback oracle

The smallest complete oracle requires no fixture or external process:

1. Construct Runtime configuration with loopback port zero and obtain the
   accepted bound-address observer before moving the HTTP service into
   `AppBuilder::register_service`.
2. Register HTTP before a channel-gated probe service when an `Initializing`
   readiness request is required. The HTTP listener acknowledges startup, the
   later service holds the application in `Initializing`, and the test sends a
   raw request to the observed address.
3. Release the gated service start, wait on `App::subscribe_lifecycle` for
   `Running`, and send complete HTTP/1.1 requests with `Connection: close` via
   `TcpStream`. Parse the status line, headers, and exact body rather than using
   an internal handler call.
4. Cover liveness, ready, not-ready when observable, wrong methods, and unknown
   routes according to the accepted matrix.
5. Trigger injected shutdown through `oneshot`. If `Stopping` must be observed
   over HTTP, use a controlled in-flight request or service registration order
   accepted by ADR-0038; do not race a new connection against listener closure.
6. Await `App::run` under a bounded timeout, verify `Stopped`, then prove the
   port can be rebound or that connection attempts fail as selected by the
   contract.
7. Repeat with a freshly built app and assert the same response matrix. Keep
   timeouts only as hang guards; use lifecycle watches and channels for every
   asserted event.
8. For bind failure, hold a Tokio listener on an allocated loopback address,
   start the app on that address, and assert named `ServiceStartFailed` plus
   `Stopped` and no HTTP task acknowledgement.

Raw HTTP avoids adding an HTTP client or direct `tower` dependency. Handler or
router tests may supplement this matrix but cannot establish public
client/server compatibility.

## Smallest coherent first slice

The evidence supports this bounded sequence after ADR acceptance:

1. Extend the immutable Runtime configuration and shared state only as required
   for a typed bind address and lifecycle-derived health observation.
2. Add a transport-neutral health snapshot whose readiness is a pure projection
   of canonical lifecycle state.
3. Add one Axum HTTP `RuntimeService` that binds during start, exposes only the
   accepted probe routes, returns the server as its owned task, and drives
   graceful shutdown from receiver-only cancellation.
4. Register that service in the thin production composition root.
5. Add public raw-loopback integration evidence and synchronize current-state
   documentation only after the production path passes.

This slice requires no new dependency, remote service, repository fixture,
semantic Coverage entry, graph change, or platform-specific API.

## Deferred scope

- Workspace lifecycle and semantic-build orchestration: Sprint 17.
- Graph and semantic query endpoints: Sprint 18.
- File watching, persistent cache, and supported CLI client: Sprints 19-21.
- MCP, LSP, IDE, and AI transports or clients: later roadmap sprints.
- TLS, authentication, authorization, CORS, compression, rate limiting, request
  IDs, metrics export, tracing export, OpenAPI, general API version negotiation,
  streaming, request bodies, domain error mapping, and proxy policy.
- Dynamic service registration, restart, retry, forced abort, and a new
  graceful-shutdown timeout.

## ADR-0038 decisions and remaining unknowns

Repository evidence is sufficient to decide ADR-0038. It must accept or reject:

1. bind-address type, default, parsing/validation phase, and exposure;
2. HTTP service identity and the bound-address observation seam;
3. exact route, method including HEAD, status, media type, body schema, and
   fallback matrix;
4. liveness meaning and total lifecycle-to-readiness mapping;
5. lifecycle observation placement without a second mutable authority;
6. service registration order required to make shutdown/readiness evidence
   observable and deterministic;
7. active-connection graceful shutdown behavior under ADR-0037's no-timeout
   policy;
8. bind and post-start error mapping, including the locked Axum serve-error
   limitation;
9. the intentionally stable compatibility surface and every deferred concern.

There is no missing external data or test blocker. Exact endpoint and schema
choices are product architecture decisions, not unknown source facts.

## Evidence and validation record

- Planning baseline: `7a6e8a585936` (`Plan Sprint 16 HTTP API and health`).
- Direct dependencies: `cargo tree -p oneagent-runtime --depth 1`.
- Existing test inventory: `cargo test -p oneagent-runtime -- --list` (17
  library tests and six public integration tests; no binary or doc tests).
- Locked source inspected under the Cargo registry for Axum 0.8.9
  `Router`, `Json`, `serve`, `local_addr`, and `with_graceful_shutdown`, and
  Tokio 1.53.0 `TcpListener`/`TcpStream` APIs.
- CI platform matrix verified in `.github/workflows/ci.yml`: macOS 14 and
  Windows latest.
- Runtime definitions and consumers verified by repository search under
  `apps/runtime`, workspace crates, tests, and documentation.
