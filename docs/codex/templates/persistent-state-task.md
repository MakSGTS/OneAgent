# Persistent State Task Template

## Purpose

Use this template for one accepted persisted-state schema, storage,
invalidation, compatibility, migration, corruption-recovery, or integration
slice.

## Recommended profile

- `docs/codex/profiles/persistent-state-implementation.md`

## Required task-specific sections

- Authoritative ADRs / architecture documents
- Prerequisites / required gate
- Task
- Canonical authority and persisted-state owner
- Persisted envelope, payload, and schema version
- Cache identity and deterministic invalidation inputs
- Compatibility and migration contract
- Corruption classification and recovery behavior
- Filesystem, replacement, and cleanup behavior
- Runtime lifecycle and observability compatibility, when applicable
- Clean-rebuild equivalence oracle
- Scope
- Included
- Excluded
- Acceptance Criteria
- Task-specific Validation
- Suggested commit message (recommendation only)

## Additional acceptance requirements

- Use only schema, identity, invalidation, compatibility, migration, corruption,
  recovery, and filesystem behavior established by accepted architecture or the
  task scope.
- Keep canonical semantic state authoritative and validate the complete loaded or
  migrated result before publication.
- Reject incompatible, corrupt, partial, stale, or unverifiable state through the
  accepted deterministic outcome without guessing or silent partial repair.
- Make replacement and cleanup behavior explicit and prove that failed or
  interrupted writes cannot publish a partial current entry.
- Prove valid-hit and recovery results equivalent to a clean build from the same
  accepted inputs.
- Use disposable, task-owned storage and deterministic failure injection; do not
  depend on host-global cache state or arbitrary sleeps.

## Additional report sections

- Schema and ownership model
- Identity and invalidation evidence
- Compatibility and migration matrix
- Corruption and recovery evidence
- Filesystem replacement and cleanup evidence
- Clean-rebuild equivalence evidence

## Additional validation

- Run non-zero focused tests for valid hits, misses, invalidation, incompatible
  versions, corruption, failed replacement, recovery, and equivalence applicable
  to the slice.
- Run public Runtime integration tests when startup, publication, health,
  shutdown, or consumer behavior is claimed.
- Run affected package checks and complete workspace validation when production
  Rust, public APIs, Cargo manifests, Runtime behavior, or persisted formats
  change.
