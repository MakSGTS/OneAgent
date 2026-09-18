---
prompt_contract: v2
task_kind: implementation
profile: docs/codex/profiles/refactoring-safe-edits-implementation.md
template: docs/codex/templates/refactoring-safe-edits-task.md
fresh_context: required
context_static_max_percent: 15
context_authorities_max_percent: 20
context_prework_hard_stop_percent: 50
context_working_min_percent: 35
context_reserve_min_percent: 15
---

# Implement Sprint 40 Immutable Source Evidence

## Reporting

- Communicate with the user in Russian.
- Keep code, APIs, tests, docs, errors, and the commit message in English.

## Context manifest

### Must read

- `AGENTS.md` — sections: repository scope, change discipline, validation, and
  Git branch/release workflow.
- `docs/adr/0063-refactoring-planner.md` — sections: immutable source-document
  owner, content version, exact occurrence ranges, bounds, failures,
  compatibility, and Task 3 implementation slice.
- `docs/architecture/refactoring-planner-investigation.md` — sections: missing
  source evidence, current BSL ranges, and accepted implementation oracle.
- `docs/Roadmap.md` — sections: Sprint 40 amendment, exclusions, and Task 3.
- `docs/codex/profiles/refactoring-safe-edits-implementation.md`,
  `docs/codex/templates/refactoring-safe-edits-task.md`, and
  `docs/codex/workflows/refactoring-safe-edits.md` — complete selected contract.
- `crates/common/src/source.rs` — complete typed path/position/span baseline and
  direct consumers found by bounded `rg`.
- `crates/bsl/src/{lib,calls}.rs` — symbols: `BslSymbol`, `BslCall`, extractors,
  constructors, source scanning, errors, and focused tests.

### Lookup on demand

- `crates/bsl/src/{queries,query_language}.rs` — trigger: ADR-0063 accepts an
  occurrence family already modeled there; exact range types and constructors.
- adapter constructors — trigger: a public BSL constructor migration requires
  an exact direct consumer inventory; bounded query for that constructor only.
- source hashing code — trigger: ADR-0063 accepts reuse or relocation of the
  dependency-free Designer digest; exact module and tests only.

### Excluded from initial context

- complete adapters, Graph, Workspace, Runtime, protocol, and client code;
- unrelated BSL grammar, full fixture corpora, and historical diffs;
- semantic planner, source reads outside captured input, and all mutation.

### Preflight

- Record effective window or `unknown`, measurement basis, admitted material,
  and `pass|warning|blocked` before implementation.
- Narrow constructor/consumer queries at warning and stop at the hard limit.

## Prerequisites / required gate

- `HEAD` is exactly the committed Task 2 result with subject
  `Define Sprint 40 Refactoring Planner`.
- ADR-0063 accepts an implementable repository-owned source evidence contract,
  and the task-owned worktree is clean.

## Task

Implement the accepted source-independent immutable source-document evidence
and exact BSL declaration/reference occurrence ranges without adapter,
Workspace, planner, or filesystem integration.

## Scope

### Included

- Accepted confined path, bounded immutable text/bytes, deterministic content
  version, coordinate/byte-range contract, occurrence kind, equality/order,
  duplicate/conflict behavior, checked bounds, and closed redacted failures.
- Exact identifier ranges for the accepted BSL declaration and reference forms,
  with additive constructor/API migration and focused tests.

### Excluded

- Adapter file discovery or reads, Graph emission, Workspace snapshots,
  refactoring-plan types/evaluation, previews, protocols, and mutation.

## Acceptance criteria

- One immutable document binds path, exact content, deterministic version, and
  ranges that are validated against UTF-8 boundaries and content length.
- Accepted BSL extractors return exact identifier occurrences across Russian/
  English syntax, whitespace, Unicode, line endings, repeats, malformed input,
  exact/over bounds, and deterministic re-extraction.
- Existing declaration/call semantic identity and resolution behavior remains
  compatible; no production dependency is added.
- Errors contain closed categories and counts, not source content or absolute
  paths.

## Task-specific validation

- Run non-zero Common source-document and BSL exact-occurrence test targets,
  plus existing unfiltered BSL declaration/call/resolution tests.
- Run affected package checks and the canonical validation triggered by
  `docs/codex/core/validation.md`.

## Suggested commit message

`Implement Sprint 40 immutable source evidence`

## Final report additions

- Report the document/version/range contract, constructor compatibility,
  meaningful test counts, dependency/API audit, and deferred adapter/planner
  integration.
