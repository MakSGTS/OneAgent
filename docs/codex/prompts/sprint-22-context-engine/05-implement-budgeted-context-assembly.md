# Implement Sprint 22 Budgeted Context Assembly

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
- committed Tasks 3-4 Context Engine boundary and selection

## Prerequisites / Required gate

Require committed Task 4, all selection validation successful, and clean
task-owned state. Stop rather than changing accepted cost, budget, explanation,
or rendering architecture.

## Task

Implement only ADR-0044 deterministic candidate cost, budget admission,
omission/truncation, provenance-backed bundle assembly, per-item explanations,
and stable semantic rendering from the ordered Task 4 candidates.

## Canonical snapshot and data boundary

Assemble only canonical graph/query facts and provenance retained by Task 4.
Do not read filesystem/source-adapter state or fabricate unavailable source text.

## Request, seed, and policy contract

Preserve Task 3 validated values and Task 4 order. Assembly cannot silently
reinterpret request intent, seed identity, or selection policy.

## Selection, relevance, and ordering contract

Treat Task 4 candidate order and relevance evidence as fixed. Budget behavior
may omit candidates only through the ADR-0044 admission contract.

## Budget, cost, and truncation contract

Implement exact budget unit, deterministic item and overhead costs, minimum and
boundary behavior, overflow containment, admission order, used/remaining
accounting, explicit omissions, and any accepted truncation state.

## Provenance, explanation, and rendering contract

Every included item must retain canonical provenance and one deterministic
explanation with accepted seed/path/relevance/cost data. Rendering must be exact,
stable, semantic-only, and consistent with bundle order and accounting.

## Evaluation corpus and oracle

Use exact graphs/candidates and expected strings for empty, minimum, exact,
over-budget, partial-admission, duplicate, provenance, explanation, omission,
ordering, Unicode/encoding if applicable, and repeated cases.

## Compatibility and consumer impact

Preserve Task 3-4 public behavior, canonical graph/query facts, and current
analysis APIs. Add no provider, Runtime, protocol, or source dependency.

## Scope

### Included

- Accepted cost estimator and checked accounting.
- Deterministic admission, omission/truncation summaries, bundle population,
  canonical provenance, explanations, semantic fragments, exact rendering, and
  focused tests.

### Excluded

Source file/range extraction, raw source text, tokenizer/provider/model calls,
embeddings, vector search, prompt templates, conversation state, Runtime/HTTP/
CLI/MCP/IDE integration, persistence/cache, graph mutation, architecture
reselection, current-state docs, sprint transition, and prompt retirement.

## Acceptance Criteria

- Total accepted cost never exceeds the validated budget and arithmetic cannot
  overflow silently.
- Exact-boundary, insufficient, empty, overhead-only, multi-item, omitted, and
  any accepted truncation cases match ADR-0044 and report explicit accounting.
- Admission preserves deterministic candidate order and cannot depend on hash,
  insertion, provenance, or seed order beyond accepted comparison keys.
- Every included item has canonical identity/provenance and one exact
  explanation; every omitted item/count is observable as accepted.
- Rendered output exactly corresponds to the bundle and remains equal across
  equivalent reordered graphs and repeated requests.
- No unavailable source text, model/provider result, quality score, or graph fact
  is invented.
- Focused tests are non-zero and existing analysis/graph behavior remains green.

## Repository Safety

Modify only Context Engine files under `crates/analysis/src/` and minimum focused
tests. Preserve manifests unless separately approved, graph, Runtime, adapters,
docs, prompts, `.codex/`, and unrelated paths. Stage task-owned files only.

## Task-specific Validation

- Run non-zero cost/admission/boundary/overflow/omission/provenance/explanation/
  rendering/reorder/repetition tests.
- `cargo test -p oneagent-analysis`
- Run the canonical complete workspace validation.
- Verify diff scope and `git status --short`.

## Suggested commit message

`Implement Sprint 22 budgeted context assembly`

## Final report additions

Report cost and budget contract, admission/truncation, bundle/provenance/
explanations, rendering, deterministic evidence, compatibility, focused/full
validation, changed paths, commit, and final Git state.
