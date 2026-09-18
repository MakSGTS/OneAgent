# Runtime Service Task Template

## Purpose

Use this template for one accepted long-running service, Runtime lifecycle,
transport-adapter, health, observability, or supported client implementation
slice.

## Recommended profile

- `docs/codex/profiles/runtime-service-implementation.md`

## Required base template

- `docs/codex/templates/task-prompt.md`

## Required task-specific sections

- Runtime and service ownership
- Lifecycle and state transitions
- Concurrency and task ownership
- Cancellation, failure, and shutdown policy
- Health, readiness, and observability contract, when applicable
- Transport and client compatibility, when applicable

## Additional acceptance requirements

- Use only lifecycle, failure, cancellation, timeout, and compatibility behavior
  established by accepted architecture or the task scope.
- Keep every spawned task, listener, channel, lock, and long-lived resource under
  one explicit owner and prove its terminal behavior.
- Preserve the composition-root boundary and keep adapters free from hidden
  dependency construction or semantic authority.
- Distinguish startup failure, service failure, cancellation, graceful shutdown,
  and forced termination whenever those outcomes are applicable.
- Keep health and readiness derived from owned lifecycle evidence rather than
  mutable labels or log messages.
- Prove transport and client behavior through public integration entry points
  when they are included.
- Use deterministic synchronization and bounded time control; arbitrary sleeps
  are not acceptance evidence.

## Additional report sections

- Ownership and lifecycle model
- Concurrency and resource inventory
- Cancellation and shutdown behavior
- Health and observability evidence
- Transport/client compatibility
- Repeated-run and resource-cleanup evidence

## Additional validation

- Run focused Runtime lifecycle, failure, cancellation, shutdown, and resource
  cleanup tests applicable to the slice.
- Run non-zero public client/server integration tests when transport or client
  behavior is claimed.
- Run affected package checks and the complete workspace validation when Rust,
  public APIs, Cargo manifests, lifecycle, or service behavior changes.
