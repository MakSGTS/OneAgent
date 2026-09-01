# Diagnostics Engine Implementation Profile

## Purpose

Use this profile for deterministic source-independent diagnostic identity,
normalization, orchestration, suppression, bounded reports, immutable snapshot
composition, or diagnostic-result projection over accepted canonical evidence.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/context-management.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/diagnostics-engine.md`
- `docs/codex/workflows/persistent-state.md` when diagnostic reports or policy
  enter persisted state
- `docs/codex/workflows/runtime-service.md` when immutable Runtime snapshots,
  lifecycle, transport, or supported clients change
- the applicable protocol or IDE workflow when public diagnostic projections
  or UI behavior change

## Task-family expectations

- Keep accepted semantic facts, validation, provenance, and source locations in
  their canonical owners; a Diagnostics Engine consumes and reports evidence.
- Define typed family and identity, observable content, total ordering,
  duplicate/collision behavior, suppression authority, bounds, summaries,
  errors, and sensitive-data rules.
- Make every suppressed, omitted, truncated, or failed outcome explicit and
  reconcile every summary counter.
- Prove deterministic results across input reorder, exact duplicates,
  conflicting evidence, repeated execution, snapshot rebuild, and persistence
  when applicable.
- Keep general rule registration/execution, new producers, diagnostics UI,
  mutable documents, fixes, edits, and telemetry outside the task unless
  separately accepted and explicitly included.
