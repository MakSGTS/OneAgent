# Base Task Prompt Template

## Purpose

Use this base contract for every newly generated executable child prompt.
Specialized task templates add only family-specific sections and evidence.

## Required front matter

```yaml
---
prompt_contract: v2
task_kind: <investigation|architecture|implementation|review>
profile: docs/codex/profiles/<profile>.md
template: docs/codex/templates/<template>.md
fresh_context: required
context_static_max_percent: 15
context_authorities_max_percent: 20
context_prework_hard_stop_percent: 50
context_working_min_percent: 35
context_reserve_min_percent: 15
---
```

The selected Profile and specialized Template paths must exist. Historical
legacy prompts are not required to adopt this front matter retroactively.

## Required sections

- Reporting
- Context manifest
  - Must read
  - Lookup on demand
  - Excluded from initial context
  - Preflight
- Prerequisites / required gate
- Task
- Scope
  - Included
  - Excluded
- Acceptance criteria
- Task-specific validation
- Suggested commit message
- Final report additions, when the base final report is insufficient

## Context requirements

- Follow `docs/codex/core/context-management.md`.
- A `Must read` entry for a large document must identify exact headings,
  symbols, ranges, diffs, or bounded queries.
- Task-specific authorities belong in the Context Manifest instead of a second
  unbounded authority list.
- The preflight must complete before substantive investigation.
- Do not execute the prompt unless the current context is guaranteed fresh.

## Composition rule

The child prompt references rather than copies permanent Repository Safety,
Context Management, Validation, Final Report, and Workflow text. Concrete scope,
acceptance, authorities, validation additions, and report additions remain in
the child prompt.
