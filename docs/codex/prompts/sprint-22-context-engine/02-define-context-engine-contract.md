# Define Sprint 22 Context Engine Contract

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 22 execution plan
- `docs/architecture/context-engine-investigation.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/v0.4-release-review.md`

## Prerequisites / Required gate

Require committed Task 1 evidence that every first-slice decision has a
repository-owned canonical input and deterministic oracle. Stop if the
investigation reports missing or conflicting evidence.

## Task

Create and accept `docs/adr/0044-context-engine.md`, defining the smallest
complete deterministic source-independent Context Engine contract. Synchronize
only planning-level architecture text required to make the decision unambiguous.
Implement no Rust.

## Scope

### Included

- Canonical immutable snapshot authority, crate ownership, dependency direction,
  request lifetime, and relationship to graph/query/index/Runtime boundaries.
- Closed first-slice intent, seed, policy, default, bound, validation, resolution,
  failure, and deterministic precedence contracts.
- Allowed node/edge inputs, direction, depth, confidence/derived-fact behavior,
  candidate limit, cycle handling, relevance inputs, comparison order,
  tie-breaking, deduplication, and stable output order.
- Budget unit, cost estimator, overhead, admissibility, exact-boundary and
  overflow behavior, deterministic admission, omissions, and truncation.
- Bundle/item identity and content, canonical provenance, explanations,
  semantic-fragment assembly, rendering format/order, and absent source-content
  behavior.
- Repository-owned evaluation corpus, exact oracle, repeatability, compatibility,
  dependency choice, public test strategy, first slice, rejected alternatives,
  and deferred scope.

### Excluded

Rust/Cargo/fixture changes, graph mutation or new semantic kinds, source parsing
or extraction, Runtime/HTTP/CLI/protocol surfaces, embeddings/vector search,
floating or learned scoring, LLM/provider/model behavior, conversation state,
tool policy, MCP/IDE, persistence/incremental repair, new production dependencies
without approval, sprint completion, and prompt retirement.

## Acceptance Criteria

- ADR-0044 answers every Task 1 decision question with one canonical contract
  grounded in live evidence and accepted architecture.
- Context selection consumes one immutable canonical graph snapshot through
  accepted read-only APIs and does not create semantic facts or adapter authority.
- Request/seed/policy/error vocabularies, validation and resolution precedence,
  traversal/filtering, bounds, relevance comparison, ties, and ordering are
  closed and observable.
- Budget unit, exact cost, overhead, admission, omission, and truncation behavior
  are deterministic and cannot silently exceed the accepted bound.
- Every included item has exact canonical provenance and one deterministic
  explanation; unavailable source text/ranges are explicitly not fabricated.
- Rendering and public evaluation have exact repository-owned oracles covering
  boundary, negative, reordered, cyclic, and repeated cases.
- Dependency choice is explicit. If a new production dependency is required,
  Task 3 remains gated on separate explicit user approval.
- Rejected alternatives, compatibility, first slice, implementation order,
  Coverage impact, Sprint 23 hand-off, and later deferrals are explicit.
  Sprint 22 remains `next`; current-state docs do not claim implementation.

## Repository Safety

Create only `docs/adr/0044-context-engine.md` and modify only the minimum
planning-level architecture document if required. Preserve `.codex/`, Rust,
manifests, lockfile, fixtures, prompts, Roadmap state, current implementation
claims, and unrelated files. Stage only ADR-owned paths when authorized.

## Task-specific Validation

- Verify decision/evidence consistency with Task 1 and cited public contracts.
- Validate internal links, ADR status, closed authority/request/seed/policy/
  error/selection/budget/bundle/provenance/rendering/evaluation matrices,
  alternatives, prerequisites, accepted/deferred scope, and `git diff --check`.
- `git status --short`

## Suggested commit message

`Define Sprint 22 Context Engine contract`

## Final report additions

Report accepted authority, request/seed/policy, selection, relevance, budget,
bundle, provenance, explanation, rendering, evaluation, compatibility,
dependency, first-slice, and deferred decisions; changed paths; validation;
commit; Git state; and whether Task 3 is unblocked.
