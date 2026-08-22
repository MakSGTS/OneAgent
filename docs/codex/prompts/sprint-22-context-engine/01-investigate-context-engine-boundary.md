# Investigate Sprint 22 Context Engine Boundary

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 22 execution plan
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/v0.4-release-review.md`
- `docs/codex/profiles/context-engine-implementation.md`
- `docs/codex/workflows/context-engine.md`
- `docs/codex/templates/context-engine-task.md`

## Prerequisites / Required gate

Require the committed Sprint 22 planning baseline containing this complete
prompt suite and matching Roadmap manifest. Require Sprint 22 to be the unique
eligible target and preserve a clean task-owned state.

## Investigation objective

Create `docs/architecture/context-engine-investigation.md` with verified
evidence for the smallest complete, deterministic, source-independent Context
Engine slice and the exact questions ADR-0044 must decide. Do not select
architecture, add a dependency, or modify production behavior.

## Questions to answer

- Which crate owns derived source-independent analysis today, which public
  library boundary can own Context Engine behavior, and what consumers or
  dependency directions constrain it?
- Which exact immutable graph/query/index APIs expose nodes, names, kinds,
  ownership, relations, traversal, identity, payload, provenance, confidence,
  and resolution state without source-adapter access?
- Which conceptual seed forms are actually resolvable from current canonical
  APIs? Inventory exact missing, ambiguous, incompatible, duplicate, empty, and
  invalid cases and any unsupported qualified-name/UUID/source-position forms.
- Which intents, policy dimensions, edge directions/kinds, node filters,
  confidence or derived-fact rules, traversal bounds, candidate limits, cycle
  behavior, relevance inputs, comparison order, and tie-breakers require an ADR?
- Which deterministic budget units and cost estimators can be implemented and
  tested without a tokenizer or provider dependency? Inventory overhead,
  minimum item, exact boundary, overflow, omission, and truncation behavior.
- Which provenance, explanation, semantic-fragment, bundle, and rendering data
  can be produced exactly? What source text/range/content is unavailable and
  must remain deferred?
- Which repository-owned constructed or production-analysis graphs can form a
  reproducible public evaluation corpus with exact inclusion, exclusion,
  ordering, cost, explanation, rendering, and repetition oracles?
- Would a complete first slice require a production dependency, Cargo change,
  graph mutation/API change, Runtime/transport integration, new fixture, or
  external data?
- Which compatibility, platform, persistence, incremental, performance,
  security, provider, tool, MCP, IDE, and later-sprint concerns are unsupported?

## Evidence scope

- `crates/analysis/` public API, tests, history, dependencies, and consumers.
- `crates/graph/` query, semantic index, identity, node, edge, provenance,
  validation, report, diff, and impact boundaries relevant to read-only context.
- Current Workspace/Runtime snapshot ownership only as a compatibility input;
  do not plan a transport surface without evidence.
- Cargo manifests/lockfile/tree, current documentation, accepted ADRs, release
  review, Context Engine framework, CI platforms, and recent prompt suites.

## Evidence sources / fixtures

At minimum inspect:

- `crates/analysis/Cargo.toml`
- `crates/analysis/src/lib.rs`
- `crates/graph/src/lib.rs`
- `crates/graph/src/query.rs`
- `crates/graph/src/semantic_index.rs`
- `crates/graph/src/node.rs`
- `crates/graph/src/edge.rs`
- `crates/graph/src/provenance.rs`
- `crates/graph/src/impact.rs`
- relevant graph tests and existing production-analysis tests

Record exact provenance for every evaluation oracle. Do not make external
models/services, network access, arbitrary sleeps, ignored local corpora,
unapproved dependencies, or unavailable source text prerequisites.

## Excluded

ADR acceptance, Rust/Cargo/public API/fixture changes, Context Engine
implementation, graph mutation, Runtime routes, source adapters/parsers,
current-state documentation, prompt retirement, Roadmap transition, external
research, benchmarks, and unsupported quality/security/performance claims.

## Completion Criteria

- The document separates confirmed evidence, accepted constraints,
  compatibility-sensitive behavior, unsupported cases, unknowns, and decisions.
- It inventories exact ownership, APIs, consumers, dependencies, platforms,
  seed/resolution capabilities, policy and relevance inputs, budget/rendering
  feasibility, provenance limits, and non-zero deterministic oracles.
- It defines the minimum ADR matrix for authority, request, seeds, policy,
  errors, selection, ordering, budget, truncation, bundle, provenance,
  explanations, rendering, evaluation, compatibility, and deferred scope.
- It states whether implementation can remain on approved dependencies and
  which exact addition, if any, would require separate approval.
- Missing or conflicting evidence blocks Task 2 instead of being replaced by an
  invented source, scoring, token, model, provider, or transport contract.
- No production, manifest, fixture, Roadmap-state, current-state, or prompt-suite
  file is changed.

## Repository Safety

Create only `docs/architecture/context-engine-investigation.md`. Preserve
`.codex/`, production code, manifests, fixtures, prompts, Roadmap state, and
unrelated files. Stage only the investigation document when commit mode is
authorized.

## Task-specific Validation

- Verify every cited path, API, dependency, platform, fixture, test, consumer,
  and oracle from the live repository.
- `cargo test -p oneagent-analysis`
- Use non-mutating focused `--list` or graph tests only when needed; report zero
  matches separately.
- Validate links and `git diff --check`.
- `git status --short`

## Suggested commit message

`Investigate Sprint 22 Context Engine`

## Final report additions

Report confirmed authority/API/provenance boundaries, seed and budget findings,
rendering/evaluation oracles, dependency and platform impact, unresolved ADR
questions, decision readiness, changed path, validation, commit, and Git state.
