# Sprint 31 Navigation and Symbol Search Execution Loop

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
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/reviews/sprint-30-vscode-extension-foundation.md`
- `docs/codex/profiles/graph-implementation.md`
- `docs/codex/profiles/mcp-protocol-implementation.md`
- `docs/codex/profiles/ide-extension-implementation.md`

## Sprint objective and state

Add one bounded semantic symbol-search and source-navigation experience to the
supported desktop VS Code workspace extension, backed by canonical graph facts,
typed source locations, and the public `oneagent-mcp` process, without adding
LSP, diagnostics UI, editor-neutral protocol behavior, mutable workspace
reload, or TypeScript-owned semantics.

Execution requires the committed planning baseline with subject
`Plan Sprint 31 Navigation and Symbol Search`. Sprint 30 must remain completed
and Sprint 31 must be the unique eligible target. Recheck live HEAD, Roadmap,
authorities, fixtures, and working-tree state before Task 1.

Commit mode is authorized only when the current user instruction explicitly
requests one separate commit per successfully completed task. This stored prompt
does not authorize commits by itself.

The current user's launch authorizes exactly one mandatory fresh-context
read-only reviewer for Task 7. Launch it automatically at the review gate
without separate confirmation. No other delegation is authorized.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-navigation-symbol-search.md` | Sprint 31 planning commit | Pinned editor/API evidence, source-location inventory, ownership map, and decision-ready test matrix | Documentation/source evidence, fixture inventory, focused baseline tests, `git diff --check` | `Investigate Sprint 31 navigation and symbol search` |
| 2 | `02-define-navigation-symbol-search.md` | Task 1 | Accepted ADR-0053 | ADR structure, source consistency, link and scope audit, `git diff --check` | `Define Sprint 31 navigation and symbol search` |
| 3 | `03-implement-source-location-model.md` | Task 2 | Typed source-location graph prerequisite and accepted producer evidence | Common/BSL/Graph/adapter focused tests and canonical Rust workspace gate | `Implement Sprint 31 source location model` |
| 4 | `04-implement-navigation-mcp-tools.md` | Task 3 | Bounded Tool-Policy-gated MCP symbol search and navigation projection | Protocol/Runtime/tool-policy/in-memory/public-process matrices and canonical Rust workspace gate | `Implement Sprint 31 navigation MCP tools` |
| 5 | `05-integrate-vscode-navigation-search.md` | Task 4 | VS Code symbol search and source navigation UX over the accepted MCP contract | Typecheck, unit, Extension Host, real-process, package inventory, required Rust checks | `Integrate Sprint 31 VS Code navigation and search` |
| 6 | `06-complete-navigation-search-evidence.md` | Task 5 | Cross-platform public evidence, compatibility audits, and current-state docs | Complete extension and canonical Rust workspace gates plus dependency/scope/link audits | `Complete Sprint 31 navigation and search evidence` |
| 7 | `07-sprint-31-integration-review.md` | Task 6 and all validation | Independent review, reconciliation, decision, hand-off, and conditional Sprint 30 suite retirement | Reviewer and primary complete matrices plus same-reviewer artifact consistency | `Complete Sprint 31 navigation and symbol search review` |

Execute strictly in order. Read every child prompt and every selected Profile,
Template, Core, and Workflow module completely before acting. Stop after the
first blocking failure. Do not combine tasks or partially commit a dependent
task. A zero-match test is not passing evidence.

## Previous-suite retirement

The verified immediately preceding suite is exactly
`docs/codex/prompts/sprint-30-vscode-extension-foundation/` with these eight
tracked files and no untracked additions at planning time:

- `00-sprint-30-execution-loop.md`
- `01-investigate-vscode-extension-foundation.md`
- `02-define-vscode-extension-foundation.md`
- `03-establish-vscode-extension-package.md`
- `04-implement-mcp-runtime-client.md`
- `05-integrate-vscode-runtime-lifecycle.md`
- `06-complete-vscode-extension-evidence.md`
- `07-sprint-30-integration-review.md`

Only Task 7 may retire those exact files, and only after the complete
non-blocking review, successful validation, and same-reviewer artifact
consistency gate.

## Final report additions

Report every task outcome, timestamp, elapsed time, exact commit and subject,
validation, available token telemetry, preserved pre-existing paths, source-
location and protocol compatibility impact, reviewer identity and read-only/
fresh-context evidence, reconciliation, previous-suite retirement, final state,
and Sprint 32 eligibility.
