# Review Sprint 27 Tool Execution Policy

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative ADRs and architecture documents

- `docs/Roadmap.md`, Sprint 27 execution plan
- `docs/adr/0049-tool-execution-policy.md`
- `docs/architecture/tool-execution-policy-investigation.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0044-context-engine.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-26-ollama-integration.md`
- `docs/codex/workflows/review.md`

## Prerequisites / Required gate

- Tasks 1-6 are committed in dependency order or proven `already_complete`.
- Every focused and full implementation validation required by the manifest has
  succeeded, with no zero-match filter or uncommitted task-created change.
- The exact planning-through-Task-6 commit range is known.
- The current user instruction explicitly authorizes one separate fresh-context
  read-only reviewer agent.

## Independent fresh-context read-only reviewer procedure

Before any review-owned edit or state transition, start exactly one reviewer
agent with no inherited implementation conversation turns. Give it only:

- repository root `/Users/maxim_tomshin/Development/oneagent`;
- the exact planning-through-Task-6 commit range and observed review `HEAD`;
- the authoritative files listed above;
- the Sprint 27 objective, included scope, excluded scope, ordered task
  acceptance criteria, and validation matrix from the committed Roadmap plan;
- the required output contract below.

Do not give the reviewer an expected decision, primary implementation
rationale, acceptance summary, proposed finding, or private reasoning. Require
the reviewer to remain read-only, not delegate, and report:

- exact reviewed range, initial `HEAD`, and initial/final `git status --short`;
- one recommendation: `pass`, `pass with non-blocking follow-ups`, or `blocked`;
- an acceptance-evidence matrix;
- blocking and non-blocking findings with exact file/line evidence;
- missing evidence separately from defects;
- every command and exact outcome, including zero-match or unexecuted checks;
- scope and exclusion conformance;
- residual risks and recommended next action.

Reviewer mutation, incomplete output, an unavailable fresh context, or a
working-tree discrepancy blocks the review. Preserve the independent report
separately from primary evidence.

## Primary review task

After the independent report returns, independently inspect the same exact
range and rerun the complete required matrix. Reconcile every criterion,
finding, missing-evidence item, command result, scope conclusion, and risk. The
effective decision must not be less severe than the reviewer recommendation;
an unresolved disagreement is `blocked`.

Only after reconciliation may the primary agent draft
`docs/reviews/sprint-27-tool-execution-policy.md` with the independent handoff,
reviewer result, primary evidence, reconciliation, decision, range, acceptance
matrix, findings, missing evidence, validation, scope/exclusions, dependency/
public-surface/sensitive-state/no-real-effect audits, risks, and repository
state. Make no implementation fix during review.

Before any state transition, prompt deletion, staging, or commit, send the
draft artifact to the same reviewer for a final read-only consistency check.
Require confirmation that every reviewer finding, missing-evidence item,
decision, validation result, scope conclusion, and risk is preserved without
weakening. A failed, incomplete, or unavailable check blocks the review.

Only after a non-blocking effective decision, successful independent and
primary validation, and a passing consistency check, update `docs/Roadmap.md`
to mark Sprint 27 `completed` and Sprint 28 MCP Server the unique `next` target,
synchronize only minimal current-state hand-off text when required, and perform
the conditional previous-suite retirement below as the final bounded action.

## Scope

### Included

- Independent and primary read-only review of the exact sprint range.
- One review artifact after independent evidence exists.
- Conditional Roadmap/current-state hand-off and exact Sprint 26 prompt
  deletions only after every review gate passes.

### Excluded

- Silent production fixes, new architecture or implementation, widened support
  claims, Sprint 28 planning, real tool execution, reviewer mutation or
  delegation, and deletion outside the exact verified previous suite.

## Acceptance Criteria

- Every ADR-0049 and committed task criterion/exclusion has independent and
  primary repository evidence or an explicit finding/missing-evidence entry.
- Non-zero focused/public/compatibility and canonical full workspace validation
  pass without real tool effects or external state.
- Decision severity is at least the reviewer's recommendation; a blocked result
  leaves Sprint 27 incomplete and retains the previous suite.
- The same reviewer passes the final artifact-consistency check before state or
  deletion changes.
- A non-blocking result atomically includes the review artifact, truthful state/
  hand-off changes, and authorized exact deletions.
- Current Sprint 27 prompts and `run-next-sprint.md` remain intact.

## Previous sprint prompt-suite retirement

The verified immediately preceding suite is exactly
`docs/codex/prompts/sprint-26-ollama-integration/`. Its planned tracked inventory
is:

- `00-sprint-26-execution-loop.md`
- `01-investigate-ollama-integration.md`
- `02-define-ollama-integration.md`
- `03-implement-ollama-client.md`
- `04-implement-ollama-discovery.md`
- `05-implement-ollama-generation.md`
- `06-complete-ollama-evidence.md`
- `07-sprint-26-integration-review.md`

After all review gates pass, re-enumerate tracked, filesystem, and untracked
inventories. If and only if they still match exactly, delete these eight
tracked files explicitly through `apply_patch`; do not use recursive deletion
or globs. Stop before deletion on ambiguity or an untracked addition. Stage
every deleted path explicitly with the review artifact and state transition in
the single final review commit. If committed evidence proves the suite already
absent, record `already_retired` and keep the ordinary review commit.

## Repository Safety

Before any review-owned edit, print a Change Contract listing the review
artifact, possible Roadmap/current-state files, and all eight conditional
deletions. Preserve `.codex/`, current/non-adjacent prompt suites,
implementation files, user work, and unrelated files.

## Task-specific Validation

- Both reviewer and primary inspect the exact planning-through-Task-6 range and
  task commit paths independently.
- Both list and run non-zero Tool Policy unit and public conformance tests.
- Both run complete LLM and Analysis tests plus affected Runtime targets.
- Both run dependency, public API, scope, redaction, ignored-test, external/live-
  state, real-side-effect, documentation-link, and prompt-inventory audits.
- Both run the canonical full workspace validation; the primary reruns it after
  review/state/retirement changes.
- Run `git diff --check` and verify the current suite remains complete.

## Suggested commit message

`Complete Sprint 27 tool execution policy review`

## Final report additions

Report reviewer task identity and fresh-context/read-only confirmation,
recommendation, findings, missing evidence, exact range and commands, primary/
reviewer discrepancies, effective decision, artifact-consistency result, state
transition, Sprint 28 eligibility, previous-suite result and every deleted path,
review commit hash, changed/preserved paths, and final Git state.
