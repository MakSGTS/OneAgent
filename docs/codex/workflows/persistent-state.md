# Persistent State Workflow

Use this workflow for persisted semantic state, on-disk caches, deterministic
invalidation, compatibility, migration, corruption handling, and recovery.

## Authority and schema ownership

- Identify the canonical in-memory authority, persisted representation owner,
  storage owner, and every reader and writer before implementation.
- Define the complete persisted envelope and payload, stable schema identity,
  version ownership, required metadata, and deterministic ordering from accepted
  architecture. Do not serialize incidental private layout as an implicit
  compatibility contract.
- Keep persisted derived state from becoming a second semantic authority. Loaded
  state must pass the accepted structural, semantic, provenance, and compatibility
  validation before publication or use.
- Define which state is intentionally not persisted and how it is reconstructed.

## Identity and deterministic invalidation

- Define cache identity and every input that can change the validity of persisted
  state, including source content, builder semantics, schema version, options,
  and workspace/configuration identity when applicable.
- Base validity on deterministic repository or source evidence rather than wall
  clock time, directory enumeration order, process identity, or platform-specific
  absolute paths unless accepted architecture explicitly requires them.
- Treat incomplete, ambiguous, mismatched, or unverifiable validity evidence as
  an invalid or unusable entry according to the accepted contract; never repair
  it by guessing.
- Prove that a valid hit is equivalent to a clean build for the same accepted
  inputs and that every accepted invalidating change prevents stale publication.

## Compatibility and migration

- State the exact readable and writable schema versions and how current, older,
  newer, and unknown versions are classified.
- Implement migration only for versions and transformations explicitly accepted
  by architecture. Preserve source evidence and validate the complete migrated
  result before replacement or publication.
- Define rollback and interruption behavior for writes and migrations. Do not
  expose a partial replacement as current persisted state.
- Keep forward compatibility, downgrade support, and lossy conversion out of
  scope unless they are explicitly accepted and tested.

## Corruption, recovery, and filesystem safety

- Classify missing, truncated, malformed, incompatible, semantically invalid,
  unreadable, and partially written state through stable accepted outcomes.
- Define whether each rejected state is ignored, retained for diagnosis,
  replaced, or surfaced as a failure. Recovery must not publish rejected bytes
  or mutate canonical source inputs.
- Keep cache paths contained under the accepted owner and make temporary-file,
  replacement, cleanup, permission, and symlink behavior explicit when those
  concerns are in scope.
- Prove interrupted or failed writes preserve the last accepted state or produce
  a deterministic miss/failure according to the accepted contract.

## Integration and deterministic testing

- Place blocking filesystem and encoding work at the accepted execution boundary
  and preserve Runtime cancellation, shutdown, health, and observer contracts
  when integration is in scope.
- Test valid hits, misses, every accepted invalidation input, incompatible
  versions, corruption classes, write failure, migration, recovery, repeated
  fresh runs, deterministic bytes when promised, and clean-rebuild equivalence as
  applicable to the slice.
- Prefer disposable directories, controlled readers/writers, explicit failure
  injection, and repository-owned fixtures over timing races or host-global
  cache state.
- Run affected package tests and complete workspace validation when production
  Rust, public APIs, Cargo manifests, Runtime behavior, or persisted formats
  change.

## Boundary

This workflow does not choose a serialization format, schema, cache key,
fingerprint algorithm, checksum, compression, encryption, storage location,
atomic-write primitive, migration policy, eviction policy, or Runtime lifecycle.
Those decisions belong to accepted ADRs or the current task. It does not require
cross-process locking, remote storage, performance claims, or security claims
unless the accepted scope includes them.
