# Diagnostics Engine Investigation

## Status

Decision-ready repository investigation for Sprint 36.

The investigation was performed from committed planning baseline `cc890879` on
branch `codex/v0.7-sprint-36`. The v0.6 release review is non-blocking and Sprint
36 is the unique eligible sprint. No production behavior, public API, Cargo
dependency, fixture, protocol, or Coverage status changes in this task.

## Objective

Establish the current diagnostic producers, canonical evidence, identities,
ordering, summaries, snapshots, cache behavior, protocol projections,
compatibility constraints, and executable oracles needed for ADR-0058. Separate
confirmed repository facts from the architecture choices that remain open.

## Confirmed authorities and constraints

- Graph owns semantic facts, recoverable `SemanticDiagnostic` values,
  `SemanticGraphValidationIssue` values, provenance, source locations, graph
  reports, and build diffs.
- Recoverable semantic diagnostics describe source/build/resolution problems
  that do not necessarily make a graph build fail. Graph validation describes
  invalid or degraded already-built graph or build-result state and does not
  read source or rerun resolution.
- Workspace publishes one immutable `WorkspaceConfigurationSnapshot` containing
  the graph, ordered recoverable diagnostics, reference requests/statistics,
  and `SemanticGraphReport`.
- ADR-0051 exposes recoverable diagnostics separately from graph validation
  through the existing MCP `oneagent.diagnostics` and `oneagent.validation`
  tools. Both are read-only and Tool Policy gated.
- ADR-0054 exposes only located recoverable diagnostics through pull-only LSP
  `textDocument/diagnostic`. It deliberately does not expose graph validation
  issues, suppression, push/workspace diagnostics, or mutable-document state.
- The v0.6 release hand-off requires Sprint 36 to preserve Graph authority and
  define orchestration, rules, ordering, suppression, bounds, and reporting
  before implementation. Sprint 37 separately owns the general Rules Engine.

## Current diagnostic evidence families

### Recoverable semantic diagnostics

`crates/graph/src/diagnostic.rs` defines `SemanticDiagnostic` with:

- 17 stable `SemanticDiagnosticCode` values;
- 17 corresponding `SemanticDiagnosticKind` values;
- closed `Warning` and `Error` severities;
- message, semantic reference, optional source node, expected node kinds,
  optional actual kind, candidate node IDs, and provenance;
- total ordering over provenance, source node, code, kind, reference, expected
  kinds, actual kind, candidates, and message.

The current code families are:

| Family | Stable codes | Confirmed production source |
|---|---|---|
| Query language | malformed syntax, unsupported structure or persistent namespace, virtual table, temporary table, external/parameter source | EDT BSL/query parsing and resolution |
| Data Composition | nested data set, field folder, unsupported data-set type, unsupported field type | EDT report Data Composition emission |
| Semantic reference | malformed format, unsupported prefix, unresolved, ambiguous, incompatible kind, invalid owner | EDT reference-request resolution and projections |
| Edge request | duplicate semantic edge request | EDT graph-emission request handling |

EDT producers sort the final diagnostic collection and prove deterministic
repeated builds. Identical reference requests are canonicalized by the request
ledger and can produce one diagnostic; different source nodes targeting the
same missing entity remain distinct. Query, Data Composition, Event
Subscription, Form navigation, Writes, Grants, Includes, XDTO/service, and
generic reference tests exercise positive and negative diagnostic production.

`oneagent-analysis::AnalysisDiagnostics` is an earlier BSL-analysis result for
unresolved local and cross-module calls. It is not stored in Workspace and is
not a third canonical engine input: the EDT adapter projects accepted unresolved
call evidence into Graph-owned `SemanticDiagnostic` values before publication.
Parser-local diagnostic collections are likewise producer evidence, not
Workspace-level inputs.

Designer XML currently publishes an empty recoverable diagnostic collection.
This is an observed adapter limitation, not evidence that Designer input cannot
produce future diagnostics.

### Graph validation issues

`crates/graph/src/validation.rs` defines `SemanticGraphValidationIssue` with:

- 19 stable `SemanticGraphValidationCode` values;
- closed `Error` and `Warning` severities;
- four kinds: `Structural`, `Semantic`, `Provenance`, and `BuildConsistency`;
- node IDs, optional edge ID and kinds, optional reference-request ID,
  normalized provenance, invariant name, and message;
- total ordering over severity, code, kind, typed identities, invariant,
  provenance, and message, followed by exact deduplication.

The code families are:

| Family | Confirmed checks |
|---|---|
| Structure | missing edge source or target |
| Semantics | invalid endpoints/owner, multiple owners, forbidden self-loop, cycle |
| Provenance | missing node or edge provenance |
| Build consistency | inconsistent reference/diagnostic statistics or report; non-terminal/missing/incompatible request evidence; missing or unexpected edge projection; missing diagnostic projection |

`SemanticGraphValidator::validate` checks only the graph.
`validate_build_result*` additionally checks diagnostics, reports, reference
requests, and statistics. Recoverable diagnostics are not themselves validation
failures. Warning-only missing provenance leaves a graph consumable; an error
makes `SemanticGraphValidationResult::is_valid()` false.

Validation issues have no public free-form constructor. That prevents Runtime
or protocol code from manufacturing Graph validation evidence and is an
important ownership constraint for ADR-0058.

## Existing identity, ordering, and duplicate behavior

Three distinct contracts exist today:

| Area | Identity/order contract | Duplicate behavior |
|---|---|---|
| `SemanticDiagnostic::Ord` | Full observable typed content, including provenance and message | Equal values can coexist in a raw slice; producers normally sort and may deduplicate earlier requests |
| `DiagnosticDiff` | `DiagnosticIdentity` = code + kind + optional source node + semantic reference | `diagnostic_index` maps one value per identity; severity, message, kinds, candidates, and provenance become modified aspects |
| Graph validation result | Full typed issue order including invariant and message | Exact equal issues are sorted and deduplicated by `SemanticGraphValidationResult` |

The existing build-diff identity is reusable evidence but not automatically a
complete cross-family engine identity. It has no family discriminator and no
validation-issue form. Multiple recoverable diagnostics with the same diff
identity but different observable content collapse to one `BTreeMap` entry in a
diff. ADR-0058 must decide whether to preserve this behavior, strengthen the
identity for reporting, or distinguish diff identity from report identity.

No stable serialized engine-result identifier, suppression identifier, or
cross-family collision policy exists.

## Existing summaries and reports

`DiagnosticSummary` counts raw recoverable diagnostics by code, severity, kind,
and provenance presence. `recoverable` currently equals `total`.
`SemanticGraphValidationSummary` separately counts validation issues by code,
kind, severity, and provenance issue count. `SemanticGraphReport` includes the
recoverable summary but does not include the validation result.

There is no unified active/suppressed/omitted report, no cross-family severity
summary, no explanation of suppression, and no engine-level bounded-result
status. Existing graph and build reports must remain canonical for their
current meanings unless ADR-0058 explicitly defines a compatible migration.

## Source locations and sensitive evidence

Recoverable diagnostics carry optional provenance, but current LSP projection
does not derive a range from diagnostic provenance. It resolves
`diagnostic.source_node()` in the same Configuration, then requires exactly one
distinct typed source-node location with a span confined below both Workspace
and Configuration roots.

Confirmed cases are:

- no source node: raw/MCP evidence remains available, LSP omits it;
- missing source node in the graph: LSP omits it;
- missing, span-less, ambiguous, conflicting, or escaping source-node location:
  LSP omits it rather than guessing;
- valid but numerically unrepresentable LSP position: request fails internally;
- exact confined span: LSP returns the source-node declaration range, which is
  not claimed as the exact offending token range.

MCP excludes root paths and provenance paths. It returns code, severity, kind,
message, optional source node ID, and candidate IDs. LSP returns only range,
severity, code, source `oneagent`, and message. ADR-0058 must preserve these
sensitive-data and confinement boundaries.

## Workspace, rebuild, and cache ownership

`WorkspaceConfigurationSnapshot` stores recoverable diagnostics as
`Arc<[SemanticDiagnostic]>`. `snapshot_from_parts` creates the immutable value
after graph construction and Configuration identity validation. File watching
replaces the whole snapshot generation atomically.

ADR-0042 cache DTOs serialize every raw diagnostic field, including provenance
and typed source locations. Cache decode:

1. reconstructs graph and diagnostics;
2. requires diagnostics to be in non-decreasing canonical order;
3. reconstructs requests/statistics and recomputes `SemanticGraphReport`;
4. reruns complete build validation;
5. rejects invalid semantic content; and
6. re-encodes to prove canonical normalization.

Equal adjacent diagnostics are not rejected by the cache ordering check. No
Diagnostics Engine result is currently serialized. The evidence supports two
implementation candidates without a new dependency: recompute the immutable
engine report from restored canonical inputs, or version and serialize it with
an equality check. ADR-0058 must choose; silent cache divergence is not safe.

## Current protocol projections and bounds

### MCP

`oneagent.diagnostics` accepts exact fields `configurationId` and optional
`limit`. The shared default is 50 and maximum is 100. Runtime takes the first
ordered raw diagnostics, returns `total` and `truncated`, and relies on the
Tool Policy 65,536-byte output bound for a final fail-closed guard.

`oneagent.validation` independently runs `configuration.graph().validate()` and
returns graph-only issues. It does not run complete build-result validation.
Any unified reporting migration must keep the seven-tool catalog, modern and
legacy MCP projections, schema truth, Tool Policy execution, deterministic
ordering, and error separation consistent.

### LSP

LSP pull diagnostics use one immutable startup snapshot and a complete-result
limit of 100. Runtime collects all located candidates before encoding and fails
the request when the count is over 100; it never truncates. A valid confined
document with no located result returns a full empty report. No result ID,
related information, tags, data, workspace diagnostics, push diagnostics,
refresh, or suppression configuration is advertised.

The different MCP and LSP bound semantics are intentional accepted contracts.
ADR-0058 must decide whether engine-level bounds are independent from adapter
bounds and how each adapter proves completeness.

## Consumer and compatibility inventory

| Consumer | Current dependency on diagnostic behavior |
|---|---|
| EDT adapter and tests | Produces sorted recoverable diagnostics, reference statistics, reports, build validation, and diffs |
| Designer XML adapter | Publishes an empty recoverable diagnostic collection and valid graph report |
| Graph report/diff/validator | Uses raw diagnostic counts, diff identity, and request-to-diagnostic consistency |
| Workspace service and watcher | Publishes and atomically replaces raw diagnostics with the graph snapshot |
| Persistent cache | Serializes raw values, reconstructs reports, reruns build validation, and proves canonical bytes |
| MCP Runtime and clients | Exposes separate diagnostics and validation tools through every supported MCP revision and Tool Policy |
| LSP Runtime and clients | Exposes located recoverable diagnostics only through pull document diagnostics |
| HTTP and CLI | Do not currently expose a diagnostic endpoint; they consume the same Workspace lifecycle and must remain compatible |
| VS Code extension | Uses MCP graph/symbol/context tools but has no diagnostics UI or LSP migration |
| EDT extension | Runs discovery-only MCP compatibility probe and owns no semantic diagnostics |
| Codex/Cursor compatibility | Depends on the immutable seven-tool catalog and version-correct tool result envelopes |

## Suppression evidence

No repository-owned diagnostic suppression configuration, directive, identity
list, persistence schema, protocol field, UI, or producer flag exists. Matches
for “suppress” in product code refer to protocol notification responses or stale
UI work, not semantic diagnostic suppression.

ADR-0058 therefore must explicitly choose one bounded first-slice policy:

1. no suppression support, with an observable zero-suppressed contract;
2. a fixed source-independent policy justified entirely by existing diagnostic
   fields and recorded with explicit suppressed outcomes; or
3. an in-memory exact-identity suppression input with no discovery,
   persistence, pattern language, or rule execution.

Adding configurable registration, discovery, scripts, third-party rules, or
rule-produced diagnostics would cross the Sprint 37 boundary and is not
supported by current evidence.

## Dependency and ownership candidates

### Graph-owned orchestration

Advantages: both canonical input families and their stable vocabularies already
live in `oneagent-graph`; no dependency changes are needed. Risk: reporting
policy can become inseparable from Graph fact and validation authority.

### Analysis-owned orchestration

`oneagent-analysis` already depends on Graph and owns the source-independent
Context Engine precedent; Runtime already depends on Analysis. This separates
reporting policy from Graph producers without a new dependency. Risks are the
existing `AnalysisDiagnostics` name collision and the need to keep validation
execution and semantic vocabularies Graph-owned.

### Runtime-owned orchestration

Runtime already sees Workspace inputs, but this would make the engine difficult
to reuse and could mix domain policy with cache/protocol composition. It is a
weak candidate unless ADR-0058 limits Runtime to construction only.

### New crate

No current dependency cycle or reusable consumer requires a new production
crate. A new crate would add manifest, API, documentation, and review cost
without evidence of necessity.

## Decision-ready first-slice boundary

Repository evidence is sufficient for a first slice that:

- consumes immutable Graph-owned recoverable diagnostics and graph/build
  validation evidence;
- creates a typed source-independent result with a family-discriminated stable
  identity, deterministic order, explicit active/suppressed disposition, and
  aggregate report;
- records every omitted or bounded outcome instead of silently discarding it;
- is constructed once with each immutable Workspace Configuration snapshot and
  reconstructed equivalently after cache load or rebuild;
- migrates only the existing MCP diagnostics result and LSP located projection
  as accepted by ADR-0058; and
- preserves raw diagnostics, graph validation APIs, the seven-tool catalog,
  Tool Policy, LSP capability truth, and existing producer semantics.

The exact owner, identity fields, suppression policy, bounds, error model,
snapshot field, cache strategy, and protocol shapes remain architecture
decisions rather than investigation conclusions.

## Deterministic acceptance matrix for ADR-0058

| Area | Required cases and oracle |
|---|---|
| Domain | empty; each family/severity/category; valid identity; invalid identity; exact/over string and collection bounds; error redaction |
| Identity | same input repeated; reordered input; exact duplicate; same identity with modified content; cross-family code collision; distinct source nodes |
| Orchestration | semantic-only; validation-only; mixed; missing optional location; suppressed/active; exact/over result bounds; deterministic summaries |
| Producer preservation | query, Data Composition, unresolved/ambiguous/incompatible reference, duplicate request, graph schema, provenance, and build-consistency regressions |
| Workspace | EDT, Designer, mixed root, empty root, later adapter failure, duplicate Configuration, repeated fresh build, watcher replacement and shutdown |
| Cache | cold/write, warm hit, equal raw and engine evidence, version/source invalidation, corruption and write-failure recovery, canonical re-encode |
| MCP | schema/list/catalog, positive/empty/mixed, filters if accepted, exact/over limits, truncation status, Tool Policy denial, modern/legacy envelopes, public process repetition |
| LSP | located/empty, unlocated omission, missing source node, conflicting/escaping/span-less location, exact/over 100, URI confinement, lifecycle, channel purity, repetition |
| Compatibility | Graph report/diff/validation, Workspace/cache, HTTP/CLI, VS Code, EDT probe, Codex/Cursor tool catalog, adapters, Coverage state |

## Baseline validation evidence

The following commands completed successfully at `cc890879`:

```text
cargo test -p oneagent-graph diagnostic
```

Six Graph unit tests and one request-aware integration test matched and passed.
Several unrelated Graph integration binaries reported zero matches; those
zero-match binaries are not counted as evidence.

```text
cargo test -p oneagent-graph --test validation
```

All 55 validation integration tests passed.

```text
cargo test -p oneagent-runtime --test workspace_service
cargo test -p oneagent-runtime --test mcp_semantic_tools
cargo test -p oneagent-runtime --test lsp_process public_lsp_process_pulls_located_diagnostics_and_empty_full_reports
```

The commands passed 6 Workspace service tests, 6 MCP semantic-tool tests, and
the one selected public LSP diagnostic-process test respectively. No required
row was skipped.

## ADR-0058 questions

ADR-0058 must decide:

1. the exact canonical input families and whether engine construction receives
   a validation result or invokes a Graph validator;
2. the owning crate and dependency direction;
3. family-discriminated stable identity and its relationship to existing
   `DiagnosticIdentity`;
4. normalized severity, category, origin, location, message, semantic ID,
   provenance, and explanation fields;
5. duplicate, same-identity/different-content, and cross-family collision rules;
6. the supported first-slice suppression policy and observable suppressed
   evidence;
7. engine-level string/count/result bounds and fail/truncate/report behavior;
8. deterministic ordering and summary arithmetic;
9. construction failure, partial-result, and redaction policy;
10. immutable Workspace field and raw-diagnostic compatibility;
11. cache serialization versus deterministic recomputation and schema impact;
12. exact MCP schema/result migration across all supported revisions;
13. exact LSP active/located projection and interaction with the 100-result
    complete bound;
14. API migration for Graph, Workspace, Runtime, tests, and downstream clients;
15. complete evidence and documentation gates; and
16. explicit deferral of the Rules Engine, new producers, UI, mutable documents,
    fixes, telemetry, remote transport, and broad performance/security claims.

## Evidence gaps and blockers

There is no blocking data gap for architecture or implementation planning. The
absence of a suppression source and unified identity is confirmed evidence that
ADR-0058 must select a bounded contract; it is not permission to invent a
configuration format. No current evidence justifies a new dependency, new
crate, diagnostics UI, or Sprint 37 rule-registry work.

## Recommended next action

Accept ADR-0058 before changing production code. The ADR should select the
smallest source-independent owner and first slice that preserves raw Graph
evidence, makes suppression and bounds explicit, supports immutable Workspace
composition, and keeps MCP/LSP projections truthful without expanding the
Rules Engine or IDE scope.
