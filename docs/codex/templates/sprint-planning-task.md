# Sprint Planning Task Template

## Purpose

Use this template for a sprint kickoff that audits live readiness, resolves the
minimum required architecture work, and records an executable ordered plan
without implementing production behavior.

## Recommended profile

- `docs/codex/profiles/architecture.md`

## Required base template

- `docs/codex/templates/task-prompt.md`

## Required task-specific sections

- Sprint objective and current Roadmap state
- Readiness-audit scope
- Source and repository evidence
- Template readiness decision
- Ordered task plan and dependency graph
- Task-owned outcomes and exclusions
- Sprint state gates
- Completion Criteria

## Additional acceptance requirements

- Recheck the live implementation, tests, fixtures, Coverage Registry, accepted
  ADRs, Git history, and working tree before decomposing tasks.
- Separate already implemented compatibility constraints from new sprint
  deliverables.
- Audit the smallest sufficient Profile, Workflow, and Template set before
  writing child prompts. Record evidence instead of creating speculative
  framework modules when existing contracts are sufficient.
- Generate only Prompt Contract v2 child prompts. Give large authorities exact
  section, symbol, range, diff, or query selectors and put optional evidence
  behind an explicit `Lookup on demand` trigger.
- Give every task one coherent owned outcome, explicit prerequisite, scope
  boundary, acceptance evidence, validation additions, and commit boundary.
- For Sprint 41 and later, before production implementation, commit an
  ADR-invariant matrix that maps every applicable accepted production invariant
  to its exact production location, the operation or retention point that it
  must precede when ordering matters, one negative production oracle, and the
  focused validation that proves the placement. Keep documentation, reporting,
  review-process, and explicitly deferred requirements outside this production
  mapping and record their evidence separately.
- For a cross-layer, parser, persistence, filesystem, concurrency, security, or
  other architecture-sensitive Sprint 41 or later change, require the targeted
  pre-implementation design review defined by
  `docs/codex/workflows/review.md`. Commit its non-blocking decision before
  implementation begins. This review does not replace the final integration
  review.
- For every Sprint 41 or later implementation task, commit the following
  baseline in both the authoritative Roadmap execution plan and generated
  master prompt: `expected_path_count`, an integer upper bound for unique
  task-owned changed paths; `expected_text_line_churn`, an integer upper bound
  for additions plus deletions on textual paths; the expected binary-path
  inventory; and the exact focused/full validation budget. Require renewed user
  agreement before continuing when live evidence exceeds twice either numeric
  bound or introduces an unexpected binary path.
- Order tasks so that every implementation prompt begins from a committed or
  explicitly proven prerequisite.
- Define `already_complete`, blocked-review, sprint-completion, and next-sprint
  hand-off gates.
- Require one fresh-context runner per sequential child and one separate
  fresh-context read-only reviewer when the Review workflow applies. Keep the
  master prompt as dispatcher and ledger rather than an implementation context.
- Keep production implementation, unsupported source forms, and later-sprint
  concerns out of the planning change.
- Do not mark the sprint `completed` during planning.

## Additional report sections

- Readiness findings
- Template readiness decision
- Accepted planning baseline
- Committed ADR-invariant matrix and targeted design-review decision
- `expected_path_count`, `expected_text_line_churn`, expected binary paths, and
  validation budget
- Ordered task manifest
- Dependency and state gates
- Deferred scope
- Recommended next action

## Additional validation

- Validate Markdown consistency, internal links, task numbering, dependency
  order, and Roadmap status.
- Run `scripts/validate-codex-prompts.sh` against every generated child prompt.
- Do not run production tests unless implementation files are changed.
