# Sprint 32 LSP Adapter Execution Loop

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
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/adr/0053-navigation-symbol-search.md`
- `docs/reviews/sprint-31-navigation-symbol-search.md`

## Sprint objective and state

Expose the supported immutable navigation, symbol, and recoverable diagnostic
evidence through one bounded editor-neutral LSP 3.17 stdio process without
moving semantic authority into protocol code, reading source after snapshot
construction, or claiming mutable-document analysis.

Execution requires the committed planning baseline with subject
`Plan Sprint 32 LSP Adapter`. Sprint 31 must remain completed and Sprint 32 must
be the unique eligible target. Resolve commit mode only from the current user
instruction; this stored prompt does not authorize commits by itself.

The current user's launch authorizes exactly one mandatory fresh-context
read-only reviewer for Task 8. Launch it automatically at the review gate
without separate confirmation. No other delegation is authorized.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-lsp-adapter.md` | Sprint 32 planning commit | Pinned LSP authority, repository ownership/compatibility inventory, decision-ready first slice and test matrix | Source/link/fixture evidence, focused baselines, `git diff --check` | `Investigate Sprint 32 LSP adapter` |
| 2 | `02-define-lsp-adapter.md` | Task 1 | Accepted ADR-0054 | ADR/source/link/scope consistency and `git diff --check` | `Define Sprint 32 LSP adapter` |
| 3 | `03-implement-lsp-protocol-core.md` | Task 2 | Transport-independent bounded LSP message, lifecycle, capability, and dispatch core | Non-zero protocol matrices and canonical Rust workspace gate | `Implement Sprint 32 LSP protocol core` |
| 4 | `04-implement-lsp-runtime-lifecycle.md` | Task 3 | Public `oneagent-lsp` stdio process lifecycle over an immutable Workspace snapshot | Runtime in-memory/process lifecycle, framing, channel, cleanup, and canonical Rust gate | `Implement Sprint 32 LSP runtime lifecycle` |
| 5 | `05-implement-lsp-navigation-symbols.md` | Task 4 | Accepted LSP navigation and symbol capabilities over canonical graph locations | Focused protocol/Runtime/public-process semantic matrices and canonical Rust gate | `Implement Sprint 32 LSP navigation and symbols` |
| 6 | `06-implement-lsp-diagnostics.md` | Task 5 | Accepted bounded document diagnostic projection from canonical recoverable diagnostics | Graph/adapter/Runtime/LSP/public-process diagnostic matrices and canonical Rust gate | `Implement Sprint 32 LSP diagnostics` |
| 7 | `07-complete-lsp-evidence.md` | Task 6 | Cross-platform, compatibility, scope, CI, and current-state evidence | Complete Rust/public-process matrices plus dependency/scope/link audits | `Complete Sprint 32 LSP evidence` |
| 8 | `08-sprint-32-integration-review.md` | Task 7 and all validation | Independent review, reconciliation, artifact consistency, Sprint 33 hand-off, and conditional Sprint 31 suite retirement | Reviewer and primary complete matrices plus same-reviewer consistency check | `Complete Sprint 32 LSP adapter review` |

Execute strictly in order. Read every child prompt and selected framework module
completely before acting. Stop after the first blocking failure. Do not combine
tasks or treat a zero-match filter as passing evidence.

## Previous-suite retirement

The verified immediately preceding suite is exactly
`docs/codex/prompts/sprint-31-navigation-symbol-search/` with these eight
tracked files and no untracked additions at planning time:

- `00-sprint-31-execution-loop.md`
- `01-investigate-navigation-symbol-search.md`
- `02-define-navigation-symbol-search.md`
- `03-implement-source-location-model.md`
- `04-implement-navigation-mcp-tools.md`
- `05-integrate-vscode-navigation-search.md`
- `06-complete-navigation-search-evidence.md`
- `07-sprint-31-integration-review.md`

Only Task 8 may retire those exact files after a non-blocking review, complete
validation, and the same reviewer's artifact-consistency confirmation.

## Final report additions

Report every task outcome, timestamp, elapsed time, exact commit/subject,
validation, available token telemetry, preserved pre-existing paths, protocol
and compatibility impact, reviewer identity and fresh/read-only evidence,
reconciliation, previous-suite retirement, final state, and Sprint 33
eligibility.
