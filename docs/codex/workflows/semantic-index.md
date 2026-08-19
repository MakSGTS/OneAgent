# Semantic Index Workflow

Use this workflow for deterministic derived indexes over a canonical semantic
graph, including complete-snapshot indexes and, when separately authorized by
accepted architecture, incremental index maintenance.

## Canonical authority

- Keep the semantic graph as the canonical owner of facts, identities,
  provenance, and validation state.
- Treat index contents as a derived view; building or querying an index must not
  add, remove, normalize, or reinterpret graph facts.
- Reuse accepted graph identity, name, ownership, and resolution policies. Do
  not create a competing resolution authority.
- Define every lookup key, value, collision rule, and deterministic result order.

## Snapshot index requirements

- State the source snapshot and the lifetime or ownership relationship between
  the graph, index, and query facades.
- Define when an index becomes stale and how callers obtain a fresh index.
- Preserve observable query and typed resolution behavior, including empty,
  duplicate-name, ambiguous, missing, and invalid-ownership cases.
- Prove indexed results are equivalent to the accepted scan-based or canonical
  behavior for every lookup dimension in scope.
- Audit public and internal consumers before changing query, resolution, or
  construction APIs.
- Preserve deterministic construction and result ordering across source
  insertion orders and repeated builds.

## Incremental extension requirements

Apply these requirements only when accepted architecture explicitly includes
incremental maintenance:

- Define the workspace or graph change input and the exact invalidation rules.
- Define how unaffected index state is retained without making retained state a
  second semantic authority.
- Define batch ordering, duplicate-change, deletion, failure, and retry behavior.
- Prove incremental results are equivalent to a clean full rebuild after every
  supported change class.
- Keep persistence, cache formats, and runtime orchestration excluded unless
  they are explicitly part of the accepted contract.

## Required evidence

- Focused positive, empty, duplicate, invalid-state, and regression tests as
  applicable.
- Determinism tests across insertion orders and repeated construction.
- Equivalence tests against the canonical query or full-rebuild behavior.
- Existing Query, Resolution, Validation, Diff, Impact, Coverage, and producer
  integration suites remain green when their observable behavior can be
  affected.
- Performance claims use a reproducible benchmark baseline; otherwise report
  complexity and measured behavior without an unsupported target.

## Boundaries

Do not pull source-adapter state, new semantic inference, persistence, Runtime,
transport, IDE, or incremental behavior into an index task unless accepted
architecture and the task scope explicitly include it.
