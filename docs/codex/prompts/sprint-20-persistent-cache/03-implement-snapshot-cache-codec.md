# Implement Sprint 20 Snapshot Cache Codec

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/persistent-state-implementation.md`

## Template

`docs/codex/templates/persistent-state-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 20 execution plan
- `docs/architecture/persistent-cache-investigation.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`

## Prerequisites / Required gate

Require committed Task 2 with accepted ADR-0042, successful documentation
validation, and clean task-owned state. If ADR-0042 requires a new production
dependency, require explicit current user approval before changing a manifest or
lockfile. Stop rather than substituting another format, schema, payload,
compatibility, validation, or error contract in code.

## Task

Implement the accepted versioned complete `WorkspaceSnapshot` cache codec with
checked domain reconstruction, complete validation, deterministic encoding, and
focused round-trip and rejection evidence. Perform no filesystem storage or
Runtime publication in this task.

## Canonical authority and persisted-state owner

- Keep in-memory `WorkspaceSnapshot` and each canonical `SemanticGraph` as the
  semantic authority; codec records are private source-neutral persistence
  values only.
- Reconstruct through accepted checked constructors and insertion/validation
  APIs rather than serializing private indexes or bypassing invariants.

## Persisted envelope, payload, and schema version

- Implement exactly the ADR-0042 envelope, complete configuration/graph/build-
  evidence payload, schema/build versions, field vocabulary, ordering, and
  excluded/reconstructed state.
- Preserve every accepted node payload, edge, provenance, diagnostic, reference
  request/statistic, report, format, root, configuration identity, and order.

## Compatibility, corruption, and equivalence

- Distinguish current, incompatible, malformed, partial, duplicate, invalid,
  inconsistent, and unsupported values through accepted typed outcomes.
- Validate the reconstructed graph and complete build evidence before returning
  a usable snapshot; never publish or silently repair rejected state.
- Prove a valid round trip equivalent to a clean build and deterministic bytes
  where ADR-0042 promises them.

## Scope

### Included

- Codec implementation and focused tests for empty, EDT, Designer XML, mixed,
  payload/provenance-rich, diagnostic/reference-rich, deterministic reorder,
  repeated encoding, current-version round trip, incompatible version,
  malformed/partial input, duplicates, invalid endpoints/payloads/graph/report,
  and clean-build equivalence.
- Manifest/lockfile changes only when explicitly approved and required by the
  accepted ADR.

### Excluded

Filesystem paths, cache keys/fingerprints, load/write/replacement, Runtime
configuration or service integration, watcher changes, HTTP/CLI APIs, fixture or
current-state documentation changes, new semantic facts, incremental state,
compression, encryption, migration beyond the accepted codec boundary,
performance claims, and prompt/Roadmap changes.

## Acceptance Criteria

- Encoding and decoding implement every accepted ADR-0042 field and closed value
  without `Debug`/`Display` parsing, lossy fixture-only projections, private-index
  persistence, or an unchecked fallback for future variants.
- Every current production graph payload, provenance, diagnostic, request,
  statistic, report, configuration field, and ordering rule has non-zero
  positive round-trip evidence.
- Incompatible/malformed/partial/duplicate/invalid/inconsistent values are
  rejected before a snapshot becomes usable and have deterministic typed tests.
- Re-encoded accepted state is deterministic as promised, reconstructed graphs
  pass canonical validation, and complete observations match a clean build.
- No filesystem I/O, Runtime publication, lifecycle change, wire change, or
  deferred capability is introduced.

## Repository Safety

Modify only codec implementation, its module wiring, explicitly approved
manifests/lockfile, and focused test paths proved necessary by ADR-0042. Preserve
`.codex/`, prompt suites, Roadmap, ADRs, tracked fixtures, current-state docs,
Runtime lifecycle/orchestration, HTTP schemas, graph/adapter semantics, and
unrelated files. Stage only enumerated task-owned paths.

## Task-specific Validation

- List and run exact non-zero focused codec, variant-coverage, deterministic-
  bytes, rejection, validation, and clean-build equivalence tests.
- Run affected Runtime and graph/package tests as applicable.
- Run the canonical complete workspace validation from
  `docs/codex/core/validation.md`.
- `git status --short`

## Suggested commit message

`Implement Sprint 20 snapshot cache codec`

## Final report additions

Report schema/payload implementation, reconstruction/validation boundary,
variant and rejection coverage, deterministic encoding, clean-build equivalence,
dependency approval/changes, focused/full validation, changed paths, commit,
and final Git state.
