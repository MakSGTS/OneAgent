# Complete Sprint 22 Context Engine Evidence

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/context-engine-implementation.md`

## Template

`docs/codex/templates/context-engine-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 22 execution plan
- `docs/architecture/context-engine-investigation.md`
- `docs/adr/0044-context-engine.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- committed Tasks 3-5 implementation

## Prerequisites / Required gate

Require committed Task 5, all focused implementation validation successful, and
clean task-owned state. Stop rather than repairing architecture or production
defects inside an evidence-only task.

## Task

Add the complete public repository-owned Context Engine evaluation evidence
required by ADR-0044 and synchronize truthful current-state documentation. Do
not mark Sprint 22 completed.

## Canonical snapshot and data boundary

Exercise the public `oneagent-analysis` Context Engine over canonical
provenance-backed graph snapshots and the existing production analysis boundary.
Do not use adapter internals, ignored corpora, source filesystem reads, or
external model/service data as an oracle.

## Request, seed, and policy contract

Cover every accepted public variant, default, bound, validation failure,
resolution outcome, and deterministic precedence.

## Selection, relevance, and ordering contract

Cover accepted traversal/filter/direction/depth, relevance keys, ties, cycles,
deduplication, candidate bounds, input/graph reordering, and repeated equality.

## Budget, cost, and truncation contract

Cover exact cost/accounting, minimum, boundary, over-budget, partial admission,
omission/truncation, and overflow containment as applicable.

## Provenance, explanation, and rendering contract

Assert exact bundle contents, canonical provenance, explanation reasons/costs,
omissions, stable semantic fragments, and rendered output. Assert unavailable
source text is not fabricated.

## Evaluation corpus and oracle

Create the minimum public integration target under `crates/analysis/tests/`
using checked-in Rust test graphs/modules with exact expected outcomes. Record
why each case is representative and ensure the target executes non-zero tests.

## Compatibility and consumer impact

Prove existing analysis pipeline and affected graph/query behavior remain green.
Synchronize implemented truth and explicit deferred scope without exposing a
Runtime, provider, or protocol surface.

## Scope

### Included

- Public evaluation for positive, empty, invalid, missing, ambiguous,
  incompatible, duplicate, reordered, cyclic, candidate-bound, exact-budget,
  over-budget, provenance, explanation, omission, rendering, and repeated cases
  as applicable to ADR-0044.
- `README.md`, `docs/Architecture.md`, and
  `docs/architecture/semantic-model-2.md` current-state synchronization.
- Dependency/API/graph/analysis regression evidence and full workspace checks.

### Excluded

Production fixes, architecture reselection, new dependencies, source/adapters,
Runtime routes, CLI/protocol, providers/models/embeddings, tools, MCP/IDE,
performance/quality claims, Coverage transitions without an existing capability,
Sprint 23 work, sprint completion, and prompt retirement.

## Acceptance Criteria

- A non-zero public evaluation target proves every accepted request, selection,
  budget, provenance, explanation, omission, and rendering contract through the
  public library boundary.
- Exact expected bundles and strings act as reproducible oracles; no test relies
  on hash order, arbitrary sleeps, network, external services, or ignored data.
- Equivalent reordered graphs/requests and fresh repeated evaluations are equal.
- Existing SemanticAnalysisPipeline and affected graph/query tests remain green;
  canonical semantic facts and public APIs outside the accepted additive slice
  stay compatible.
- Current-state docs agree on the implemented first slice and explicitly defer
  source text, providers/models, Runtime/transport, persistence, MCP, and IDE.
- Sprint 22 remains incomplete pending Task 7.

## Repository Safety

Create only the minimum public test files under `crates/analysis/tests/`; modify
only `crates/analysis/Cargo.toml` for repository-local dev dependencies proven
necessary, `Cargo.lock` only if mechanically required, and the three current-
state docs listed above. Preserve production code, graph/adapters/Runtime,
prompts, Roadmap, `.codex/`, and unrelated paths. Stage task-owned paths only.

## Task-specific Validation

- List and run the exact non-zero public Context Engine evaluation target.
- `cargo test -p oneagent-analysis`
- Run affected graph query tests.
- Run the canonical complete workspace validation.
- Validate doc links/claims, diff scope, and `git status --short`.

## Suggested commit message

`Complete Sprint 22 Context Engine evidence`

## Final report additions

Report the public evaluation matrix, canonical inputs, exact oracles, request/
selection/budget/rendering outcomes, provenance/explanations, repetition,
preserved contracts, current-state docs, deferred scope, focused/full
validation, changed paths, commit, and final Git state.
