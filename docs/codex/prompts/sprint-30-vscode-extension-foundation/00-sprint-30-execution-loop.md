# Sprint 30 VS Code Extension Foundation Execution Loop

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
- `docs/adr/0002-runtime-composition-root.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/reviews/sprint-29-mcp-semantic-tools.md`
- `docs/codex/profiles/ide-extension-implementation.md`
- `docs/codex/templates/ide-extension-task.md`
- `docs/codex/workflows/ide-extension.md`

## Sprint objective and state

Establish one reproducible desktop VS Code workspace extension that packages
from tracked sources, activates on explicit OneAgent demand, validates bounded
workspace-scoped configuration, owns one `oneagent-mcp` stdio child lifecycle,
reports deterministic connection state, and proves cleanup through public
extension-host and real-process evidence.

The required governance parent is `90695c74`, `Add IDE extension task
framework`. Execution requires the committed planning baseline with subject
`Plan Sprint 30 VS Code Extension Foundation`. Recheck live HEAD and Roadmap
state before Task 1.

Commit mode is authorized only when the current user instruction explicitly
requests one separate commit per successfully completed task. This stored prompt
does not authorize commits by itself.

The current user's launch authorizes exactly one mandatory fresh-context
read-only reviewer for Task 7. Launch it automatically at the review gate
without a separate confirmation. No other delegation is authorized.

## Ordered task manifest

| Order | Prompt | Required committed prerequisite | Task-owned outcome | Validation additions | Suggested commit message |
|---:|---|---|---|---|---|
| 1 | `01-investigate-vscode-extension-foundation.md` | Sprint 30 planning commit | Pinned platform/toolchain evidence and decision-ready investigation | Documentation links, evidence inventory, `git diff --check` | `Investigate Sprint 30 VS Code extension foundation` |
| 2 | `02-define-vscode-extension-foundation.md` | Task 1 | Accepted ADR-0052 | ADR structure, source consistency, `git diff --check` | `Define Sprint 30 VS Code extension foundation` |
| 3 | `03-establish-vscode-extension-package.md` | Task 2 | Locked extension package, activation entry point, build/unit and VSIX inventory | Non-zero typecheck, build, unit, package inventory | `Establish Sprint 30 VS Code extension package` |
| 4 | `04-implement-mcp-runtime-client.md` | Task 3 | Bounded MCP stdio client and real-process evidence | Client unit matrix, real `oneagent-mcp` process tests, required Rust checks | `Implement Sprint 30 MCP runtime client` |
| 5 | `05-integrate-vscode-runtime-lifecycle.md` | Task 4 | Configuration, commands, status, lifecycle, cleanup, extension-host evidence | Unit, extension-host, failure, repetition, cleanup | `Integrate Sprint 30 VS Code runtime lifecycle` |
| 6 | `06-complete-vscode-extension-evidence.md` | Task 5 | CI, package, public matrix, docs, dependency/scope audits | Complete extension and canonical Rust workspace gates | `Complete Sprint 30 VS Code extension evidence` |
| 7 | `07-sprint-30-integration-review.md` | Task 6 and all validation | Independent review, reconciliation, decision, hand-off, conditional retirement | Reviewer and primary complete matrices, consistency check | `Complete Sprint 30 VS Code extension review` |

Execute strictly in order. Read every child prompt and every selected Profile,
Template, Core, and Workflow module completely before acting. Stop after the
first blocking failure. Do not combine tasks or partially commit a dependent
task. A zero-match test is not passing evidence.

## Previous-suite retirement

The verified immediately preceding suite is exactly
`docs/codex/prompts/sprint-29-mcp-semantic-tools/` with these nine tracked
files and no untracked additions:

- `00-sprint-29-execution-loop.md`
- `01-investigate-mcp-semantic-tools.md`
- `02-define-mcp-semantic-tools.md`
- `03-implement-mcp-tool-protocol.md`
- `04-implement-semantic-graph-tools.md`
- `05-implement-impact-context-tools.md`
- `06-integrate-mcp-semantic-tools.md`
- `07-complete-mcp-semantic-tool-evidence.md`
- `08-sprint-29-integration-review.md`

Only Task 7 may retire those exact files, and only after the complete
non-blocking review, successful validation, and same-reviewer artifact
consistency gate.

## Final report additions

Report every task outcome, timestamp, elapsed time, exact commit, validation,
available token telemetry, preserved pre-existing paths, reviewer identity and
read-only/fresh-context evidence, reconciliation, package inventory result,
previous-suite retirement, final state, and Sprint 31 eligibility.
