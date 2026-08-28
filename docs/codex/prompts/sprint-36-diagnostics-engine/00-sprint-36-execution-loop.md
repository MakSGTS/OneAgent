# Sprint 36 Diagnostics Engine Execution Loop

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
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0024-reference-request-provenance.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0054-lsp-adapter.md`
- `docs/reviews/v0.6-release-review.md`
- the repository diagnostic producers, validators, reports, snapshots, cache,
  MCP/LSP projections, fixtures, tests, and consumers pinned by Task 1
- `docs/adr/0058-diagnostics-engine.md` after Task 2

## Sprint objective and state

Build one source-independent deterministic diagnostic orchestration and
reporting boundary over preserved recoverable semantic diagnostics and graph
validation evidence, then publish its accepted immutable results through the
existing Workspace, MCP, and LSP boundaries without moving Graph authority,
inventing source facts, or implementing the Sprint 37 Rules Engine.

Execution requires the committed planning baseline with subject
`Plan Sprint 36 Diagnostics Engine`. v0.6 must remain completed and Sprint 36
must be the unique eligible target. Resolve commit mode from the current user
instruction; this stored prompt does not authorize commits by itself.

After Task 1, execution also requires the committed reusable Diagnostics Engine
framework prerequisite with subject `Establish Diagnostics Engine task
framework`. It corrects the initial planning-readiness decision before Task 2
and selects the dedicated profile, workflow, and template without changing
production behavior. The separate Rules Engine framework remains deferred to
Sprint 37.

The current user's launch authorizes exactly one mandatory fresh-context
read-only reviewer for Task 8. Launch it automatically at the review gate
without separate confirmation. No other delegation is authorized.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-diagnostics-engine.md` | Sprint 36 planning commit | Producer, consumer, identity, ordering, suppression, bounds, compatibility, and oracle investigation | Source/API/fixture/test/consumer/link gates | `Investigate Sprint 36 diagnostics engine` |
| 2 | `02-define-diagnostics-engine.md` | Task 1 and committed Diagnostics Engine framework prerequisite | Accepted ADR-0058 | ADR/evidence/identity/ownership/compatibility/scope consistency | `Define Sprint 36 diagnostics engine` |
| 3 | `03-implement-diagnostic-domain.md` | Task 2 | Typed diagnostic identity, result, suppression, summary, report, bounds, and ordering domain | Focused domain/API/ordering/bound tests and workspace gate | `Implement Sprint 36 diagnostic domain` |
| 4 | `04-implement-diagnostic-orchestration.md` | Task 3 | Deterministic accepted-input orchestration | Positive/negative/duplicate/suppression/bound/repetition tests and workspace gate | `Implement Sprint 36 diagnostic orchestration` |
| 5 | `05-integrate-diagnostic-snapshots.md` | Task 4 | Immutable Workspace and cache/rebuild diagnostic result composition | Snapshot/cache/rebuild/lifecycle/repetition matrices and workspace gate | `Integrate Sprint 36 diagnostic snapshots` |
| 6 | `06-integrate-diagnostic-reporting.md` | Task 5 | Accepted MCP and LSP diagnostic projections | Schema/capability/policy/confinement/bound/public-process matrices and workspace gate | `Integrate Sprint 36 diagnostic reporting` |
| 7 | `07-complete-diagnostics-evidence.md` | Task 6 | Complete evidence, audits, and current-state documentation | Focused/public/full/audit/documentation gates | `Complete Sprint 36 diagnostics evidence` |
| 8 | `08-sprint-36-integration-review.md` | Task 7 and all validation | Independent review, reconciliation, artifact consistency, Sprint 37 hand-off, and conditional Sprint 35 suite retirement | Reviewer and primary complete matrices | `Complete Sprint 36 diagnostics engine review` |

Execute strictly in order. Read every child prompt and selected framework module
completely before acting. Stop after the first blocking failure. Do not combine
tasks or treat a zero-match filter as passing evidence.

## Preserved compatibility boundary

Graph remains the authority for graph facts, recoverable semantic diagnostics,
validation issues, provenance, source locations, and graph reports. The new
engine may normalize and report only inputs accepted by ADR-0058. It must not
invent source facts, infer diagnostics from missing evidence, mutate snapshots,
or become a general rule registry.

Preserve the existing seven-tool MCP catalog and Tool Policy gate, LSP lifecycle
and truthful capabilities, immutable Workspace publication, cache recovery,
HTTP/CLI/IDE behavior, adapter semantics, and source confinement except for the
exact diagnostic result migrations accepted by ADR-0058.

## Previous-suite retirement

The verified immediately preceding suite is exactly
`docs/codex/prompts/sprint-35-external-ai-client-compatibility/` with these seven
tracked files and no untracked additions at planning time:

- `00-sprint-35-execution-loop.md`
- `01-investigate-external-ai-client-compatibility.md`
- `02-define-external-ai-client-compatibility.md`
- `03-implement-legacy-mcp-protocol.md`
- `04-integrate-mcp-client-lifecycle.md`
- `05-complete-external-client-evidence.md`
- `06-sprint-35-integration-review.md`

Only Task 8 may retire those exact files after a non-blocking review, complete
primary validation, and the same reviewer's artifact-consistency confirmation.

## Final report additions

Report every task outcome, exact commit and subject, focused and full
validation, diagnostic inputs and ownership, identity/ordering/suppression/
bounds/reporting behavior, Workspace/cache equivalence, MCP/LSP compatibility,
dependency/API impact, reviewer identity and fresh/read-only evidence,
reconciliation, previous-suite retirement, final state, and Sprint 37
eligibility.
