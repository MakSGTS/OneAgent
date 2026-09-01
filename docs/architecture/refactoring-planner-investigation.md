# Refactoring Planner Investigation

## Status and scope

This investigation is the Task 1 evidence for Sprint 40 Refactoring Planner. It
starts from amended planning baseline
`1319674fd5a9ae37ec01418c3fe8c3602f143621` on
`codex/v0.7-sprint-40` with a clean working tree. The baseline descends from
planning commit `2bc6afb7` and framework commit `5c273da1`; Sprint 40 is the
unique `next` target before this task changes it to `active`.

The objective is to make ADR-0063 decision-ready for one bounded deterministic
read-only semantic plan and preview. This document records repository evidence
and alternatives; it does not accept architecture, add an edit capability, or
authorize source, repository, cache, Workspace, editor, protocol, or Git
mutation.

The effective context window and runtime token telemetry were unavailable. The
preflight admitted only the child prompt, selected framework contract, bounded
Roadmap and Sprint 39 hand-off sections, named source symbols, direct consumers,
tests, fixtures, and Cargo boundaries. The decision was `pass`; the complete
Roadmap, Architecture, semantic-model document, fixture corpora, generated
outputs, and successful command logs remained excluded.

## Investigation conclusion

No current refactoring family has all of the semantic, immutable source,
content-version, exact-range, conflict, and paired-format evidence needed for
planning. This is not external missing data. The repository contains the source
corpus, parsers, adapters, mixed Workspace fixture, controlled changes, and
public-process oracles needed to close the gap.

The smallest coherent first-slice candidate is:

> Rename one top-level BSL `Procedure` or `Function` and every supported direct
> call occurrence that resolves uniquely to that target within one complete
> Configuration publication, for EDT and Designer XML BSL modules. Reject the
> whole request when the target, desired name, source document, content version,
> exact declaration/reference ranges, resolution, completeness, or conflict
> evidence is missing or ambiguous.

This family has the narrowest existing semantic vocabulary: stable pre-rename
Graph identity, closed `Procedure`/`Function` kinds, one owning Module, EDT
local and qualified call resolution, deterministic Graph queries, paired EDT /
Designer declaration fixtures, source-location projections, and controlled
source-change/publication oracles. It still requires Tasks 3–4 to add one
source-independent immutable source-document contract, deterministic content
version, exact declaration and call occurrence ranges, and paired adapter
capture before planner evaluation can start.

The Sprint data and testability gate therefore passes. The planner-evaluation
gate remains intentionally closed until Tasks 3–4 prove that internal
prerequisite. `SPRINT_BLOCKED_MISSING_DATA` does not apply.

## Accepted constraints

- Graph remains the only semantic identity, containment, relation, query, diff,
  and impact authority. Planner types may consume Graph evidence but may not
  duplicate its traversal or reinterpret paths, statuses, diagnostics, or
  impact as semantic identity.
- Only one immutable complete Workspace publication may supply one plan. Source
  content and occurrence evidence must be captured before publication and must
  not be re-read while planning or previewing.
- Planning and preview remain read-only phases. A valid plan is not edit
  authorization and creates no source, repository, Workspace, cache, editor,
  protocol, or Git mutation capability.
- Tasks 3–4 own the repository-implementable immutable source document,
  deterministic content version, exact occurrence, and paired-adapter evidence
  prerequisite. Task 2 must accept ADR-0063 before either implementation task.
- Existing Runtime lifecycle, Tool Policy enforcement, supported MCP revisions,
  source confinement, closed public failures, and existing tool behavior remain
  compatibility constraints. Sprint 40 adds no HTTP, CLI, LSP, or IDE UI.
- No production dependency, external corpus, credential, network service, user
  repository, or GUI validation is required for the bounded first slice.

## Confirmed semantic and source evidence

### Common source domain

`crates/common/src/source.rs` owns:

- `SourcePath`, a slash-normalized UTF-8 path bounded at 4,096 bytes. It rejects
  empty, malformed, traversal-bearing, and over-bound values, but deliberately
  accepts absolute or relative paths and leaves containment to consumers.
- `SourcePosition`, with one-based line and column coordinates.
- `SourceSpan`, an ordered half-open range whose equal endpoints explicitly
  represent a navigation point.
- `SourceLocation`, which binds one `SourcePath` to an optional `SourceSpan`.

These types can represent an exact occurrence but do not prove that a producer
captured one. They contain no source content, encoding, line-ending policy,
content digest, document identity, snapshot identity, or confinement root.

### Graph identity, provenance, query, and impact

`GraphNode` in `crates/graph/src/node.rs` owns canonical `EntityId`,
`EntityName`, `NodeKind`, payload, and a provenance vector. Procedure and
Function identifiers produced by the BSL extractor contain their current name,
so a rename changes the resulting semantic node identity. The planner target
must therefore retain the accepted pre-rename node ID and separately describe
the expected post-plan identity; the display name is not plan identity.

`Provenance` in `crates/graph/src/provenance.rs` keeps opaque source identity and
typed `SourceLocation` as independent optional evidence. Location participates
in provenance equality but does not change source identity. Nodes and edges can
carry multiple records; consumers must not select one by encounter order.

`SemanticGraphQuery` in `crates/graph/src/query.rs` owns read-only stable node,
exact-name/kind, owner/child, edge, adjacency, dependency, usage, and traversal
queries. Nodes are returned in `NodeId` order, edges in stable `EdgeId` order,
neighbors are deduplicated, and `Calls` is an accepted dependency edge.
`SemanticGraph` storage replaces a duplicate node ID and deduplicates an equal
edge ID, so these storage behaviors cannot serve as rename collision policy.

`SemanticImpactAnalyzer` in `crates/graph/src/impact.rs` owns deterministic
impact over a caller-supplied canonical previous/current diff. Impact can
explain affected callers, but it does not identify text occurrences, establish
source completeness, authorize an edit, or become planner target identity.
ADR-0063 must keep Graph query/diff/impact ownership unchanged and prohibit a
second semantic traversal.

### BSL extraction and resolution

`BslSymbol` and `BslCall` in `crates/bsl/src/{lib,calls}.rs` retain one-based
lines but not columns or identifier end positions. The line-oriented
declaration extractor accepts English and Russian Procedure/Function keywords
and records whether a declaration is exported. Local and qualified call
resolvers compare names using lowercase normalization; qualified calls require
an exported destination. Exact rename conflict comparison must use the same
accepted BSL name equivalence, not `EntityName`'s exact string ordering.

The current extractors are semantic precursors, not edit-range parsers. They do
not return exact identifier tokens, byte/character coordinate units, raw-source
anchors, or an occurrence completeness contract. Task 3 must define those
source-independent contracts; Task 4 must prove both production adapters map
only validated exact occurrences to canonical targets.

### EDT adapter

`AnalyzedBslModule` and `analyze_module` in
`adapters/edt/src/bsl_graph.rs` read one module with `read_to_string`, extract
declarations, calls, and queries, and retain a path-only opaque source ID plus a
typed path while the build is in progress. `insert_declarations` emits
Procedure/Function nodes and `Contains` edges with exact declared provenance.
`declaration_location` creates only `(line, 1)..(line, 1)` point spans.

EDT resolves local and qualified direct calls and emits `Calls` edges. A call
edge carries module source provenance but no call location. Equal calls between
the same semantic endpoints collapse into one Graph edge, so edge provenance
does not preserve occurrence multiplicity. Unresolved calls become diagnostics
with source identity and line encoded in the opaque source fragment, not a
typed location or canonical reference-request ledger entry.

The EDT Semantic Coverage Registry marks BSL call references supported for
parsing, resolution, diagnostics, edge emission, provenance, statistics, and
tests. That coverage is semantic Graph evidence only; it does not claim exact
editable occurrences or immutable source retention.

### Designer XML adapter

`DesignerXmlModuleSourceEvidence` and `DesignerXmlModuleDescriptor` in
`adapters/designer-xml/src/module_reader.rs` retain adapter-local raw bytes,
artifact path, and normalized UTF-8 text. Normalization strips an optional BOM
and converts CRLF/CR line endings to LF. Descriptors are canonically ordered and
reject unsupported, orphan, mismatched, non-UTF-8, and symlinked module inputs.

`emit_module_and_declarations` in
`adapters/designer-xml/src/semantic_graph.rs` hashes raw bytes into an opaque
source ID and emits Module, Procedure, Function, and `Contains` facts.
Declarations receive the same one-based `(line, 1)..(line, 1)` point location.
The private `source_id` SHA-256 value is content-sensitive, but it is mixed with
path and fact fragments, is not a typed reusable document version, and is not
published with the captured bytes.

Designer XML currently runs only the declaration extractor. It emits no BSL
`Calls` edges, unresolved-call diagnostics, or call occurrences, and its
Coverage Registry contains Module/Procedure/Function and ownership/provenance
capabilities but no Designer BSL-call capability. Task 4 must close this paired
adapter asymmetry for the accepted family instead of treating the EDT-only call
graph as cross-format complete.

### Paired and controlled fixtures

`adapters/designer-xml/tests/fixtures/sprint14_conformance/` contains tracked
paired EDT and Designer representations of the same Configuration and common
module. The common module contains one exported
`FillSecurityCollection` Procedure. `adapters/designer-xml/tests/conformance.rs`
proves non-empty canonical Module/Procedure/Function ownership equality,
repeatability, complete validation, and content-sensitive Designer provenance;
its projection intentionally excludes `Calls` and strips provenance.

`adapters/designer-xml/tests/fixtures/modules/{edt,designer}/` proves that BOM
and line-ending normalization feeds the same BSL declaration analyzer.
`adapters/edt/tests/module_emission.rs` proves real EDT nodes, ownership,
resolved `Calls`, provenance, and repeated-build equality. Controlled Designer
synonym and EDT source changes prove deterministic Graph diffs, but no current
fixture proves exact rename ranges, multiple occurrences, collisions, preview,
or paired rename-plan equality.

## Confirmed immutable publication and consumer boundaries

### Workspace publication

`WorkspaceConfigurationSnapshot` in `apps/runtime/src/workspace/mod.rs` owns one
Configuration root, format, canonical ID/name, immutable Graph, diagnostics,
reference evidence, report, validation, rules report, and diagnostics report.
`WorkspaceSnapshot` owns canonically ordered Configurations, root path, and
process-local adjacent-publication impact evidence. An observer clones one
`Arc<WorkspaceSnapshot>` atomically.

The publication does not retain the accepted BSL source bytes, normalized
document text, deterministic document version, exact declaration/call
occurrences, or a source-evidence completeness marker. EDT discards its read
text after build; Designer descriptor bytes are adapter-local and discarded
after Graph construction. A planner that re-reads files after publication
would violate the Sprint objective and could mix Graph and source versions.

The existing `ChangeImpactPublicationId` identifies a successful process-local
Workspace publication, but it is owned by the Change Impact domain. ADR-0063
must decide whether planner preconditions reuse that exact identity or introduce
a Workspace/planner-owned snapshot identity without creating two competing
publication counters. Update counters, paths, Git baselines, and statuses are
not substitutes.

Workspace complete rebuilds, validation, atomic publication, failed-attempt
retention, recovery, equal-end-state behavior, and filesystem/Git trigger
equivalence are reusable oracles. The planner must borrow one complete
publication for the whole evaluation and publish no partial plan when any
Configuration or source-evidence precondition fails.

### Cache

`apps/runtime/src/workspace/cache.rs` owns private cache schema `1` and semantic
compatibility `5`. Its source-state envelope already retains canonically ordered
relative entries and exact regular-file bytes for validation and invalidation,
while the semantic Workspace DTO retains Graph/provenance evidence. Cache decode
reconstructs derived Workspace reports without running adapters.

Those private bytes are useful reproducibility evidence but are not currently a
published source-document contract and cannot be reached by a planner. If
immutable source documents and occurrences become snapshot fields, cache load
must either reconstruct and validate exactly the same evidence from its
accepted source entries or reject/rebuild through an explicit schema/semantic
compatibility change. Silently publishing Graph-only warm snapshots would make
planner availability depend on cold versus warm startup.

### Diagnostics and impact

Diagnostics can expose unresolved or ambiguous call evidence and Graph
validation can prove structural validity. Change Impact can explain adjacent
semantic change. Neither owns occurrence completeness, source versions, plan
identity, conflicts, preview, or authorization. They may be fail-closed planner
preconditions only if ADR-0063 names exact relevant evidence; unrelated
diagnostics must not make results order-dependent.

### Runtime, Tool Policy, MCP, and clients

Runtime currently clones one current immutable snapshot for each MCP call.
Every semantic tool is declared and authorized only as `ReadOnly`; Tool Policy
bounds encoded arguments and outputs at 65,536 bytes. MCP bounds frames at 1
MiB, dispatches sequentially, supports revisions `2025-06-18`, `2025-11-25`,
and `2026-07-28`, and exposes exactly seven lexicographically ordered tools.

`oneagent.symbols` is the closest public precedent. It accepts only bounded
queries/kinds/limits, requires one distinct `SourceLocation`, confines paths to
the Configuration and Workspace roots, returns Workspace-relative paths, and
omits ambiguous, missing, cross-Configuration, or escaping locations. It proves
public path confinement and deterministic Procedure/Function discovery, but its
point locations are navigation-only and it truncates search results.

No MCP planner schema, Tool Policy rule, Runtime service, HTTP/CLI/LSP planner,
or source-edit tool exists. The VS Code client pins the exact seven-tool catalog
and has typed support for `oneagent.symbols` but no plan method or edit UI.
Adding an eighth tool requires a synchronized accepted catalog/client/process
compatibility change; overloading an existing semantic tool would change its
schema and validation precedence. Sprint 40 excludes a new IDE UI and every
mutation capability.

Public plan output must not expose absolute paths, repository configuration,
opaque provenance source IDs, hashes used as internal versions, raw source,
unbounded replacement text, credentials, environment data, or internal error
chains. Whether a bounded preview may contain source/replacement snippets is an
explicit ADR-0063 sensitive-data decision, not implied by read-only policy.

## Alternatives and decision-ready ADR-0063 questions

Every row below is unresolved architecture scope, not an accepted choice. The
repository constraint and the exact decision surface are sufficient for Task 2
to accept one alternative or record an explicit deferral without new external
evidence.

| Decision | Confirmed constraint | Decision required before implementation |
|---|---|---|
| First family | BSL callables have the narrowest cross-layer target evidence; no family is currently edit-ready. | Accept or reject the bounded Procedure/Function rename family, exact EDT/Designer module roles, local/qualified direct-call forms, exported/non-exported targets, and explicit unsupported syntax. |
| Domain owner | Graph owns semantics; adapters own parsing; Runtime owns publication; protocol owns wire shape. `oneagent-analysis` already depends on Common, BSL, and Graph and Runtime depends on Analysis. | Select a source-independent planner/report owner, preferably Analysis unless a dependency audit disproves it; prohibit Graph, adapter, Runtime, or MCP from reimplementing planning semantics. |
| Target identity | Callable IDs are stable for the captured old name but name-derived across rename. Owner query exposes one Module. | Bind target to Configuration ID, exact pre-rename Node ID/kind, one owner Module ID, and accepted source occurrence set; define expected post-plan ID separately. |
| Snapshot identity | Workspace has one process-local publication ID through Change Impact; update counters are separate. | Reuse or generalize exactly one publication identity owner and define process lifetime, equality, staleness, initial/warm behavior, and Configuration matching. |
| Source document | `SourceLocation` has path/range only; Workspace lacks content. Designer raw/normalized forms differ. | Task 3 must define confined document identity, exact captured content representation, encoding/BOM/line-ending policy, deterministic content version, equality, and validated half-open range unit. |
| Adapter occurrence evidence | EDT declaration/call models are line-only; Designer has declarations only; Graph edges collapse equal calls. | Task 4 must capture declaration and every accepted call occurrence with exact ranges and canonical target mapping for both formats, plus a completeness/failure marker. |
| Desired name and collision | `EntityName` only rejects blank; BSL resolution lowercases names. Graph storage replacement is not a conflict rule. | Define BSL identifier grammar/byte bound, case-equivalence, keyword policy, same-name no-op, sibling collision, target-ID collision, and exported qualified-name effects. |
| Plan identity | No plan type exists. Display labels, preview, and runtime metadata are mutable projections. | Define deterministic identity from canonical family/request, target, publication/source preconditions, and accepted desired name; exclude preview rendering, timestamps, paths, and encounter order. |
| Operations and identity | No edit operation vocabulary exists; `SourceSpan` can represent an exact range. | Define a closed declaration/reference replacement operation with document ID/version, exact range, expected captured token, replacement, and stable operation ID. No apply operation belongs to Sprint 40. |
| Ordering and duplicates | Graph/query order is stable, but filesystem/discovery/request order is not authority. | Select one total order, such as confined document identity, descending non-overlapping range for projection, operation kind, and operation ID; define exact duplicate collapse before summaries. |
| Conflicts | Multiple calls collapse in Graph; no planner collision policy exists. | Reject same-range different replacements, overlaps, incompatible document versions, duplicate target identities, missing/ambiguous occurrences, desired-name collisions, and ordering cycles atomically; never use last-writer-wins. |
| Completeness and summary | Current symbol/MCP results may truncate; a usable plan cannot appear complete after omission. | Define requested/planned/conflicted/rejected/omitted/returned counts with checked arithmetic and one explicit complete read-only state; reject over-bound internal plans rather than silently truncate operations. |
| Preview | No preview exists and no post-publication source read is allowed. | Define a deterministic projection produced only from captured immutable documents and canonical operations; decide bounded context/snippets, redaction, line endings, and whether public projection may truncate only presentation while retaining explicit incompleteness. |
| Bounds | Common path is 4,096 bytes; Tool Policy args/output are 65,536 bytes; MCP frame is 1 MiB; current tool result limits are at most 100. | Set exact target, document, operation, dependency, identifier, desired-name, expected/replacement, preview, error-detail, and serialized output bounds with exact/one-over tests before cloning/publication. |
| Failures | Existing public tools use closed `invalid_arguments`, `not_found`, `policy_denied`, `execution_failed`, and `result_too_large`. | Define closed domain failures and deterministic validation precedence for missing/stale/incompatible/incomplete/conflicting/over-bound evidence; map them without leaking source or paths. |
| Workspace lifecycle | One Arc is immutable; failed rebuilds retain prior publication; cache can bypass adapters. | Bind evaluation to one Arc, publish no mutable planner state, define repeated/equal calls, rebuild races, stop/cancel behavior, cold/warm equivalence, and source-evidence absence as fail-closed. |
| MCP and compatibility | Catalog and VS Code client expect exactly seven tools; all current semantic tools are read-only. | Select an additive planner tool or another explicitly migrated surface, preserve supported revisions and legacy tools, add only `ReadOnly` policy, update exact catalog consumers, and keep edit/application absent. |
| Persistence | Cache stores private exact source entries but not published documents/occurrences. | Choose deterministic reconstruction versus schema/semantic-version migration and prove cold/warm source-evidence and plan equality. Do not persist plan history in Sprint 40. |

## Required deterministic oracles

### Focused domain and Graph oracles

- Empty, one declaration, one local call, multiple calls on one line/across
  lines, qualified exported call, unsupported/unresolved/ambiguous call,
  Procedure and Function, English and Russian keywords, repeated and reordered
  inputs.
- Missing/multiple owner, wrong node kind, stale publication, stale content
  version, missing document/range, reversed/out-of-content/UTF-8-boundary range,
  exact duplicate, same-anchor conflict, overlap, collision, no-op desired name,
  exact limits, one-over limits, and checked-summary reconciliation.
- Exact operation and plan identity equality across request, Graph insertion,
  adapter discovery, and occurrence input order; inequality for every semantic
  precondition or operation change.
- Synthetic unit graphs may supplement conflict/cycle cases but cannot replace
  production adapter evidence.

### Paired adapter and controlled-change oracles

- Extend the tracked paired EDT/Designer common-module fixture with the same
  declaration and supported local/qualified call occurrences and assert equal
  canonical source-document/occurrence/plan projections.
- Preserve exact raw bytes and deterministic versions across accepted BOM and
  CRLF/CR/LF cases while mapping exact identifier ranges to the raw captured
  document contract.
- Add controlled declaration, one-call, repeated-call, collision, unresolved,
  malformed, missing-module, non-UTF-8, symlink, and reordered-layout cases.
- Assert that the current Graph, report, validation, provenance, Coverage, and
  complete-rebuild behavior remain equal outside the controlled rename evidence.

### Workspace, cache, and no-hidden-read oracles

- Build and publish one snapshot, then change, delete, or make the source file
  unreadable before planning. Repeated plans over the retained Arc must remain
  byte-for-byte equal, proving no post-publication filesystem read.
- A new complete publication from changed source must receive different source
  versions/preconditions; planning with old evidence against the new
  publication must fail stale rather than mix snapshots.
- Prove initial, equal rebuild, failed rebuild, recovery, cancellation, stopped
  observer, Configuration add/remove, source-format mismatch, and filesystem/Git
  trigger-equivalent semantic end states.
- Prove cold build and validated warm cache decode publish equal complete source
  evidence and plans, or explicitly reject/rebuild an incompatible cache.

### Public-process and compatibility oracles

- Catalog/schema/annotation and Tool Policy allow/deny tests; invalid field,
  malformed type, duplicate list value, lookup, stale, conflict, exact/one-over
  bound, output-size, and redacted failure precedence.
- Same request over one immutable publication is byte-for-byte repeatable;
  reordered JSON fields are equivalent; later calls may observe a newer atomic
  publication without changing an in-flight result.
- Modern `2026-07-28` and negotiated `2025-06-18`/`2025-11-25` public-process
  lifecycles expose the same accepted read-only payload and clean EOF/shutdown.
- Preserve every existing tool input/result and update exact seven-tool catalog
  assertions and the VS Code connection contract only through the accepted
  migration. No client edit action is added.

### Full validation gate

Tasks 3–9 must run their non-zero focused suites plus the canonical workspace
format, check, test, clippy, and Rustdoc matrix. Task 9 must also audit public
exports, Cargo dependencies, cache compatibility, protocol schema, exact tool
catalog, sensitive data, source scope, current documentation, and unrelated
changes. Task 10 supplies independent fresh-context review and artifact
consistency before Sprint completion.

## Existing focused evidence executed for this investigation

All 45 meaningful selected tests passed with zero failures, ignored, or
measured tests:

| Boundary | Command | Meaningful count |
|---|---|---:|
| Common source | `cargo test -p oneagent-common source_` | 3 |
| Graph provenance | `cargo test -p oneagent-graph provenance` | 34 |
| EDT callable emission | `cargo test -p oneagent-edt --test module_emission module_emission_builds_canonical_nodes_owners_symbols_calls_and_provenance` | 1 |
| Paired adapter | `cargo test -p oneagent-designer-xml --test conformance` | 3 |
| Workspace lifecycle | `cargo test -p oneagent-runtime --test workspace_service public_workspace_snapshot_and_health_follow_owned_lifecycle` | 1 |
| Workspace end-state order | `cargo test -p oneagent-runtime --test git_change_workspace public_git_input_publishes_equal_complete_end_states_across_operation_orders` | 1 |
| MCP location projection | `cargo test -p oneagent-runtime --test mcp_semantic_tools symbol_search_preserves_matching_filtering_ordering_and_locations` | 1 |
| MCP public process | `cargo test -p oneagent-runtime --test mcp_process public_mcp_process_serves_every_semantic_tool_family_repeatably` | 1 |

The package-wide Graph name filter also invoked six test binaries with zero
matching tests: `build_diff`, `data_composition`, `diff`, `query`, `report`, and
`xdto_service`. They are reported as zero-match outcomes and are not counted as
evidence.

## Decision readiness and next action

ADR-0063 can now select the first family, ownership, identities, preconditions,
operation/preview/failure contracts, bounds, lifecycle, cache, MCP migration,
and deferrals without external research or implementation guessing. The
recommended next action is Task 2 architecture acceptance. Tasks 3–4 must then
implement and prove immutable source documents, deterministic versions, and
exact paired occurrence evidence before any planner evaluation. Sprint 40
remains strictly read-only; edit application, transactions, atomicity,
rollback, reversibility, backups, post-edit rebuild, and source mutation remain
Sprint 41 scope.
