# Review Sprint 39 Change Impact Analysis

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
- `docs/architecture/change-impact-analysis-investigation.md`
- `docs/architecture/change-impact-analysis-evidence.md`
- `docs/architecture/git-change-adapter-evidence.md`
- `docs/architecture/mcp-semantic-tools-investigation.md`
- `docs/adr/0017-depends-on-semantics.md`
- `docs/adr/0027-incremental-semantic-index-maintenance.md`
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0041-file-watching.md`
- `docs/adr/0042-persistent-cache.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0060-git-change-adapter.md`
- `docs/adr/0061-change-impact-analysis.md`
- `docs/reviews/sprint-29-mcp-semantic-tools.md`
- `docs/reviews/sprint-38-git-change-adapter.md`
- all committed Task 1–6 code, tests, fixtures, docs, and validation evidence

## Prerequisites / immutable review range

- Tasks 1–6 are committed in order with their required validation.
- Resolve the Sprint 39 planning commit and Task 6 head; review that exact
  immutable range and record both hashes.
- The working tree is clean before review drafting.
- The current launch authorizes exactly one fresh-context read-only reviewer.
  Launch it automatically without separate confirmation and launch no other
  subagent.

## Task

Review the integrated Sprint 39 baseline. Create
`docs/reviews/sprint-39-change-impact-analysis.md` only for `pass` or `pass with
non-blocking follow-ups`. After independent review, primary reconciliation,
complete validation, and same-reviewer artifact consistency, transition Sprint
39 to `completed`, make Sprint 40 — Refactoring Planner the unique `next`
target, and atomically retire the verified Sprint 38 prompt suite.

## Independent review gate

- Spawn exactly one reviewer with `fork_turns="none"` or the runtime's
  equivalent guaranteed fresh context. Supply only the repository root, exact
  immutable range, canonical authorities, exact acceptance criteria and
  exclusions, required report/Graph/Workspace/cache/lifecycle/equivalence/MCP/
  policy/client matrices, canonical validation commands, and this output
  contract. Do not supply the primary's expected decision, rationale,
  acceptance summary, or proposed findings.
- Require exact reviewed range and observed initial HEAD/status; one decision
  (`pass`, `pass with non-blocking follow-ups`, or `blocked`); an acceptance-
  evidence matrix; findings by severity with exact file/line evidence; missing
  evidence separate from defects; commands, counts, exits, skips, zero matches,
  and environment limits; scope/exclusion/API/dependency/cache/schema/client/
  sensitive-data audits; residual risks; and recommended next action.
- The reviewer must remain read-only: no edit, create, delete, stage, commit,
  configure, download, repository mutation, network, or delegation.

## Primary reconciliation and validation

- Independently inspect the exact range and reproduce every reviewer claim.
  Classify each as accepted, rejected with evidence, or unresolved. The final
  decision must not be less severe than the reviewer's recommendation; any
  unresolved disagreement blocks completion.
- Verify every ADR-0061 authority, canonical input, publication/Configuration
  identity, report identity/status/availability/reason/order/duplicate/
  conflict/completeness/summary/bound/failure/redaction, Workspace publication,
  cache, lifecycle, filesystem/Git equivalence, MCP schema/projection/error,
  Tool Policy, public-process, client, compatibility, and evidence criterion
  plus every explicit exclusion.
- Independently rerun the complete non-zero Graph/Analysis/Workspace/cache/
  watching/Git-input/Runtime/protocol/Tool Policy/MCP/client/public-process
  matrix and canonical full workspace validation.
- Confirm Graph remains semantic/diff/impact authority; only complete validated
  previous/current graphs feed the product workflow; repository paths/statuses
  never become impact identity, seeds, reasons, summaries, snapshot/cache, or
  wire values; and no selective mutation, new graph fact, diagnostic/rule,
  scoring, refactoring/edit flow, Git mutation/remote behavior, unsupported UI,
  telemetry, or false Coverage claim entered the range.
- Do not silently fix findings. A production or evidence fix requires a
  separate authorized task commit and a fresh review of the new immutable
  range.

## Review artifact and state transition

The review records the immutable range; requirement-to-evidence matrix;
independent findings; primary reconciliation; exact focused/public/full
commands and outcomes; authority/input/report/Workspace/cache/lifecycle/
equivalence/MCP/policy/client/compatibility/API/dependency/sensitive-data/scope
audits; follow-ups; and effective decision.

Before any Roadmap transition, prompt deletion, staging, or commit, send the
drafted review, Roadmap/current-state diff, and exact retirement diff to the
same reviewer. Require explicit read-only confirmation that every finding,
missing-evidence item, decision, validation result, environment limit, risk,
Sprint 40 hand-off, and retirement path is preserved without weakening. A
failed or unavailable consistency check blocks completion.

Only after all gates pass:

- mark Sprint 39 `completed` in `docs/Roadmap.md`;
- make Sprint 40 — Refactoring Planner the unique `next` target;
- delete exactly the eight verified tracked files under
  `docs/codex/prompts/sprint-38-git-change-adapter/` and no others;
- stage only the review, Roadmap, exact current-state documents explicitly
  owned by the review, and exact retirement paths.

The exact retirement inventory is:

- `docs/codex/prompts/sprint-38-git-change-adapter/00-sprint-38-execution-loop.md`
- `docs/codex/prompts/sprint-38-git-change-adapter/01-investigate-git-change-adapter.md`
- `docs/codex/prompts/sprint-38-git-change-adapter/02-define-git-change-adapter.md`
- `docs/codex/prompts/sprint-38-git-change-adapter/03-implement-change-set-domain.md`
- `docs/codex/prompts/sprint-38-git-change-adapter/04-implement-git-repository-reader.md`
- `docs/codex/prompts/sprint-38-git-change-adapter/05-integrate-workspace-change-inputs.md`
- `docs/codex/prompts/sprint-38-git-change-adapter/06-complete-git-change-adapter-evidence.md`
- `docs/codex/prompts/sprint-38-git-change-adapter/07-sprint-38-integration-review.md`

## Blocking conditions

Any blocking authority/input/identity/order/completeness/summary/bound/failure/
redaction/Workspace/cache/lifecycle/equivalence/MCP/policy/client/
compatibility/evidence/dependency/scope/documentation finding; Graph-authority
duplication; repository-path/status impact seeding; failed or zero-match
required validation; reviewer mutation or incomplete output; unresolved
disagreement; inconsistent artifact; retirement inventory drift; unrelated
change; or failed commit/push blocks the review. Preserve Sprint 39 as
incomplete and keep the Sprint 38 suite.

## Suggested commit message

`Complete Sprint 39 Change Impact Analysis review`

## Final report additions

Report reviewer identity and fresh/read-only proof, immutable range, findings
and reconciliation, exact focused/public/full validation, effective decision,
review artifact, state transition, Sprint 40 eligibility, exact retired files,
artifact-consistency result, preserved paths, commit, and remaining changes.
