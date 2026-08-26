# Review Sprint 25 LM Studio Integration

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 25 execution plan and completion gate
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0046-openai-compatible-provider.md`
- `docs/adr/0047-lm-studio-integration.md`
- `docs/architecture/lm-studio-integration-investigation.md`
- `docs/reviews/sprint-24-openai-compatible-provider.md`
- committed Sprint 25 planning and Tasks 1-6

## Prerequisites / Required gate

Require every Sprint 25 Task 1-6 outcome committed or proven
`already_complete`, clean task-owned state, exact ordered commit/path inventory,
and successful focused plus full validation. Stop before review outputs when any
prerequisite or evidence is incomplete.

## Review target

Review the exact committed Sprint 25 planning-through-Task-6 range against
ADR-0045, ADR-0046, ADR-0047, the Roadmap plan, every child prompt acceptance
criterion, and current repository behavior. Do not fix findings.

## Reviewed baseline / commit range

Resolve and record the exact planning baseline, every task commit, review head,
changed paths, dependency graph, and initial/final Git status from live history.
Do not copy historical hashes from prompt text as proof.

## Scope

- Planning/readiness, investigation quality, accepted architecture, dependency
  approval, implementation order, and scope conformance.
- Provider ownership, identity, construction, locality, authentication,
  discovery type filtering, generation mapping, bounds, errors, redaction,
  timeout, cancellation, one-attempt behavior, cleanup, and determinism.
- Public LM Studio conformance, complete generic adapter and provider-neutral
  regressions, Context/Runtime compatibility, dependency direction, and docs.
- Evidence that acceptance requires no installed/running LM Studio, downloaded
  model, credential, external network, ignored artifact, or response quality.

## Excluded

- Fixing production/test/docs findings during review.
- New features, broader LM Studio compatibility, live testing, model/server
  lifecycle, Runtime integration, chat/streaming/tools/MCP/embeddings, prompt
  policy, Sprint 26 implementation, and v0.5 release review.

## Repository Safety

Do not modify implementation or test files, fix findings, contact a live LM
Studio server, inspect developer-local state, or broaden the reviewed range.
Preserve unrelated changes, `.codex/`, the Sprint 25 suite, non-adjacent prompt
suites, and every path not explicitly authorized as a review output. Staging,
committing, state transition, and Sprint 24 prompt retirement require the exact
gates below and current launch authorization.

## Review Criteria

- Every Task 1-6 acceptance criterion has executed evidence.
- ADR-0047 is evidence-backed and preserves ADR-0045/0046.
- Discovery cannot advertise embedding-only entries as `TextGeneration`.
- Generation preserves exact provider/model identity, local byte bounds/usage,
  accepted finish, typed terminal failures, and no fallback.
- Sensitive URL/header/body/input/output/credential/provider payload content is
  absent from implicit formatting, diagnostics, fixtures, and review artifacts.
- Timeout/cancellation precedence, one request, cleanup, and fresh repetition
  are deterministic.
- Generic adapter behavior and provider-neutral/consumer ownership remain
  compatible.
- Documentation and deferred scope match implemented truth.

## Acceptance evidence matrix

Create a criterion-by-criterion table with exact code/tests/commands/results.
Separate defects, missing evidence, accepted residual risks, and unsupported
claims. Zero matched tests, unexecuted live assumptions, or local-state
dependence are missing evidence, not passes.

## Authorized review outputs and state transition

Only after issuing `pass` or `pass with non-blocking follow-ups` and completing
all required validation may this task:

- create `docs/reviews/sprint-25-lm-studio-integration.md`;
- transition Sprint 25 from `next`/`active` to `completed` in
  `docs/Roadmap.md`;
- make Sprint 26 Ollama Integration the unique `next` target;
- synchronize only minimal hand-off text in `README.md`,
  `docs/Architecture.md`, and `docs/architecture/semantic-model-2.md` when live
  documentation requires it;
- conditionally retire the exact Sprint 24 suite below in the same final review
  commit when commit mode is authorized.

A blocked decision creates only the explicitly justified review artifact/state
allowed by the live contract, leaves Sprint 25 incomplete, preserves Sprint 24
prompts, and does not start Sprint 26.

## Previous-suite retirement gate

The verified immediately preceding suite is
`docs/codex/prompts/sprint-24-openai-compatible-provider/` with exactly:

- `00-sprint-24-execution-loop.md`
- `01-investigate-openai-compatible-provider.md`
- `02-define-openai-compatible-provider.md`
- `03-implement-openai-compatible-client.md`
- `04-implement-openai-compatible-discovery.md`
- `05-implement-openai-compatible-generation.md`
- `06-complete-openai-compatible-evidence.md`
- `07-sprint-24-integration-review.md`

Before deletion, re-enumerate tracked, filesystem, and untracked inventories.
Delete only those exact tracked files with explicit file edits and staging, only
after a non-blocking decision and successful validation, and only when commit
mode is authorized. Stop when the inventory differs or contains an endangered
untracked file. Preserve the Sprint 25 suite, `run-next-sprint.md`, non-adjacent
suites, and `.codex/`.

## Task-specific Validation

- List and run the exact non-zero LM Studio package unit and public targets
  selected by committed ADR-0047 and the live manifests.
- `cargo test -p oneagent-openai-compatible --lib --offline`
- `cargo test -p oneagent-openai-compatible --test conformance --offline`
- `cargo test -p oneagent-llm --offline`
- `cargo test -p oneagent-analysis --offline`
- `cargo test -p oneagent-runtime --lib --offline`
- Exact direct/reverse dependency, feature, redaction, no-live-state, scope,
  prompt-inventory, Markdown/link, and diff audits.
- Canonical full workspace validation from `docs/codex/core/validation.md`.
- After authorized retirement: revalidate current-suite completeness, previous
  tracked-suite absence, links, `git diff --check`, and final status.

## Required commit

After every task-specific validation command succeeds, stage only the exact
task-owned paths, create one commit with the exact message below, verify its
paths and resulting `HEAD`, and continue only from clean task-owned state:

`Complete Sprint 25 LM Studio review`

Do not commit after failed validation or when unrelated changes cannot be
excluded.

## Final report additions

Report reviewed range and commits, acceptance matrix, findings/missing evidence,
exact validation, decision, state transition, changed/preserved/deleted paths,
Sprint 24 retirement result, Sprint 26 eligibility, commit, and final Git state.
