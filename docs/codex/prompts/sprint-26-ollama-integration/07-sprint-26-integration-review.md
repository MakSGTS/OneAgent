# Review Sprint 26 Ollama Integration

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative ADRs and architecture documents

- `docs/Roadmap.md`, Sprint 26 execution plan
- `docs/adr/0048-ollama-integration.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0046-openai-compatible-provider.md`
- `docs/adr/0047-lm-studio-integration.md`
- `docs/architecture/ollama-integration-investigation.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-25-lm-studio-integration.md`

## Prerequisites / Required gate

- Tasks 1-6 are committed in dependency order or proven `already_complete`.
- Every focused and full implementation validation required by the manifest has
  succeeded, with no zero-match filter or uncommitted task-created change.
- The exact planning-through-Task-6 commit range is known.

## Task

Review the integrated Sprint 26 baseline without silently fixing findings.
Create `docs/reviews/sprint-26-ollama-integration.md` with a decision of `pass`,
`pass with non-blocking follow-ups`, or `blocked`; reviewed range; acceptance
matrix; findings; missing evidence; focused and complete validation; scope and
exclusion conformance; dependency/public-surface/sensitive-state audits; risks;
and repository state.

Only after a non-blocking decision and successful complete validation, update
`docs/Roadmap.md` to mark Sprint 26 `completed` and Sprint 27 Tool Execution
Policy the unique `next` target, synchronize only minimal current-state hand-off
text when required, and perform the conditional previous-suite retirement below
as the final bounded action.

## Scope

### Included

- Read-only review of the exact sprint range and working tree.
- One review artifact.
- Conditional Roadmap/current-state hand-off and exact Sprint 25 prompt
  deletions only after the review gate passes.

### Excluded

- Silent production fixes, new implementation, widened support claims, Sprint 27
  planning, live/cloud provider acceptance, and deletion outside the exact
  verified previous suite.

## Acceptance Criteria

- Every ADR-0048 criterion and exclusion has repository evidence or a finding.
- Focused/public/regression/compatibility and canonical full workspace validation
  pass without live provider state or external network.
- Decision severity matches findings; a blocked decision leaves Sprint 26
  incomplete and retains the previous suite.
- A non-blocking decision transitions state truthfully and atomically includes
  the review artifact, state/hand-off changes, and authorized exact deletions.
- Current Sprint 26 prompts and `run-next-sprint.md` remain intact.

## Previous sprint prompt-suite retirement

The verified immediately preceding suite is exactly
`docs/codex/prompts/sprint-25-lm-studio-integration/`. Its planned tracked
inventory is:

- `00-sprint-25-execution-loop.md`
- `01-investigate-lm-studio-integration.md`
- `02-define-lm-studio-integration.md`
- `03-implement-lm-studio-client.md`
- `04-implement-lm-studio-discovery.md`
- `05-implement-lm-studio-generation.md`
- `06-complete-lm-studio-evidence.md`
- `07-sprint-25-integration-review.md`

After issuing `pass` or `pass with non-blocking follow-ups` and completing all
validation, re-enumerate tracked, filesystem, and untracked inventories. If and
only if they still match exactly, delete these eight tracked files explicitly
through `apply_patch`; do not use recursive deletion or globs. Stop before
deletion on ambiguity or an untracked addition. Stage every deleted path
explicitly with the review artifact and state transition in the single final
review commit. If committed evidence proves the suite already absent, record
`already_retired` and keep the ordinary review commit.

## Repository Safety

Before any review-owned edit, print a Change Contract listing the review
artifact, possible Roadmap/current-state files, and all eight conditional
deletions. Preserve `.codex/`, current/non-adjacent prompt suites, implementation
files, user work, and unrelated files.

## Task-specific Validation

- Inspect the exact planning-through-Task-6 range and task commit paths.
- List and run non-zero Ollama unit and public conformance tests.
- Run complete OpenAI-compatible, LM Studio, provider-neutral, Analysis, and
  Runtime regression targets.
- Run dependency, public API, scope, redaction, ignored-test, live-state,
  documentation-link, prompt-inventory, and deletion-boundary audits.
- Run the canonical full workspace validation before the decision and again
  after review/state/retirement changes.
- Run `git diff --check` and verify the current suite remains complete.

## Suggested commit message

`Complete Sprint 26 Ollama review`

## Final report additions

Report decision, findings, exact reviewed range, focused/public/full validation,
state transition, Sprint 27 eligibility, previous-suite result and every
deleted path, review commit hash, changed/preserved paths, and final Git state.
