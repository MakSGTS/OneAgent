# Sprint Planning Task Template

## Purpose

Use this template for a sprint kickoff that audits live readiness, resolves the
minimum required architecture work, and records an executable ordered plan
without implementing production behavior.

## Recommended profile

- `docs/codex/profiles/architecture.md`

## Required task-specific sections

- Authoritative ADRs / architecture documents
- Sprint objective and current Roadmap state
- Prerequisites / required gate
- Readiness-audit scope
- Source and repository evidence
- Template readiness decision
- Ordered task plan and dependency graph
- Task-owned outcomes and exclusions
- Sprint state gates
- Completion Criteria
- Task-specific Validation
- Suggested commit message (recommendation only)

## Additional acceptance requirements

- Recheck the live implementation, tests, fixtures, Coverage Registry, accepted
  ADRs, Git history, and working tree before decomposing tasks.
- Separate already implemented compatibility constraints from new sprint
  deliverables.
- Audit the smallest sufficient Profile, Workflow, and Template set before
  writing child prompts. Record evidence instead of creating speculative
  framework modules when existing contracts are sufficient.
- Give every task one coherent owned outcome, explicit prerequisite, scope
  boundary, acceptance evidence, validation additions, and commit boundary.
- Order tasks so that every implementation prompt begins from a committed or
  explicitly proven prerequisite.
- Define `already_complete`, blocked-review, sprint-completion, and next-sprint
  hand-off gates.
- When the sprint requires an independent integration reviewer, require both
  the generated final review prompt and master execution prompt to state that
  their current user invocation authorizes exactly one fresh-context read-only
  reviewer. Require automatic launch at the review gate without a separate
  confirmation request, while leaving all other delegation unauthorized.
- Keep production implementation, unsupported source forms, and later-sprint
  concerns out of the planning change.
- Do not mark the sprint `completed` during planning.

## Additional report sections

- Readiness findings
- Template readiness decision
- Accepted planning baseline
- Ordered task manifest
- Dependency and state gates
- Deferred scope
- Recommended next action

## Additional validation

- Validate Markdown consistency, internal links, task numbering, dependency
  order, and Roadmap status.
- Do not run production tests unless implementation files are changed.
