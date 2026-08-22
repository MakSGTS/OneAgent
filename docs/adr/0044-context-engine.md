# ADR-0044: Deterministic Semantic Context Engine

## Status

Accepted

## Context

Sprint 22 must build deterministic semantic context selection and assembly
before any LLM provider, tool, MCP, or IDE integration exists. The canonical
repository authority is `SemanticGraph`; accepted queries and indexes are
read-only views over one immutable graph snapshot.

The [Context Engine investigation](../architecture/context-engine-investigation.md)
confirms that `oneagent-analysis` already owns source-independent derived
analysis and depends only on local BSL, common, and graph crates. The graph query
surface exposes stable node/edge identities, exact names and kinds, typed
payloads, incoming/outgoing relations, ownership, dependencies, bounded
traversal, and node/edge provenance. No production crate currently consumes
`oneagent-analysis`.

The repository has no generic qualified-name, metadata-UUID, source-file, or
source-position query, no public source span/content store, no tokenizer,
embedding/vector index, Context API, renderer, model provider, Runtime context
route, or Context Coverage capability. Runtime Graph Query projections omit
payload and provenance by design. A safe first slice must therefore remain an
in-memory source-independent library capability over one supplied graph.

## Decision

### Canonical authority and ownership

`oneagent-analysis` owns the Context Engine implementation and public Context
domain types. `oneagent-graph` remains the sole authority for graph facts,
identity, payload, provenance, validation, indexes, and query behavior.

One synchronous request borrows exactly one immutable `SemanticGraph` snapshot,
builds a temporary read-only query view, and returns an owned result. The engine
does not retain the graph, mutate facts, resolve new semantics, read source
files, or interpret adapter-specific provenance. It owns no task, thread,
listener, cache, global state, clock, random source, Runtime observer, or model.

The public entry point is an additive stateless Context Engine value that accepts
`&SemanticGraph` and `&ContextRequest` and returns
`Result<ContextBundle, ContextError>`. Exact Rust names may follow nearby style,
but this ownership and result shape are normative.

### Request and intent

The first slice has one closed intent, `Explain`. The value is retained in the
request and bundle so future additive intents do not change the meaning of an
existing request. `Explain` means select and render bounded graph evidence around
the requested seeds; it does not generate natural language or infer new facts.

A request contains:

- one `ContextIntent`;
- one to sixteen raw `ContextSeed` values;
- one `ContextBudget` measured in rendered UTF-8 bytes;
- one `ContextPolicy`.

Requests and all public result values are owned, cloneable, equality-comparable,
and free of borrowed graph references. No serialization contract is accepted.

### Seed vocabulary and resolution

The accepted seed variants are:

1. exact node identifier plus an optional expected `NodeKind`;
2. exact canonical `EntityName` plus an optional required `NodeKind`.

Node identifiers are validated through the non-empty `EntityId` contract before
lookup. Exact-name lookup is case-sensitive and performs no trimming,
localization, aliasing, normalization, qualified-name parsing, or fuzzy match.

All raw seeds are converted to a canonical sortable descriptor and processed in
that order, so failure selection is independent from caller order. Resolution is
all-or-nothing:

- absent exact ID or exact name is `MissingSeed`;
- an ID whose node kind differs from its expected kind is `IncompatibleSeed`;
- an exact name with a required kind is filtered to that kind; zero filtered
  matches with at least one unfiltered match is `IncompatibleSeed`;
- more than one compatible exact-name match is `AmbiguousSeed` with candidates
  in stable node-ID order;
- identical resolved node IDs are deduplicated;
- a resolved seed excluded by the policy node-kind set is `IncompatibleSeed`.

No partial result or candidate survives a seed failure. After deduplication,
seed IDs are in stable node-ID order. Their count must not exceed the policy
candidate maximum.

Qualified-name, metadata-UUID, source-file, source-position, selected-text, and
editor-symbol seeds are deferred.

### Policy vocabulary and bounds

`ContextPolicy` has exactly these fields:

- `direction`: `Outgoing`, `Incoming`, or `Both`;
- a non-empty ordered set of allowed `EdgeKind` values;
- an optional non-empty ordered set of allowed `NodeKind` values;
- `max_depth` in `0..=4`;
- `max_candidates` in `1..=128`.

The default policy uses `Both`, all current graph edge kinds, no node-kind
restriction, `max_depth = 2`, and `max_candidates = 32`. A node-kind restriction
applies to seeds and discovered candidates. A disallowed node is neither
selected nor traversed through. Edge filtering applies before visiting the
related node.

The current edge vocabulary is closed at the accepted graph variants. Future
edge or node kinds require an explicit mapping before Context rendering can
accept them; a wildcard string or `Debug` fallback is forbidden.

Validation precedence is:

1. budget bounds;
2. policy depth, candidate maximum, non-empty edge set, and non-empty optional
   node-kind set;
3. raw seed count;
4. canonical per-seed syntax;
5. canonical seed resolution;
6. policy compatibility and unique-seed versus candidate-limit compatibility;
7. candidate selection;
8. mandatory-seed budget admission.

The first failing category terminates the request with no partial bundle.

### Deterministic candidate discovery

Every unique resolved seed is a candidate at depth zero. Seeds are mandatory
and ordered by node ID.

Traversal follows accepted graph edges to `max_depth`:

- `Outgoing` follows edge source to target;
- `Incoming` follows edge target to source;
- `Both` considers outgoing before incoming only as the accepted direction
  tie-breaker; it does not materialize reverse edges.

Traversal is cycle-safe. A candidate node can be discovered by several seeds or
paths, but only its best reason is retained. A path step contains direction,
canonical edge kind, stable edge ID, and canonicalized edge provenance. Path
comparison is lexicographic by:

1. path length;
2. per-step edge priority;
3. per-step direction (`Outgoing` before `Incoming`);
4. per-step stable edge ID;
5. seed node ID.

The explicit edge priority from highest to lowest is:

1. `Contains`;
2. `Calls`;
3. `References`;
4. `Reads`;
5. `Writes`;
6. `DependsOn`;
7. `Opens`;
8. `Triggers`;
9. `Includes`;
10. `Extends`;
11. `Grants`.

This ordinal is a deterministic first-slice relevance policy, not a probability,
confidence score, or quality claim. Depth always precedes edge priority. Node
kind does not add another hidden priority.

Final candidates are ordered as mandatory seeds by node ID, followed by related
candidates using the same best-path comparison and stable candidate node ID as
the final tie-breaker. Candidates beyond `max_candidates` are omitted as one
explicit candidate-limit truncation count. The limit includes seeds. Selection
computes the exact eligible distinct count within the depth bound before
truncation, so the omission count is observable.

Graph/query insertion order, seed order, duplicate edges, duplicate paths,
provenance insertion order, and hash iteration cannot affect the result.

### Provenance projection and explanations

Each admitted Context item retains:

- canonical node ID, exact canonical name, and closed node kind;
- canonicalized node provenance;
- depth and selected seed ID;
- the selected path with edge IDs, directions, kinds, and canonicalized edge
  provenance;
- one typed inclusion reason: `Seed` or `Related`;
- exact rendered fragment and byte cost.

Provenance records are sorted and deduplicated by this tuple:

```text
optional source identity
producer identity
fact origin
confidence
resolution state
```

Absent source sorts before present source. Existing enum order is accepted for
origin, confidence, and resolution only inside this canonical projection. The
original graph vectors are not modified.

A `Seed` explanation names the seed node and depth zero. A `Related`
explanation names the selected seed, exact depth, and complete selected path.
The cost is stored on the Context item. Natural-language explanations,
alternative paths, source locations, raw source text, inferred payload
summaries, and model-generated rationale are deferred.

### Stable kind vocabulary

Semantic rendering uses a Context-owned closed machine vocabulary. Metadata
nodes render as `metadata.<MetadataKind::as_str()>`. Other node kinds render as:

```text
module
procedure
function
query
data_composition_schema
data_set
data_composition_field
xdto_type
http_service_url_template
http_service_method
web_service_operation
web_service_parameter
form
command
attribute
standard_attribute
tabular_section
dimension
resource
measure
role
access_right
subsystem
unknown
```

Edge kinds render as:

```text
contains
calls
references
reads
writes
grants
includes
extends
depends_on
opens
triggers
```

Directions render as `outgoing` and `incoming`. These mappings are exhaustive;
`Debug` output is not a compatibility surface.

### Fragment grammar and rendering

Every candidate is converted to one complete UTF-8 semantic fragment before
budget admission. A fragment is exactly two newline-terminated lines.

The node line is:

```text
node kind=<kind> id=<byte-length>:<id> name=<byte-length>:<name>\n
```

For a seed, the reason line is:

```text
reason seed=<byte-length>:<seed-id> depth=0\n
```

For a related item, the reason line is:

```text
reason seed=<byte-length>:<seed-id> depth=<depth> path=<step-count>:<steps>\n
```

Each step is:

```text
<direction>,<edge-kind>,<edge-id-byte-length>:<edge-id>
```

Steps are separated by `;` with no trailing separator. Decimal integers have no
leading sign or leading zero except the value zero. Lengths and costs count UTF-8
bytes, not characters or model tokens. Length prefixes make spaces, tabs,
newlines, colons, semicolons, commas, and non-ASCII data unambiguous without an
escape convention.

The bundle's rendered text is the byte-for-byte concatenation of admitted
fragments in item order. There is no global header, footer, or separator beyond
fragment content. Provenance is retained structurally in the bundle and is not
duplicated in the first rendered format.

### Budget, admission, and truncation

`ContextBudget` is an integer count of rendered UTF-8 bytes in `1..=65_536`.
It is deliberately not called a token budget. No tokenizer or caller-supplied
cost estimator exists in the first slice.

One item's cost is the checked UTF-8 byte length of its complete rendered
fragment. There is no uncounted overhead. The sum of every mandatory seed
fragment is checked first. If it exceeds the request budget, the request fails
with `InsufficientBudget { required, available }` and returns no partial bundle.

After all seeds are admitted, related candidates are considered in deterministic
relevance order. Admission is a whole-fragment prefix: admit the next fragment
only when checked addition remains within budget. At the first non-fitting
fragment, omit it and every lower-ranked remaining candidate. Partial fragments,
per-item truncation, reordering to fill gaps, and budget overflow are forbidden.

The bundle reports:

- requested byte budget;
- used bytes equal to rendered text byte length;
- remaining bytes;
- candidate-limit truncation flag and exact omitted count;
- budget truncation flag and exact omitted count.

Candidate-limit truncation is applied before budget admission; its omitted
count is distinct from budget omissions. The first slice therefore uses
explicit whole-item omission rather than textual truncation.

### Errors

The public typed error vocabulary is closed:

- invalid budget;
- invalid policy;
- invalid seed count;
- invalid seed identifier;
- missing seed;
- ambiguous seed with stable candidate IDs;
- incompatible seed with expected/actual or policy context;
- too many unique seeds for the candidate limit;
- insufficient budget for mandatory seeds;
- checked cost or accounting overflow;
- unsupported kind mapping only as an internal invariant failure if a future
  graph variant reaches code before its mapping is accepted.

Errors have stable English `Display` messages and expose typed fields through
accessors where callers need identity or bounds. No error contains a path read
from the filesystem, adapter parser state, or partial Context bundle.

### Reproducible evaluation

Focused unit tests live with the Context Engine modules. A separate non-zero
public integration target under `crates/analysis/tests/` exercises only exported
APIs.

The evaluation corpus combines:

- constructed canonical graphs with exact node/edge IDs, names, kinds,
  provenance, alternative paths, cycles, duplicate/reordered inputs, and budget
  boundaries;
- fixed `AnalysisModule` inputs passed through the existing public
  `SemanticAnalysisPipeline` to prove compatibility with production analysis
  facts and provenance.

The exact oracle covers:

- node-ID and exact-name seeds, unique, duplicate, missing, ambiguous, and
  incompatible resolution;
- validation precedence and no partial result;
- all directions, depth zero and maximum, filters, empty neighborhoods, cycles,
  alternative paths, edge priority, direction ties, seed ties, candidate limit,
  and reordered equality;
- absent and present provenance, provenance sort/dedup, selected edge paths, and
  typed reasons;
- ASCII and non-ASCII byte lengths, exact budget, one-byte-short mandatory seed,
  related prefix admission, explicit candidate/budget omissions, and checked
  arithmetic;
- exact fragment and bundle rendering and equality across fresh repeated runs;
- preservation of existing analysis and affected graph query tests.

This is contract evaluation. Sprint 22 makes no precision, recall, ranking-
quality, model-token, latency, memory, large-workspace, security, or performance
claim and adds no benchmark threshold.

### Dependencies, compatibility, and Coverage

No production dependency, Cargo manifest change, graph API change, public
Runtime/HTTP/CLI/protocol change, source fixture, or external data is required.
If implementation evidence contradicts that conclusion, work stops before a
manifest change and requests separate explicit dependency approval or a new
architecture prerequisite.

Existing `SemanticAnalysisPipeline`, graph facts and consumers, Semantic Index,
Query, Validation, Diff, Impact, Coverage, Workspace snapshots, cache, Runtime
Graph Query, HTTP, CLI, EDT, and Designer XML behavior remain unchanged.

Graph and adapter Coverage registries classify source-independent graph and
producer capabilities. The Context Engine is a read-only analysis consumer, so
Sprint 22 adds no graph/EDT/Designer Coverage capability or status transition.
Its completion evidence is ADR-0044 plus the focused/public Context evaluation
and integration review.

## Rejected alternatives

### Store `ContextBundle` as a graph node

Rejected because a request result is a derived, budget- and policy-specific
view. Persisting it would mutate the canonical graph and turn transient Context
state into competing semantic authority.

### Own Context selection in Runtime or the HTTP Graph Query projection

Rejected because Runtime is orchestration and its ADR-0040 projections omit
payload/provenance. The core must remain reusable before any transport exists.

### Read source files from provenance identifiers

Rejected because provenance source is opaque identity, not filesystem authority,
and no structured source-range/content contract exists.

### Use model tokens or add a tokenizer dependency

Rejected because no provider contract or tokenizer evidence exists before
Sprint 23. UTF-8 byte budgets are exact and reproducible without mislabeling.

### Use floating-point, learned, or Impact scores

Rejected because no labeled relevance corpus or accepted metric exists. Impact
has change-propagation semantics and is not Context relevance authority.

### Accept every conceptual seed form

Rejected because qualified names, UUIDs, source locations, selected text, and
editor symbols lack current canonical query contracts.

### Render `Debug` values or copy Runtime wire vocabularies

Rejected because `Debug` is not stable and Runtime owns a transport-specific
projection. Context rendering needs an exhaustive local semantic vocabulary.

### Skip a large item and admit lower-ranked smaller items

Rejected because it weakens relevance-prefix semantics and makes budget effects
less explainable. Whole-fragment prefix admission is deterministic and explicit.

## Consequences

- Sprint 22 can implement and test a complete source-independent Context Engine
  without a new dependency or external service.
- The first result is useful as deterministic semantic graph context and a
  future input to providers, MCP, and IDE adapters, but it is not yet exposed by
  them.
- Exact-name seeds provide repository-backed missing and ambiguity behavior;
  node-ID seeds provide canonical direct selection.
- Relevance is a transparent lexicographic policy with typed paths, not a hidden
  score or quality claim.
- UTF-8 byte budgeting is exact and provider-independent but is not a model token
  estimate.
- Bundle provenance remains structural while rendering is a stable compact
  semantic view.
- Source-text context requires a separate accepted source range/content and
  authorization boundary.

## Implementation prerequisites and order

1. Implement public request, policy, budget, result domain values, validation,
   and all-or-nothing seed resolution.
2. Implement deterministic bounded candidate discovery, best-path selection,
   relevance ordering, deduplication, provenance projection, and candidate-limit
   accounting.
3. Implement exhaustive kind vocabularies, exact fragments, byte costs,
   mandatory-seed and related-prefix admission, bundle accounting,
   explanations, and rendering.
4. Add the public reproducible evaluation target and synchronize truthful
   current-state documentation.
5. Complete an independent integration review and full workspace gate.

## Deferred scope

- additional intents and seed variants;
- qualified-name, metadata-UUID, source-file, source-position, selected-text, and
  editor-symbol resolution;
- source spans/content, raw snippets, payload summaries, natural-language
  generation, prompt templates, and arbitrary filesystem access;
- alternative/complete-path explanations, configurable edge priorities,
  confidence thresholds, derived-fact switches, floating/learned scores,
  embeddings, vector search, model tokenizers, and relevance-quality metrics;
- multi-configuration aggregation, Runtime/HTTP/CLI/protocol surfaces,
  persistence, cache integration, and incremental Context repair;
- LLM providers, model execution, streaming, conversations, tool policy,
  secrets, retries, cancellation, MCP, LSP, IDE, and UI behavior;
- benchmarks and performance/security claims.

## Completion criteria

Sprint 22 is complete only when the accepted implementation order is committed,
all focused and public evaluation oracles pass, the canonical full workspace
validation succeeds, current-state documentation preserves every deferral, and
the Sprint 22 integration review records `pass` or
`pass with non-blocking follow-ups`.
