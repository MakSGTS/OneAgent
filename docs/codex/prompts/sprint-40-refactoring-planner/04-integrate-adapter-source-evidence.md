---
prompt_contract: v2
task_kind: implementation
profile: docs/codex/profiles/source-adapter-implementation.md
template: docs/codex/templates/source-adapter-task.md
fresh_context: required
context_static_max_percent: 15
context_authorities_max_percent: 20
context_prework_hard_stop_percent: 50
context_working_min_percent: 35
context_reserve_min_percent: 15
---

# Integrate Sprint 40 Adapter Source Evidence

## Reporting

- Communicate with the user in Russian.
- Keep code, APIs, tests, fixtures, docs, errors, and the commit message in
  English.

## Context manifest

### Must read

- `AGENTS.md` — sections: repository scope, change discipline, validation, and
  Git branch/release workflow.
- `docs/adr/0063-refactoring-planner.md` — sections: adapter source capture,
  canonical source-document mapping, content-version/provenance agreement,
  completeness, failures, cross-format oracle, compatibility, and Task 4.
- `docs/architecture/refactoring-planner-investigation.md` — sections: EDT and
  Designer source evidence gaps and paired fixture inventory.
- `docs/Roadmap.md` — sections: Sprint 40 amendment, exclusions, and Task 4.
- `docs/codex/profiles/source-adapter-implementation.md`,
  `docs/codex/templates/source-adapter-task.md`, and workflows:
  `source-adapter.md` and `refactoring-safe-edits.md` — selected source and
  planner-precondition contracts.
- committed Task 3 source-document and BSL occurrence APIs — exact changed
  symbols and focused tests only.
- `adapters/edt/src/{module_reader,bsl_graph,lib}.rs` — symbols: module
  discovery/read, `AnalyzedBslModule`, source identity/location, graph build
  result, and direct tests.
- `adapters/designer-xml/src/{module_reader,semantic_graph,source_hash,lib}.rs`
  — symbols: raw source evidence, digest, declaration/call contribution, graph
  build result, and direct tests.
- paired and controlled source fixtures/tests under `adapters/{edt,designer-xml}/tests`
  named by ADR-0063; load only owning test functions and referenced files.

### Lookup on demand

- metadata descriptor sources — trigger: the accepted first refactoring family
  requires a non-BSL artifact; exact reader, fixture, and provenance functions.
- Graph provenance/query code — trigger: adapter output cannot preserve accepted
  semantic/source separation; exact types and consumers only.
- Workspace builders — trigger: an additive adapter result API has a direct
  compile consumer requiring migration; exact call sites only, without Task 7
  composition behavior.

### Excluded from initial context

- complete generated configurations and unrelated adapter families;
- full Graph/Workspace/Runtime/protocol/client implementations;
- planner domain/evaluation, public projection, and source mutation.

### Preflight

- Record effective window or `unknown`, measurement basis, admitted material,
  and `pass|warning|blocked` before implementation.
- Narrow adapter/fixture/consumer selectors at warning and stop at the hard
  limit.

## Prerequisites / required gate

- `HEAD` is exactly the committed Task 3 result with subject
  `Implement Sprint 40 immutable source evidence`.
- Source-document and exact BSL occurrence targets pass unfiltered and the
  task-owned worktree is clean.

## Task

Capture and publish the accepted immutable source-document evidence through the
EDT and Designer XML production adapter boundaries, preserving canonical Graph
semantics and proving the paired cross-format oracle.

## Scope

### Included

- Deterministic confined artifact enumeration/read, raw-to-canonical content
  handling, content version, exact occurrence mapping, provenance agreement,
  complete/partial classification, duplicate/conflict/bound/failure behavior,
  additive adapter result APIs, and paired production-builder evidence.

### Excluded

- Workspace storage/composition, planning logic, MCP/policy/client changes,
  persistence, edits, transactions, and unsupported source forms.

## Acceptance criteria

- EDT and Designer production builders expose equivalent source-independent
  document/occurrence evidence for the accepted paired slice while preserving
  deliberate path/format/provenance differences.
- The content version always matches the exact captured bytes/text used for
  ranges and Graph contribution; path-only or opaque-hash-only evidence is not
  accepted as complete.
- Missing, duplicate, conflicting, unreadable, malformed, non-UTF-8, escaping,
  changed-during-capture, and exact/over-bound cases follow ADR-0063 atomically.
- Existing Graph identities, reports, validation, Coverage, adapter conformance,
  and repeated-build determinism remain compatible.

## Task-specific validation

- Run non-zero EDT and Designer reader/parser/production-builder source-evidence
  targets and the exact paired cross-adapter conformance target.
- Run controlled semantic/source changes, reorder/repeat, negative/bound tests,
  affected package checks, and the canonical validation triggered by
  `docs/codex/core/validation.md`.

## Suggested commit message

`Integrate Sprint 40 adapter source evidence`

## Final report additions

- Report source capture/completeness, canonical mapping, deliberate adapter
  differences, paired oracle, changed public consumers, tests, and preserved
  Graph/no-mutation boundary.
