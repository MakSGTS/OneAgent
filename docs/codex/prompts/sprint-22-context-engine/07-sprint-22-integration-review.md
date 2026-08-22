# Review Sprint 22 Context Engine Integration

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 22 execution plan
- `docs/architecture/context-engine-investigation.md`
- `docs/adr/0044-context-engine.md`
- `docs/reviews/v0.4-release-review.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- exact committed Sprint 22 planning-through-Task-6 range

## Prerequisites / Required gate

Require Tasks 1-6 committed or proven `already_complete`, every implementation
validation successful, clean task-owned state, and exact commit/path inventory.
Stop with a blocked review if any prerequisite or acceptance evidence is absent.

## Review task

Independently review the integrated Sprint 22 baseline. Do not fix findings.
Issue `pass`, `pass with non-blocking follow-ups`, or `blocked`. Only after a
non-blocking decision and complete successful validation may this task create
the review artifact, transition Roadmap state, and retire the previous suite.

## Scope

### Included

- Exact planning-through-Task-6 commit and path range.
- Investigation quality, ADR-0044 completeness, canonical authority, request/
  seed/policy/error behavior, selection and relevance, ties/order/deduplication,
  budget/cost/admission/truncation, bundle/provenance/explanations, rendering,
  evaluation, dependencies, compatibility, docs, and deferred scope.
- Complete validation, findings, missing evidence, risk assessment, Roadmap
  transition, Sprint 23 hand-off, and conditional Sprint 21 suite retirement.

### Excluded

Implementation fixes, architecture reselection, new tests to repair evidence,
new graph facts/dependencies/source extraction/Runtime routes, providers/models,
tools, MCP/IDE, performance/quality claims, Sprint 23 implementation, release
review, and unrelated cleanup.

## Review Criteria

- Every committed task owns exactly its planned outcome and satisfies its
  acceptance criteria without excluded scope.
- ADR-0044 is implemented exactly and Context Engine remains a read-only
  source-independent consumer of one immutable canonical graph snapshot.
- Request/seed/policy validation and failures, traversal/filtering, relevance,
  ties, bounds, cycles, deduplication, budget accounting, admission, omissions,
  truncation, provenance, explanations, bundle order, and rendering are closed,
  deterministic, and completely evidenced.
- Public repository-owned evaluation proves positive/negative/boundary/reorder/
  repetition behavior without external services, unavailable source text,
  arbitrary sleeps, or unsupported quality claims.
- Existing analysis and graph/query behavior, dependencies, platforms, docs,
  semantic authority, and deferred provider/Runtime/MCP/IDE scope remain intact.
- Complete focused and workspace validation succeeds.

## Acceptance evidence matrix

Record one evidence/result row for planning readiness, investigation, accepted
architecture, canonical data boundary, public request types, validation and seed
resolution, failures and precedence, traversal/filtering, relevance and ties,
bounds/cycles/deduplication, candidate order, costs and budget accounting,
admission/omission/truncation, bundle identity/order, provenance, explanations,
rendering, public evaluation corpus/oracle, reordered and repeated equality,
dependency impact, graph/analysis compatibility, platforms, docs, and scope
containment.

## Authorized review outputs and state transition

Only after issuing `pass` or `pass with non-blocking follow-ups` and completing
all required validation:

- create `docs/reviews/sprint-22-context-engine.md`;
- transition Sprint 22 to `completed` in `docs/Roadmap.md` and make Sprint 23
  LLM Provider Abstraction the unique `next` planning target;
- synchronize only minimal current-state hand-off text in `README.md`,
  `docs/Architecture.md`, and `docs/architecture/semantic-model-2.md` if needed;
- conditionally retire the exact verified previous suite
  `docs/codex/prompts/sprint-21-cli-client/` in the same review commit.

The previous suite has exactly these planned tracked files:

- `00-sprint-21-execution-loop.md`
- `01-investigate-cli-client-boundary.md`
- `02-define-cli-client-contract.md`
- `03-implement-cli-command-boundary.md`
- `04-implement-runtime-http-client.md`
- `05-complete-cli-client-evidence.md`
- `06-sprint-21-integration-review.md`

Before deletion, re-enumerate tracked, filesystem, and untracked inventory and
stop on mismatch or danger. Delete only exact files through explicit patches;
never use recursive deletion, globs, `git clean`, or broad staging. Verify no
retained Markdown link depends on a deleted prompt. Keep the Sprint 22 suite,
`run-next-sprint.md`, non-adjacent suites, and `.codex/` untouched.

## Task-specific Validation

- List and run exact non-zero focused request, selection, budget, rendering, and
  public evaluation targets.
- `cargo test -p oneagent-analysis`
- Run affected graph query tests.
- Run the canonical complete workspace validation.
- Verify commit/path ownership, docs links, Roadmap state, prompt-retirement
  inventory, `git diff --check`, and final `git status --short`.

## Suggested commit message

`Complete Sprint 22 Context Engine review`

## Final report additions

Report reviewed range/commits, evidence matrix, findings, missing evidence,
decision, validation, review/state/doc outputs, every retired Sprint 21 path or
`already_retired` evidence, Sprint 23 eligibility, deferred scope, residual
risk, commit, and final Git state.
