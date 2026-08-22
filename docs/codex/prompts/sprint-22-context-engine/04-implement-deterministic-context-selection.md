# Implement Sprint 22 Deterministic Context Selection

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
- committed Task 3 request boundary

## Prerequisites / Required gate

Require committed Task 3 with exact request validation and seed resolution,
successful focused/full validation, and clean task-owned state. Stop rather than
reselecting relevance or traversal architecture.

## Task

Implement only ADR-0044 deterministic candidate discovery and ordering over the
canonical immutable graph query: accepted traversal and filters, bounds,
relevance comparison, stable ties, deduplication, and provenance paths. Do not
admit or render budgeted fragments.

## Canonical snapshot and data boundary

Observe exactly the Task 3 borrowed snapshot/query for one request. Selection is
a read-only derived view and cannot add, remove, normalize, or reinterpret graph
facts.

## Request, seed, and policy contract

Consume only validated/resolved Task 3 values. Preserve accepted per-intent and
per-policy behavior without introducing new defaults or variants.

## Selection, relevance, and ordering contract

Implement the exact ADR-0044 node/edge allowlists, direction, depth, confidence
or derived-fact behavior, candidate maximum, cycles, relevance keys, comparison
order, path choice, ties, deduplication, and final candidate order.

## Budget, cost, and truncation contract

Enforce only candidate-count and traversal bounds owned by selection. Item cost,
budget admission, omission, and rendered truncation remain Task 5.

## Provenance, explanation, and rendering contract

Retain canonical node/edge provenance and accepted seed/path/relevance evidence
needed for later explanations. Do not render final text.

## Evaluation corpus and oracle

Use exact constructed graphs covering every accepted edge/direction/depth and
relevance relation, duplicate/cyclic/reordered inputs, limits, and repetition.

## Compatibility and consumer impact

Preserve Task 3 public request behavior, `SemanticAnalysisPipeline`, graph/query
ordering, and existing consumers. Add no Runtime or adapter dependency.

## Scope

### Included

- Candidate model internals/public projections accepted by ADR-0044.
- Deterministic traversal, filtering, candidate bounds, relevance comparison,
  stable tie-breaking, deduplication, chosen path/provenance retention, and
  focused tests.

### Excluded

Cost estimation, budget admission, bundle rendering, source text, providers,
models, tools, Runtime/transport, graph mutation, new semantic kinds, public
evaluation/docs, sprint transition, and prompt retirement.

## Acceptance Criteria

- Only accepted graph facts and policy dimensions can produce candidates.
- Direction, depth, edge/node filters, confidence/derived policy, cycle handling,
  candidate bounds, deduplication, relevance comparison, ties, and order match
  ADR-0044 exactly.
- Equivalent graphs/requests produce equal candidate identities, order, paths,
  provenance, and relevance reasons regardless of insertion or seed order.
- Missing endpoints, duplicate paths, cycles, empty neighborhoods, exact limits,
  and over-limit candidates remain deterministic and bounded.
- No candidate mutates the graph or claims source/model evidence not present in
  canonical inputs.
- Focused tests are non-zero and existing analysis/graph behavior remains green.

## Repository Safety

Modify only Task 3 Context Engine files under `crates/analysis/src/` and the
minimum focused tests. Preserve manifests unless separately approved, graph,
Runtime, adapters, docs, prompts, `.codex/`, and unrelated paths. Stage only
task-owned files.

## Task-specific Validation

- Run non-zero traversal/filter/relevance/tie/bound/deduplication/provenance/
  reorder/repetition tests.
- `cargo test -p oneagent-analysis`
- Run affected `oneagent-graph` query tests if observable query behavior could
  be affected.
- Run the canonical complete workspace validation.
- Verify diff scope and `git status --short`.

## Suggested commit message

`Implement Sprint 22 deterministic context selection`

## Final report additions

Report selection policy, relevance and ties, bounds/cycles/deduplication,
provenance paths, determinism evidence, compatibility, focused/full validation,
changed paths, commit, and final Git state.
