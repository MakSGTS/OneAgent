# ADR-0038: HTTP API and Health

## Status

Accepted

## Context

ADR-0002 makes `oneagent-runtime` the composition root. ADR-0037 adds the
long-running service container, sequential startup acknowledgement, canonical
lifecycle, receiver-only cancellation, reverse cleanup, and complete ownership
of service tasks. It deliberately leaves HTTP paths, wire schemas, liveness,
readiness, and transport compatibility to Sprint 16.

The repository investigation in
`docs/architecture/http-api-health-investigation.md` confirms that Runtime has
no HTTP implementation or bind configuration. The locked direct dependencies
already include Axum 0.8.9, Tokio 1.53.0, Serde, and Serde JSON. Axum can serve
one router from a Tokio listener and complete graceful shutdown; Tokio can bind
port-zero loopback listeners and exercise the public server through raw TCP.
No new dependency, external service, or platform-specific fixture is required.

Sprint 16 must establish one narrow, stable public HTTP boundary without
pulling workspace, graph-query, watcher, cache, or supported CLI behavior from
Sprints 17-21 into the health slice.

## Decision

### Composition and public boundary

`oneagent-runtime` owns the HTTP adapter. Runtime composition constructs one
`HttpService` and registers it under the stable service identity `http` through
`AppBuilder`. The service remains a normal ADR-0037 `RuntimeService`; it does
not construct dependencies, mutate lifecycle, or own process shutdown policy.

`main.rs` remains a thin consumer. It loads the default configuration,
constructs and registers `HttpService`, builds `App`, supplies
`tokio::signal::ctrl_c()`, and reports the terminal result. Router construction,
listener acquisition, request handling, task execution, and graceful shutdown
belong to reusable Runtime library modules rather than `main.rs`.

The first public HTTP compatibility surface contains only the two probe routes
defined below, the bind configuration accessor, the HTTP service constructor,
and a read-only bound-address observer needed by public clients and port-zero
tests. Internal router functions, Axum state types, listener storage, and
cancellation wiring remain private.

### Bind configuration

`RuntimeConfig` gains one typed `std::net::SocketAddr` HTTP bind address. The
default is `127.0.0.1:3000`, which is loopback-only and does not expose the
development Runtime to other hosts. Existing `RuntimeConfig::new(application,
environment)` remains source-compatible and initializes the default address.
A builder-style `with_http_bind_address(SocketAddr)` override and a shared
reference or copied accessor provide explicit composition and test control.

The first slice does not parse environment variables, files, command-line
strings, hostnames, or URLs. A typed `SocketAddr` means invalid textual address
syntax is outside the production configuration boundary; no speculative parse
error is added. Port zero is valid and required for deterministic public tests.

The HTTP service reads the immutable address from `AppState` during startup. It
calls `tokio::net::TcpListener::bind` before acknowledging service startup. Bind
failure is returned as the start error, so the service container exposes it as
`RuntimeErrorKind::ServiceStartFailed` with service name `http`, performs any
earlier-service rollback, and reaches `Stopped` before returning.

### Bound-address observation

`HttpService` owns a Tokio watch sender whose value is `Option<SocketAddr>`.
Before moving the service into `register_service`, a caller may obtain a
receiver through a read-only `subscribe_bound_address` method. The value is
`None` before successful bind and becomes `Some(actual_address)` immediately
after bind and before startup acknowledgement returns.

This observer does not select the address, control the listener, report
readiness, or request shutdown. It exists so port-zero callers can connect to
the public server without inspecting private listener state. A fresh service
has fresh observation state; no process-global address registry is allowed.

### Transport-neutral health state

The canonical lifecycle watch remains solely owned and mutated by `Lifecycle`.
`AppState` receives a receiver derived from that existing watch during build and
exposes a transport-neutral immutable `RuntimeHealth` snapshot. The snapshot
contains the current `LifecycleState` and an `is_ready` projection. It owns no
sender and cannot change lifecycle.

The total readiness mapping is:

| Lifecycle state | Ready |
| --- | --- |
| `Created` | false |
| `Building` | false |
| `Initializing` | false |
| `Running` | true |
| `Stopping` | false |
| `Stopped` | false |

Only `Running` means every required service acknowledged startup and shutdown
has not begun. No boolean, atomic, lock, log message, route-local flag, or HTTP
service state may override this mapping. The health snapshot is reusable by
future transports but does not expose HTTP status or JSON concepts.

### Liveness and readiness semantics

Liveness answers whether the HTTP probe handler is alive enough to produce a
response. A reachable liveness handler returns success regardless of Runtime
readiness. It does not promise that workspace, graph, or later services are
available, and it does not serialize lifecycle.

Readiness answers whether Runtime is in canonical `LifecycleState::Running`.
It returns not-ready before all services acknowledge startup and as soon as
shutdown begins. It does not inspect workspace or semantic state because those
services do not exist until later sprints. Future required services affect this
probe automatically through the existing rule that Runtime enters `Running`
only after every registered service starts.

`Stopped` is included in the transport-neutral mapping even though a correctly
owned HTTP listener is no longer reachable then. `Created` and `Building` are
also total model states even though the server cannot yet be started.

### Stable route and wire contract

Both probe paths are intentionally unversioned. They are operational process
probes, not the future versioned workspace or graph API. Exact path matching is
used; no trailing-slash redirect is accepted.

| Request | Runtime condition | Status | Required headers | Exact body |
| --- | --- | ---: | --- | --- |
| `GET /health/live` | Handler is reachable | `200 OK` | `content-type: application/json` | `{"status":"alive"}` |
| `GET /health/ready` | lifecycle is `Running` | `200 OK` | `content-type: application/json` | `{"status":"ready"}` |
| `GET /health/ready` | any other lifecycle state | `503 Service Unavailable` | `content-type: application/json` | `{"status":"not_ready"}` |

The JSON schema is one closed object with one required string field named
`status`; no additional field is emitted. The accepted values are exactly
`alive`, `ready`, and `not_ready` in the rows above. Axum `Json` and a derived
Serde response type own serialization. The single-field encoding has no map
ordering ambiguity and contains no newline.

Only GET is accepted. Routes use Axum `on(MethodFilter::GET, ...)`, not the
`get(...)` convenience that also installs HEAD behavior. HEAD, POST, PUT,
PATCH, DELETE, OPTIONS, CONNECT, and TRACE on either registered path return
`405 Method Not Allowed`, an empty body, and `Allow: GET`. An unknown exact path,
including `/`, `/health`, `/health/live/`, and `/health/ready/`, returns
`404 Not Found` with an empty body. Error-response transport headers such as
date, connection, transfer encoding, or content length are not part of the
stable contract.

The first slice accepts HTTP/1.1 behavior exercised by raw loopback clients.
It makes no HTTP/2, proxy, host-routing, TLS, browser, caching, or content
negotiation claim. Successful probe responses do not vary by `Accept` header.

### Listener, task, and connection ownership

`HttpService::start` consumes the service, binds the configured listener,
publishes the actual address, constructs the router from the shared `AppState`,
and returns one boxed service task. Startup acknowledgement occurs only after
these steps succeed.

The returned task owns the Tokio listener through Axum's server future, the
router state, the bound-address sender, and its receiver-only Runtime
cancellation handle. The ADR-0037 running container owns and joins that task.
Axum may create connection tasks internally, but its graceful server future
does not complete until those connection tasks finish. They therefore remain
structurally nested under the single Runtime-owned service task.

No raw listener, accept task, connection handle, cancellation sender, or Axum
state may escape into global state. Request handlers clone only the immutable
shared `Arc<AppState>` or its read-only health view.

### Cancellation and graceful shutdown

The HTTP task calls `axum::serve(listener, router).with_graceful_shutdown(...)`.
The graceful signal future waits only for its receiver-only ADR-0037 service
cancellation. Runtime remains the sole cancellation requester.

When application shutdown starts, lifecycle becomes `Stopping` before reverse
service cleanup. Any already admitted readiness request observes not-ready.
When the container reaches the HTTP service, it requests cancellation; Axum
stops accepting new connections, drops the listener, and waits for existing
connections to complete before the service task returns. The Runtime joins that
task before it can reach `Stopped` or return to its caller.

ADR-0037 accepts no production graceful-shutdown timeout. Sprint 16 does not add
one. An uncooperative open connection may therefore delay HTTP task completion.
The implementation guarantees ownership and complete join after `App::run`
returns, not a bounded shutdown duration. Tests use controlled connections and
bounded timeouts only as hang guards.

### Failure policy

- Listener bind failure is a named HTTP service startup failure.
- Router construction and health serialization are infallible for the accepted
  static routes and closed string response type.
- Axum 0.8.9 types its server future as `io::Result<()>`, but its documented
  listener loop does not ordinarily return an error. The service task still
  maps any returned error into its boxed service error, which ADR-0037 would
  expose as named `ServiceFailed`; this is type-level containment, not a claim
  of a reproducible serve-error test.
- Successful HTTP service completion before Runtime cancellation remains the
  existing named `UnexpectedServiceExit`.
- Task panic remains `ServiceTaskJoinFailed`.
- Shutdown-source failure remains primary over later cooperative HTTP cleanup
  failures as defined by ADR-0037.
- No Runtime error, Rust type name, source-chain text, or diagnostic message is
  serialized by a probe route.

### Deterministic public evidence

Public integration tests use the exported Runtime, configuration, HTTP service,
bound-address observer, lifecycle receiver, and raw Tokio loopback TCP. They
must prove:

- port-zero bind publication occurs before HTTP startup acknowledgement;
- liveness success and readiness 503 while a later gated service keeps Runtime
  in `Initializing`;
- readiness 200 after lifecycle becomes `Running`;
- readiness 503 during `Stopping` while a later-registered controlled service
  delays reverse cleanup and HTTP remains reachable;
- exact status, JSON media type, and body for all three response rows;
- exact 405 plus `Allow: GET` and empty body for at least HEAD and POST on the
  registered paths;
- exact 404 and empty body for unknown and trailing-slash paths;
- address-in-use bind failure produces named `ServiceStartFailed`, no HTTP task,
  and terminal `Stopped`;
- requested shutdown joins the HTTP task and releases the listener so the same
  address can be rebound;
- two fresh port-zero applications produce the same wire matrix without shared
  address, lifecycle, or task state.

Tests coordinate startup, lifecycle, stopping, connection completion, and
shutdown with watch/oneshot/mpsc channels. Raw requests include
`Connection: close`. Bounded timeouts detect hangs but are never the event under
assertion. Handler-only tests, arbitrary sleeps, real signals, fixed ports,
external services, and Unix-only sockets are insufficient.

### First production slice

Sprint 16 implements only:

1. typed loopback-default HTTP bind configuration;
2. transport-neutral lifecycle-derived `RuntimeHealth` observation;
3. one Runtime-owned Axum HTTP service and read-only bound-address observer;
4. the exact liveness/readiness routes and negative route/method behavior;
5. thin production service registration and cancellation-driven graceful
   shutdown;
6. public raw-loopback client/server evidence and truthful current-state docs.

## Consequences

- Runtime gains a stable but deliberately narrow public network surface.
- Readiness cannot drift from application lifecycle because it has no separate
  writer or stored boolean.
- Binding is a real startup prerequisite and participates in existing rollback.
- The HTTP server and all Axum connection work remain underneath one
  Runtime-owned service task.
- Default development exposure is loopback-only; callers may explicitly choose
  another typed address.
- Port-zero observation supports deterministic public tests and embedded
  callers without exposing listener control.
- Probe wire compatibility is fixed before workspace and graph APIs are
  designed.
- Graceful shutdown remains structurally complete but unbounded in duration.

## Rejected alternatives

- **A mutable `ready` boolean in `AppState` or `HttpService`:** rejected because
  it duplicates canonical lifecycle and can drift during failure or shutdown.
- **Deriving readiness from the listener alone:** rejected because bind success
  precedes startup of later required services.
- **Treating liveness and readiness as synonyms:** rejected because process
  responsiveness does not prove readiness to serve accepted work.
- **Binding in `main.rs`:** rejected because startup failure and listener
  ownership would escape the Runtime service boundary.
- **Binding after returning the service task:** rejected because Runtime could
  enter `Running` before listener acquisition succeeds.
- **Using Axum `get(...)`:** rejected because it silently broadens the method
  surface to HEAD; the first contract accepts GET only.
- **Versioning probe paths under `/api/v1`:** rejected because operational
  probes are intentionally independent from later domain API versioning.
- **Serializing lifecycle or `RuntimeError`:** rejected because it expands the
  wire schema and exposes implementation and diagnostic details.
- **A generic health registry:** rejected because no repository requirement
  needs per-component mutable health or aggregation in this slice.
- **An HTTP client or `tower` test dependency:** rejected because raw Tokio TCP
  proves the public server boundary with the existing locked surface.
- **A fixed test port or external probe process:** rejected as parallel-unsafe
  and cross-platform-fragile.
- **A new shutdown timeout:** rejected because ADR-0037 deliberately leaves the
  product timeout/forced-abort policy unresolved.

## Deferred scope

- Workspace lifecycle and semantic-build readiness: Sprint 17.
- Graph and semantic query routes: Sprint 18.
- File watching, persistent cache, and supported CLI client: Sprints 19-21.
- MCP, LSP, IDE, and AI transports/clients: later roadmap sprints.
- TLS, authentication, authorization, CORS, compression, rate limiting, request
  IDs, metrics/tracing export, OpenAPI, general API version negotiation,
  HTTP/2 compatibility, proxy policy, request bodies, streaming, and domain
  error mapping.
- Environment/file/CLI configuration providers, hostname or URL parsing, hot
  rebind, dynamic registration, retry, restart, forced abort, and bounded
  graceful shutdown.

## Implementation prerequisites

1. Add typed bind configuration while preserving existing constructor behavior.
2. Make the existing lifecycle watch observable through immutable shared state
   and implement the total `RuntimeHealth` projection before HTTP code.
3. Add the HTTP service and bound-address watch without changing the generic
   Runtime service contract.
4. Build routes with explicit GET-only method filters and closed Serde response
   values.
5. Register HTTP through `AppBuilder` in the thin binary.
6. Add focused tests before the public loopback matrix and full workspace gate.

## Coverage Registry impact

None. Runtime health and HTTP transport are not semantic graph or source-adapter
capabilities and do not change either Coverage Registry.

## Documentation completion criteria

Architecture documents may identify this accepted boundary immediately, but
README, Architecture, and Semantic Model current-state support claims change
only after the public production path and loopback evidence are committed.
Sprint 16 becomes completed only after its integration review records a
non-blocking decision and the full validation matrix succeeds.
