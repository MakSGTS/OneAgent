# Runtime Service Workflow

Use this workflow for long-running services, application lifecycle orchestration,
Runtime-owned background work, transport adapters, and supported clients.

## Ownership and lifecycle contract

- Identify the composition root, service owner, shared-state owner, and every
  resource that must outlive one request or task.
- Define explicit construction, initialization, start, ready, stop, and stopped
  boundaries that match accepted architecture.
- Keep service registration and dependency construction separate from service
  execution. Do not hide construction or mutable global state in adapters.
- Define startup ordering, partial-start failure behavior, cleanup ownership,
  repeated start/stop behavior, and the observable terminal state.
- Do not leave detached tasks, listeners, channels, locks, or temporary resources
  after a failed start or successful shutdown.

## Concurrency, cancellation, and shutdown

- Inventory every spawned task and asynchronous resource. The Runtime must own
  their handles or an accepted structured owner must do so.
- Define how cancellation is requested, observed, propagated, and distinguished
  from service failure.
- Define deterministic shutdown ordering and how task failures are surfaced.
- State the timeout, draining, abort, and retry policy only when accepted
  architecture or the task scope defines it; do not invent defaults.
- Keep blocking work off asynchronous executor threads and make synchronization
  ownership explicit.

## Health, readiness, and observability

- When health or readiness is in scope, define each state from owned lifecycle
  evidence and distinguish process liveness from ability to serve accepted work.
- Do not report ready before required services have started or after shutdown has
  begun.
- Define stable observable events or state needed to prove startup, service
  failure, cancellation, and shutdown without relying on log wording alone.
- Keep secrets and unbounded payloads out of diagnostics and structured fields.

## Transport and client boundaries

- Keep domain and service behavior independent from HTTP, MCP, LSP, CLI, or IDE
  transport details unless accepted architecture assigns ownership otherwise.
- Define request/response compatibility, error mapping, cancellation propagation,
  and connection lifecycle at the adapter boundary.
- Prove client/server integration through public entry points when a supported
  transport or client is in scope; handler-only tests are insufficient.
- Preserve transport-independent services when adding another adapter or client.

## Deterministic testing

- Test successful startup and shutdown, partial startup failure, service failure,
  cancellation, repeated lifecycle execution, and ordering where applicable.
- Prefer explicit synchronization, bounded channels, controlled clocks, and
  deterministic probes over arbitrary sleeps or timing races.
- Assert that tasks and resources terminate and that failures reach the owning
  caller with the accepted error classification.
- Run the affected Runtime package tests and the full workspace validation when
  production Rust, public APIs, Cargo manifests, lifecycle, or service behavior
  changes.

## Boundary

This workflow does not choose a concrete executor, service registry, cancellation
primitive, shutdown timeout, health schema, transport, endpoint, or client
protocol. Those decisions belong to accepted ADRs or the current task. It does
not require transport or health work in a lifecycle-only slice, but it requires
their ownership and deferred boundary to remain explicit.
