# Sprint 37 Rules Engine Execution Loop

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

Use the profile selected by each child prompt.

## Template

`docs/codex/templates/sprint-execution-loop.md`

## Required workflow

`docs/codex/workflows/sequential-sprint-execution.md`

## Canonical authorities

- `AGENTS.md`
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
- the repository semantic graph, diagnostics domain and engine, Workspace
  snapshots/cache, Runtime projections, fixtures, tests, and consumers pinned
  by Task 1
- `docs/adr/0059-rules-engine.md` after Task 2

## Sprint objective and state

Define and implement one source-independent deterministic Rules Engine boundary
for bounded rule identity, registration, dependencies, configuration,
execution, and typed results over accepted immutable evidence, then compose its
accepted diagnostic output into Workspace snapshots without moving Graph,
validation, or Diagnostics Engine authority.

Execution requires the committed planning baseline with subject
`Plan Sprint 37 Rules Engine`, the committed framework prerequisite
`68045f0c Establish Rules Engine task framework`, and completed Sprint 36
review baseline `8240ed1a`. Sprint 37 must be the unique eligible target.
Resolve commit mode from the current user instruction; this stored prompt does
not authorize commits by itself.

The current user's launch authorizes exactly one mandatory fresh-context
read-only reviewer for Task 8. Launch it automatically at the review gate
without separate confirmation. No other delegation is authorized.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-rules-engine.md` | Sprint 37 planning commit and framework prerequisite | Complete rule-input, owner, registry, dependency, configuration, execution, result, diagnostic, compatibility, and oracle evidence | Source/API/test/consumer/history/link gates | `Investigate Sprint 37 rules engine` |
| 2 | `02-define-rules-engine.md` | Task 1 | Accepted ADR-0059 | Evidence/identity/ownership/dependency/configuration/execution/result/compatibility/scope consistency | `Define Sprint 37 rules engine` |
| 3 | `03-implement-rule-registry.md` | Task 2 | Accepted typed rule domain and deterministic registry | Focused identity/registration/duplicate/bound/API tests and workspace gate | `Implement Sprint 37 rule registry` |
| 4 | `04-implement-rule-planning.md` | Task 3 | Deterministic dependency validation, execution planning, configuration, and applicability | Dependency/cycle/order/configuration/applicability tests and workspace gate | `Implement Sprint 37 rule planning` |
| 5 | `05-implement-rule-execution.md` | Task 4 | Bounded rule execution, terminal results, failure containment, and accepted diagnostic integration | Execution/failure/cancellation/result/diagnostic/repetition tests and workspace gate | `Implement Sprint 37 rule execution` |
| 6 | `06-integrate-rule-snapshots.md` | Task 5 | Immutable Workspace and cache/rebuild rule-result composition with unchanged truthful projections | Snapshot/cache/rebuild/lifecycle/reporting/compatibility matrices and workspace gate | `Integrate Sprint 37 rule snapshots` |
| 7 | `07-complete-rules-engine-evidence.md` | Task 6 | Complete evidence, audits, and current-state documentation | Focused/public/full/audit/documentation gates | `Complete Sprint 37 rules engine evidence` |
| 8 | `08-sprint-37-integration-review.md` | Task 7 and all validation | Independent review, reconciliation, artifact consistency, Sprint 38 hand-off, and conditional Sprint 36 suite retirement | Reviewer and primary complete matrices | `Complete Sprint 37 rules engine review` |

Execute strictly in order. Read every child prompt and selected framework module
completely before acting. Stop after the first blocking failure. Do not combine
tasks or treat a zero-match filter as passing evidence.

## Preserved compatibility boundary

Graph remains authoritative for semantic facts, recoverable diagnostics,
validation, provenance, source locations, reports, and diffs. The Diagnostics
Engine remains authoritative for accepted diagnostic identity, normalization,
suppression, ordering, summaries, and complete reports. The Rules Engine may
evaluate only inputs and produce only results accepted by ADR-0059.

Preserve immutable Workspace publication, cache recovery, watcher lifecycle,
the seven-tool MCP catalog and Tool Policy gate, LSP lifecycle and capabilities,
HTTP/CLI/IDE behavior, adapters, source confinement, and existing Coverage
state except for exact additive or migrated behavior accepted by ADR-0059.

## Previous-suite retirement

The verified immediately preceding suite is exactly
`docs/codex/prompts/sprint-36-diagnostics-engine/` with these nine tracked
files and no untracked additions at planning time:

- `00-sprint-36-execution-loop.md`
- `01-investigate-diagnostics-engine.md`
- `02-define-diagnostics-engine.md`
- `03-implement-diagnostic-domain.md`
- `04-implement-diagnostic-orchestration.md`
- `05-integrate-diagnostic-snapshots.md`
- `06-integrate-diagnostic-reporting.md`
- `07-complete-diagnostics-evidence.md`
- `08-sprint-36-integration-review.md`

Only Task 8 may retire those exact files after a non-blocking review, complete
primary validation, and the same reviewer's artifact-consistency confirmation.

## Final report additions

Report every task outcome, exact commit and subject, focused and full
validation, rule inputs and ownership, identity/registry/dependency/
configuration/execution/result behavior, diagnostic and Workspace/cache
integration, compatibility, dependency/API impact, reviewer identity and
fresh/read-only evidence, reconciliation, previous-suite retirement, final
state, and Sprint 38 eligibility.
