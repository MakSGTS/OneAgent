# Runtime Service Implementation Profile

## Purpose

Use this profile for implementing one accepted long-running Runtime service,
lifecycle, transport-adapter, or supported client slice.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/runtime-service.md`

## Task-family expectations

- Reference accepted lifecycle, ownership, concurrency, cancellation, shutdown,
  health, transport, and observability decisions instead of selecting them
  during implementation.
- Keep composition, service execution, transport adaptation, and client behavior
  in their accepted ownership layers.
- Make every background task and long-lived resource owned and observable.
- Prove failure, cancellation, shutdown, and repeated execution without
  timing-dependent tests.
- Do not combine unresolved Runtime architecture with implementation; use a
  preceding architecture task.
