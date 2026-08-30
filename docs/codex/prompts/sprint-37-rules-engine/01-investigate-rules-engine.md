# Investigate Sprint 37 Rules Engine

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/investigation.md`
- `docs/codex/templates/investigation-task.md`

## Authoritative documents and evidence

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/diagnostics-engine-investigation.md`
- `docs/architecture/diagnostics-engine-evidence.md`
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/adr/0058-diagnostics-engine.md`
- `docs/reviews/sprint-36-diagnostics-engine.md`
- repository graph queries and validation, Diagnostics Engine, Workspace
  snapshots/cache/rebuild, Runtime cancellation and projections, fixtures,
  tests, and consumers

## Prerequisites / required gate

- The committed Sprint 37 planning baseline and Rules Engine framework
  prerequisite are ancestors of HEAD.
- Sprint 36 is completed and Sprint 37 is the unique eligible target.
- The working tree has no uncommitted task-created changes.

## Task

Create `docs/architecture/rules-engine-investigation.md` and update only the
Sprint 37 Roadmap state needed to record Task 1 start. Produce decision-ready
repository evidence for ADR-0059 without production implementation.

## Questions and required evidence

- Inventory every accepted immutable semantic, validation, diagnostic,
  Workspace, configuration, cancellation, and lifecycle input that a rule could
  consume without reading source or creating a competing authority. Record
  owner, API, lifetime, bounds, ordering, tests, and compatibility constraints.
- Confirm that no general Rules Engine, registry, rule identity, dependency
  graph, rule configuration source, or rule-result aggregate already exists.
  Separate similarly named Tool Policy rules, validators, parser checks, and
  test helpers from candidate Rules Engine concepts.
- Trace `SemanticGraph`, graph validation, `DiagnosticEngine`,
  `DiagnosticIdentity`, `DiagnosticReport`, Workspace Configuration
  snapshots, persistent cache reconstruction, watcher rebuild, Runtime
  cancellation, MCP/LSP projections, and every public/internal consumer likely
  affected.
- Compare evidence-supported owners and dependency directions for rule domain,
  registry, planning, execution, configuration, and aggregate results without
  selecting one. Identify Cargo/dependency impact and any approval gate.
- Define decision-ready identity, duplicate/conflicting registration, registry
  ownership/lifecycle, enumeration, and bound alternatives.
- Define dependency meanings and deterministic planning alternatives. Cover
  missing, self, duplicate, incompatible, cyclic, independent, chain, diamond,
  reordered, and exact/over-bound cases.
- Inventory repository-owned configuration authorities. If none exists, record
  the smallest static first-slice options and keep file grammar, environment,
  persistence, protocol, and UI configuration unresolved rather than invented.
- Define decision-ready applicability, disabled/inapplicable status, execution
  lifecycle, cancellation, failure containment, atomic/partial behavior,
  ordering, bounds, error redaction, result aggregation, and repeated-execution
  alternatives.
- Determine how rule-produced evidence can enter the accepted ADR-0058 identity,
  duplicate/conflict, suppression, order, summary, provenance, location, and
  completeness boundary without weakening it or manufacturing graph facts.
- Build deterministic repository-owned oracles for empty, positive,
  registration conflict, dependency topology, configuration, inapplicability,
  failure, cancellation, result collision, exact/over bound, reordered,
  repeated, snapshot rebuild, cache reconstruction, watcher replacement, MCP/
  LSP compatibility, and cleanup behavior.
- Record every ADR-0059 decision, exact likely production/test areas, migration
  and compatibility cost, rejected candidates supported by evidence, first
  rule or synthetic conformance seam, and deferred plugins, scripts, remote
  rules, source mutation, fixes, safe edits, UI, telemetry, performance, and
  security scope.

## Excluded scope

Architecture acceptance, Rust or Cargo changes, dependency additions, rule
implementation, production configuration, persistence schema, new protocol or
IDE capability, source parsing, graph-semantic changes, automatic fixes,
Coverage transition, and Sprint completion.

## Validation

Run focused non-mutating source/API/consumer/test/fixture/history inventories
and existing Graph/Analysis/Workspace/cache/watching/Runtime/MCP/LSP tests
needed to confirm the current baseline. Validate Markdown links,
`git diff --check`, Roadmap state, and unrelated-change absence. Record exact
commands, non-zero matched counts, and inconclusive evidence separately.

## Suggested commit message

`Investigate Sprint 37 rules engine`

## Final report additions

Report canonical inputs and owners, absence or presence of existing rule
concepts, owner/dependency candidates, registry/dependency/configuration/
execution/result alternatives, diagnostic integration, consumer and
compatibility inventory, deterministic oracles, unresolved ADR questions, and
unchanged production behavior.
