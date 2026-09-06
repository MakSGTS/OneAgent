# Sprint Execution Loop Template

## Purpose

Use this template for a master prompt that executes an already accepted sprint
plan strictly in dependency order.

## Required workflow

- `docs/codex/workflows/sequential-sprint-execution.md`

## Required task-specific sections

- Reporting language
- Canonical authorities
- Sprint objective and current state
- Starting-state requirements
- Ordered task manifest
- Commit authorization mode
- Fresh-context child-runner authorization and unavailable-runtime fallback
- Initial audit additions
- Sprint 41+ committed ADR-invariant matrix selector and targeted design-review
  decision
- Sprint 41+ per-task `expected_path_count`, `expected_text_line_churn`, expected
  binary paths, and focused/full validation budget
- Task-loop additions, if any
- Already-complete policy additions, if any
- Failure and integration-review gates
- Automatic mandatory reviewer authorization, when the manifest includes an
  independent integration review
- Final report additions

The ordered manifest must identify, for every task:

- order and prompt path;
- required committed prerequisite;
- task-owned outcome;
- task-specific validation additions;
- Prompt Contract v2 and Context Manifest validation state;
- suggested commit message.

For Sprint 41 and later, the master prompt must also contain one machine-readable
`Sprint efficiency contract` block with this exact record shape:

```text
sprint_efficiency_contract: v1
adr_invariant_matrix: <repository-path>::<exact-section-heading>
design_review_gate: <review-prompt-path>|<required-prerequisite>|<decision-artifact-path>|<commit-message>
implementation_baseline: <implementation-prompt-path>|<expected_path_count>|<expected_text_line_churn>|<expected-binary-paths-or-none>|<focused-check-count>|<full-gate-count>
```

Use exactly one non-empty matrix and design-review record and exactly one
implementation baseline per implementation child. For a sprint without an
architecture-sensitive change, use exactly `design_review_gate: none`; otherwise
use the four-field form and a dedicated review child. Numeric fields are base-10
non-negative integers, `full-gate-count` is `0` or `1`, and binary paths are
`none` or a comma-separated repository-relative inventory. Do not place `|` in
any field. The `full-gate-count` values across all implementation records must
sum to exactly `1`; independent reviewer and primary completion-gate validation
remain outside this implementation budget. Repeat the exact
`design_review_gate` and `implementation_baseline` records in the authoritative
Roadmap execution plan so validation can reconcile them mechanically.

## Additional acceptance requirements

- Treat the manifest as an execution plan, not proof of current repository
  state.
- Resolve every mutable baseline from the live repository before execution.
- Do not reorder, skip, combine, or partially commit dependent tasks.
- Dispatch every child in a guaranteed fresh context and retain only its compact
  verified ledger row. Never execute two children in the master context.
- Do not permanently encode commit authorization in a stored prompt. Determine
  commit mode from the current user instruction that launches the execution.
- Treat the current user's launch of the master prompt as authorization for one
  sequential fresh-context runner per manifest child and one mandatory
  fresh-context read-only reviewer when required. Neither runner nor reviewer
  may delegate further. This does not change commit authorization.
- Stop at the child boundary and emit the exact continuation prompt when the
  runtime cannot guarantee fresh context.
- Preserve prompt-suite files unless their modification is explicitly part of
  the current task scope.
- Stop after the first blocking failure.
- For Sprint 41 and later, transport the committed invariant-matrix selector,
  design-review decision, and numeric/path validation baseline to every affected
  fresh-context child; do not rely on a planning transcript or dispatcher
  memory.

## Additional report sections

- Starting and ending `HEAD`
- Ordered task outcomes
- Commits created
- Validation evidence
- Already-complete evidence
- Blocker and stopping point, if any
- Final repository state

## Additional validation

- Validate that every manifest prompt and authoritative document exists before
  starting the first task.
- Validate the Sprint 41+ master together with every Prompt Contract v2 child
  using `scripts/validate-codex-prompts.sh`; earlier suites validate their
  children under their committed contract.
- Validate that prerequisite and commit-message metadata agree with the accepted
  Roadmap execution plan.
