# Rules Engine Task Template

## Purpose

Use this template for one accepted rule identity, registration, dependency,
configuration, execution, result, diagnostic-integration, snapshot, or public
projection slice.

## Recommended profile

- `docs/codex/profiles/rules-engine-implementation.md`

## Required task-specific sections

- Authoritative ADRs / architecture documents
- Prerequisites / required gate
- Task
- Canonical inputs and ownership
- Rule identity and registration contract
- Dependency and deterministic execution-order contract
- Configuration authority, applicability, and compatibility
- Execution lifecycle, cancellation, failure, and completeness
- Result, diagnostic integration, bounds, and sensitive-data policy
- Snapshot, persistence, protocol, and consumer impact, when applicable
- Evidence corpus and deterministic oracle
- Scope
- Included
- Excluded
- Acceptance Criteria
- Task-specific Validation
- Suggested commit message (recommendation only)

## Additional acceptance requirements

- Consume only accepted canonical inputs and preserve their source owners.
- Define typed identity, registration ownership, deterministic enumeration,
  duplicate/conflict behavior, and registry bounds.
- Validate dependencies before affected execution and prove deterministic order
  for chains, independent rules, diamonds, missing dependencies, and cycles.
- Define configuration authority and distinguish disabled, inapplicable,
  invalid, dependency-blocked, cancelled, failed, partial, and successful
  outcomes.
- Apply explicit bounds before cloning or publishing evidence and keep errors
  closed, bounded, and redacted.
- Map rule-produced diagnostics only through accepted diagnostic identity,
  collision, ordering, suppression, summary, provenance, and completeness
  contracts.
- Prove registration/input reorder, repeated execution, exact and one-over
  limits, failure containment, cancellation, snapshot rebuild, and persistence
  behavior applicable to the slice.
- Audit affected graphs, validators, diagnostic reports, snapshots, caches,
  schemas, capabilities, policy gates, clients, and source-confinement rules.

## Additional report sections

- Canonical input and ownership model
- Rule identity, registry, and duplicate behavior
- Dependency validation and execution ordering
- Configuration and applicability evidence
- Execution lifecycle, cancellation, and failure containment
- Result, diagnostic integration, bounds, and sensitive-data evidence
- Snapshot/persistence/protocol compatibility
- Deterministic evaluation results
- Deferred plugins, edits, UI, and product-integration scope

## Additional validation

- Run non-zero focused identity, registration, duplicate, dependency, cycle,
  ordering, configuration, applicability, bound, failure, cancellation,
  result, diagnostic-integration, redaction, and repetition tests applicable
  to the slice.
- Run affected graph, analysis, Workspace, persistence, Runtime, diagnostics,
  protocol, public-process, or client checks when their public behavior
  changes.
- Run full workspace validation for production behavior, public APIs, Cargo,
  snapshot, cache, or protocol changes as required by
  `docs/codex/core/validation.md`.
