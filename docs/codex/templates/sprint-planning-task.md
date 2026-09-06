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
- Before production implementation, build an ADR-invariant matrix that maps
  every accepted requirement to its exact production location, the operation
  or retention point that it must precede when ordering matters, one negative
  production oracle, and the focused validation that proves the placement.
- For a cross-layer, parser, persistence, filesystem, concurrency, security, or
  other architecture-sensitive change, require one targeted read-only design
  review of the invariant matrix before implementation begins. Resolve every
  blocking design finding in the plan; this review does not replace the final
  integration review.
- Record a planning baseline for expected task-owned paths, approximate diff
  scale, and focused/full validation budget. Require renewed user agreement
  before continuing when live evidence expands either changed-path count or
  estimated diff scale beyond twice that baseline.
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
- ADR-invariant matrix and targeted design-review result
- Expected path/diff baseline and validation budget
- Ordered task manifest
- Dependency and state gates
- Deferred scope
- Recommended next action

## Additional validation

- Validate Markdown consistency, internal links, task numbering, dependency
  order, and Roadmap status.
- Run `scripts/validate-codex-prompts.sh` against every generated child prompt.
- Do not run production tests unless implementation files are changed.
