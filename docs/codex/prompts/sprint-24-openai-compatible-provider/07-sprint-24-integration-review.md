# Review Sprint 24 OpenAI-Compatible Provider Integration

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/review.md`

## Template

`docs/codex/templates/review-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 24 execution plan
- `docs/architecture/openai-compatible-provider-investigation.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0046-openai-compatible-provider.md`
- `docs/reviews/sprint-23-llm-provider-abstraction.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- committed Tasks 3-6 implementation and evidence

## Prerequisites / Required gate

Require Tasks 1-6 committed or proven `already_complete`, their focused and
full validation successful, the exact dependency approval recorded, and clean
task-owned state. Stop on any missing prerequisite or blocking defect. Do not
repair production defects inside the review.

## Task

Perform the Sprint 24 integration review, record an evidence-backed decision,
and apply state/document/prompt cleanup only when the decision is `pass` or
`pass with non-blocking follow-ups`.

## Review scope and evidence matrix

Review the committed planning and framework selection; pinned llama.cpp and
authorized live-service evidence; exact dependency approval and lock state;
adapter dependency direction and public construction; base-URL, redirect,
proxy, authentication, and redaction policy; fresh `/v1/models` discovery;
single non-streaming `/v1/completions` generation; provider/model identity and
fallback rejection; canonical mapping; bounded bodies and outputs; HTTP,
transport, protocol, timeout, and cancellation failures; cleanup and repeated
execution; public controlled-loopback conformance; existing consumer
compatibility; documentation truthfulness; and excluded-scope preservation.

Require explicit positive and negative oracles for malformed/empty/duplicate
models, unknown fields, response-model mismatch, choice/index/finish mismatch,
oversized body/output, redirects, retry absence, credential/prompt/response
sentinels, timeout, cancellation before and during work, and zero surviving
operation state where the accepted ADR requires them. Live network and
credentials are prohibited during review validation.

## Required outputs

- Create `docs/reviews/sprint-24-openai-compatible-provider.md` with findings,
  evidence matrix, validation results, decision, and non-blocking follow-ups.
- For a non-blocking decision only, transition Sprint 24 from `next` to
  `completed` and make Sprint 25 LM Studio Integration the unique `next` sprint.
- Synchronize only demonstrably stale minimum current-state text in
  `README.md`, `docs/Architecture.md`, or
  `docs/architecture/semantic-model-2.md` when review evidence requires it.
- For a non-blocking decision only, retire the exact previous Sprint 23 prompt
  suite in this same commit after revalidating its inventory.
- For a blocking decision, leave roadmap state and prompt suites unchanged,
  record the review, stop, and do not commit a successful completion message.

## Previous-suite retirement inventory

The only eligible directory is
`docs/codex/prompts/sprint-23-llm-provider-abstraction/`, containing exactly:

- `00-sprint-23-execution-loop.md`
- `01-investigate-llm-provider-boundary.md`
- `02-define-llm-provider-abstraction.md`
- `03-implement-provider-domain-model.md`
- `04-implement-capability-aware-requests.md`
- `05-implement-provider-execution-boundary.md`
- `06-complete-llm-provider-evidence.md`
- `07-sprint-23-integration-review.md`

Before deletion, prove that these eight paths are the complete tracked,
filesystem, and untracked inventory. Delete only these files. Preserve this
Sprint 24 suite, `docs/codex/prompts/run-next-sprint.md`, all non-adjacent prompt
suites, `.codex/`, ignored artifacts, and unrelated paths.

## Acceptance Criteria

- The review contains no unresolved blocking finding and traces every Sprint 24
  acceptance condition to committed implementation and successful validation.
- The public adapter conformance target is non-zero, deterministic, controlled
  loopback only, and proves the accepted ADR-0046 behavior and exclusions.
- Provider-neutral `oneagent-llm`, existing consumers, dependency direction,
  redaction, bounded failure, timeout/cancellation, and cleanup are verified.
- Roadmap, architecture/current-state docs, review decision, and prompt
  retirement are mutually consistent.
- Sprint 25 is the unique `next` sprint only after a non-blocking decision.

## Repository Safety

Review first and mutate only required review/state/current-doc paths plus the
exact eligible previous-suite files. Do not change production code, tests,
manifests, lockfiles, unrelated docs/prompts, ignored artifacts, or `.codex/`.
Do not access external services. Stage every path explicitly.

## Task-specific Validation

- Run the exact focused construction, discovery, generation, redaction,
  bounds, timeout, cancellation, cleanup, and public conformance targets.
- Run complete adapter and provider-neutral package tests.
- `cargo test -p oneagent-analysis`
- `cargo test -p oneagent-runtime --lib` with local-bind permission when needed.
- Run the canonical complete workspace validation.
- Audit dependency direction/features, public API surface, sentinels, docs,
  Roadmap unique-next state, review links, and exact previous-suite inventory.
- `git diff --check`
- Inspect the complete task diff and `git status --short` before committing.

## Suggested commit message

`Complete Sprint 24 OpenAI-compatible review`

## Final report additions

Report findings by severity, decision, evidence matrix, exact validation and
results, dependency approval, changed/preserved paths, Sprint 24/25 state,
previous-suite retirement, commit, remaining changes, and final Git state.
