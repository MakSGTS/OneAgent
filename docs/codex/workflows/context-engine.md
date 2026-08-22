# Context Engine Workflow

Use this workflow for deterministic context selection and assembly over a
canonical source-independent semantic graph or accepted read-only derived view.

## Canonical authority and data boundary

- Treat graph facts, identities, provenance, validation, and accepted derived
  query behavior as authoritative inputs rather than mutable Context Engine
  state.
- Keep source-adapter structures, parser internals, Runtime transport state,
  model-provider state, and editor-specific state outside the core selection
  boundary.
- Define the exact immutable snapshot observed by one request and how callers
  detect unavailable, stale, or incompatible inputs.
- Do not infer new semantic facts while selecting or rendering context.

## Request and seed resolution

- Define every accepted intent, seed variant, policy field, default, bound, and
  validation error as a closed contract.
- Resolve seeds through accepted graph or query interfaces with deterministic
  missing, ambiguous, incompatible, duplicate, and invalid-input behavior.
- Preserve canonical identities through candidate selection and output.

## Selection and relevance

- Define allowed node and edge kinds, direction, depth, confidence, candidate
  bounds, and whether accepted derived facts participate.
- Define the relevance inputs, comparison order, tie-breakers, deduplication,
  and final output ordering without relying on hash or insertion order.
- Keep heuristic or measured relevance distinct from semantic truth and label
  it in explanations and evaluation evidence.

## Budgeting, truncation, and assembly

- Define the budget unit, cost estimator, reserved overhead, minimum admissible
  item, and overflow behavior before implementation.
- Apply one deterministic admission policy and report used, remaining, and
  omitted capacity without silently exceeding the accepted budget.
- Make every truncation explicit, stable, and attributable to a bound or budget.
- Define fragment identity, ordering, overlap handling, deduplication, and the
  relationship between selected graph facts, source fragments, summaries, and
  rendered output.

## Provenance and explanations

- Retain enough canonical provenance to trace each included item to its graph
  fact or accepted source evidence.
- Emit one deterministic explanation per included item, including the seed or
  path, relevance reason, cost, and applicable truncation state.
- Reject or explicitly classify candidates that lack provenance required by the
  accepted contract; do not invent source locations or text.

## Reproducible evaluation

- Use repository-owned fixtures or constructed graphs with stated provenance,
  fixed requests, exact expected inclusions, exclusions, order, costs,
  explanations, and truncation outcomes.
- Cover positive, empty, invalid, missing, ambiguous, incompatible, duplicate,
  reordered, boundary-budget, over-budget, and repeated-build cases as
  applicable.
- Prove equivalent requests over equal snapshots produce equal bundles and
  rendered output regardless of insertion order.
- Treat relevance-quality claims as unsupported unless a checked-in evaluation
  corpus, oracle, command, and acceptance threshold make them reproducible.

## Boundaries

Do not pull model-provider calls, prompt execution, embeddings, remote data,
tool execution, MCP, IDE integration, graph mutation, source parsing, or
performance claims into a Context Engine task unless accepted architecture and
the task scope explicitly include them.
