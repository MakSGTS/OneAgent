# Sprint 38 Git Change Adapter Execution Loop

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
- `docs/adr/0027-incremental-semantic-index-maintenance.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/reviews/sprint-19-file-watching.md`
- `docs/reviews/sprint-37-rules-engine.md`
- the repository Git history, Workspace change source, cache, Runtime
  lifecycle, production fixtures, tests, consumers, and platform constraints
  pinned by Task 1
- `docs/adr/0060-git-change-adapter.md` after Task 2

## Sprint objective and state

Define and implement one bounded deterministic local Git Change Adapter that
converts accepted repository change evidence into source-independent Workspace
change inputs without making Git a semantic, validation, impact, or edit
authority. Preserve complete production rebuilds, immutable atomic
publication, recovery, cache, lifecycle, and supported consumer behavior.

Execution requires the committed planning baseline with subject
`Plan Sprint 38 Git Change Adapter`, committed framework prerequisite
`7eac8515 Establish Git Change Adapter task framework`, completed Sprint 37
review `b029544f`, and version integration `a1434fa0`. Sprint 38 must be the
unique eligible target. Resolve commit mode from the current user instruction;
this stored prompt does not authorize commits by itself.

The current user's launch authorizes exactly one mandatory fresh-context
read-only reviewer for Task 7. Launch it automatically at the review gate
without separate confirmation. No other delegation is authorized.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-git-change-adapter.md` | Sprint 38 planning commit and framework prerequisite | Complete repository, endpoint, state-layer, change, path, process/dependency, Workspace-equivalence, compatibility, and oracle evidence | Source/API/history/Git/test/consumer/link gates | `Investigate Sprint 38 Git Change Adapter` |
| 2 | `02-define-git-change-adapter.md` | Task 1 | Accepted ADR-0060 | Evidence/authority/endpoint/state/status/path/order/process/Workspace/compatibility/scope consistency | `Define Sprint 38 Git Change Adapter` |
| 3 | `03-implement-change-set-domain.md` | Task 2 | Accepted typed normalized change-set domain | Focused identity/status/path/order/duplicate/bound/error/API tests and workspace gate | `Implement Sprint 38 change-set domain` |
| 4 | `04-implement-git-repository-reader.md` | Task 3 | Accepted local Git repository reader boundary | Repository-state/process-or-library/negative/repetition/platform tests and workspace gate | `Implement Sprint 38 Git repository reader` |
| 5 | `05-integrate-workspace-change-inputs.md` | Task 4 | Accepted Workspace mapping, rebuild equivalence, lifecycle, cache, and consumers | Workspace/cache/watching/lifecycle/public-consumer matrices and workspace gate | `Integrate Sprint 38 Workspace change inputs` |
| 6 | `06-complete-git-change-adapter-evidence.md` | Task 5 | Complete evidence, audits, and current-state documentation | Focused/public/full/platform/dependency/API/scope/documentation gates | `Complete Sprint 38 Git Change Adapter evidence` |
| 7 | `07-sprint-38-integration-review.md` | Task 6 and all validation | Independent review, reconciliation, artifact consistency, Sprint 39 hand-off, and conditional Sprint 37 suite retirement | Reviewer and primary complete matrices | `Complete Sprint 38 Git Change Adapter review` |

Execute strictly in order. Read every child prompt and selected framework
module completely before acting. Stop after the first blocking failure. Do not
combine tasks or treat a zero-match filter as passing evidence.

## Preserved authority and compatibility boundary

Git supplies bounded repository change evidence only. Workspace discovery and
production adapters remain source authorities; Graph remains semantic
authority; Graph Diff and ADR-0027 remain authoritative after complete graph
construction; Analysis, Diagnostics, and Rules retain their accepted owners.

Preserve ADR-0041 complete observation and rebuild correctness, immutable
Workspace publication, cache validation/recovery, Runtime lifecycle,
HTTP/CLI/MCP/LSP/VS Code/EDT behavior, source confinement, and Coverage state
except for exact additive or migrated behavior accepted by ADR-0060. Do not
implement Sprint 39 impact analysis or Sprint 40–41 edit behavior.

## Previous-suite retirement

The verified immediately preceding suite is exactly
`docs/codex/prompts/sprint-37-rules-engine/` with these nine tracked files and
no untracked additions at planning time:

- `00-sprint-37-execution-loop.md`
- `01-investigate-rules-engine.md`
- `02-define-rules-engine.md`
- `03-implement-rule-registry.md`
- `04-implement-rule-planning.md`
- `05-implement-rule-execution.md`
- `06-integrate-rule-snapshots.md`
- `07-complete-rules-engine-evidence.md`
- `08-sprint-37-integration-review.md`

Only Task 7 may retire those exact files after a non-blocking review, complete
primary validation, and the same reviewer's artifact-consistency confirmation.

## Final report additions

Report every task outcome, exact commit and subject, focused and full
validation, repository/endpoint/state-layer contracts, normalized identity,
statuses, paths, ordering, bounds, reader boundary, Workspace equivalence,
lifecycle/cache/consumer compatibility, dependency/API impact, reviewer
identity and fresh/read-only evidence, reconciliation, previous-suite
retirement, final state, and Sprint 39 eligibility.
