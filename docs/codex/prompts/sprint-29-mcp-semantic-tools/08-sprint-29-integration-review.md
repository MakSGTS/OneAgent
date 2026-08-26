# Review Sprint 29 MCP Semantic Tools

Use the review profile/template and `docs/codex/workflows/review.md`. Read the
Sprint plan, investigation, ADR-0051, ADR-0040/0044/0049/0050, Sprint 28
review, exact official sources, complete implementation range and validation.

After Tasks 1-7 are committed, delegate the exact planning-through-Task-7 range
to exactly one fresh-context read-only reviewer. Provide only repository root,
range/HEAD, authorities, objective, included/excluded scope, acceptance matrix,
validation matrix, and output contract. Do not provide an expected decision or
implementation rationale. Require range and initial/final state, recommendation
(`pass`, `pass with non-blocking follow-ups`, or `blocked`), evidence matrix,
findings, missing evidence, commands/outcomes, exclusions, risks, and next
action. Reviewer mutation, delegation, incomplete evidence, or discrepancy
blocks.

The primary independently inspects and validates the same range. The effective
decision cannot be less severe. Only after reconciliation create
`docs/reviews/sprint-29-mcp-semantic-tools.md`. Before any state change,
deletion, staging, or commit, ask the same reviewer to verify the draft
artifact preserves every finding, missing-evidence conclusion, decision,
command result, scope conclusion, and risk.

After a passing consistency check and non-blocking decision only, mark Sprint
29 `completed`, make Sprint 30 the unique `next`, and explicitly delete exactly
these verified Sprint 28 files through `apply_patch`:

- `00-sprint-28-execution-loop.md`
- `01-investigate-mcp-server.md`
- `02-define-mcp-server.md`
- `03-implement-mcp-protocol-domain.md`
- `04-implement-mcp-server-dispatch.md`
- `05-implement-mcp-stdio-transport.md`
- `06-integrate-mcp-server-lifecycle.md`
- `07-complete-mcp-server-evidence.md`
- `08-sprint-28-integration-review.md`

All are under `docs/codex/prompts/sprint-28-mcp-server/`. Re-enumerate tracked,
filesystem, and untracked inventories immediately before deletion; ambiguity
blocks. Preserve the current suite, `run-next-sprint.md`, `.codex/`, production
files and all non-adjacent suites. Rerun focused/public/full validation, links,
range diff, and prompt inventories. Commit:
`Complete Sprint 29 MCP semantic tools review`.
