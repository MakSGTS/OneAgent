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

# Remediate Sprint 40.1 Refactoring Planner Contract

## Reporting

- Communicate with the user in Russian.
- Keep code, public APIs, tests, documentation, errors, and commit message in
  English.

## Context manifest

### Must read

- `AGENTS.md` — sections: repository scope, change discipline, validation, Git
  workflow, and GUI validation.
- `docs/adr/0063-refactoring-planner.md` — sections: Publication and target
  identity, Workspace and lifecycle, compatibility, and acceptance.
- `docs/Roadmap.md` — sections: Sprint 40.1 Refactoring Planner Remediation
  execution plan and Sprint 40.1 ADR invariant matrix.
- `docs/reviews/sprint-40-1-refactoring-planner-remediation-design.md` — complete
  committed `pass` decision.
- `crates/analysis/src/{lib.rs,change_impact.rs,refactoring.rs}` and
  `apps/runtime/src/workspace/mod.rs` — complete publication identity
  definitions and direct consumers.
- `docs/architecture/refactoring-planner-evidence.md` — sections: baseline,
  focused executable evidence, canonical gate, and scope audits.

### Lookup on demand

- Direct consumers/tests from bounded `rg` results — trigger: a signature or
  alias change has a consumer not covered by the initial files.
- Six governance-only commits `dce7470e`, `25019f17`, `f1698840`, `89ce1402`,
  `6e038660`, and `8a8e4734` — trigger: the final scope audit needs exact path
  and net-diff classification.
- Retained validation logs — trigger: a command fails or inventory totals do
  not reconcile.

### Excluded from initial context

- complete unrelated architecture and source trees;
- previous reviewer conversations or expected implementation details;
- Sprint 41 edit transactions, mutation, rollback, UI, or protocol expansion;
- full fixtures and successful logs.

### Preflight

- Record effective context window or `unknown`, measurement basis, admitted
  material, and `pass|warning|blocked`; narrow selectors at warning and stop at
  the hard limit.

## Prerequisites / required gate

- Exact start is the committed Task 1 `pass` artifact on
  `codex/v0.7-sprint-40.1` with a clean task-owned worktree.
- The matrix selector and design decision are committed and consistent.
- Enforce the baseline: at most 10 expected task-owned paths, 600 textual line
  changes, no binary paths, nine focused checks, and one stable full gate.

## Task

Make `oneagent-analysis::publication::WorkspacePublicationId` the canonical
checked non-zero publication identity required by ADR-0063. Preserve
`ChangeImpactPublicationId` as a source-compatible alias or exact projection of
the same type and counter, migrate Runtime's public Workspace vocabulary to the
canonical name, add focused regressions, and synchronize Sprint 40 evidence
from live validation. Separately classify the six governance-only commits so
they are not presented as Refactoring Planner product behavior.

## Canonical semantic, source, repository, and authorization boundaries

- Analysis owns the single process-local identity; Change Impact and
  Refactoring share it.
- Runtime owns publication sequencing and atomic snapshot composition.
- Graph, source adapters, cache schema, protocol, Tool Policy, clients, and
  source bytes remain behaviorally unchanged.
- No plan or publication identity grants edit authorization.

## Supported refactoring family and production source entry point

- Preserve only `bsl_callable_rename_v1` and the existing read-only planner.
- This task changes publication type ownership and documentation evidence, not
  supported syntax or operations.

## Target, snapshot, source-version, and precondition contract

- Preserve all existing snapshot, target, source-version, stale, overflow, and
  failed-publication behavior while exposing the canonical type path.

## Plan and operation identity, vocabulary, ordering, and dependencies

- No plan/operation hashing input, vocabulary, ordering, dependency, or output
  shape may change.

## Duplicate, overlap, conflict, stale-input, and incompatibility behavior

- Preserve existing closed failures and publication equality semantics.

## Bounds, completeness, preview, failures, and sensitive-data policy

- Preserve all ADR-0063 bounds, redaction, complete-plan, and no-snippet preview
  behavior.

## Runtime, policy, protocol, client, cache, and watcher impact

- Runtime returns the canonical type name in its Rust API while wire values and
  lifecycle are unchanged. Existing source imports of
  `ChangeImpactPublicationId` remain valid.
- Cache schema/semantic version, MCP schemas/revisions/catalog, VS Code, file
  watching, and Git-input behavior remain unchanged.

## Repository-owned evidence corpus and deterministic oracle

- Add an exact public-path and alias-identity regression, preserve checked
  zero/overflow tests, and rerun the existing Change Impact, Refactoring,
  Workspace, adapters, cache, and public-process evidence.

## Scope

### Included

- Canonical publication module/newtype, compatibility alias, direct Rust
  consumer migration, focused regression tests, live test enumeration, evidence
  and Roadmap count synchronization, and governance-only scope classification.

### Excluded

- ADR amendment, second counter or conversion, behavioral protocol/cache/Graph
  changes, supported-family expansion, source mutation, transactions, UI,
  unrelated framework changes, and sprint state transition.

## Acceptance criteria

- The exact canonical public path exists and owns the only newtype.
- The old public name is source-compatible and type-identical; no conversion or
  second sequence exists.
- Runtime public snapshot/impact signatures use the canonical vocabulary.
- Non-zero construction, initial value, checked successor, overflow failure,
  failed-attempt retention, and fresh-service reset remain covered.
- Evidence reports exact live non-zero suite and all-target counts without stale
  values; governance-only commits are explicitly separated from product scope.
- Actual path/churn/binary scope stays within the committed stop-loss or work
  stops for renewed user agreement.
- Focused and the single stable canonical gate pass with a clean task-owned diff.

## Task-specific validation

- Run non-zero publication identity/alias, Change Impact, Refactoring Planner,
  Runtime Workspace, BSL, EDT, Designer, cache, and public MCP process checks.
- Enumerate all workspace targets/tests and reconcile evidence counts.
- Audit public API consumers, duplicate identity definitions, manifests,
  protocol/cache invariants, governance/product scope, sensitive data, and
  tracked artifacts.
- Run the canonical full gate once after the diff is stable.

## Suggested commit message

`Remediate Sprint 40.1 Refactoring Planner contract`

## Final report additions

- Report canonical/alias paths, migrated consumers, preserved behavior,
  planned/actual scope, exact counts, governance classification, validations,
  changed files, commit, push, and remaining status.
