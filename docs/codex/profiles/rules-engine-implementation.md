# Rules Engine Implementation Profile

## Purpose

Use this profile for deterministic source-independent rule identity,
registration, dependency validation, configuration, execution, result
production, or integration with accepted diagnostic evidence.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/rules-engine.md`
- `docs/codex/workflows/diagnostics-engine.md` when rule results enter the
  accepted diagnostic domain or report
- `docs/codex/workflows/persistent-state.md` when rule configuration or
  execution state enters persisted state
- `docs/codex/workflows/runtime-service.md` when immutable Runtime snapshots,
  lifecycle, cancellation, or supported clients change
- the applicable protocol or IDE workflow when public rule configuration,
  execution, results, or UI behavior change

## Task-family expectations

- Keep semantic facts, validation, diagnostics, provenance, and source
  locations in their accepted owners; a Rules Engine evaluates accepted
  immutable inputs without becoming a competing authority.
- Define typed rule identity, registration ownership, duplicate behavior,
  dependency validation and ordering, configuration authority, applicability,
  execution lifecycle, bounds, errors, and result contracts.
- Make cycles, missing dependencies, disabled or inapplicable rules,
  cancellation, rule failure, partial execution, and conflicting results
  explicit and deterministic.
- Prove equivalent results across registration and input reorder, repeated
  execution, dependency graphs, configuration changes, snapshot rebuild, and
  persistence when applicable.
- Keep dynamic plugin loading, scripting, remote rule acquisition, source
  mutation, automatic fixes, safe edits, telemetry, and performance or
  security claims outside the task unless separately accepted and explicitly
  included.
