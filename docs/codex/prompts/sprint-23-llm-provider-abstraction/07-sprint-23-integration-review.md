# Review Sprint 23 LLM Provider Integration

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 23 execution plan
- `docs/architecture/llm-provider-investigation.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0044-context-engine.md`
- `docs/reviews/sprint-22-context-engine.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- exact committed Sprint 23 planning-through-Task-6 range

## Prerequisites / Required gate

Require Tasks 1-6 committed or proven `already_complete`, every implementation
validation successful, clean task-owned state, and exact commit/path inventory.
Stop with a blocked review if any prerequisite or acceptance evidence is absent.

## Review task

Independently review the integrated Sprint 23 baseline. Do not fix findings.
Issue `pass`, `pass with non-blocking follow-ups`, or `blocked`. Only after a
non-blocking decision and complete successful validation may this task create
the review artifact, transition Roadmap state, and retire the previous suite.

## Scope

### Included

- Exact planning-through-Task-6 commit and path range.
- Framework readiness, investigation quality, ADR-0045 completeness, ownership/
  dependencies, identities, discovery, capabilities, request/response/usage/
  finish contracts, secret handling, error taxonomy, async execution, timeout/
  retry/cancellation policy, cleanup, conformance evidence, Context/Runtime
  compatibility, docs, and deferred scope.
- Complete validation, findings, missing evidence, risk assessment, Roadmap
  transition, Sprint 24 hand-off, and conditional Sprint 22 suite retirement.

### Excluded

Implementation fixes, architecture reselection, new tests to repair evidence,
concrete OpenAI-compatible/LM Studio/Ollama adapters, wire protocols, live
providers/credentials, Runtime/CLI/protocol surfaces, prompt/tool policy,
tokenizers/streaming/conversations, MCP/IDE, performance/quality/security claims,
Sprint 24 implementation, release review, and unrelated cleanup.

## Review Criteria

- Every committed task owns exactly its planned outcome and satisfies its
  acceptance criteria without excluded scope.
- ADR-0045 is implemented exactly and the provider-neutral crate remains
  independent from concrete wire schemas, Runtime transport, and live state.
- Identity/discovery/capability, request/response/usage/finish, configuration/
  secret, validation/precedence, error, execution, timeout/retry/cancellation,
  cleanup, ordering, and repeatability contracts are closed and evidenced.
- Public repository-owned conformance proves positive/negative/boundary/
  incompatible/redaction/cancellation/reorder/repetition behavior without live
  services, credentials, arbitrary sleeps, or unsupported provider claims.
- Existing Context Engine and Runtime behavior, dependencies, CI platforms,
  docs, ownership boundaries, and deferred concrete-provider/MCP/IDE scope
  remain intact.
- Complete focused and workspace validation succeeds.

## Acceptance evidence matrix

Record one evidence/result row for framework/planning readiness, investigation,
accepted architecture, ownership/dependencies, provider/model identity,
discovery/capabilities, public domain values, request validation/precedence,
compatibility, response/usage/finish, configuration/secrets, redaction, error
taxonomy, async substitution, timeout/retry policy, cancellation/cleanup,
public conformance corpus/oracle, reordered/repeated equality, dependency impact,
Context/Runtime compatibility, platforms, docs, and scope containment.

## Authorized review outputs and state transition

Only after issuing `pass` or `pass with non-blocking follow-ups` and completing
all required validation:

- create `docs/reviews/sprint-23-llm-provider-abstraction.md`;
- transition Sprint 23 to `completed` in `docs/Roadmap.md` and make Sprint 24
  OpenAI-Compatible Provider the unique `next` planning target;
- synchronize only minimal current-state hand-off text in `README.md`,
  `docs/Architecture.md`, and `docs/architecture/semantic-model-2.md` if needed;
- conditionally retire the exact verified previous suite
  `docs/codex/prompts/sprint-22-context-engine/` in the same review commit.

The previous suite has exactly these planned tracked files:

- `00-sprint-22-execution-loop.md`
- `01-investigate-context-engine-boundary.md`
- `02-define-context-engine-contract.md`
- `03-implement-context-request-boundary.md`
- `04-implement-deterministic-context-selection.md`
- `05-implement-budgeted-context-assembly.md`
- `06-complete-context-engine-evidence.md`
- `07-sprint-22-integration-review.md`

Before deletion, re-enumerate tracked, filesystem, and untracked inventory and
stop on mismatch or danger. Delete only exact files through explicit patches;
never use recursive deletion, globs, `git clean`, or broad staging. Verify no
retained Markdown link depends on a deleted prompt. Keep the Sprint 23 suite,
`run-next-sprint.md`, non-adjacent suites, and `.codex/` untouched.

## Task-specific Validation

- List and run exact non-zero focused domain, request/compatibility, provider/
  execution/cancellation, and public conformance targets.
- Run the complete provider-neutral package tests.
- `cargo test -p oneagent-analysis`
- `cargo test -p oneagent-runtime --lib` with only local loopback permission
  when required by the sandbox.
- Run the canonical complete workspace validation.
- Verify commit/path ownership, dependency tree, docs links, Roadmap state,
  prompt-retirement inventory, `git diff --check`, and final
  `git status --short`.

## Suggested commit message

`Complete Sprint 23 LLM Provider review`

## Final report additions

Report reviewed range/commits, evidence matrix, findings, missing evidence,
decision, validation, review/state/doc outputs, every retired Sprint 22 path or
`already_retired` evidence, Sprint 24 eligibility, deferred scope, residual
risk, commit, and final Git state.
