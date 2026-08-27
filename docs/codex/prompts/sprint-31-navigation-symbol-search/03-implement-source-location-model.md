# Implement Sprint 31 Source Location Model

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/graph-implementation.md`

## Template

`docs/codex/templates/graph-model-task.md`

## Authoritative documents

- `docs/adr/0053-navigation-symbol-search.md`
- `docs/architecture/navigation-symbol-search-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`

## Prerequisites / Required gate

Task 2 and accepted ADR-0053 are committed. The exact typed location model,
coordinate system, producer slice, compatibility behavior, and migration
boundary are fixed.

## Task

Implement the accepted source-location graph prerequisite and its bounded
production producer evidence.

## Scope

### Included

The accepted Common source path/span primitives; optional graph fact or node
location representation; construction/access/validation/equality/ordering
behavior; exact serialization or persistence compatibility policy if affected;
accepted EDT and/or Designer producer projection; preservation of BSL
declaration lines already extracted by `oneagent-bsl`; deterministic
deduplication and repeated builds; positive, missing, malformed, incompatible,
ambiguous, Unicode/line-ending, path-normalization, and regression evidence;
and architecture/current-state synchronization owned by this prerequisite.

### Excluded

MCP tools, Runtime search projection, VS Code code, new symbol semantics,
filesystem reads in graph/domain consumers, parsing opaque provenance in a
client, unsupported producer families, LSP, diagnostics, references, and
Coverage transitions not explicitly required by ADR-0053.

## Acceptance Criteria

- The public model exactly implements ADR-0053 with deterministic identity,
  coordinates, equality, ordering, validation, and compatibility behavior.
- Existing node/edge identities, query behavior, provenance, graph validation,
  cache/repeated-build behavior, and unsupported-location handling are
  preserved.
- Every accepted producer emits locations from parsed source evidence; missing
  or unsupported locations remain explicit and never guessed.
- Focused producer and graph tests prove real repository fixtures and repeated
  builds with non-zero counts.

## Task-specific Validation

- Run non-zero focused `oneagent-common`, `oneagent-bsl`,
  `oneagent-graph`, accepted adapter, Workspace/cache, and compatibility
  tests named by ADR-0053.
- Run the canonical full Rust workspace gate and `git diff --check`.

## Suggested commit message

`Implement Sprint 31 source location model`

## Final report additions

Report created/changed public types, coordinates, accepted producer coverage,
missing-location behavior, API/serialization/persistence impact, focused test
counts, repeated-build evidence, and preserved graph behavior.
