# Review Sprint 38 Git Change Adapter

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/review.md`
- `docs/codex/templates/review-task.md`

## Required workflow

`docs/codex/workflows/review.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/git-change-adapter-investigation.md`
- `docs/architecture/git-change-adapter-evidence.md`
- `docs/adr/0027-incremental-semantic-index-maintenance.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/adr/0060-git-change-adapter.md`
- `docs/reviews/sprint-19-file-watching.md`
- `docs/reviews/sprint-37-rules-engine.md`
- all committed Task 1–6 code, tests, fixtures, docs, and validation evidence

## Prerequisites / immutable review range

- Tasks 1–6 are committed in order with their required validation.
- Resolve the Sprint 38 planning commit and Task 6 head; review that exact
  immutable range and record both hashes.
- The working tree is clean before review drafting.
- The current launch authorizes exactly one fresh-context read-only reviewer.
  Launch it automatically without separate confirmation and launch no other
  subagent.

## Task

Review the integrated Sprint 38 baseline. Create
`docs/reviews/sprint-38-git-change-adapter.md` only for `pass` or `pass with
non-blocking follow-ups`. After independent review, primary reconciliation,
complete validation, and same-reviewer artifact consistency, transition Sprint
38 to `completed`, make Sprint 39 — Change Impact Analysis the unique `next`
target, and atomically retire the verified Sprint 37 prompt suite.

## Independent review gate

- Spawn exactly one reviewer with `fork_turns="none"` or the runtime's
  equivalent guaranteed fresh context. Supply only the repository root, exact
  immutable range, canonical authorities, exact acceptance criteria and
  exclusions, required domain/reader/repository-state/Workspace/cache/
  lifecycle/platform/consumer matrices, canonical validation commands, and
  this output contract. Do not supply the primary's expected decision,
  rationale, acceptance summary, or proposed findings.
- Require exact reviewed range and observed initial HEAD/status; one decision
  (`pass`, `pass with non-blocking follow-ups`, or `blocked`); an acceptance-
  evidence matrix; findings by severity with exact file/line evidence; missing
  evidence separate from defects; commands, counts, exits, skips, zero matches,
  and environment limits; scope/exclusion/API/dependency/platform/security
  audits; residual risks; and recommended next action.
- The reviewer must remain read-only: no edit, create, delete, stage, commit,
  configure, download, repository mutation, network, or delegation.

## Primary reconciliation and validation

- Independently inspect the exact range and reproduce every reviewer claim.
  Classify each as accepted, rejected with evidence, or unresolved. The final
  decision must not be less severe than the reviewer's recommendation; any
  unresolved disagreement blocks completion.
- Verify every ADR-0060 authority, repository, endpoint, state-layer, status,
  path, identity, order, duplicate/conflict, rename/copy, bound, failure,
  reader/process-or-library, Workspace mapping, equivalence, rebuild, cache,
  lifecycle, platform, sensitive-data, consumer, and evidence criterion plus
  every explicit exclusion.
- Independently rerun the complete non-zero domain/reader/temporary-repository/
  Workspace/cache/watching/Runtime/Graph/Analysis/protocol/client/public-
  process matrix and canonical full workspace validation.
- Confirm Git remains input evidence only; production source adapters and Graph
  remain semantic authority; ADR-0027 remains canonical after complete graph
  builds; no remote operation, credential access, repository mutation,
  semantic impact, diagnostic/rule inference, selective semantic update,
  refactoring, edit flow, unsupported protocol/UI, telemetry, or false Coverage
  claim entered the range.
- Do not silently fix findings. A production or evidence fix requires a
  separate authorized task commit and a fresh review of the new immutable
  range.

## Review artifact and state transition

The review records the immutable range; requirement-to-evidence matrix;
independent findings; primary reconciliation; exact focused/public/full
commands and outcomes; authority, repository/endpoint/state/status/path/order,
reader, Workspace/cache/lifecycle, platform, compatibility, API, dependency,
sensitive-data, scope, and deferred-scope audits; follow-ups; and effective
decision.

Before any Roadmap transition, prompt deletion, staging, or commit, send the
drafted review, Roadmap/current-state diff, and exact retirement diff to the
same reviewer. Require explicit read-only confirmation that every finding,
missing-evidence item, decision, validation result, environment limit, risk,
Sprint 39 hand-off, and retirement path is preserved without weakening. A
failed or unavailable consistency check blocks completion.

Only after all gates pass:

- mark Sprint 38 `completed` in `docs/Roadmap.md`;
- make Sprint 39 — Change Impact Analysis the unique `next` target;
- delete exactly the nine verified tracked files under
  `docs/codex/prompts/sprint-37-rules-engine/` and no others;
- stage only the review, Roadmap, exact current-state documents explicitly
  owned by the review, and exact retirement paths.

The exact retirement inventory is:

- `docs/codex/prompts/sprint-37-rules-engine/00-sprint-37-execution-loop.md`
- `docs/codex/prompts/sprint-37-rules-engine/01-investigate-rules-engine.md`
- `docs/codex/prompts/sprint-37-rules-engine/02-define-rules-engine.md`
- `docs/codex/prompts/sprint-37-rules-engine/03-implement-rule-registry.md`
- `docs/codex/prompts/sprint-37-rules-engine/04-implement-rule-planning.md`
- `docs/codex/prompts/sprint-37-rules-engine/05-implement-rule-execution.md`
- `docs/codex/prompts/sprint-37-rules-engine/06-integrate-rule-snapshots.md`
- `docs/codex/prompts/sprint-37-rules-engine/07-complete-rules-engine-evidence.md`
- `docs/codex/prompts/sprint-37-rules-engine/08-sprint-37-integration-review.md`

## Blocking conditions

Any blocking authority, repository, endpoint, state-layer, status, path,
ordering, confinement, rename/copy/conflict, bound, failure, reader/process,
Workspace/cache/lifecycle/platform/consumer/compatibility/evidence/dependency/
scope/documentation finding; failed or zero-match required validation;
reviewer mutation or incomplete output; unresolved disagreement; inconsistent
artifact; retirement inventory drift; unrelated change; or failed commit/push
blocks the review. Preserve Sprint 38 as incomplete and keep the Sprint 37
suite.

## Suggested commit message

`Complete Sprint 38 Git Change Adapter review`

## Final report additions

Report reviewer identity and fresh/read-only proof, immutable range, findings
and reconciliation, exact focused/public/full validation, effective decision,
review artifact, state transition, Sprint 39 eligibility, exact retired files,
artifact-consistency result, preserved paths, commit, and remaining changes.
