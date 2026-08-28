# ADR-0058: Diagnostics Engine

## Status

Accepted.

## Context

Sprint 36 follows the completed v0.6 MCP and IDE release. The release already
has two Graph-owned diagnostic evidence families: ordered recoverable
`SemanticDiagnostic` values produced during source and reference processing,
and deterministic `SemanticGraphValidationIssue` values produced by Graph
validation over an already-built graph or complete build result.

Workspace preserves raw recoverable diagnostics, reference evidence, and graph
reports in one immutable Configuration snapshot. The persistent cache encodes
the raw evidence and reconstructs and validates it. MCP exposes recoverable
diagnostics and graph-only validation through separate tools. LSP pull
diagnostics expose only recoverable diagnostics with one confined source span.

The existing contracts have no unified identity, disposition, suppression,
summary, or report. Recoverable diagnostic ordering includes all observable
content, while build diff uses a narrower identity. Validation issues have a
separate identity and exact deduplication rule. MCP truncates to a requested
prefix and reports `truncated`; LSP fails a complete request over 100 items.
There is no repository-owned suppression configuration or Rules Engine.

The evidence is recorded in the
[Diagnostics Engine investigation](../architecture/diagnostics-engine-investigation.md).
The dedicated reusable task contract is recorded by the Diagnostics Engine
[profile](../codex/profiles/diagnostics-engine-implementation.md),
[workflow](../codex/workflows/diagnostics-engine.md), and
[template](../codex/templates/diagnostics-engine-task.md).

## Decision

OneAgent adds a source-independent Diagnostics Engine owned by
`oneagent-analysis`. Graph remains the sole authority for semantic facts,
recoverable diagnostic vocabularies and values, validation vocabularies and
execution, provenance, source locations, graph reports, and build diffs.

The engine normalizes immutable Graph-owned evidence into one complete bounded
`DiagnosticReport`. It does not parse source, resolve references, validate
graph semantics independently, invent a diagnostic, mutate a graph, or register
rules.

## Ownership and dependency direction

The accepted direction is:

```text
oneagent-graph
    -> oneagent-analysis::diagnostics
        -> oneagent-runtime Workspace composition
            -> MCP and LSP projections
```

`oneagent-analysis` already depends on Graph and owns the source-independent
Context Engine precedent. Runtime already depends on Analysis. No Cargo
manifest or production dependency change is required.

The module is named `oneagent_analysis::diagnostics` to remain distinct from
the existing producer-side `AnalysisDiagnostics` BSL result. That existing type
is not renamed in Sprint 36.

## Canonical inputs

The first slice accepts exactly:

1. an immutable slice of Graph-owned `SemanticDiagnostic` values; and
2. an immutable `SemanticGraphValidationResult` produced by Graph validation.

The engine does not invoke a validator. The caller supplies the exact
validation result appropriate to its boundary. Workspace supplies complete
build-result validation over graph, raw diagnostics, the canonical
reference-request ledger, legacy reference statistics, and
`SemanticGraphReport`.

Producer-side BSL or parser diagnostics are not direct inputs. They remain
adapter evidence until projected into `SemanticDiagnostic`. Runtime, transport,
provider, and UI errors are not engine inputs.

## Family, severity, and category

Every finding has one closed `DiagnosticFamily`: `Semantic` or `Validation`.

Every finding has one closed normalized `DiagnosticSeverity`: `Warning` or
`Error`. Both input families map their existing severity exactly; the engine
never upgrades or downgrades it.

Every finding has one closed `DiagnosticCategory`:

- `Source` for query-language and Data Composition recoverable diagnostics;
- `Semantic` for semantic-reference and duplicate-edge recoverable diagnostics
  and Graph semantic validation issues;
- `Structural` for Graph structural validation issues;
- `Provenance` for Graph provenance validation issues; and
- `BuildConsistency` for Graph build-consistency validation issues.

The original typed code and kind remain available. Category is a reporting
projection and does not replace either Graph vocabulary.

## Stable identity

`DiagnosticIdentity` is a closed tagged typed value with one variant per family.
The family tag prevents cross-family collisions.

Semantic identity contains exactly code, kind, optional source node, and
`SemanticReference`. This matches the fields of the existing build-diff
identity while adding the family tag. Severity, message, expected kinds, actual
kind, candidates, and provenance are observable content, not identity.

Validation identity contains exactly code, issue kind, canonical node IDs,
optional edge ID, optional reference-request ID, optional edge/source/target
kinds, and invariant string. Severity, message, and provenance are observable
content, not identity.

Identity is a public typed Rust value with total ordering. Sprint 36 defines no
hash, UUID, editable string grammar, persistence key, or protocol-generated
opaque ID.

## Finding content and anchors

`DiagnosticFinding` contains identity, family, severity, category, original
stable code and kind projections, bounded existing message, disposition,
canonical node anchors, optional edge and reference-request anchors, original
typed evidence needed by in-process consumers, and normalized provenance count.

Semantic findings use the optional source node as their primary node anchor and
retain candidates as related nodes. Validation findings use their canonical
node list. The engine never derives a location. Workspace and adapters resolve
a node anchor through canonical graph provenance and accepted confinement.

## Duplicate and collision policy

Inputs are normalized independently of input order.

- Exact equal evidence with the same identity is one finding.
- Same identity and different observable content is a deterministic
  `ConflictingEvidence` error; the engine never picks one by order.
- Different identities remain different even when code or message is equal.
- Semantic and validation identities cannot collide because family is tagged.

This is intentionally stricter than the current build-diff `BTreeMap` collapse.
The build-diff API does not change in Sprint 36.

## Ordering

The complete report uses total order:

1. disposition (`Active` before `Suppressed`);
2. severity (`Error` before `Warning`, independent of source enum order);
3. category;
4. family;
5. typed identity; and
6. observable content as a defensive tie-breaker.

Input order, producer iteration order, cache load, and repeated execution cannot
affect the result.

## Suppression

The first slice supports only an exact in-memory identity suppression set in
`DiagnosticPolicy`. A finding is `Suppressed` only when its complete identity is
present; otherwise it is `Active`.

The report retains suppressed findings after active findings and counts them
separately. Suppression does not delete Graph evidence, alter Graph reports,
change severity, or satisfy a missing invariant.

The default policy contains no suppressions. Workspace uses only the default in
Sprint 36, so production snapshots normally report zero suppressed findings.
This establishes semantics without adding configuration, persistence, pattern
matching, code-wide ignores, directives, rule registration, or UI.

## Bounds and errors

The engine is complete and never truncates. Accepted hard limits are:

| Item | Limit |
|---|---:|
| Input semantic diagnostics | 65,536 |
| Input validation issues | 65,536 |
| Total normalized findings | 65,536 |
| Exact suppression identities | 4,096 |
| Message bytes per finding | 4,096 |
| Node anchors per finding | 256 |
| Provenance records observed per finding | 256 |

Existing typed codes, kinds, IDs, references, edge IDs, and request IDs remain
validated by their source types. The engine checks messages and collections
before cloning them into the report.

Closed `DiagnosticError` kinds are `TooManySemanticDiagnostics`,
`TooManyValidationIssues`, `TooManyFindings`, `TooManySuppressions`,
`MessageTooLarge`, `TooManyNodeAnchors`, `TooManyProvenanceRecords`, and
`ConflictingEvidence`.

Errors expose only kind and bounded counts. They never echo message, reference,
path, node, candidate, provenance, rejected identity, or internal error chain.
No partial report is returned.

The 65,536 limit is an internal immutable-snapshot bound, not a public protocol
result limit. Protocol adapters retain stricter accepted bounds.

## Summary, report, and filters

`DiagnosticSummary` records total, active, and suppressed findings; counts by
family, severity, and category; and active/suppressed counts by stable code
string. Arithmetic is checked, active plus suppressed equals total, and every
distribution reconciles with its applicable total.

`DiagnosticReport` owns the ordered complete findings and summary. It provides
read-only deterministic filtering by family, severity, category, and
disposition without rerunning orchestration. Filtering preserves report order
and does not reinterpret a finding.

Existing Graph `DiagnosticSummary`, `SemanticGraphReport`, validation summary,
and build diff retain their meanings and public APIs.

## Workspace composition

`WorkspaceConfigurationSnapshot` retains `diagnostics()` and adds read-only
accessors for the complete Graph validation result used by the engine and the
complete `DiagnosticReport`.

Workspace constructs both before publication from the graph, raw diagnostics,
request ledger, total statistics split into request-represented and legacy
portions, and existing report. A validation error follows the existing atomic
semantic-build failure policy; no invalid or mismatched snapshot is published.
Validation warnings remain reportable findings.

The engine uses the default no-suppression policy. No task, lazy validation,
request-time source read, global state, watcher, lock, channel, or lifecycle is
added. Watch rebuilds construct the complete report before replacing the
immutable generation.

## Persistent cache

Sprint 36 does not change cache schema or serialize validation/report derived
values. Cache continues to store canonical graph, raw diagnostics, requests,
statistics, and provenance.

After decode, Runtime reconstructs complete Graph validation and the engine
report from restored inputs. Cache acceptance retains the same valid
build-result invariant. Cold, warm, rebuilt, and repeated snapshots must expose
equal validation and engine reports. Persisting a report later requires a new
cache schema decision.

## MCP reporting contract

The catalog remains exactly seven lexicographically ordered tools and
`oneagent.diagnostics` remains read-only and Tool Policy gated.

Its input schema accepts required `configurationId`; optional unique non-empty
`families` (`semantic|validation`); optional unique non-empty `severities`
(`error|warning`); optional unique non-empty `categories`
(`source|semantic|structural|provenance|build_consistency`); optional
`includeSuppressed` boolean defaulting to `false`; and optional `limit` with
default 50 and maximum 100. Unknown, duplicate, empty, or invalid values are
`invalid_arguments`.

The result retains `configurationId`, `diagnostics`, `total`, and `truncated`
and adds `summary`. `diagnostics` is the ordered matching set limited by the
adapter. `total` is the complete matching count before the limit and
`truncated` compares total with returned length. `summary` is the complete
unfiltered report summary, never reconstructed from a filtered or truncated
prefix.

Each item contains `family`, `code`, `severity`, `category`, `kind`, `message`,
and `disposition`; compatibility `sourceNodeId` and `candidateNodeIds`; and
canonical `nodeIds`, optional `edgeId`, and optional `referenceRequestId` for
validation evidence. Semantic items preserve their current code, severity,
kind, message, source node, and candidates.

No item contains a root or source path, source content, raw reference,
provenance, producer, hash, rejected value, or internal error. The result
evolution is accepted because output schemas remain unadvertised under
ADR-0051. Modern and legacy revisions use the same tool payload through their
existing version-specific MCP envelopes. Other tools, ordering, policy,
lifecycle, framing, and error precedence do not change.

## LSP reporting contract

LSP retains pull-only `textDocument/diagnostic`, full reports, no result ID, no
document synchronization, and the complete-result limit of 100.

Runtime consumes only `Active` findings. A finding is projectable when it has
exactly one primary node anchor in the same Configuration, that node exists,
and it has one exact confined typed span for the requested URI. Semantic
findings use their source node. Validation findings are projectable only when
their node list contains exactly one node. Related/candidate nodes are not
alternate locations.

Missing, multiple, conflicting, span-less, escaping, or incompatible location
evidence is omitted rather than guessed. Code, severity, source `oneagent`, and
message retain their current wire meanings. Category, family, disposition,
identity, candidates, node lists, edges, requests, provenance, and paths are not
added to LSP output.

Runtime computes the complete located active set first. More than 100 remains
`RequestFailed`; no prefix is presented as complete. URI validation, UTF-16
positions, lifecycle, framing, channel purity, and sensitive-data rules remain.

## Public API and compatibility impact

- Graph public types and behavior remain unchanged.
- Analysis gains additive diagnostic domain and engine APIs.
- Workspace Configuration snapshots gain additive validation/report accessors;
  raw diagnostics and graph report accessors remain.
- Cache bytes and schema remain unchanged; derived evidence is recomputed.
- MCP keeps the tool name, catalog, semantic fields, policy, and envelopes while
  adding filters, normalized fields, summary, and validation findings.
- LSP capability and payload shape remain unchanged while its source becomes
  active located engine findings.
- HTTP, CLI, VS Code lifecycle, EDT probe, adapters, graph query, Context,
  Impact, and Coverage behavior remain unchanged.

Exact Codex/Cursor binaries need not be rerun unless repository-owned evidence
finds an incompatibility. The seven-tool catalog and supported protocol
lifecycle must be revalidated.

## Sensitive-data policy

The engine retains typed in-memory evidence needed for deterministic reporting,
but generic errors and protocol projections never expose source roots, source
content, raw references, opaque provenance, hashes, credentials, rejected
inputs, or internal chains. Only LSP may expose a file URI after existing
Workspace/Configuration confinement. MCP exposes no path.

## Implementation sequence

Task 3 implements typed identity, finding, policy, disposition, summary, report,
limits, errors, ordering, and domain tests without orchestration.

Task 4 implements deterministic normalization, exact suppression,
duplicate/conflict handling, checked summaries, filters, and mixed-input tests.

Task 5 constructs complete build validation and the default-policy report in
Workspace, recomputes both after cache load, and proves cold/warm, rebuild,
watching, failure, lifecycle, and repeated equality.

Task 6 migrates MCP and LSP projections and proves schemas, Tool Policy,
versions, confinement, exact/over bounds, public processes, channel purity,
EOF, shutdown, and repetition.

Task 7 completes compatibility, audit, cross-platform, and current-state
evidence without changing the accepted behavior.

## Required evidence

Completion requires non-zero tests for every family, severity, category,
disposition, limit, and error; exact duplicate, conflict, cross-family
collision, reorder, and repetition; default and exact suppression; empty,
single-family, and mixed reports; summary/filter stability; Workspace cold/warm,
rebuild, corruption/write recovery, and repetition; MCP schemas, filters,
policy, versions, and public process; LSP located/empty/unlocated/validation,
exact/over 100, confinement, lifecycle, and public process; plus affected Graph,
adapter, report, diff, validation, cache, HTTP, CLI, VS Code, EDT, catalog, and
Coverage regressions.

Production Rust and public API changes require the complete workspace gate.

## Coverage impact

No Semantic Coverage Registry capability changes in Sprint 36. The engine
orchestrates existing evidence and adds no graph node, edge, parser, producer,
or supported semantic capability. Architecture acceptance alone changes no
Coverage status or count.

## Rejected alternatives

- Graph-owned reporting policy is rejected because Graph already owns facts and
  validation; Analysis provides the existing source-independent layer.
- Runtime-owned domain is rejected because Runtime owns composition and
  transports, not reusable diagnostic policy.
- A new crate is rejected because no dependency cycle or consumer requires it.
- Reconstructing the report from MCP/LSP is rejected because adapters are
  differently bounded lossy projections.
- Full observable content as identity is rejected because changed severity,
  message, candidates, or provenance should modify one problem.
- Silent same-identity selection is rejected; conflicting evidence fails.
- Engine truncation is rejected; adapters retain explicit independent bounds.
- Code/pattern/file/configuration/rule suppression is rejected for the first
  slice because no such authority exists.
- Persisting derived values in the current cache schema is rejected; canonical
  recomputation is sufficient.
- Replacing raw diagnostics, Graph reports, validation, or build diff is
  rejected because their established meanings remain required.

## Deferred scope

Deferred beyond Sprint 36 are rule registration, discovery, dependencies,
configuration, execution, plugins, scripts, and rule-produced findings;
persisted/user/project suppression, directives, patterns, severity overrides,
baselines, expiration, and UI; new parser/adapter/graph/validation producers;
exact offending-token ranges without evidence; push/workspace/related/tagged
diagnostics, result IDs, refresh, mutable documents, diagnostics UI, code
actions, fixes, and edits; HTTP/CLI diagnostic endpoints, remote transport,
authentication, telemetry, analytics, publication, stable API guarantees,
benchmarks, and broad performance/security claims.

## Consequences

OneAgent gains one deterministic bounded report over current Graph diagnostic
evidence while preserving canonical producers and existing graph reports.
Workspace can publish and cache-reconstruct equal derived evidence. MCP can
query both families with explicit filters and truncation, and LSP can project
only active located findings without changing capability truth.

Production uses no suppressions in Sprint 36. Exact-identity suppression and
typed diagnostic identity establish a boundary that the future Rules Engine or
configuration layer can consume without redefining what one diagnostic means.
