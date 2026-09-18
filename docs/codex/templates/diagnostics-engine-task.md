# Diagnostics Engine Task Template

## Purpose

Use this template for one accepted diagnostic identity, normalization,
orchestration, suppression, reporting, snapshot, or public-projection slice.

## Recommended profile

- `docs/codex/profiles/diagnostics-engine-implementation.md`

## Required base template

- `docs/codex/templates/task-prompt.md`

## Required task-specific sections

- Canonical diagnostic inputs and authority
- Identity, vocabulary, ordering, and collision contract
- Suppression and disposition contract
- Bounds, completeness, summaries, and errors
- Location, provenance, and sensitive-data policy
- Snapshot, persistence, protocol, and consumer impact, when applicable
- Evidence corpus and deterministic oracle

## Additional acceptance requirements

- Consume only accepted canonical inputs and preserve their source owners.
- Keep identity fields distinct from observable modified content and define
  exact duplicate and conflict behavior.
- Define suppression authority and retain the accepted suppressed evidence;
  never treat suppression as validation success or evidence deletion.
- Apply explicit bounds before cloning or projecting producer-owned data and
  reconcile total, active, suppressed, omitted, and returned counts.
- Prove input-order independence, exact and one-over limits, error redaction,
  repeated execution, and every changed snapshot or protocol boundary.
- Audit affected reports, diffs, validators, caches, schemas, capabilities,
  policy gates, clients, and source-confinement rules.
- Do not implement a general Rules Engine or new diagnostic producer unless a
  separate accepted task explicitly owns it.

## Additional report sections

- Canonical input and ownership model
- Identity, ordering, duplicate, and conflict behavior
- Suppression and disposition evidence
- Bounds, completeness, summary, and error behavior
- Location, provenance, and sensitive-data evidence
- Snapshot/persistence/protocol compatibility
- Deterministic evaluation results
- Deferred Rules Engine and product-integration scope

## Additional validation

- Run non-zero focused family, identity, duplicate, conflict, suppression,
  ordering, bound, summary, filter, redaction, and repetition tests applicable
  to the slice.
- Run affected graph, analysis, Workspace, persistence, Runtime, protocol,
  public-process, or client checks when their public behavior changes.
- Run full workspace validation for production behavior, public APIs, Cargo,
  snapshot, cache, or protocol changes as required by
  `docs/codex/core/validation.md`.
