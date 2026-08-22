# Implement Sprint 22 Context Request Boundary

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
- `docs/architecture/semantic-model-2.md`

## Prerequisites / Required gate

Require committed accepted ADR-0044, clean task-owned state, and any explicit
approval ADR-0044 requires before a production manifest change. Stop rather than
selecting a new Context Engine contract during implementation.

## Task

Implement only the accepted public Context Engine domain and request boundary in
`oneagent-analysis`: request, intent, seed, policy, budget, bundle/result domain
types needed by later tasks, deterministic validation, accepted seed resolution,
and typed failures. Keep candidate selection behind the Task 4 boundary.

## Canonical snapshot and data boundary

Consume one borrowed immutable `SemanticGraph` or its accepted query facade.
Do not mutate graph facts, retain adapter/parser state, load source files, or
introduce Runtime/model ownership.

## Request, seed, and policy contract

Implement only the closed variants, defaults, bounds, precedence, and failure
behavior accepted by ADR-0044. Invalid or unresolved requests must terminate
before candidate traversal or rendering.

## Selection, relevance, and ordering contract

Expose only the accepted typed inputs required by Task 4. Do not implement
candidate traversal, ranking, or admission in this task.

## Budget, cost, and truncation contract

Represent and validate the accepted budget boundary without calculating item
cost or admitting fragments.

## Provenance, explanation, and rendering contract

Define only accepted public result shapes required by later tasks. Do not emit
final explanations or rendered text.

## Evaluation corpus and oracle

Use focused constructed canonical graphs with exact IDs, names, kinds, and
provenance to prove every accepted and rejected request/seed path.

## Compatibility and consumer impact

Keep existing `SemanticAnalysisPipeline` and all graph/query APIs compatible.
Add no consumer integration or public Runtime surface.

## Scope

### Included

- Minimum accepted module/export structure under `crates/analysis/src/`.
- Public domain types and accessors, request/policy/budget validation, exact seed
  resolution, duplicate normalization or rejection, typed errors, stable error
  messages, and focused deterministic tests.

### Excluded

Candidate traversal/scoring, budget cost/admission, bundle population,
explanations, rendering, source fragments, graph/API mutation, Runtime/transport,
providers/models/tools, public evaluation target, docs, sprint transition, and
prompt retirement.

## Acceptance Criteria

- Public types cannot represent behavior outside the ADR-0044 first slice.
- Validation and seed resolution exactly implement accepted precedence and
  deterministic positive, empty, duplicate, invalid, missing, ambiguous,
  incompatible, minimum, maximum, and overflow behavior as applicable.
- Resolved seed identity remains canonical and input order cannot affect the
  accepted normalized result.
- Failures create no candidates, partial bundle, graph fact, source read, or
  background work.
- Existing analysis and graph APIs remain compatible and all focused tests are
  non-zero, deterministic, and cross-platform.
- No production dependency or unrelated behavior changes.

## Repository Safety

Modify only `crates/analysis/src/lib.rs`, create only the minimum accepted
Context Engine modules under `crates/analysis/src/`, and modify
`crates/analysis/Cargo.toml`/`Cargo.lock` only if ADR-0044 requires and separate
dependency approval is present. Preserve all other paths and stage task-owned
files only.

## Task-specific Validation

- Run exact non-zero request/validation/seed-resolution tests.
- `cargo test -p oneagent-analysis`
- Run the canonical complete workspace validation from
  `docs/codex/core/validation.md`.
- Verify diff scope and `git status --short`.

## Suggested commit message

`Implement Sprint 22 context request boundary`

## Final report additions

Report implemented domain/request boundary, snapshot authority, validation and
seed behavior, failures, compatibility, focused/full validation, dependency
impact, changed paths, commit, and final Git state.
