# Refactoring and Safe Edits Task Template

## Purpose

Use this template for one accepted refactoring target, immutable plan,
precondition, preview, conflict, edit-transaction, rollback, reversibility, or
post-edit semantic-validation slice.

## Recommended profile

- `docs/codex/profiles/refactoring-safe-edits-implementation.md`

## Required base template

- `docs/codex/templates/task-prompt.md`

## Required task-specific sections

- Canonical semantic, source, repository, and authorization boundaries
- Supported refactoring family and production source entry point
- Target, snapshot, source-version, and precondition contract
- Plan and operation identity, vocabulary, ordering, and dependencies
- Duplicate, overlap, conflict, stale-input, and incompatibility behavior
- Bounds, completeness, preview, failures, and sensitive-data policy
- Transaction, filesystem confinement, atomicity, rollback, and reversibility,
  when mutation is included
- Post-edit production rebuild and semantic validation, when mutation is
  included
- Runtime, policy, protocol, client, cache, and watcher impact, when applicable
- Repository-owned evidence corpus and deterministic oracle

## Additional acceptance requirements

- Preserve canonical semantic and source owners; do not derive edit authority
  from impact, diagnostics, Git evidence, paths, or model output.
- Bind every plan to an immutable complete snapshot and explicit preconditions.
  Reject missing, ambiguous, conflicting, stale, incompatible, incomplete, or
  out-of-bound evidence atomically.
- Define stable plan and operation identity, a closed vocabulary, canonical
  ordering, dependencies, duplicates, overlaps, conflicts, and exact/over
  bounds independently of encounter order.
- Make preview deterministic, complete or explicitly non-executable, redacted,
  and read-only. Reconcile every plan and projection counter.
- For mutation, recheck preconditions before writing; confine paths and the
  complete write set; prove the accepted atomicity, rollback, reversibility,
  cancellation, cleanup, concurrent-change, and injected-failure behavior.
- Validate successful edits through the accepted production adapter and
  complete semantic rebuild. Prove unaffected-source preservation and pre-edit
  equivalence after rollback.
- Audit every affected Workspace snapshot, filesystem watcher, Git adapter,
  cache, diagnostics, impact, Tool Policy, Runtime, protocol, process, and
  client boundary.

## Additional report sections

- Authority and authorization boundary
- Supported refactoring and source boundary
- Target, snapshot, and precondition evidence
- Plan identity, operations, ordering, duplicates, and conflicts
- Bounds, completeness, preview, failure, and redaction evidence
- Transaction, confinement, atomicity, rollback, and reversibility evidence
- Post-edit semantic validation and unaffected-source evidence
- Runtime, policy, protocol, client, and compatibility impact
- Deferred refactoring, mutation, Git, remote, UI, and performance scope

## Additional validation

- Run non-zero focused target, precondition, identity, ordering, duplicate,
  overlap, conflict, bound, completeness, preview, redaction, stale-input,
  incompatible-input, reorder, and repetition tests applicable to the slice.
- When mutation is included, run non-zero confined-filesystem, concurrent-change,
  injected-failure, cancellation, atomicity, rollback, rollback-failure,
  reversibility, cleanup, production-rebuild, semantic-validation, and
  unaffected-source tests.
- Run affected Graph, Analysis, Workspace, adapter, cache, Runtime, Tool Policy,
  protocol, public-process, and client checks when their observable behavior
  changes.
- Run full workspace validation for production behavior, public APIs, Cargo,
  source mutation, snapshots, cache, or protocol changes as required by
  `docs/codex/core/validation.md`.
