# Implement Sprint 37 Rule Execution

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/rules-engine-implementation.md`
- `docs/codex/templates/rules-engine-task.md`

## Required workflows

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/rules-engine.md`
- `docs/codex/workflows/diagnostics-engine.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/rules-engine-investigation.md`
- `docs/adr/0008-semantic-model-2-knowledge-graph.md`
- `docs/adr/0058-diagnostics-engine.md`
- `docs/adr/0059-rules-engine.md`

## Prerequisite

Task 4 is committed, its complete validation passes, and registry, dependency
planning, configuration, and applicability match ADR-0059.

## Task

Implement the accepted bounded Rules Engine execution and terminal result
contract, including only ADR-0059-approved diagnostic evidence integration.
Keep Workspace/cache and public protocol composition outside this task.

## Required behavior and evidence

- Execute only one validated immutable plan over the exact accepted canonical
  inputs without source reads, graph mutation, hidden validation, mutable
  globals, or a competing semantic/diagnostic authority.
- Apply the accepted sequential or structured execution order, applicability,
  cancellation checkpoints, dependency-blocking, failure containment,
  continuation/fail-closed policy, bounds, and cleanup exactly.
- Produce one typed terminal result per admitted rule and one deterministic
  complete aggregate result. Preserve distinctions among disabled,
  inapplicable, dependency-blocked, cancelled, failed, partial, and successful
  outcomes.
- Map rule-produced diagnostic evidence only through ADR-0058 and ADR-0059
  accepted identity, code/kind/family, duplicate/conflict, order, suppression,
  summary, provenance, location, bounds, error, and completeness contracts.
  Never invent graph facts or silently select a cross-rule conflict.
- Keep errors closed, bounded, deterministic, and free of rejected
  configuration, source content, paths, secrets, provenance payloads, or
  internal chains. Return no unrecorded partial result.
- Add focused repository-owned conformance rules and tests for empty, positive,
  independent, dependency chain/diamond, disabled, inapplicable, rule failure,
  dependency block, cancellation at every accepted boundary, result collision,
  exact/over result bounds, reordered equivalent inputs, repeated execution,
  error redaction, and cleanup.
- Preserve existing Graph validation and reports, Diagnostics Engine behavior,
  adapters, Workspace/cache, Runtime projections, MCP/LSP contracts, and
  Coverage outside accepted additive domain APIs.

## Excluded scope

Workspace snapshot or cache integration, watcher lifecycle changes, protocol or
IDE schema/capability changes, external configuration, dynamic rules, plugins,
scripts, remote execution, source mutation, automatic fixes, safe edits,
current-state documentation, and Sprint completion.

## Validation

Run non-zero focused execution/status/dependency-block/failure/cancellation/
result/diagnostic-collision/bound/redaction/reorder/repetition/cleanup tests;
affected rule-domain and Diagnostics Engine regressions; then the canonical full
Rust workspace gate and `git diff --check`.

## Suggested commit message

`Implement Sprint 37 rule execution`

## Final report additions

Report execution ownership and lifecycle, terminal statuses, failure and
cancellation containment, result aggregation, diagnostic mapping and
collisions, bounds/errors/cleanup, exact focused tests/counts, preserved
authority and compatibility, API/dependency impact, and full validation.
