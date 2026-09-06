---
prompt_contract: v2
task_kind: review
profile: docs/codex/profiles/review.md
template: docs/codex/templates/review-task.md
fresh_context: required
context_static_max_percent: 15
context_authorities_max_percent: 20
context_prework_hard_stop_percent: 50
context_working_min_percent: 35
context_reserve_min_percent: 15
---

# Review Sprint 40.1 Refactoring Planner Remediation Design

## Reporting

- Communicate with the user in Russian.
- Keep the decision artifact and commit message in English.

## Context manifest

### Must read

- `AGENTS.md` — sections: repository scope, validation, Git branch/release
  workflow, and GUI validation.
- `docs/adr/0063-refactoring-planner.md` — sections: Publication and target
  identity, Workspace and lifecycle, compatibility, and acceptance.
- `docs/Roadmap.md` — sections: Sprint 40.1 Refactoring Planner Remediation
  execution plan and Sprint 40.1 ADR invariant matrix.
- `docs/codex/workflows/review.md` — section: Targeted pre-implementation design
  review.
- `crates/analysis/src/{lib.rs,change_impact.rs,refactoring.rs}` and
  `apps/runtime/src/workspace/mod.rs` — symbols matching
  `ChangeImpactPublicationId|WorkspacePublicationId|publication_id`.

### Lookup on demand

- Direct Rust consumers and tests — trigger: the bounded symbol inventory does
  not establish source compatibility, ownership, or negative-oracle placement.
- Sprint 40 evidence and review observations — trigger: a matrix row lacks an
  exact baseline discrepancy or expected count.

### Excluded from initial context

- implementation conversation transcripts and the primary's expected decision;
- complete unrelated architecture documents and historical prompt suites;
- Sprint 41 transaction design and all source-mutation behavior;
- successful build logs and complete fixture corpora.

### Preflight

- Record effective context window or `unknown`, measurement basis, admitted
  material, and `pass|warning|blocked`; narrow selectors at warning and stop at
  the hard limit.

## Prerequisites / required gate

- Run from the committed Sprint 40.1 planning baseline on
  `codex/v0.7-sprint-40.1` with a clean task-owned worktree.
- The Roadmap and master prompt contain byte-identical matrix, design-gate, and
  implementation-baseline records.
- Launch exactly one guaranteed fresh-context read-only reviewer without prior
  implementation turns or an expected decision.

## Task

Perform the targeted pre-implementation design review required by the opted-in
new workflow. Decide whether the matrix and one-task correction place the
canonical type, source-compatible alias, checked increment, Runtime boundary,
negative oracle, evidence synchronization, and governance-scope audit at the
correct owners without changing ADR-0063.

## Review target

- Exact committed Sprint 40.1 planning range resolved from version head
  `427a78cd809a16bae2ee160867b20bb64c1d415e` through the planning commit.
- Product contract: ADR-0063 publication identity and existing no-mutation
  boundary.

## Reviewed baseline / commit or diff range

- Record full endpoint hashes, parent/subject, branch, status, path inventory,
  and the exact invariant-matrix selector before reviewer dispatch.

## Review Criteria

- Canonical owner and stable public module path are unambiguous.
- `ChangeImpactPublicationId` remains the exact same type and counter through a
  source-compatible alias or projection, never a second newtype or conversion.
- Checked non-zero construction and overflow behavior remain deterministic.
- Runtime and all direct consumers migrate to the canonical vocabulary without
  protocol, cache, Graph, publication-sequence, or behavioral change.
- One public negative compile/runtime oracle and focused validation are mapped.
- Evidence counts are derived live, and governance-only commits are audited
  separately from product acceptance.
- Scope and numeric baseline are sufficient and do not include Sprint 41 edits.

## Acceptance evidence matrix

- Review every row under `Sprint 40.1 ADR invariant matrix` and return
  `pass|blocked`, exact owner/location, negative oracle, missing evidence, and
  scope result.

## Independent reviewer contract and output

- Follow the targeted design-review contract in
  `docs/codex/workflows/review.md`; remain read-only, do not delegate, and return
  one decision, findings, missing evidence, scope conformance, and next action.

## Automatic independent-reviewer authorization

- The current Sprint 40.1 launch authorizes exactly this one fresh-context
  read-only design reviewer and no mutation or further delegation.

## Authorized review outputs and state transition

- After reviewer `pass`, the primary may create only
  `docs/reviews/sprint-40-1-refactoring-planner-remediation-design.md`, validate
  it, commit it separately, and push. Do not change product code or sprint
  completion state.

## Scope

### Included

- Read-only design review, matrix verification, scope-baseline verification,
  and the predeclared decision artifact after `pass`.

### Excluded

- Production implementation, evidence-count edits, integration review, sprint
  closure, prompt retirement, and Sprint 41 behavior.

## Acceptance criteria

- Every accepted production invariant has one owner/location, ordering point
  where applicable, negative oracle, and focused validation.
- Reviewer returns `pass`; otherwise create no artifact or commit and stop.
- The committed decision artifact exactly records the reviewed range and result.

## Task-specific validation

- Run bounded symbol/API/diff checks, Markdown link checks, prompt validation,
  and `git diff --check`; do not run the full production gate.

## Suggested commit message

`Approve Sprint 40.1 remediation design`

## Final report additions

- Report reviewer identity, fresh/read-only proof, range, matrix disposition,
  findings, missing evidence, scope decision, artifact, and commit/push.
