# Investigate Sprint 36 Diagnostics Engine

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
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0024-reference-request-provenance.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0054-lsp-adapter.md`
- `docs/reviews/v0.6-release-review.md`
- repository diagnostic producers, graph validator, reports and diffs,
  Workspace snapshots/cache, MCP/LSP projections, fixtures, tests, and consumers

## Prerequisites / required gate

- The committed Sprint 36 planning baseline is HEAD.
- v0.6 is completed and Sprint 36 is the unique eligible target.
- The working tree has no uncommitted task-created changes.

## Task

Create `docs/architecture/diagnostics-engine-investigation.md` and update only
the Sprint 36 Roadmap state needed to record Task 1 start. Produce
decision-ready repository evidence for ADR-0058 without production
implementation.

## Questions and required evidence

- Inventory every current typed diagnostic and validation input family,
  producer, code/severity/kind vocabulary, identity field, provenance/location
  source, ordering rule, duplicate behavior, recoverability meaning, and
  positive/negative fixture.
- Trace `SemanticDiagnostic`, validation issues, report summaries, build diffs,
  reference-request evidence, Workspace snapshots, persistent cache DTOs,
  rebuild/watching paths, MCP `oneagent.diagnostics`, LSP pull diagnostics, and
  every public or internal consumer and test.
- Distinguish Graph-owned facts and validation from orchestration-owned
  normalization, filtering, suppression, bounds, summaries, and projections.
  Identify layering and dependency candidates without selecting one.
- Determine which existing diagnostics have a stable source node and exact
  confined source span, which are configuration-level only, and how missing,
  ambiguous, incompatible, duplicate, reordered, and repeated evidence is
  currently represented.
- Compare evidence-supported identity, deduplication, severity/category,
  suppression, truncation versus fail-closed bounds, summary, and report models.
  Record stability and compatibility costs for each candidate.
- Define decision-ready options for immutable engine inputs and output,
  orchestration timing, Workspace/cache ownership, error behavior, public API
  migration, MCP schema/result compatibility, and LSP located-only projection.
- Build a deterministic test matrix for empty, positive, mixed-family,
  duplicate, collision, suppressed, exact-bound, one-over, reordered,
  repeated-build, cache round-trip, rebuild, missing-location, invalid-location,
  MCP Tool Policy, LSP confinement, public-process, and compatibility cases.
- Identify the exact boundary with Sprint 37. Configurable rule registration,
  discovery, third-party rules, scripting, dynamic execution, and rule-owned
  diagnostics must remain deferred unless repository evidence proves a smaller
  prerequisite contract is mandatory.
- Record every decision ADR-0058 must make, exact production areas and tests
  likely affected, dependency approval gates, accepted evidence gaps, rejected
  candidates supported by evidence, first slice, and deferred UI, mutable
  document, edits, telemetry, remote, and performance scope.

## Excluded scope

Architecture acceptance, Rust changes, Cargo changes, new dependencies, new
diagnostic producers, parser or graph-semantic changes, MCP/LSP implementation,
rule-registry implementation, diagnostics UI, Coverage transition, and Sprint
completion.

## Validation

Run focused non-mutating source/API/consumer/fixture/test/history inventories,
existing diagnostic/validation/Workspace/MCP/LSP tests needed to confirm the
current baseline, Markdown link checks, `git diff --check`, and an
unrelated-change audit. Record exact commands, matched test counts, and
inconclusive evidence; zero-match results are not proof.

## Suggested commit message

`Investigate Sprint 36 diagnostics engine`

## Final report additions

Report confirmed input families and owners, identity/order/location behavior,
consumer and compatibility inventory, candidate boundaries, test oracles,
unresolved ADR questions, Sprint 37 separation, and unchanged production
behavior.
