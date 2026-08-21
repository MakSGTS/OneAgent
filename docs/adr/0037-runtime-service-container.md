# ADR-0037: Runtime Service Container

## Status

Accepted

## Context

ADR-0002 makes `oneagent-runtime` the composition root: `AppBuilder` constructs
the application, `AppState` contains immutable shared state, `Lifecycle` owns
explicit state transitions, and `main.rs` remains a thin process boundary. The
current Runtime is binary-only and synchronous. `App::run` enters `Running`,
prints a banner, and immediately enters `Stopping` and `Stopped`; it owns no
long-lived service, task, cancellation source, or shutdown wait.

Sprint 15 must establish the first reusable long-running Runtime boundary
without pulling the Sprint 16 HTTP contract or later workspace, graph, watcher,
persistence, and CLI services forward. The repository investigation in
`docs/architecture/runtime-service-container-investigation.md` confirms that
Tokio 1.53 already supplies the required structured task, channel, executor,
and cross-platform signal primitives. No new production dependency or external
test fixture is required.

## Decision

### Composition and public library boundary

`oneagent-runtime` remains the only composition root and gains a library target.
The library owns reusable application, service-container, lifecycle,
cancellation, configuration, state, and Runtime error contracts. The binary is
a consumer of that library. It may choose the default configuration, register
production services, supply the process shutdown future, initialize diagnostics,
and map the terminal result to process exit. It must not own service execution,
task handles, domain behavior, or adapter infrastructure.

The library exposes only the transport-independent types required to construct
and run an application and to implement a Runtime service. Concrete registry
storage, task collections, cancellation senders, and lifecycle mutation remain
private. No global registry, singleton executor state, or adapter-owned
composition is permitted.

`AppBuilder` remains the construction entry point. It owns configuration and
ordered service registrations until `build`. `build` creates immutable
`AppState`, the initial `Lifecycle`, and exactly one service container, then
transfers all of them to `App`. Each separately constructed builder and app has
independent registry, cancellation, and terminal state. `App::run` consumes the
app, so one built app has one run; restart is represented by a fresh build.

### Service identity and registration

Every service registration has an explicit, owned, non-empty UTF-8 name.
Service identity is the name, not Rust `TypeId`, task ID, registration position,
or an adapter route. Names are unique within one builder/container and stable in
errors and deterministic test evidence.

Registration is allowed only before `build`. Registration order is preserved
and is the startup order. A duplicate name is rejected synchronously at
registration, before any service starts. An empty registry is valid: it supports
composition tests and a Runtime with no production service yet, but a running
app still waits for shutdown rather than immediately succeeding.

The public service contract separates initialization from execution without an
`async-trait` dependency:

1. A registered service value is owned by the container.
2. The container calls its boxed asynchronous start operation in registration
   order with an immutable `AppState` handle and a receiver-only cancellation
   handle scoped to that service.
3. Successful start returns an owned, boxed service task future. Returning that
   task is the service's startup acknowledgement.
4. The container spawns the acknowledged task and owns its handle. A start
   error identifies the service and triggers rollback.

The exact Rust aliases and method names may remain implementation details where
they do not affect these semantics. A service must be `Send + 'static`; its
start and task futures must be `Send + 'static`; service failures must implement
`Error + Send + Sync + 'static`.

### Ownership inventory

| Resource | Sole owner | Terminal rule |
| --- | --- | --- |
| Configuration and shared dependencies | Immutable `AppState`, shared by owned handles | Dropped after all service tasks terminate |
| Registered, not-yet-started service values | Built service container | Started once in order or dropped during rollback |
| Started service task handles | Running service container | Every handle is joined before run returns |
| Per-service cancellation sender | Running service container | Used at most once logically; dropped only after its task terminates |
| Receiver-only cancellation handle | Corresponding service start/task | Observes an idempotent request; cannot cancel siblings |
| Mutable application lifecycle | `App` through `Lifecycle` | Reaches `Stopped` after every started-task cleanup path |
| Process shutdown future | `App::run` for the duration of the run | Its success requests shutdown; its error is a Runtime failure |
| First classified terminal cause and cleanup failures | Running container, then `App::run` | Returned to the application caller after complete cleanup |

No spawned task may be detached, leaked through an abort handle, or transferred
to a transport adapter. A service may own child work only when it implements an
equivalent structured lifetime and completes that work before its service task
returns; the Runtime still owns and joins the top-level service task.

### Startup and rollback

`App::run` begins with lifecycle state `Initializing`. The container starts
services sequentially in registration order. While awaiting a later service's
start acknowledgement, it also observes already-started task termination. An
early task completion or failure interrupts further startup and is handled as a
terminal running-service outcome; the in-progress start future is dropped and
all acknowledged tasks are rolled back.

After every registration acknowledges startup, the application transitions
once from `Initializing` to `Running`. It must not report `Running` earlier.

If service `N` fails to start, services `N+1..` are never started. Already
started services receive cancellation one at a time in reverse registration
order and each top-level task is joined before rollback advances. The start
failure for `N` is the primary returned cause. A cleanup failure is retained as
structured secondary context and diagnostics; it cannot replace or hide the
start failure. Rollback completes before the lifecycle reaches `Stopped` or the
caller receives the error.

The Sprint 15 implementation has no production startup timeout. Service start
is therefore cooperative and may remain pending. This limitation is explicit
and does not imply a bounded-start guarantee.

### Running and terminal causes

After startup, `App::run` remains pending until either the supplied shutdown
future completes or a service task terminates. A completed service task is
never silently discarded:

- shutdown future success is a requested shutdown;
- shutdown future error is a shutdown-source failure;
- a service returning `Err` is a named service failure;
- a service returning `Ok` before its cancellation was requested is a named
  unexpected service exit;
- a panicked or externally cancelled top-level task is a named task-join
  failure.

Any service/task terminal cause requests cleanup of all remaining services.
Requested shutdown is successful only when every service terminates normally
after its cancellation request and every handle joins successfully. A service
error during requested shutdown remains a service failure; shutdown does not
hide it.

If multiple abnormal task outcomes become observable in one cleanup cycle, the
primary service/task cause is selected by lowest registration order, not by
scheduler completion order. Startup failure always remains primary over rollback
failures. A shutdown-source error remains primary over later cleanup failures,
but an independently completed service failure already observed before the
source error remains a service failure. All additional failures are retained as
secondary cleanup context and emitted as structured diagnostics.

### Cancellation and deterministic shutdown

Cancellation is cooperative and receiver-only for services. The container owns
one logical cancellation source per started service. It requests cancellation
in reverse registration order and joins that service before requesting the next
one. This makes shutdown request and completion order deterministic and permits
stack-like resource dependencies. Repeated requests are idempotent.

Dropping the shutdown future does not cancel service tasks. Dropping a service's
cancellation receiver is not a successful task termination. `abort_all` may be
used only as a last-resort internal cleanup after a join-level failure makes
cooperative progress impossible; every aborted handle must still be joined and
the run must return a failure.

Sprint 15 accepts no wall-clock graceful-shutdown timeout. Consequently it
guarantees complete ownership and no detached work after `run` returns, but it
does not guarantee that an uncooperative service will let `run` return within a
bounded duration. A timeout and forced-abort product policy requires a later ADR
or an explicit extension of this one.

### Lifecycle and terminal state

The existing state sequence remains canonical:

```text
Created -> Building -> Initializing -> Running -> Stopping -> Stopped
```

`configure` performs `Created -> Building`, and `build` performs
`Building -> Initializing`, as today. Successful startup performs
`Initializing -> Running`. Requested shutdown or a running-service terminal
cause performs `Running -> Stopping`. Startup failure performs
`Initializing -> Stopping`; this transition is added because rollback is real
shutdown work. After all acknowledged tasks terminate, both success and failure
paths perform `Stopping -> Stopped`.

Invalid transitions remain Runtime errors. A terminal operational error is
returned only after `Stopped` has been recorded. Lifecycle state does not encode
the error cause; the returned Runtime error does.

### Error taxonomy

The Runtime error surface preserves the existing missing-configuration and
invalid-transition variants and adds distinguishable classifications for:

- invalid or duplicate service identity;
- named service startup failure;
- named unexpected service exit;
- named service execution failure;
- named task join failure;
- shutdown-source failure;
- lifecycle failure while preserving an already selected operational cause;
- secondary cleanup failures associated with a primary cause.

Errors carry stable service names and implement source chaining where a source
exists. They are library errors, not HTTP response or serialized wire schemas.
Exact `Display` prose is diagnostic and is not a readiness or integration-test
oracle.

### Internal observation and test boundary

Sprint 15 may expose lifecycle subscription/snapshots and transport-neutral
structured service identity/results needed for deterministic tests. Lifecycle
observation must show `Running` only after all startup acknowledgements and
`Stopped` only after all owned handles terminate. Service implementations may
emit typed probe events over injected in-memory channels.

These facilities do not define liveness, readiness, availability, HTTP paths,
status codes, JSON fields, polling frequency, or wire compatibility. Sprint 16
must derive any public health/readiness contract from owned Runtime state and
accept it separately.

Production `main` supplies `tokio::signal::ctrl_c()` as the shutdown future.
Tests inject a one-shot or channel-backed future and never send a real process
signal. Signal registration errors propagate as shutdown-source failures.

### Deterministic evidence

Focused unit tests and public library integration tests use in-memory services
and explicit `oneshot`, `mpsc`, or `watch` handshakes. They must prove:

- unique and duplicate registration, startup order, and repeated independent
  construction;
- a pending app after every service acknowledges start and before shutdown is
  released;
- reverse cancellation/completion order and zero unjoined tasks on success;
- partial-start rollback and the primary start error;
- unexpected successful exit, service error, and task panic classifications;
- sibling cleanup and `Stopped` after every terminal path;
- repeated fresh build/run equivalence.

Arbitrary sleeps, scheduler-yield counts, stdout/log matching, real signals,
network listeners, filesystem fixtures, and external services are not accepted
as correctness evidence. A bounded timeout may guard a test from hanging, but
the timeout is not the event under assertion.

### First production slice

Sprint 15 implements only:

1. the public Runtime library extraction;
2. explicit service registration and the ordered container;
3. receiver-only per-service cancellation and complete task ownership;
4. sequential startup, reverse rollback/shutdown, terminal classification, and
   cleanup;
5. asynchronous `App::run` with injected shutdown and thin `ctrl_c` binary
   wiring;
6. deterministic in-memory unit and public integration evidence.

An empty production registry is acceptable in this slice. The Runtime process
is genuinely long-running because it awaits its shutdown source, not because a
placeholder HTTP listener or fake background loop is registered.

## Consequences

- ADR-0002 ownership is preserved while the crate becomes reusable by later
  transports and integration tests.
- Services have explicit identity, startup acknowledgement, cancellation, task
  ownership, error propagation, and terminal handling.
- Registration/start order and reverse cleanup order are deterministic.
- `App::run` becomes asynchronous and consuming; binary and test callers must
  await it and supply a shutdown future.
- `Lifecycle` gains the accepted `Initializing -> Stopping` failure transition
  and a transport-neutral observation seam.
- Runtime errors expand without changing current variant meanings.
- Cooperative services can be proven leak-free without a new dependency,
  external service, port, OS-signal test, or arbitrary delay.
- Shutdown is structurally complete but intentionally has no bounded-duration
  guarantee in Sprint 15.

## Rejected alternatives

- **Detached `tokio::spawn` tasks:** rejected because the composition root could
  return while work or resources remain alive.
- **One shared cancellation sender exposed to services:** rejected because any
  service could stop siblings and reverse dependency shutdown could not be
  enforced.
- **Global registry or mutable `AppState` service locator:** rejected because it
  breaks independent construction and ADR-0002 immutable shared state.
- **Adapter-owned composition:** rejected because HTTP/MCP/LSP/CLI services must
  reuse Runtime behavior rather than own it.
- **Rust type identity as service identity:** rejected because it is unsuitable
  for stable diagnostics, multiple instances, and later adapter composition.
- **Concurrent startup or broadcast-at-once shutdown:** rejected for the first
  slice because ordering, rollback, and resource dependency behavior would be
  less deterministic.
- **Treating early `Ok` as successful app shutdown:** rejected because one
  service must not silently terminate the Runtime or its siblings.
- **Dropping or aborting tasks without joining:** rejected because it cannot
  prove cleanup or surface panics deterministically.
- **A hard-coded shutdown timeout:** rejected because no product requirement or
  repository evidence selects a duration or forced-abort policy.
- **Pulling Axum/HTTP health into Sprint 15:** rejected because Sprint 16 owns
  transport and public health/readiness compatibility.
- **Arbitrary-sleep and real-signal tests:** rejected as timing-dependent and
  cross-platform-fragile.
- **A new DI, cancellation, or async-trait dependency:** rejected because the
  locked Tokio and standard-library surface is sufficient.

## Deferred scope

- HTTP listener, routes, liveness/readiness semantics, schemas, and client
  compatibility: Sprint 16.
- Workspace lifecycle and semantic-build orchestration services: Sprint 17.
- Graph query API: Sprint 18.
- File watching and update orchestration: Sprint 19.
- Persistent cache and invalidation: Sprint 20.
- Supported CLI client behavior: Sprint 21.
- MCP, LSP, IDE integration, packaging, performance claims, and a general plugin
  service model: later roadmap work.
- Bounded graceful-shutdown timeout, forced abort, retry, restart, and dynamic
  post-build registration: future explicit architecture work when required.

## Implementation prerequisites

1. Extract the existing private Runtime modules behind `src/lib.rs` without
   changing configuration-provider semantics or moving ownership to another
   crate.
2. Define boxed service start/task futures and receiver-only cancellation using
   the already locked Tokio API.
3. Implement named ordered registration and a single structured container owner
   with complete joins on every path.
4. Extend lifecycle transitions and observations before integrating the async
   app loop.
5. Keep service-container primitives independently unit-testable before wiring
   `AppBuilder`, `App`, and async `main`.
6. Add public integration tests only after the public production path exists;
   keep every probe in memory and cross-platform.

## Coverage Registry impact

None. This ADR changes Runtime architecture only and neither adds semantic graph
capabilities nor changes source-adapter coverage.
