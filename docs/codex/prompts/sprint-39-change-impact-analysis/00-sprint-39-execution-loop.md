# Sprint 39 Change Impact Analysis Execution Loop

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
- `docs/adr/0017-depends-on-semantics.md`
- `docs/adr/0027-incremental-semantic-index-maintenance.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0060-git-change-adapter.md`
- `docs/reviews/sprint-29-mcp-semantic-tools.md`
- `docs/reviews/sprint-38-git-change-adapter.md`
- the Graph diff/impact, Workspace publication, cache, filesystem/Git input,
  Runtime, MCP, public-process, fixture, test, and consumer evidence pinned by
  Task 1
- `docs/adr/0061-change-impact-analysis.md` after Task 2

## Sprint objective and state

Define and implement one bounded deterministic product-facing Change Impact
Analysis workflow over complete previous/current semantic Configuration graphs
and canonical `SemanticGraphDiff`. Publish the accepted immutable report
through existing Runtime and MCP boundaries while preserving Graph authority,
complete Workspace rebuilds, failure recovery, cache, Tool Policy, supported
consumers, and source confinement.

Execution requires the committed planning baseline with subject
`Plan Sprint 39 Change Impact Analysis`, completed Sprint 38 review `56bb3004`,
and version integration `295a5454`. Sprint 39 must be the unique eligible
target. Resolve commit mode from the current user instruction; this stored
prompt does not authorize commits by itself.

The current user's launch authorizes exactly one mandatory fresh-context
read-only reviewer for Task 7. Launch it automatically at the review gate
without separate confirmation. No other delegation is authorized.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-change-impact-analysis.md` | Sprint 39 planning commit | Complete Graph/snapshot/identity/lifecycle/cache/MCP/compatibility/sensitive-data/oracle evidence | Source/API/history/test/consumer/link gates and focused baseline | `Investigate Sprint 39 Change Impact Analysis` |
| 2 | `02-define-change-impact-analysis.md` | Task 1 | Accepted ADR-0061 | Evidence/authority/input/identity/completeness/snapshot/protocol/compatibility/scope consistency | `Define Sprint 39 Change Impact Analysis` |
| 3 | `03-implement-change-impact-report.md` | Task 2 | Accepted immutable typed Change Impact report | Focused input/identity/order/duplicate/bound/summary/error/repetition tests and workspace gate | `Implement Sprint 39 Change Impact report` |
| 4 | `04-integrate-workspace-impact-snapshots.md` | Task 3 | Accepted Workspace composition, matching, recovery, cache, lifecycle, and input equivalence | Workspace/watching/Git/cache/lifecycle/public-consumer matrices and workspace gate | `Integrate Sprint 39 Workspace impact snapshots` |
| 5 | `05-integrate-product-impact-reporting.md` | Task 4 | Accepted compatible MCP impact workflow | Schema/handler/policy/bound/error/repetition/public-process matrices and workspace gate | `Integrate Sprint 39 product impact reporting` |
| 6 | `06-complete-change-impact-evidence.md` | Task 5 | Complete evidence, audits, and current-state documentation | Focused/public/full/dependency/API/scope/documentation gates | `Complete Sprint 39 Change Impact evidence` |
| 7 | `07-sprint-39-integration-review.md` | Task 6 and all validation | Independent review, reconciliation, artifact consistency, Sprint 40 hand-off, and conditional Sprint 38 suite retirement | Reviewer and primary complete matrices | `Complete Sprint 39 Change Impact Analysis review` |

Execute strictly in order. Read every child prompt and selected framework
module completely before acting. Stop after the first blocking failure. Do not
combine tasks or treat a zero-match filter as passing evidence.

## Preserved authority and compatibility boundary

Graph remains the only owner of semantic facts, canonical graph diff, and
impact traversal semantics. The product workflow may consume complete immutable
Graph results but must not create another diff, dependency, or propagation
authority. Workspace remains the complete snapshot and publication owner.
Repository paths and statuses may request a complete rebuild but are never
Configuration selectors, semantic identities, impact seeds, reasons, or
authority.

Preserve complete production discovery/build/validation, atomic last-valid
publication, cache validation/recovery, Runtime lifecycle, current MCP revision
and Tool Policy, supported HTTP/CLI/MCP/LSP/VS Code/EDT behavior, source
confinement, and Coverage state except for exact behavior accepted by ADR-0061.
Do not implement Sprint 40 refactoring or Sprint 41 edit transactions.

## Previous-suite retirement

The verified immediately preceding suite is exactly
`docs/codex/prompts/sprint-38-git-change-adapter/` with these eight tracked files
and no untracked additions at planning time:

- `00-sprint-38-execution-loop.md`
- `01-investigate-git-change-adapter.md`
- `02-define-git-change-adapter.md`
- `03-implement-change-set-domain.md`
- `04-implement-git-repository-reader.md`
- `05-integrate-workspace-change-inputs.md`
- `06-complete-git-change-adapter-evidence.md`
- `07-sprint-38-integration-review.md`

Only Task 7 may retire those exact files after a non-blocking review, complete
primary validation, and the same reviewer's artifact-consistency confirmation.

## Final report additions

Report every task outcome, exact commit and subject, focused and full
validation, canonical inputs and authority, report identity and completeness,
ordering/bounds/failures, Workspace matching/publication/cache/lifecycle,
filesystem/Git equivalence, MCP schema/policy/public-process behavior,
dependency/API impact, reviewer identity and fresh/read-only evidence,
reconciliation, previous-suite retirement, final state, and Sprint 40
eligibility.
