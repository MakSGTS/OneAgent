# Runtime Service Container Investigation

## Purpose

This investigation records the repository evidence and decision surface for
ADR-0037 and the smallest safe Sprint 15 implementation of a long-running
Runtime service container. It does not accept an architecture decision, define
an HTTP health contract, or change production behavior.

## Confirmed repository evidence

### Current application boundary

- `apps/runtime/src/main.rs::main` is the only executable entry point. It builds
  an `App` with `DefaultConfigurationProvider` and calls `App::run`. The crate
  has no `src/lib.rs`, so no other workspace crate or external integration test
  can currently import its application types.
- `apps/runtime/src/app/builder.rs::AppBuilder` owns an optional
  `RuntimeConfig` and a `Lifecycle`. `configure` performs the
  `Created -> Building` transition and loads the provider; `build` requires
  configuration, performs `Building -> Initializing`, and constructs
  `AppState` and `App`.
- `apps/runtime/src/config/provider.rs::ConfigurationProvider` is the existing
  construction seam. It returns a boxed, `Send + Sync` error and permits a
  later provider without changing the builder consumer. The default provider
  returns `RuntimeConfig::default`.
- `apps/runtime/src/state/mod.rs::AppState` is cloneable, contains only an
  immutable `RuntimeConfig`, and exposes it by shared reference. It does not
  own services, mutable lifecycle state, task handles, cancellation state, or
  terminal failures.
- `apps/runtime/src/app/lifecycle.rs::Lifecycle` owns mutable
  `LifecycleState`. The accepted transitions are strictly
  `Created -> Building -> Initializing -> Running -> Stopping -> Stopped`.
  Its state accessor is compiled only for unit tests.
- `apps/runtime/src/app/runtime.rs::App` owns `AppState` and `Lifecycle`.
  `App::run` is synchronous: it enters `Running`, prints the banner, and then
  immediately enters `Stopping` and `Stopped`. It neither awaits a shutdown
  request nor starts a service.
- `apps/runtime/src/error/mod.rs::RuntimeError` represents only missing
  configuration and invalid lifecycle transitions. Service registration,
  startup, task exit, task panic/cancellation, and shutdown failures have no
  current error representation.
- The five unit tests in `apps/runtime/src/app/builder.rs`,
  `apps/runtime/src/app/lifecycle.rs`, and `apps/runtime/src/config/mod.rs`
  cover required/default configuration, application construction, and valid
  and invalid lifecycle transitions. `cargo test -p oneagent-runtime -- --list`
  confirms that there is no public integration-test target.

### Accepted architectural constraints

- `docs/adr/0002-runtime-composition-root.md` assigns dependency construction
  and application lifecycle to Runtime, requires `main.rs` to contain no domain
  or infrastructure logic, assigns construction to `AppBuilder`, immutable
  shared state to `AppState`, and explicit transitions to `Lifecycle`, and
  rejects a dependency-injection framework.
- The same ADR requires services to remain reusable by future HTTP, MCP, LSP,
  and CLI adapters and requires startup and shutdown to remain testable.
- `docs/architecture/semantic-model-2.md`, section `oneagent-runtime`, assigns
  runtime configuration, state, lifecycle, and future transport composition to
  the Runtime crate; graph meaning remains in domain crates and source loading
  remains in adapters.
- `docs/Architecture.md`, Runtime layer, describes the current crate as
  composition/configuration/state/lifecycle foundations and explicitly says it
  is not yet the v0.4 long-running Runtime API.
- `docs/codex/workflows/runtime-service.md` requires structured ownership of
  spawned work and resources, deterministic startup cleanup and shutdown,
  explicit cancellation and failure propagation, and tests without arbitrary
  sleeps. It deliberately leaves the concrete registry, executor,
  cancellation primitive, timeout, health schema, and transport to an ADR or
  bounded task.
- `.github/workflows/ci.yml` runs workspace check and tests on macOS 14 and
  Windows. Sprint 15 therefore cannot rely on Unix-only signals, descriptors,
  or process control.
- `docs/reviews/v0.3-release-review.md` keeps long-running Runtime services and
  transport APIs outside v0.3 and names Sprint 15 as the next target. Existing
  domain, graph, and source-adapter behavior is compatibility-sensitive and is
  not a Runtime-container migration surface.

### Available dependency capabilities

`apps/runtime/Cargo.toml` and `cargo tree -p oneagent-runtime --depth 1` confirm
that no new production dependency is required for the bounded slice:

| Capability | Existing dependency and locally verified API | Intended boundary |
| --- | --- | --- |
| Async executor | `tokio = 1.53.0`, feature `full` | Async `main`, application execution, and test synchronization |
| Owned task set | `tokio::task::JoinSet::{spawn, join_next, abort_all}` | One container-owned structured set of service tasks |
| Broadcast state/cancellation | `tokio::sync::watch::{channel, Sender::subscribe}` | Cloneable cancellation observation and deterministic probes |
| Production shutdown | `tokio::signal::ctrl_c` | Cross-platform production trigger supplied outside service logic |
| Structured diagnostics | `tracing = 0.1.44` | Diagnostics only; logs are not a test oracle or health state |
| Future HTTP adapter | `axum = 0.8.9` | Already available but explicitly deferred to Sprint 16 |

`serde`, `serde_json`, and `tracing-subscriber` are also direct Runtime
dependencies, but they are not needed to define service ownership or the
shutdown oracle. Introducing a cancellation-token crate, async-trait crate,
DI framework, or test-only timing dependency would be unjustified by the
current evidence.

## Compatibility-sensitive behavior and migration surfaces

1. Keep `App::builder().configure(...).build()` as the composition path and
   keep configuration-provider failures propagating to the caller.
2. Preserve all lifecycle state names and the legal forward sequence. Making
   execution asynchronous changes when transitions occur, not their order.
3. Keep `AppState` immutable after construction. Runtime-owned mutable control
   state must not turn it into a global service locator.
4. Keep `main.rs` declarative. Production signal selection may be wired there,
   while registration, execution, cleanup, and failure classification belong to
   reusable library code.
5. Preserve the current banner content unless a later task explicitly changes
   it. A long-running implementation may emit it after successful startup, but
   must not use stdout text as readiness evidence.
6. Preserve current `RuntimeError` messages for existing variants. New variants
   need stable classifications, but this investigation does not promise a
   public wire format.
7. Add a library target as a migration surface rather than moving composition
   into an adapter. The binary should become a thin consumer of the same public
   Runtime boundary used by integration tests and later transport adapters.

There are no current Runtime consumers outside the binary and its unit tests,
as confirmed by repository symbol search. The library extraction is therefore
locally bounded, but future consumers make its ownership decisions durable.

## Ownership model for ADR-0037

The following allocation is the smallest model consistent with ADR-0002 and
the Runtime Service Workflow. It is a candidate for acceptance, not an accepted
decision in this investigation.

| Concern | Candidate owner | Reason |
| --- | --- | --- |
| Service definitions and registration order | `AppBuilder` before `build` | Construction remains separate from execution and duplicate identity can fail before tasks start |
| Immutable configuration/shared dependencies | `AppState` | Preserves the existing immutable shared-state boundary |
| Registered service values until start | A container constructed by `AppBuilder` and owned by `App` | Service lifetimes cannot escape the composition root |
| Spawned task handles | The running container, through one `JoinSet` | Every task is joined or explicitly aborted; no detached work |
| Cancellation sender | The running container | The owner that starts tasks must request their shutdown |
| Cancellation receivers | Each started service and deterministic test probe | Observation is cloneable but request authority stays centralized |
| Mutable lifecycle | `App`, through `Lifecycle` | Retains the existing single transition authority |
| First terminal service failure | The running container, returned through `App::run` | Failure reaches the composition-root caller after cleanup |
| Shutdown trigger | Injected runner input; `main` supplies `ctrl_c` | Tests do not depend on OS signals and services do not own process policy |

Registration identity should be a stable explicit name or newtype rather than
Rust type identity: duplicates then have deterministic diagnostics and the
registry can report which service failed. Registration order is the candidate
startup order and reverse registration order is the candidate logical shutdown
order. A single `JoinSet` alone does not enforce reverse service-specific drain,
so ADR-0037 must either accept cooperative shared cancellation followed by full
join, or define an explicit per-service stop phase.

## Failure decision matrix

ADR-0037 must decide every row before production implementation claims a
complete lifecycle contract.

| Case | Required classification and cleanup question | Candidate direction |
| --- | --- | --- |
| Duplicate registration | Does `build` reject the second name before execution? | Reject deterministically; start nothing |
| Empty registry | Is an app with no services valid? | Permit for construction tests, but it must still wait for injected shutdown when run |
| Startup failure before any task | Which error identifies the service and state? | Return named startup failure; move through cleanup to `Stopped` |
| Partial startup failure | How are already-started services cancelled and joined? | Cancel all started services, join all handles, then return the original named failure plus cleanup context if needed |
| Unexpected successful service exit | Is early `Ok` a failure while the app is running? | Treat as unexpected exit unless cancellation was already requested |
| Service-reported error | Which service identity and source reach the caller? | Cancel siblings, join everything, return named service failure |
| Join panic | How is `JoinError` classified? | Treat as terminal service-task failure and clean up siblings |
| Task cancellation | Was cancellation container-requested or external/abort-driven? | Normal only after owned cancellation; otherwise terminal failure |
| External shutdown request | What wins if a service fails concurrently? | ADR must define deterministic precedence; service failure should not be hidden |
| Repeated registration/build | Can builders and separately built apps be independent? | Yes; no process-global registry or cancellation state |
| Repeated `run` on one app | Consuming `self` currently prevents it | Preserve single-run ownership unless an explicit restart contract is later accepted |
| Repeated cancellation request | Is shutdown idempotent? | Yes; it must not skip joining or alter the terminal result |
| Graceful shutdown stalls | Is there a timeout and abort policy? | Leave timeout value unselected; Sprint 15 must not invent one silently |
| Cleanup failure after primary failure | Which error is returned? | Preserve the primary failure and retain cleanup failure as structured context |

The unresolved timeout row is not a blocker for a cooperative, deterministic
first slice whose test services always observe cancellation. It is a blocker to
claiming bounded shutdown for arbitrary production services. ADR-0037 must state
that limitation explicitly if no timeout is accepted.

## Internal observable state boundary

Sprint 15 may expose transport-neutral, in-process evidence required to prove
the container contract:

- lifecycle state (`Initializing`, `Running`, `Stopping`, `Stopped`);
- ordered service-start and service-stop events carrying stable service names;
- whether cancellation has been requested;
- active/remaining owned task count;
- terminal result classified as shutdown, service exit, service error, or join
  failure.

This state is an application/testing boundary, not the Sprint 16 HTTP contract.
Sprint 15 must not define endpoint paths, status codes, JSON fields, liveness or
readiness promises, probe timeouts, or public wire compatibility. In
particular, `Running` can be used as a deterministic internal milestone only
after required services have acknowledged startup; it must not be described as
HTTP readiness until Sprint 16 accepts that mapping.

Logs and the current banner are diagnostic output, not observable state. Tests
must use owned channels or state snapshots rather than matching log/stdout text.

## Candidate public library boundary

Add `apps/runtime/src/lib.rs` and publicly re-export the smallest transport-free
surface needed by the binary, future adapters, and `apps/runtime/tests/`:

- `App` and `AppBuilder` for composition and execution;
- `AppState` and immutable `RuntimeConfig` access;
- `LifecycleState` as an in-process lifecycle observation;
- a service registration/definition trait or concrete task factory with stable
  service identity and a cancellation receiver;
- `RuntimeError` with named registration/start/run/join classifications;
- an injectable shutdown future or handle accepted by `App::run`.

Private implementation details should include the concrete registry storage,
`JoinSet`, cancellation sender, and transition mutation. The public service
boundary must be implementable by deterministic test services without external
fixtures and reusable by an HTTP adapter without depending on Axum types.

The binary remains responsible only for choosing default configuration,
registering production services, selecting `tokio::signal::ctrl_c`, and
reporting a terminal process error. This satisfies ADR-0002 without exporting
`main` or placing composition inside an adapter.

## Deterministic test oracle

No external fixture, socket, filesystem watch, child process, or remote service
is required. In-memory test services can coordinate with bounded Tokio channels
and cancellation receivers.

### Positive oracle

1. Register two controlled services in a known order.
2. Each service sends a startup acknowledgement and then awaits cancellation.
3. The test drives `App::run` with a one-shot shutdown future and waits for both
   acknowledgements through a bounded channel, not elapsed time.
4. Before releasing shutdown, assert that the run future has not completed and
   that lifecycle/event evidence reports both services started. This proves the
   application is genuinely long-running.
5. Release shutdown, have both services acknowledge cancellation and exit, and
   assert the accepted stop ordering, `Stopped` terminal lifecycle, successful
   result, and zero remaining owned tasks.

### Negative oracles

- Register the same identity twice and assert a named duplicate error before
  any startup acknowledgement.
- Make the second service fail its startup acknowledgement; assert cancellation
  and join acknowledgement from the first service, the second service identity
  in the terminal error, `Stopped`, and zero remaining tasks.
- Let a running service return an error before shutdown; assert sibling
  cancellation/join, named error propagation, and no detached task.
- Panic one controlled task and assert a named join-failure classification,
  sibling cleanup, and zero remaining tasks.
- Construct and run two independent apps sequentially to prove registry and
  cancellation state are not global.

Tests should use `tokio::sync::{mpsc, oneshot, watch}` handshakes and poll/join
completion directly. A bounded Tokio timeout may guard a test against a hang and
produce a clear failure, but it must not be used as the event being tested.
There must be no `sleep`, scheduler-yield counting, stdout matching, real
`ctrl_c`, real network listener, or platform-specific process signal.

## Candidate implementation slice

1. Extract the existing Runtime modules behind a library target without moving
   ownership out of Runtime or changing current configuration semantics.
2. Add an ordered, build-time service registry with explicit duplicate
   rejection and no global state.
3. Add container-owned cancellation and task ownership using existing Tokio
   APIs; require every started task to reach a joined terminal state.
4. Make `App::run` asynchronous and wait on the injected shutdown request or a
   terminal service outcome before entering `Stopping`.
5. Keep production `ctrl_c` wiring in the thin async binary and prove behavior
   through public in-memory integration tests.

HTTP routes, readiness schema, workspace/graph services, file watching,
persistence, and CLI behavior remain deferred to Sprints 16-21. The slice must
not register a placeholder transport merely to keep the process alive.

## ADR decisions and remaining unknowns

ADR-0037 can proceed from repository evidence. It must accept or reject:

1. the stable service identity type and duplicate-registration phase;
2. the public service factory/runner signature and how startup acknowledgement
   differs from task execution;
3. cooperative shared cancellation versus an explicit reverse-order stop hook;
4. startup, early-exit, service-error, join-error, and concurrent shutdown/error
   precedence;
5. the public internal observation mechanism and which pieces remain test-only;
6. whether Sprint 15 explicitly has no production shutdown timeout, or accepts
   a timeout/abort policy;
7. the exact public exports from the new library target.

No evidence blocker requires external research or a new dependency. The only
material product-policy unknown is the graceful-shutdown timeout. It can remain
deferred only if ADR-0037 clearly limits Sprint 15 to cooperative services and
does not claim a bounded shutdown guarantee.

## Evidence and validation record

- Repository baseline: commit `9c2beeaa7b8968f9279722607d8e5da138f03bb9`.
- Direct Runtime dependencies verified with
  `cargo tree -p oneagent-runtime --depth 1`.
- Existing test inventory verified with
  `cargo test -p oneagent-runtime -- --list`: five tests, zero benchmarks.
- Tokio 1.53.0 source installed by the lockfile was inspected for
  `JoinSet::spawn`, `JoinSet::join_next`, `JoinSet::abort_all`,
  `watch::Sender::subscribe`, and `signal::ctrl_c`.
- Runtime history currently traces to the repository stabilization commit
  `2c286712`; no later Runtime implementation migration is hidden in path
  history.
