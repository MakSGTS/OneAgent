# Context Engine Investigation

## Status and scope

This document records the live repository evidence used to prepare ADR-0044
for Sprint 22 Context Engine. The investigation baseline is committed Sprint 22
planning head `7c58f3d559fc23db3539c7e7c1f2606cc51b910b`.

The investigation is read-only with respect to production behavior. It does not
select a request vocabulary, relevance policy, budget unit, renderer, public
Runtime surface, or implementation API. Those remain ADR-0044 decisions.

## Confirmed repository baseline

- The [v0.4 release review](../reviews/v0.4-release-review.md) records `pass` and
  makes Sprint 22 the unique `next` target.
- The [Context Engine Profile](../codex/profiles/context-engine-implementation.md),
  [Workflow](../codex/workflows/context-engine.md), and
  [Template](../codex/templates/context-engine-task.md) are committed.
- `oneagent-analysis` is a public library crate with local dependencies only on
  `oneagent-bsl`, `oneagent-common`, and `oneagent-graph`. It has no external
  dependency and no dependent production crate in the workspace.
- `cargo test -p oneagent-analysis` executes four unit tests and passes. The
  crate's existing production pipeline builds deterministic Module, Procedure,
  Function, `Contains`, and `Calls` facts with provenance from supplied BSL
  modules.
- Repository CI runs format, check, test, and Clippy on `macos-14` and
  `windows-latest`.
- No Context Engine request, seed, policy, candidate, budget, bundle,
  explanation, renderer, evaluation target, Runtime route, CLI command,
  protocol contract, Coverage capability, tokenizer, embedding, vector-search,
  or provider implementation exists.

## Ownership and dependency boundary

[Architecture](../Architecture.md) assigns source-independent derived analysis
to `oneagent-analysis` and canonical graph identity, facts, provenance,
validation, indexes, and queries to `oneagent-graph`. The Context Engine concept
in [Semantic Model 2.0](semantic-model-2.md) consumes the Knowledge Graph through
query interfaces and must not depend on EDT structures or parser internals.

The current dependency graph permits `oneagent-analysis` to consume graph and
common domain APIs without a cycle. No Runtime, adapter, protocol, filesystem,
serialization, or async dependency is needed for an in-memory first slice.
There is no current production consumer to migrate.

Runtime owns immutable `WorkspaceSnapshot` values. Each
`WorkspaceConfigurationSnapshot::graph()` returns a canonical
`&SemanticGraph`, so a later Runtime consumer can supply one selected graph to a
source-independent Context Engine without moving graph authority. The existing
Runtime `GraphQueryService` cannot be the Sprint 22 core input: its owned node,
relation, and traversal projections intentionally omit payload and provenance,
and its HTTP routes expose only the bounded ADR-0040 first slice. Adding a
Context route or aggregate multi-configuration behavior is a separate later
decision.

## Canonical graph and query evidence

`SemanticGraph` stores nodes in `BTreeMap<EntityId, GraphNode>` and edges in a
`BTreeSet<GraphEdge>`. `SemanticGraphQuery` is a borrowed read-only facade over
one graph snapshot. Its module contract explicitly forbids graph mutation,
source reads, rebuilds, semantic resolution, and diagnostic creation.

| Input or operation | Live behavior relevant to Context Engine |
|---|---|
| Node identity | `GraphNode::id()` returns a stable `EntityId`; `NodeId` is the public query wrapper. |
| Node content | Query returns `GraphNode`, exposing exact name, typed kind, typed payload, and provenance. |
| Edge identity | `SemanticGraphQuery::edge_id` centralizes stable identity from source, target, and kind. |
| Edge content | Query returns `GraphEdge`, exposing source, target, kind, and provenance. |
| Exact lookup | `node` and `node_by_entity_id` return one exact node or absence. |
| Name lookup | `nodes_by_name` is exact canonical-name matching and returns stable `NodeId` order; it can return zero, one, or several nodes. |
| Kind lookup | `nodes_by_kind` is stable and insertion-order independent. |
| Ownership | `owner`, `owners`, `owner_edges`, and `children` expose canonical `Contains` direction and invalid multiple-owner states. |
| Relations | Incoming/outgoing and per-kind queries return deterministic edge order. |
| Dependencies | Direct dependency/usage helpers use the closed Calls, References, Reads, Writes, DependsOn, and Opens policy. |
| Traversal | `traverse` performs deterministic breadth-first traversal in one direction with mandatory depth, edge filtering, optional start inclusion, cycle containment, deduplication, and first-discovery edge identity. |
| Missing inputs | Query methods return absence or empty results rather than typed Context errors. |
| Candidate bound | Graph traversal has a depth bound but no maximum-candidate bound or truncation result. |
| Node filtering | Traversal has no node-kind filter. |
| Confidence filtering | Traversal does not filter node or edge provenance confidence/resolution. |

The public graph `query` integration target lists 19 non-zero tests covering
lookup, kind/name order, containment, relations, dependencies/usages, cycles,
depth, upstream traversal, edge filters, index equivalence, and insertion-order
independence. These are compatibility evidence; they do not by themselves
define Context relevance or budget semantics.

## Seed feasibility

The conceptual Semantic Model lists node ID, qualified name, source position,
source file, metadata UUID, selected text, and editor symbol as possible seeds.
The live graph supports only a narrower evidence-backed set.

| Conceptual seed | Live canonical evidence | Decision status for ADR-0044 |
|---|---|---|
| Node identifier | Exact `NodeId`/`EntityId` lookup exists. Empty or whitespace-only entity identifiers are invalid in `oneagent-common`; arbitrary non-empty unknown IDs remain valid values with no graph match. | Implementable without another model prerequisite. ADR must define request validation and missing behavior. |
| Exact canonical name | Exact ordered lookup exists and naturally produces missing, unique, or ambiguous results. | Implementable if ADR defines ambiguity, optional kind qualification, duplicate inputs, and order. |
| Qualified name | `GraphNode` exposes only one canonical `EntityName`; no separate qualified-name field or query index exists. Some IDs encode scope, but their syntax is not a general qualified-name contract. | Unsupported without a new accepted model/API decision. |
| Metadata UUID | Stable UUIDs commonly participate in existing entity IDs, but no generic public UUID field/index exists on `GraphNode`. | Unsupported as a distinct seed in the first slice without new graph API evidence. |
| Source file | Provenance has an optional opaque `EntityId` source, but there is no generic source-to-node query. Source identifiers can include path fragments and producer context. | Not safely resolvable as a file seed from current APIs. |
| Source position | `oneagent-common` has no source span type and public graph provenance has no span. | Unsupported. |
| Selected text/editor symbol | No editor, document, position, selection, or workspace-text contract exists. | Deferred to IDE work. |

Duplicate seed normalization, mixed seed variants, empty seed lists, partial
resolution, ambiguity, incompatible kind constraints, and error precedence are
not decided by current code. ADR-0044 must close them before implementation.

## Provenance and source-content boundary

Every `GraphNode` and `GraphEdge` exposes a provenance vector. Each `Provenance`
contains optional opaque source identity, producer identity, fact origin,
confidence, and resolution state. This is sufficient to explain which canonical
fact and producer supported a selected node or relation.

Important limits are confirmed:

- provenance has no structured source range or raw source content;
- an opaque source ID can contain a path-like value plus encoded context, but
  consumers cannot treat that value as an authorized filesystem path;
- generic `GraphNode` and `GraphEdge` constructors retain provenance insertion
  order and do not sort or deduplicate it automatically;
- graph identity and query order are deterministic, while a Context projection
  that exposes multiple provenance records must define its own canonical
  comparison/deduplication tuple;
- the Runtime cache serializes graph provenance privately, but that persistence
  format is not a Context Engine API or source-text store.

The first slice can assemble semantic fragments from node/edge identity, name,
kind, selected relation/path, and canonicalized provenance metadata. It cannot
produce source snippets, line ranges, or claims about file contents. Missing
source content is a required explicit deferral rather than an empty or invented
fragment.

## Selection and relevance evidence

The live query traversal supplies deterministic discovery order but does not
define Context relevance. `SemanticImpactAnalyzer` provides a useful
compatibility pattern—typed options, strict depth, BTree-based state, typed
reasons, stable paths, and deterministic output—but its change-impact meaning is
not Context relevance and must not be reused as semantic authority.

Repository evidence supports deterministic implementation of the following
mechanisms after ADR acceptance:

- bounded incoming or outgoing graph traversal;
- exact allowed edge-kind and node-kind filters;
- optional use of accepted dependency or ownership relations;
- integer/enum/identity comparison keys derived from depth, relation kind, node
  kind, confidence/resolution metadata, and canonical IDs;
- stable tie-breaking and deduplication through ordered collections;
- explicit maximum-candidate truncation and chosen-path retention;
- cycle safety and insertion/seed-order equivalence.

No repository evidence selects an intent vocabulary, bidirectional traversal
composition, node/edge allowlist, confidence threshold, derived-fact rule,
priority table, score formula, path preference, maximum depth, or maximum
candidate count. Floating-point, learned, execution-reachability, source-
proximity, and data-dependency quality claims are unsupported. ADR-0044 must
choose a closed deterministic relevance comparison rather than describe a
future collection of possible factors.

## Budget and truncation evidence

The workspace has no tokenizer or LLM-provider capability. Adding a provider-
specific token estimator would pull Sprint 23 scope into Sprint 22 and would
require an external production dependency or an unverified local algorithm.

The standard library can provide reproducible checked costs based on exact
rendered UTF-8 bytes, Unicode scalar values, or another explicitly defined
semantic unit. Byte cost has direct `String::len()` behavior and simple exact
cross-platform fixtures; scalar-value cost has deterministic `chars().count()`
behavior but is not a model token count. Either remains only a Context budget
unit and must not be labeled provider tokens. A caller-supplied estimator is
technically possible but would add compatibility and trust questions not
required by current evidence.

ADR-0044 must decide:

- budget type, non-zero/minimum/maximum validation, and overflow containment;
- exact item representation before cost calculation;
- fixed/header/separator overhead and whether an empty bundle has cost;
- whether a seed item is mandatory and how an unaffordable seed fails;
- admission order, whole-item versus partial-item behavior, and whether an
  oversized item is omitted or deterministically truncated;
- used, remaining, omitted-count, and truncation reporting;
- whether candidate and budget truncation are distinct typed states.

Checked integer arithmetic, exact boundary fixtures, and stable admission order
provide a reliable oracle. Silent overflow, implicit model-token claims, and
post-render budget checking are unsafe.

## Bundle, explanation, and rendering evidence

The conceptual `ContextBundle` in Semantic Model 2.0 is design evidence, not a
live Rust API. Current graph types provide enough data for a first semantic-only
bundle containing canonical seed IDs, included node projections, selected edge
or path IDs, canonical provenance projections, costs, explanations, omissions,
and rendered text after ADR-0044 defines exact ownership and order.

`NodeKind` and `EdgeKind` are closed ordered enums but do not expose a stable
source-independent `as_str`/`Display` vocabulary. Runtime defines private owned
wire enums and string mappings for ADR-0040; copying that transport authority
into Context Engine would be a dependency inversion. ADR-0044 must decide
whether rendering defines an explicit Context-owned vocabulary or omits kind
strings from the first format. Rust `Debug` output is not a stable public
rendering contract.

Every explanation can refer to accepted seed identity, graph path/relation,
depth, relevance reason, cost, and canonical provenance. No current evidence
supports natural-language generation, summaries inferred from payloads, source
snippets, prompt templates, or model-ready provider token counts.

## Reproducible evaluation boundary

The smallest repository-owned evaluation boundary requires no new fixture or
dependency:

1. construct small canonical `SemanticGraph` snapshots with exact node/edge IDs,
   names, kinds, provenance, duplicates, cycles, and reordered insertions;
2. use `SemanticAnalysisPipeline` with fixed in-test BSL modules for a public
   production-analysis graph containing declared ownership and resolved Calls;
3. execute the public Context Engine library API from a non-zero integration
   target under `crates/analysis/tests/`;
4. assert exact accepted and rejected requests, candidate order/path, budget
   accounting, bundle contents, provenance, explanations, omissions,
   truncation, rendered bytes, and equality across reordered/fresh runs.

The evaluation matrix must include, where accepted by ADR-0044:

| Area | Required observable cases |
|---|---|
| Request and seeds | positive, empty, invalid, duplicate, missing, ambiguous, incompatible, reordered |
| Traversal and filtering | depth zero/boundary, incoming/outgoing, accepted/rejected kinds, empty neighborhood, cycle, duplicate paths |
| Relevance and bounds | different priorities, exact ties, deterministic tie-break, candidate boundary, over-bound omission |
| Budget | minimum, exact boundary, one-unit-short, multi-item admission, oversized item, checked overflow |
| Provenance/explanation | node and edge evidence, absent optional source, duplicate/reordered provenance, exact reason/cost |
| Rendering | empty and populated bundle, exact order/separators/vocabulary, Unicode if applicable, repeated equality |
| Compatibility | existing four analysis tests and affected graph query tests remain green |

This is a deterministic contract evaluation, not a relevance-quality benchmark.
No current labeled corpus or accepted metric supports precision, recall,
ranking-quality, latency, memory, or large-workspace performance claims.

## Compatibility and platform constraints

- The Context Engine must remain additive to `oneagent-analysis`; existing
  `SemanticAnalysisPipeline`, errors, tests, and crate dependencies remain valid.
- `SemanticGraph`, Query, Semantic Index, Validation, Diff, Impact, Coverage,
  Runtime Workspace/Graph Query, adapters, cache, HTTP, and CLI behavior must not
  change for the bounded library slice.
- Public result values should own what callers need after a request; borrowed
  request execution may consume only one immutable graph snapshot.
- Standard-library ordered collections and checked integer arithmetic are
  cross-platform. Test paths must not require Unix behavior, sockets, signals,
  clocks, or filesystem order.
- No production dependency is required by the evidenced slice. Any ADR choice
  that requires one must block Task 3 pending separate explicit approval.

## Unsupported and deferred scope

- qualified-name, UUID, source-file, source-position, selected-text, and editor
  seeds without separately accepted model/API prerequisites;
- source ranges, source text, arbitrary filesystem reads, parser/adaptor access,
  and adapter-specific provenance interpretation;
- graph mutation, persisted `ContextBundle` nodes/edges, cache integration,
  incremental context repair, aggregate multi-configuration selection;
- model tokenization, embeddings, vector search, floating or learned scoring,
  remote data, relevance-quality or performance claims;
- provider/model requests, prompt execution, streaming, conversation state,
  tools, authorization, secrets, retries, and cancellation;
- Runtime routes, HTTP, CLI, protocol activation, MCP, LSP, IDE, and UI behavior.

## ADR-0044 decision matrix

ADR-0044 must close every row before implementation begins.

| Decision area | Required decision |
|---|---|
| Authority and ownership | Owning crate/module, one-snapshot request lifetime, graph/query dependency, owned result boundary, no semantic mutation. |
| Request | Intent vocabulary, required fields, defaults, limits, validation precedence, equality and ordering. |
| Seeds | Accepted variants, exact resolution, kind constraints, duplicates, empty/mixed inputs, missing/ambiguous/incompatible outcomes. |
| Policy | Direction, edge/node allowlists, depth, candidate maximum, confidence/resolution/derived-fact behavior, cycle rules. |
| Relevance | Exact comparison keys and order, path selection, ties, deduplication, final candidate order, no unsupported quality meaning. |
| Budget | Unit/name, bounds, checked cost, overhead, mandatory items, admission, omission, truncation, accounting, candidate-versus-budget truncation. |
| Bundle | Public owned fields, item/path identities, order, equality, empty result, completeness/truncation state. |
| Provenance | Canonical projection, sort/dedup tuple, absent-source behavior, required evidence, no source-path authority. |
| Explanation | Typed reason vocabulary, seed/path/relevance/cost association, one or multiple reasons, deterministic order. |
| Rendering | Exact stable vocabulary, grammar, separators, escaping, Unicode/cost relationship, no `Debug` contract or fabricated text. |
| Errors | Closed typed errors, messages, validation/resolution/admission precedence, no partial result on rejected requests. |
| Evaluation | Public target, constructed and production-analysis cases, exact oracle, repetition/reordering, compatibility matrix. |
| Dependencies and consumers | No-new-dependency baseline, approval gate, additive API impact, future Runtime/MCP/IDE integration boundary. |
| First slice and deferrals | Exact included variants and explicit unsupported source/model/provider/transport/persistence/performance scope. |

## Decision readiness

The repository provides enough evidence to accept and test a bounded
source-independent Context Engine without external data or a new production
dependency. ADR-0044 is unblocked if it chooses only mechanisms supported by
the matrices above. It must block implementation if it requires source text,
model tokenization, a new graph identity/index, external relevance data, or an
unapproved dependency without first creating the corresponding prerequisite.
