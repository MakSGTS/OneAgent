# Change Impact Analysis Investigation

## Status and scope

This investigation is the committed Task 1 evidence for Sprint 39 Change
Impact Analysis. It starts from planning baseline
`6d9fd0ff1bbe58e83683f471c3db5fdcf2415c56` on
`codex/v0.7-sprint-39` with a clean working tree.

The objective is to make ADR-0061 decision-ready for one bounded
product-facing workflow. This document does not accept architecture or change
production behavior. It distinguishes the existing Graph-owned diff and impact
semantics from product orchestration over successive complete Workspace
publications.

Repository-owned source, tests, fixtures, accepted ADRs, current-state
documentation, and Git history provide sufficient evidence. No external
service, credential, user repository, source format, or live client is required
for the bounded first slice.

## Accepted constraints

- `SemanticGraph` remains the canonical semantic fact owner.
- `SemanticGraphDiff` remains the canonical directional graph change input.
- `SemanticImpactAnalyzer` remains the canonical dependency/ownership impact
  traversal owner. A product workflow may consume its owned result but must not
  redefine dependency classification, propagation, seed, or reason semantics.
- Workspace discovery, EDT and Designer XML adapters, complete Graph
  validation, and immutable atomic publication remain the only production path
  from source state to a published semantic graph.
- ADR-0060 Git evidence may request one source-neutral complete rebuild.
  Repository baselines, paths, statuses, completeness, and operation order are
  not semantic identities, Configuration selectors, impact seeds, reasons,
  summaries, cache values, or public impact output.
- The MCP server remains pinned to revision `2026-07-28`; Tool Policy remains a
  mandatory execution gate even for read-only semantic tools.
- Sprint 39 does not implement selective or incremental source rebuilding,
  refactoring, source edits, transactions, rollback, remote Git, or product UI.

## Confirmed Graph authority and behavior

### Directional graph changes

`crates/graph/src/diff.rs` owns `SemanticGraphDiff::between(previous, current)`.
It returns stable added, removed, and modified node and edge collections plus a
summary. Node identity present in both graphs is modified when semantic content
or normalized provenance differs. Edge identity includes source, target, and
kind, so an endpoint or kind change is represented as removal plus addition.
The implementation uses canonical graph order and produces deterministic
owned snapshots.

`crates/graph/src/build_diff.rs` owns the broader
`SemanticGraphBuildDiff`. It reuses `SemanticGraphDiff` and additionally
compares semantic diagnostics, reference requests, resolution statistics,
Graph report metrics, and provenance coverage. No production consumer uses it
as a Workspace publication transition or product impact report. ADR-0061 must
decide whether the product workflow is strictly graph impact or whether any
non-impact build-diff summary is exposed separately; it must not silently treat
diagnostic or report changes as Graph impact seeds.

### Impact inputs, options, and result

`crates/graph/src/impact.rs` owns the stateless
`SemanticImpactAnalyzer::analyze(previous_graph, current_graph, diff, options)`.
The caller supplies all three canonical inputs. The analyzer rejects a diff
whose node or edge seed is missing from its required snapshot.

`SemanticImpactOptions` contains:

- an unbounded-in-type `usize` maximum depth;
- a `SemanticGraphEdgeFilter` applied to dependency-to-usage propagation;
- `OwnershipImpactMode::{Disabled, ChildToOwner, OwnerToChild,
  Bidirectional}`; and
- `ProvenanceImpactMode::{Exclude, DirectOnly, Propagate}`.

The default is depth `1`, all accepted dependency edge kinds, ownership
disabled, and provenance-only changes direct-only. Query dependency/usage
classification currently includes `Calls`, `References`, `Reads`, `Writes`,
`DependsOn`, and `Opens`; `Contains` is optional ownership propagation.
`Grants`, `Includes`, `Extends`, and `Triggers` do not propagate through the
first impact policy.

The result owns unique `AffectedNode` values in `NodeId` order. Each value has
an optional `NodeKind`, `DirectlyChanged`, `TransitivelyAffected`, or `Removed`
status, `PreviousOnly`, `CurrentOnly`, or `Both` availability, minimum depth,
and sorted/deduplicated typed reasons. Reasons retain the originating node or
edge seed, source node, edge identity/kind, depth, previous/current snapshot,
and dependency/ownership propagation direction.

`SemanticImpactSummary` contains seed-node and seed-edge changes, directly
changed, transitively affected, removed, previous-only, and current-node
counts, maximum reached and requested depth, and total affected nodes.
`ImpactCompleteness` currently has only
`CompleteWithinRequestedDepth`. It describes traversal depth, not input
validity, product-history availability, item/reason truncation, or output-size
completeness.

The Graph analyzer has no maximum graph, seed, affected-node, reason, or output
bound. Summary arithmetic uses ordinary `usize` addition and counting because
Graph inputs are already in memory. Product-level admission, cloning, reason,
result, and serialization bounds remain unresolved for ADR-0061.

`ImpactAnalysisError` includes the missing `NodeId` or `EdgeId` in its display
text. Existing MCP behavior maps it to a closed generic `execution_failed`
message, so product errors must not expose the domain error directly without an
explicit sensitive-data decision.

### Existing executable Graph evidence

`crates/graph/tests/impact.rs` contains 18 passing tests covering:

- empty/equal graphs;
- modified dependency propagation and multiple reasons;
- `Reads` plus `DependsOn`, `Opens`, and excluded `Triggers`/`Includes`;
- repeated and insertion-order-independent results;
- depth zero, edge filters, and optional ownership propagation;
- added/removed nodes and edges with correct previous/current snapshots;
- provenance-only modes;
- typed member and conditional access-right changes;
- cycles and self-loops without duplicate nodes; and
- inconsistent diff rejection.

Production EDT tests exercise impact for command references, form navigation,
ownership, report data composition, registers/queries, conditional Grants,
Writes, and XDTO services. Coverage registries describe Graph edge propagation
support; Sprint 39 needs no new source or edge evidence for the product
workflow.

## Confirmed Workspace publication boundary

### Snapshot and Configuration identity

`apps/runtime/src/workspace/mod.rs` owns one immutable `WorkspaceSnapshot` for
one configured root. It stores Configuration snapshots sorted by canonical
Configuration `EntityId`. Each `WorkspaceConfigurationSnapshot` owns its root,
source format, Configuration ID/name, `Arc<SemanticGraph>`, diagnostics,
reference ledger/statistics, report, complete validation, rule execution
report, and normalized diagnostic report.

The Configuration ID and name are read from exactly one
`Metadata(Configuration)` node in the complete graph. Zero or multiple
Configuration nodes reject the build. Duplicate IDs across discovered roots
reject the complete Workspace snapshot. A name change with stable ID remains
the same Configuration identity; an ID change currently appears only as a
different key in a new complete snapshot. No transition matcher exists.

`WorkspaceSnapshot` has no publication revision, predecessor identifier,
build-attempt identity, source fingerprint, or impact report. Its observer is a
`watch::Receiver<Option<Arc<WorkspaceSnapshot>>>` and exposes only the current
complete snapshot. Replacing the watched value drops Workspace ownership of the
previous value; consumers that already cloned the `Arc` may retain it, but no
history service or paired previous/current observation is published.

### Build, publication, failure, and recovery

Initial startup observes the complete source, loads or builds one complete
snapshot, publishes it, then starts the portable change source. Update status
separately owns `attempt`, `published`, phase, and failure category counters.
Those counters are not part of snapshot identity and may overflow only through
a closed Runtime failure.

Each filesystem or explicit source-neutral change input requests a complete
serialized rebuild. The Runtime performs complete discovery, EDT/Designer XML
build, complete validation, diagnostic/rule composition, stable source rescan,
cache write, and then one atomic `send_replace`. It does not compute or retain a
diff between the old and new snapshots.

Build, validation, observation, or task failure changes update status to
`Failed` and retains the last published snapshot. A later accepted source
observation or explicit input may recover with a new complete publication.
Equal complete rebuilds are currently published and increment the `published`
counter. ADR-0061 must decide whether they produce a complete empty report,
reuse a prior report, or produce a distinct no-change transition.

The capacity-one explicit input and filesystem observation are coalesced into
complete rebuild requests. At most one follow-up request remains pending while
a build runs. Cancellation closes the change receiver, joins active blocking
work and the source task, clears the snapshot, publishes `Stopped`, and closes
observers. No product impact task, channel, or observer exists.

### Configuration transition cases requiring a decision

Repository behavior supports deterministic evidence for stable IDs, additions,
removals, and complete empty Workspaces, but no policy interprets them as
product impact:

- same ID in both publications can supply two complete graphs directly;
- previous-only ID could be compared with an empty graph or represented as a
  Configuration-level removal without calling Graph impact;
- current-only ID has the symmetric choice;
- an ID change cannot be proven to be a rename from source path or Git status;
- a source-format change with stable canonical ID is representable but may
  require explicit compatibility or failure policy;
- an initial publication and a warm cache hit have no earlier live publication;
  treating either as an empty baseline would claim impact that may not have
  occurred during the process lifetime; and
- failed attempts must not become the previous or current semantic endpoint.

ADR-0061 must select these outcomes explicitly and must not infer Configuration
continuity from root path, display name, discovery order, or Git evidence.

## Confirmed persistent-cache boundary

`apps/runtime/src/workspace/cache.rs` owns JSON cache schema `1` and manual
semantic compatibility `4`. The envelope contains complete source state and a
Workspace DTO with Configuration roots/formats, nodes, edges, diagnostics,
reference requests, and reference statistics. It does not contain publication
history, update counters, diagnostic/rule reports, Git evidence, or impact.

Decode validates format, schema, semantic version, exact complete source state,
checksum, canonical structure, graph semantics, provenance, reports, and
Configuration order. `snapshot_from_parts` recomputes validation, rule
execution, and diagnostic report before publication. Cold/warm equality is
therefore already an oracle for deterministic derived evidence.

Adding impact to serialized state would require an explicit schema and
semantic-compatibility decision and a validity input representing its previous
endpoint. Recomputing a transition report after warm decode is possible only
when a distinct previous live publication exists. A cache hit alone supplies
one current semantic endpoint and cannot prove the immediately preceding
product transition.

## Confirmed filesystem and Git input boundary

The portable filesystem watcher compares complete private byte state and emits
only a changed/observation-failed revision. It does not expose changed paths to
Workspace semantic composition.

ADR-0060 adds a bounded explicit Git reader and normalized `GitChangeSet`.
`WorkspaceChangeInputHandle::submit` discards the Git baseline and completeness,
maps normalized changes into a private request, and uses the non-empty set only
to request a complete rebuild. The request records are not published, cached,
or used to select source adapters, Configurations, graph nodes, diagnostics,
rules, or incremental operations.

`apps/runtime/tests/git_change_workspace.rs` already proves equal complete
semantic end states across opposite filesystem/Git operation orders, complete
failure retention and recovery, Graph Query compatibility, cold/warm cache
behavior, and fresh service ownership. The Sprint 39 oracle can extend the same
test boundary: equal previous/current complete graphs must produce equal impact
reports regardless of which accepted input caused the rebuild. No repository
value is needed or permitted in the expected report.

## Confirmed MCP and Tool Policy boundary

### Current tool contract

`apps/runtime/src/mcp_tools.rs` constructs a static seven-tool catalog in
lexicographic order. `oneagent.impact` is read-only and currently requires:

- `previousConfigurationId` string;
- distinct `currentConfigurationId` string;
- optional `maxDepth` from `0` through `4`, default `1`; and
- optional `limit` from `1` through `100`, default `50`.

Unknown fields are rejected. Both IDs select Configurations from the same
immutable `WorkspaceSnapshot`; equal IDs are invalid. The handler computes a
fresh Graph diff and Graph impact on every call. This compares two different
Configuration identities in one Workspace, not two publications of one
Configuration.

The result includes both Configuration IDs; Graph summary fields except
`previousOnlyNodes`, `currentNodes`, and `ImpactCompleteness`; bounded affected
nodes with node ID/kind/status/depth and every reason; `total`; and `truncated`
when affected-node count exceeds `limit`. It does not project availability or
an explicit completeness value. Reasons are not independently bounded, so the
Tool Policy output bound is the final fail-closed protection for a node with a
large reason set.

The semantic handler holds `Arc<WorkspaceSnapshot>` for the whole process.
`apps/runtime/src/bin/oneagent-mcp.rs` builds exactly one snapshot from the
current working directory before reading frames and never observes Workspace
updates. Existing public MCP impact therefore cannot expose a live successor
publication without an accepted composition change. The long-running Runtime
Workspace service does not currently construct this semantic MCP handler.

### Policy and failures

Every known tool call creates one exact actor/tool/request/revision Tool Policy
request with only `ReadOnly`, evaluates the fixed allow policy, and invokes
`execute_tool`. Denial maps to `policy_denied`; failed, partial, timed-out, or
cancelled execution maps to `execution_failed`. Invalid arguments, missing
Configuration, oversized results, and domain failures use the existing closed
tool-error vocabulary. Tool descriptions and annotations do not authorize
execution.

The MCP protocol bounds frames to 1 MiB and Tool Policy bounds arguments and
output to 65,536 bytes. The semantic projection must continue to avoid roots,
provenance paths, source content, repository values, policy internals, and raw
error chains.

### Consumers and compatibility

- VS Code pins revision `2026-07-28` and asserts the seven tool names, but has no
  typed impact request/result API and does not call `oneagent.impact` in
  production.
- HTTP Graph Query, CLI, LSP, and EDT expose no impact operation.
- External Codex and Cursor compatibility evidence verifies discovery and a
  seven-tool call catalog. Its negative call uses `{}` for impact, so the
  current mandatory-argument rejection is observable compatibility evidence.
- ADR-0051 owns the current two-Configuration request and result. Replacing it
  silently would be a breaking schema migration even if the tool name remains.

## Decision-ready architecture alternatives

### Product owner and dependency direction

1. A source-independent product report in `oneagent-analysis`, with Runtime
   composing it from Graph-owned results. This reuses the existing Analysis
   dependency and Diagnostics-style report precedent while keeping product
   identity/completeness out of Graph.
2. A Runtime-owned report domain next to Workspace publication. This minimizes
   dependencies but couples product semantics to Runtime and limits reuse.
3. Extending `oneagent-graph` with publication and product contracts. This
   keeps diff/impact types together but risks making Graph own Workspace
   history, compatibility, truncation, and product presentation.
4. Protocol-owned computation is incompatible with current layering and must
   be rejected because protocols do not own Graph or Workspace semantics.

ADR-0061 must select one owner and state whether it consumes Graph graphs and
computes the canonical diff itself, or consumes a caller-supplied canonical
diff plus Graph impact. It must prohibit a second traversal implementation.

### Publication composition

1. Embed an owned latest-transition report in the newly published
   `WorkspaceSnapshot`. Construction can borrow the prior Arc and new complete
   snapshot before atomic publication. Consumers receive one consistent
   current snapshot/report pair, while full prior graphs need not be retained
   after the owned report is built.
2. Publish a separate paired transition observer atomically coordinated with
   the snapshot. This avoids changing `WorkspaceSnapshot` but creates a
   consistency problem unless one owner and revision binds both values.
3. Retain two complete snapshots and compute on demand. This simplifies option
   selection but adds history lifetime/memory, stale selection, and observer
   contracts not currently present.
4. Keep only MCP's same-snapshot cross-Configuration calculation. This does not
   satisfy the Roadmap objective for change impact across a product update.

ADR-0061 must define initial/warm-start absence, equal rebuild, failed attempt,
Configuration addition/removal, ID/format transition, report lifetime,
publication identity, and whether callers may choose impact options.

### Completeness and bounds

The architecture must distinguish at least:

- complete Graph inputs and validation;
- availability of a previous published semantic endpoint;
- complete traversal within requested depth;
- complete set of matched Configuration transitions;
- report-level admission or omission;
- item and reason projection truncation; and
- protocol output-bound failure.

Choices include fail-closed whole-report bounds, explicit bounded partial
reports with omitted counts, or complete in-memory reports plus projection-only
truncation. The current Graph result alone does not settle this product policy.
Identity, duplicate/conflict behavior, checked summary reconciliation, maximum
Configurations, affected items, reasons, depth, component bytes, and error
redaction all require explicit ADR values.

There is no diagnostic suppression authority relevant to impact. ADR-0061 must
state that impact items are included, omitted, truncated, or rejected only by
the accepted impact/completeness policy; diagnostic suppression must not hide
semantic impact.

### Persistence

1. Do not serialize transition reports. Initial/warm startup has explicit no-
   predecessor state; later live publications compute deterministic reports.
   Cache schema can remain `1`, while semantic version changes only if decoded
   snapshot reconstruction behavior changes.
2. Serialize the latest report with explicit previous/current endpoint
   fingerprints. This needs schema/version migration, source-state validity,
   corruption handling, and proof that a cached historical transition is still
   meaningful after process restart.
3. Serialize two complete snapshots or a history log. This is substantially
   broader persistent-history scope and lacks a current eviction/identity
   contract.

ADR-0061 must select one and must not claim a warm cache hit has a predecessor
unless persisted evidence proves it.

### MCP compatibility

1. Add a product-report selector such as one exact `configurationId` while
   retaining the ADR-0051 legacy pair as an explicit compatibility mode.
   Validation must reject mixed modes and define which mode is the default.
2. Replace the two-ID schema with the product report. This is a breaking
   migration for ADR-0051 and existing validation evidence.
3. Add a new tool name while retaining `oneagent.impact`. This changes the
   static catalog, Tool Policy, VS Code and external-client expectations, and
   needs separate justification.
4. Return product reports only from a new long-running Runtime transport while
   leaving the immutable MCP process unchanged. No accepted transport currently
   owns that exposure.

ADR-0061 must select an executable first slice, preserve schema/handler
validation agreement, define no-report/unavailable/equal-report results, and
state whether current same-snapshot cross-Configuration comparison remains a
supported product capability or only a legacy compatibility path.

## Required deterministic evidence for Tasks 3-6

| Area | Required cases and oracle |
|---|---|
| Report domain | Empty/equal, direct, transitive, removed, previous/current-only, each reason/status/availability, reordered and repeated inputs, exact duplicates, conflicts, inconsistent canonical diff, exact/over bounds, completeness and summary reconciliation, redacted failures. |
| Configuration matching | Same ID, stable ID/name change, previous-only, current-only, empty Workspace, multiple ordered Configurations, ID change without rename inference, source-format transition, duplicate ID rejection. |
| Publication | Initial/no predecessor, successful replacement, equal rebuild, failed build retention, later recovery, coalesced follow-up, observer atomicity, cancellation during owned work, shutdown clearing and closure, repeated fresh service. |
| Cache | Cold/warm equality, explicit no-predecessor or persisted-history behavior, current/older/newer versions, source invalidation, corrupt/incompatible entry, write failure, watched replacement, clean rebuild equality. |
| Input equivalence | Equivalent complete previous/current graphs after filesystem and Git triggers, opposite operation order, non-empty trigger with equal end state, no repository value in report or failure. |
| MCP | Schema/catalog truth, each accepted selector mode, missing/extra/mixed/unknown/malformed arguments, exact/over depth/item/reason/output bounds, complete summary versus bounded items, availability/completeness vocabulary, policy allow/deny/failure, deterministic repeated JSON, channel purity, EOF and fresh process. |
| Compatibility | Graph/Analysis/Workspace/diagnostics/rules/cache/HTTP/CLI/MCP/LSP/VS Code/EDT, external-client catalog, dependencies/features/licenses/unsafe, public API and Rustdoc, Coverage and documentation. |

## Confirmed focused baseline

The Task 1 focused matrix completed successfully at planning head `6d9fd0ff`:

| Command | Result |
|---|---|
| `cargo test -p oneagent-graph --test impact` | 18 passed; 0 failed/ignored/measured/filtered |
| `cargo test -p oneagent-runtime --test workspace_service` | 6 passed; 0 failed/ignored/measured/filtered |
| `cargo test -p oneagent-runtime --test file_watching` | 2 passed; 0 failed/ignored/measured/filtered |
| `cargo test -p oneagent-runtime --test git_change_workspace` | 3 passed; 0 failed/ignored/measured/filtered |
| `cargo test -p oneagent-runtime --test persistent_cache` | 4 passed; 0 failed/ignored/measured/filtered |
| `cargo test -p oneagent-runtime --test mcp_semantic_tools` | 7 passed; 0 failed/ignored/measured/filtered |
| `cargo test -p oneagent-runtime --test mcp_process` | 17 passed; 0 failed/ignored/measured/filtered |

These 57 tests are non-zero executable oracles. They prove the current
boundaries, not the future ADR-0061 decisions.

## Expected implementation and test areas

The exact owner and file placement remain ADR decisions. Current consumers make
the following areas likely and must be rechecked after ADR-0061:

- `crates/analysis/src/` if Analysis owns the product report;
- `crates/graph/src/{diff.rs,impact.rs}` only for compatibility or a proven
  missing canonical primitive, not product orchestration;
- `apps/runtime/src/workspace/{mod.rs,cache.rs}` for accepted immutable
  publication and persistence composition;
- `apps/runtime/src/{mcp_tools.rs,bin/oneagent-mcp.rs}` for the accepted MCP
  projection or composition migration;
- `apps/runtime/tests/{workspace_service.rs,file_watching.rs,
  git_change_workspace.rs,persistent_cache.rs,mcp_semantic_tools.rs,
  mcp_process.rs}` plus focused Analysis tests;
- `extensions/vscode/src/mcp-client.ts` and its unit test only if catalog or
  supported client assumptions change; and
- README, Architecture, Semantic Model, Roadmap, ADR, investigation/evidence,
  and final review documents in their assigned tasks.

No current evidence requires a new production dependency, source fixture
family, protocol revision, Graph edge/node kind, Coverage transition, external
client run, GUI process, or network access.

## Decision readiness and blockers

ADR-0061 is decision-ready. Repository evidence answers what exists, who owns
it, which consumers are observable, and how to test the bounded alternatives.
The architecture task must decide:

1. product owner and exact Graph input boundary;
2. previous/current publication and Configuration identity;
3. report identity, vocabulary, duplicate/conflict behavior, order,
   completeness, summaries, bounds, and errors;
4. initial/equal/failure/recovery/cancellation/shutdown lifecycle;
5. persistence and cache compatibility;
6. filesystem/Git complete-end-state equivalence;
7. MCP compatibility and public-process composition; and
8. first-slice deferrals and completion evidence.

No missing-data blocker remains. Production implementation must still stop if
ADR-0061 cannot resolve these decisions consistently with the confirmed code or
if it would require an unapproved dependency or breaking consumer migration.

## Deferred scope

Selective/incremental parsing or Graph mutation; changed-path-to-node inference;
new semantic facts, dependency kinds, or propagation policy; diagnostic/rule
production; impact scoring, probability, or risk prediction; history queries;
cross-workspace or remote impact; mutable documents; refactoring plans; code
actions; source edits; transactions; rollback; repository mutation; remote Git;
new HTTP/CLI/LSP/VS Code/EDT UI; telemetry; benchmarks; and broad performance or
security claims remain outside Sprint 39.
