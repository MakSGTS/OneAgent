---
prompt_contract: v2
task_kind: implementation
profile: docs/codex/profiles/implementation.md
template: docs/codex/templates/implementation-task.md
fresh_context: required
context_static_max_percent: 15
context_authorities_max_percent: 20
context_prework_hard_stop_percent: 50
context_working_min_percent: 35
context_reserve_min_percent: 15
---

# Example Prompt Contract v2 Task

Continue project development.

## Reporting

- Repository artifacts: English.
- User-visible reports: use the language required by repository instructions.

## Context manifest

### Must read

- `AGENTS.md` — sections: repository scope, change discipline, validation, and
  Git workflow.
- `docs/adr/NNNN-example.md` — sections: accepted decision and first-slice
  boundary.
- `src/example.rs` — symbols: `ExampleRequest`, `ExampleResult`, and direct
  consumers discovered with `rg`.

### Lookup on demand

- `docs/Architecture.md` — trigger: the accepted ADR references an unresolved
  ownership term; sections: only the matching ownership section.
- integration logs — trigger: a focused validation command fails and its bounded
  output is insufficient to diagnose the failure.

### Excluded from initial context

- complete Roadmap history;
- unrelated ADRs and completed sprint prompts;
- generated artifacts and successful full command logs.

### Preflight

- Effective context window: obtain from runtime telemetry or record `unknown`.
- Admission basis: runtime telemetry or a labelled conservative estimate.
- Record static and authority allocation plus `pass|warning|blocked` before
  substantive investigation.

## Prerequisites / required gate

- The accepted example ADR is committed.
- The working tree contains no conflicting task-created change.

## Task

Implement one bounded example behavior accepted by the ADR.

## Scope

### Included

- The accepted domain behavior and its focused tests.
- Documentation required by a changed public contract.

### Excluded

- Architecture reselection.
- Unrelated refactoring and dependency changes.

## Acceptance criteria

- The accepted behavior is implemented deterministically.
- Focused positive, negative, and regression evidence passes.
- Existing supported behavior remains unchanged.

## Task-specific validation

- Run the narrow example test target discovered from the repository.
- Apply the canonical validation contract from
  `docs/codex/core/validation.md` when its trigger conditions apply.

## Suggested commit message

`Implement bounded example behavior`

## Final report additions

- Report the admitted context sources and preflight decision.
- Report any complete log retained under `local-artifacts/codex-runs/`.
