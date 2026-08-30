# Implement Sprint 37 Rule Planning

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

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/rules-engine-investigation.md`
- `docs/adr/0058-diagnostics-engine.md`
- `docs/adr/0059-rules-engine.md`

## Prerequisite

Task 3 is committed, its complete validation passes, and the registry matches
ADR-0059.

## Task

Implement only the accepted rule dependency validation, deterministic execution
planning, configuration, and applicability boundary. Do not execute rules or
produce diagnostics yet.

## Required behavior and evidence

- Implement the exact accepted dependency vocabulary and validate every
  dependency against one immutable registry before returning a plan.
- Handle missing, self, duplicate, incompatible, and cyclic dependencies
  exactly as ADR-0059 requires, with closed bounded redacted errors and no
  partial plan when the accepted contract is fail-closed.
- Produce the accepted canonical total order for independent rules, chains,
  branches, and diamonds independently from registration and dependency input
  order.
- Implement only the accepted first-slice configuration authority, defaults,
  identity, precedence, validation, compatibility, scope, and bounds. Add no
  unsupported file, environment, persistence, protocol, or UI grammar.
- Represent disabled, inapplicable, unsupported, invalidly configured, and
  dependency-blocked rules exactly as accepted. Preserve distinctions from
  execution failure, cancellation, diagnostic suppression, and success.
- Add focused tests for empty/single/independent/chain/diamond plans, every
  invalid dependency case, reordered equivalent inputs, exact/over dependency
  bounds, default and explicit configuration, unknown/duplicate/incompatible/
  exact-over configuration, applicability, deterministic errors, and repeated
  planning.
- Preserve registry immutability, Graph and Diagnostics authority, and every
  existing producer, Workspace, cache, Runtime, protocol, adapter, and Coverage
  behavior.

## Excluded scope

Rule body execution, concurrency, cancellation observation, terminal execution
results, diagnostic production, Workspace/cache integration, external
configuration sources, protocols, UI, plugins, scripts, current-state docs, and
Sprint completion.

## Validation

Run non-zero focused dependency/missing/self/duplicate/cycle/order/configuration/
applicability/bound/redaction/repetition tests and registry regressions, then the
canonical full Rust workspace gate and `git diff --check`.

## Suggested commit message

`Implement Sprint 37 rule planning`

## Final report additions

Report dependency semantics and validation, canonical plan ordering,
configuration authority and compatibility, applicability/status behavior,
bounds and errors, exact focused tests/counts, preserved registry and consumer
behavior, API/dependency impact, and full validation outcomes.
