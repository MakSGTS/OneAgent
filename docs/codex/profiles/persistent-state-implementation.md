# Persistent State Implementation Profile

## Purpose

Use this profile for implementing one accepted persisted-state schema, storage,
invalidation, compatibility, migration, corruption-recovery, or integration
slice.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/persistent-state.md`

## Task-family expectations

- Reference accepted schema, ownership, compatibility, invalidation, migration,
  corruption, recovery, and filesystem decisions instead of selecting them
  during implementation.
- Preserve the canonical in-memory semantic authority and treat persisted state
  only as the accepted recoverable representation or derived cache.
- Keep encoding, storage, Runtime integration, and public consumers in their
  accepted ownership layers.
- Prove deterministic invalidation, rejected-state containment, recovery, and
  clean-rebuild equivalence through public or production-representative paths.
- Do not combine unresolved persistence architecture with implementation; use a
  preceding architecture task.
