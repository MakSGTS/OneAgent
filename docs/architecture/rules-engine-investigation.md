# Rules Engine Investigation

## Status and baseline

This document records the read-only Sprint 37 Task 1 investigation at committed
planning head `3c0222dc`, after the committed Rules Engine framework
prerequisite `68045f0c` and completed Sprint 36 review merge `8240ed1a`.
Sprint 37 is the active execution target. No production Rust, Cargo manifest,
fixture, protocol, cache, Coverage, or public API change is part of this
investigation.

The investigation separates:

- **confirmed repository evidence** from current code, tests, manifests, and
  accepted review artifacts;
- **accepted constraints** from ADR-0008, ADR-0039, ADR-0042, and ADR-0058;
- **decision candidates** that ADR-0059 must resolve; and
- **unsupported behavior** that remains deferred.

## Confirmed current boundary

### Canonical immutable semantic evidence

| Evidence | Owner and API | Lifetime and order | Existing oracle | Constraint for rules |
|---|---|---|---|---|
| Semantic facts | `oneagent-graph::SemanticGraph`; ordered node and edge storage plus `query()` | Owned by one immutable Workspace Configuration snapshot; ordered collections provide deterministic iteration | Graph query, validation, report, diff, impact, and adapter tests | Rules may borrow the graph or its public queries but must not mutate it, read source, replace Graph validation, or create a second fact authority. |
| Recoverable diagnostics | `oneagent_graph::SemanticDiagnostic` slice in `WorkspaceConfigurationSnapshot::diagnostics()` | Ordered producer evidence retained with the snapshot | Graph, adapter, Workspace, cache, Diagnostics Engine, MCP, and LSP tests | These remain Graph-owned inputs. A rule must not restate an existing producer diagnostic as a new rule finding. |
| Complete validation | `SemanticGraphValidationResult` from complete build-result validation and `WorkspaceConfigurationSnapshot::validation()` | Owned and immutable; issues are sorted and deduplicated | Graph validation tests and Workspace atomic-failure tests | Rules may consume caller-supplied validation evidence only if ADR-0059 accepts it. They must not invoke hidden validation or reinterpret `is_valid`. |
| Normalized diagnostics | `oneagent_analysis::diagnostics::DiagnosticReport` from stateless `DiagnosticEngine::build` | Complete, bounded, immutable, and canonically ordered | 25 focused domain/engine tests and 3 public engine tests passed in this investigation | ADR-0058 admits exactly Semantic and Validation families. Rule evidence cannot enter the report through the current public domain without an explicit additive migration. |
| Graph report and reference evidence | `SemanticGraphReport`, request ledger, and reference statistics accessors on the Configuration snapshot | Same immutable generation as graph, diagnostics, and validation | Graph report/reference and Workspace/cache tests | Rules may read only fields accepted by ADR-0059 and must not make a derived result authoritative over canonical reports or ledgers. |

`SemanticGraph` is cloneable, but Workspace exposes only `&SemanticGraph`
through an immutable `Arc`. Its public mutation APIs are builder/producer
capabilities, not authority for a Rules Engine that evaluates a published
snapshot.

### Diagnostics Engine contract that rule results must preserve

The current diagnostics domain is deliberately closed:

- `DiagnosticFamily` contains only `Semantic` and `Validation`;
- `DiagnosticCode`, `DiagnosticKind`, `DiagnosticIdentity`, and
  `DiagnosticEvidence` have one typed variant for each of those two families;
- `DiagnosticFinding` construction is limited to Graph-owned semantic or
  validation evidence;
- identity conflicts fail closed independently from input order;
- exact duplicates collapse, the complete report is canonically ordered, and
  the summary reconciles active and suppressed findings;
- `DiagnosticEngine::build` accepts only
  `&[SemanticDiagnostic]`, `&SemanticGraphValidationResult`, and one
  exact in-memory `DiagnosticPolicy`;
- the hard complete-report bound is 65,536 findings and there is no partial
  engine report.

Therefore a rule-specific diagnostic is not representable by relabeling an
existing value. ADR-0059 must either:

1. define an additive typed Rule family and migrate every exhaustive
   Diagnostics Engine consumer; or
2. keep rule results separate from `DiagnosticReport` in the first slice and
   make their public visibility explicitly unavailable.

Passing a rule message through `SemanticDiagnostic` is unsupported because it
would make Graph appear to own evidence it did not produce and would require a
false `SemanticDiagnosticCode`, `SemanticDiagnosticKind`, and
`SemanticReference`.

### Workspace, cache, watching, and lifecycle

`WorkspaceConfigurationSnapshot` currently owns immutable shared graph,
recoverable diagnostics, request evidence, graph report, complete validation,
and normalized diagnostic report values. `snapshot_from_parts` constructs
complete validation and the default-policy diagnostic report before returning
the Configuration snapshot. A semantic-build or diagnostic-domain failure
prevents publication.

The Workspace service:

- publishes a complete `Arc<WorkspaceSnapshot>` only after successful private
  construction;
- replaces generations atomically through a watch channel;
- preserves the last accepted generation after a recoverable rebuild failure;
- owns its update status, blocking build tasks, cancellation, shutdown, and
  cleanup;
- exposes graph, validation, diagnostic, cache, watcher, HTTP, CLI, MCP, and LSP
  consumers through the same immutable generation.

The cache uses schema version `1` and semantic compatibility version `3`.
It serializes canonical graph and producer evidence, not derived validation or
`DiagnosticReport`; decode calls the same `snapshot_from_parts` boundary and
recomputes both. An accepted rule result could therefore be:

- deterministically recomputed after decode, requiring every rule registry,
  configuration, input, and engine-compatibility change to participate in
  invalidation and normally requiring a semantic-version advance; or
- serialized, which would require a new cache schema/DTO decision and complete
  validation before publication.

ADR-0042 gives no permission to serialize incidental rule trait objects,
closures, executor state, or errors.

The Runtime service `Cancellation` is a Tokio watch receiver owned by the
service container. Separate source-independent cancellation traits already
exist in `oneagent-llm` and `oneagent-tool-policy`, but they are private to
those domains and there is no shared cancellation abstraction. ADR-0059 must
choose whether rule execution is synchronous and bounded inside snapshot
construction, owns a Rules Engine-specific read-only cancellation signal, or is
explicitly non-interruptible in its first slice. Reusing an unrelated provider
or Tool Policy trait would create an unjustified dependency.

### Current configuration authorities

`RuntimeConfig` contains only application name, environment, HTTP bind
address, and Workspace root. `ConfigurationProvider` loads that value.
`DiagnosticPolicy` is an exact in-memory suppression set, and production
Workspace always supplies its empty default. No repository-owned value defines:

- registered rule identities;
- enabled or disabled rules;
- per-rule parameters;
- dependency overrides;
- severity or category overrides;
- rule execution limits;
- project, Workspace, user, environment, protocol, or UI rule settings; or
- persisted rule preferences.

The absence is confirmed evidence. ADR-0059 must not invent a configuration
file, environment variable, cache payload, protocol schema, or UI. Evidence-
supported first-slice candidates are an immutable programmatic configuration
constructed with the registry, or an all-default static configuration with
explicit disabled/inapplicable terminal states reserved by the domain.

## No existing general Rules Engine

Focused source searches found no production `RuleEngine`, `RuleRegistry`,
`RuleId`, rule dependency graph, rule execution plan, rule configuration
domain, or aggregate rule result.

Similarly named concepts are not reusable Rules Engine implementations:

| Existing concept | Actual responsibility | Why it remains separate |
|---|---|---|
| `oneagent_tool_policy::ToolRule` and `RuleAction` | Immutable actor/tool/effect authorization precedence for external tool side effects | It owns request-wide deny/confirm/allow policy, not semantic analysis, dependencies, diagnostics, or rule execution. Tool Policy must remain independent from Graph and Analysis. |
| `SemanticGraphValidator` and schema functions | Canonical Graph and build-result invariant validation | Rewrapping validators as rules would duplicate validation authority and could change build validity. |
| Parser and adapter checks | Source-format parsing, normalization, and recoverable producer evidence | They read source-specific artifacts and must remain in adapters/producers. |
| `DiagnosticEngine` | Normalization and reporting of already-produced evidence | ADR-0058 explicitly forbids rule registration and evidence invention. |
| Graph `DependsOn` edges and dependency query APIs | Semantic relationships among graph nodes | They are semantic facts, not dependencies among executable rules. |
| Test helper closures and fake services | Local deterministic test seams | They have no stable public identity, registry, dependency, or result contract. |

Tool Policy does provide useful non-normative implementation precedent:
bounded input rule counts, canonical sorting/deduplication, typed closed actions,
default-deny behavior, and source-independent tests. ADR-0059 may reuse the
design qualities, but not its types, dependency, identity, or authorization
semantics.

## Evidence-supported ownership candidates

### Candidate A — `oneagent-analysis`

Confirmed advantages:

- it already depends on `oneagent-common`, `oneagent-graph`, and
  `oneagent-bsl`;
- it already owns source-independent Context and Diagnostics engines;
- Runtime already depends on Analysis;
- placing rule domain and execution here requires no new dependency for
  Graph or Runtime;
- it can borrow canonical Graph and Diagnostics values without source-adapter
  dependencies.

Required separation:

- a dedicated `rules` module must not become a Graph builder, validator,
  protocol handler, cache codec, Runtime service, or source reader;
- BSL source ownership already present in the crate does not authorize rules to
  consume unconfined source text;
- cancellation and executable rule abstraction must remain object-safe,
  bounded, deterministic, and independent from Tokio if ADR-0059 requires them.

### Candidate B — `oneagent-graph`

This would give immediate graph access but conflicts with accepted ownership:
Graph owns canonical facts, validation, query, provenance, and reports; it must
not own higher-level configurable analysis orchestration. A Rules Engine in
Graph would also force diagnostic and configuration concepts into the semantic
authority. This candidate is evidence-backed only if ADR-0059 limits the work
to a new Graph invariant, in which case it should be a validator change rather
than a general Rules Engine.

### Candidate C — `oneagent-runtime`

Runtime owns snapshot composition, lifecycle, cache, and transports, so it is
the integration owner. Making it the rule domain owner would couple rule
identity, dependencies, and results to Tokio, filesystem/service lifecycle, and
protocol composition. This is inconsistent with source-independent reuse and
would make deterministic unit evidence harder. Runtime remains the strongest
composition candidate, not the domain candidate.

### Candidate D — new crate

A new crate could enforce isolation but current evidence shows no dependency
cycle or independent reuse requirement that `oneagent-analysis` cannot
satisfy. It would add manifest and lockfile surface without a proven benefit.
ADR-0059 must reject or justify it explicitly; any new production dependency
requires current user approval before implementation.

## Identity and registration decision candidates

ADR-0059 must define:

- a validated typed rule identity and stable public string projection;
- whether identity is a bounded opaque label, a structured namespace/name
  pair, or another repository-owned type;
- whether version participates in identity, observable metadata, compatibility,
  or is deferred;
- one immutable registration representation separating metadata from executable
  behavior;
- one registry owner and construction boundary;
- exact duplicate versus same-identity/different-content behavior;
- deterministic enumeration and a hard rule-count bound;
- whether empty registry is valid;
- whether a registry contains built-in executable objects, descriptors plus an
  executor dispatch table, or another object-safe seam; and
- which metadata is observable and therefore participates in conflict
  detection.

Evidence supports `BTreeMap` or sort-then-validate normalization. It does not
support last-wins, insertion-order precedence, hash-order enumeration, mutable
global registration, filesystem discovery, or dynamic code loading.

## Dependency and execution-plan decisions

No current type defines rule dependencies. ADR-0059 must choose the meaning of
the first-slice relation:

- ordering-only prerequisites;
- availability prerequisites whose absence rejects the registry or plan;
- execution-success prerequisites that can block downstream rules;
- result-consuming dependencies; or
- an explicitly bounded combination.

Each meaning changes terminal statuses and failure containment. The ADR must
define validation and outcomes for missing, self, equivalent duplicate,
conflicting duplicate, incompatible, and cyclic dependencies before execution.

Evidence supports a deterministic topological plan with the complete typed rule
identity as the tie-breaker for independent ready nodes. Required oracles are:

- empty and single registry;
- independent rules registered in every relevant order;
- chain and diamond;
- repeated equivalent dependency declarations;
- missing dependency;
- self-dependency;
- cycle independent from input order;
- exact and one-over dependency counts; and
- repeated planning equality.

DFS iteration or queue order without a canonical ready-set tie-breaker is not
acceptable evidence.

## Configuration and applicability decisions

The first slice needs an immutable source-independent configuration value even
if production uses only its default. ADR-0059 must decide:

- whether configuration is registry-wide, per-rule, or both;
- stable identity and equality;
- enabled/disabled default;
- accepted parameter vocabulary, or explicit absence of parameters;
- unknown and duplicate rule entries;
- bounds;
- compatibility with a registry;
- whether invalid configuration rejects planning or produces a terminal status;
- whether applicability is a pure rule predicate, a planning outcome, or an
  execution outcome; and
- whether disabled and inapplicable rules appear in the aggregate result.

No evidence supports user-editable configuration, persistence, precedence
across sources, environment selection, or protocol/UI administration.

## Execution, cancellation, and result decisions

ADR-0059 must define:

- exact immutable execution context and canonical inputs;
- synchronous or asynchronous rule seam;
- sequential versus concurrent execution;
- cancellation signal ownership and observation boundaries;
- whether a rule failure stops all work, blocks dependents only, or allows
  independent continuation;
- whether an engine-level invariant failure returns no result;
- exact terminal per-rule status vocabulary;
- complete aggregate ordering and summary reconciliation;
- input, rule, dependency, configuration, diagnostic, message, anchor,
  provenance, result, and error bounds;
- closed redacted domain versus rule failures;
- panic containment if executable Rust rules are object-safe trait objects;
- repeated execution and cleanup guarantees; and
- whether partial execution is representable and how it is distinguished from
  a complete successful run.

The current Workspace build is sequential and blocking, and current diagnostics
orchestration is synchronous and stateless. That is useful precedent for a
deterministic first slice, but it does not by itself decide rule failure or
cancellation semantics.

## Diagnostic-result integration candidates

### Add a typed Rule diagnostic family

This makes rule findings available through the existing complete report and
projections, but requires an explicit additive migration of
`DiagnosticFamily`, `DiagnosticCode`, `DiagnosticKind`,
`DiagnosticIdentity`, `DiagnosticEvidence`, constructors, summaries,
filters, error bounds, MCP argument/result vocabulary, LSP projection tests,
cache semantic compatibility, and every exhaustive match.

The rule identity and finding identity must remain distinct. The ADR must define
whether one finding identity contains rule ID plus rule-local code and semantic
anchors, how cross-rule collisions work, and whether equal diagnostic content
from two rules is one or two findings.

### Keep typed rule evidence separate

This avoids migrating ADR-0058 in the first slice but makes production rule
results invisible to existing diagnostics consumers unless a new public surface
is accepted later. The Workspace may still expose an in-process
`RuleExecutionReport`, but MCP/LSP behavior would remain unchanged.

### Reuse Semantic or Validation families

Rejected by current evidence. It would falsify the producer and typed source
vocabulary, weaken identity, and blur Graph or validator authority.

## Public consumer and compatibility inventory

| Consumer | Current dependency | Potential migration if rule findings are integrated |
|---|---|---|
| Analysis public API | Diagnostics types and `DiagnosticEngine` | Additive `rules` API and possibly exhaustive diagnostic-family migration. |
| Workspace snapshot | Graph, validation, and `DiagnosticReport` accessors | Add immutable registry/configuration/execution result accessors and compose complete results before publication. |
| Persistent cache | Canonical source/graph/producer DTOs; derived diagnostics recomputed | Add deterministic invalidation/recompute inputs or a versioned schema; semantic version cannot remain unchanged if equal source bytes produce different snapshot semantics. |
| File watching | Rebuild and atomically replace a complete snapshot | Rule planning/execution must finish before replacement; failure recovery and generation counters remain truthful. |
| MCP `oneagent.diagnostics` | Families `semantic|validation`, complete summary, limit 100, seven-tool catalog, Tool Policy | A Rule family requires schema/argument/result and modern/legacy parity tests but no new tool is required. |
| LSP pull diagnostics | Active `DiagnosticFinding` values with exactly one confined span, complete limit 100 | Rule findings require one accepted anchor/location and must preserve omission, confinement, bounds, capability, and lifecycle. |
| HTTP, CLI, VS Code, EDT | Workspace/Runtime lifecycle or unchanged MCP catalog | Must remain compatible unless ADR-0059 proves an exact changed payload contract. |
| Graph/adapter/Coverage | Canonical facts and source-produced evidence | No rule result may change facts, producer evidence, or capability status in Sprint 37. |

## Deterministic testability matrix

| Area | Required cases and oracle |
|---|---|
| Identity and registry | Empty, valid, invalid, duplicate, conflict, reordered, exact/over bound, stable accessors, error redaction, and repeated construction through focused Analysis tests. |
| Dependencies | Independent, chain, diamond, missing, self, duplicate, incompatible, cycle, reordered, exact/over bound, and repeated planning with exact typed order assertions. |
| Configuration | Default, explicit, disabled, unknown, duplicate, incompatible, exact/over bound, registry mismatch, and deterministic equality. |
| Applicability | Applicable, inapplicable, disabled, unsupported if accepted, and dependency-blocked without conflating them with execution failure. |
| Execution | Empty, success, order, failure, independent continuation or fail-closed behavior, downstream blocking, cancellation at each accepted checkpoint, result bound, panic policy if applicable, repetition, and cleanup. |
| Diagnostic integration | Every accepted rule code/kind/severity/category, anchors/provenance, duplicate and cross-rule conflict, suppression, summary, ordering, exact/over bound, missing location, and error redaction. |
| Snapshot and cache | Cold/warm equality, semantic/configuration/registry invalidation, corruption, write failure, clean recovery, atomic publication, rebuild failure, and repeated fresh-process reuse. |
| Watching and lifecycle | Reordered/repeated source generations, accepted rule failure, retained previous snapshot where required, cancellation, shutdown, observer status, and no detached work. |
| MCP/LSP | Exact catalog/capability truth, Tool Policy, arguments, modern/legacy parity, bounds, confinement, malformed input, public process, EOF, cancellation, channel purity, and cleanup if the payload changes. |
| Compatibility | Graph, Diagnostics Engine, Workspace, cache, HTTP, CLI, VS Code, EDT, adapters, Coverage, dependency, API, sensitive-data, and documentation audits. |

The baseline commands executed during this investigation all exited zero:

| Command | Result |
|---|---|
| `cargo test -p oneagent-analysis --lib diagnostics::` | 25 passed; 27 unrelated tests filtered; no failure or ignored test. |
| `cargo test -p oneagent-analysis --test diagnostics_engine` | 3 passed; no filtered test. |
| `cargo test -p oneagent-runtime --test workspace_service` | 6 passed. |
| `cargo test -p oneagent-runtime --test persistent_cache` | 4 passed. |
| `cargo test -p oneagent-runtime --test file_watching` | 2 passed. |
| `cargo test -p oneagent-runtime --test mcp_semantic_tools` | 7 passed. |
| `cargo test -p oneagent-runtime --test lsp_stdio` | 5 passed. |

Every named target executed at least one test. No zero-match filter or skipped
required row is used as evidence.

## Likely implementation and validation areas

ADR-0059 must pin exact paths, but current evidence identifies these affected
areas:

- `crates/analysis/src/`, `crates/analysis/tests/`, and public exports for
  rule domain, registry, planning, execution, results, and diagnostic mapping;
- `apps/runtime/src/workspace/mod.rs` and focused Workspace unit/integration
  tests for composition and immutable access;
- `apps/runtime/src/workspace/cache.rs` and persistent-cache tests for
  invalidation, recomputation or schema behavior;
- `apps/runtime/src/mcp_tools.rs`, `apps/runtime/src/lsp.rs`, protocol and
  public-process tests only if the accepted diagnostic result changes;
- `docs/Architecture.md`, `docs/architecture/semantic-model-2.md`,
  `README.md`, and this Roadmap for verified current-state synchronization;
- existing Graph, diagnostics, Workspace, cache, watcher, Runtime, MCP/LSP,
  HTTP/CLI, adapter, IDE, EDT, Coverage, dependency, and full workspace
  regression matrices.

Current manifests already provide the likely dependency direction:
`oneagent-analysis -> oneagent-graph` and
`oneagent-runtime -> oneagent-analysis`. No new external dependency is
required by confirmed evidence. Any different crate or dependency decision
requires explicit justification and current user approval before production
use.

## ADR-0059 decision checklist

ADR-0059 is decision-ready only when it explicitly fixes:

1. engine owner and dependency direction;
2. exact canonical immutable inputs and prohibited source reads/mutations;
3. rule identity, metadata, registration owner/lifecycle, duplicate/conflict
   behavior, enumeration, and bounds;
4. dependency meaning, validation, canonical order, cycles, and bounds;
5. configuration authority, defaults, identity, validation, compatibility,
   applicability, and bounds;
6. execution seam, ordering, cancellation, failure containment, partial versus
   complete behavior, panic policy, and cleanup;
7. per-rule and aggregate terminal result types, ordering, summaries, and
   bounds;
8. diagnostic mapping or explicit first-slice separation, including identity,
   family/code/kind, collisions, suppression, anchors, provenance, locations,
   sensitive data, and bounds;
9. immutable Workspace publication, cache serialization or recomputation,
   invalidation, watching, lifecycle, and public compatibility;
10. first production registry and conformance rule policy;
11. public API and migration impact;
12. repository-owned focused, integration, public-process, and complete
    validation evidence; and
13. rejected alternatives and deferred scope.

## First-slice and deferred boundaries

Repository evidence supports a bounded source-independent engine and synthetic
or built-in repository-owned conformance rules. It does not identify an
unimplemented semantic check that may safely be promoted to a production rule
without a separate meaning and producer decision. ADR-0059 must therefore
choose explicitly between an empty production registry with executable
repository conformance rules in tests and one evidence-backed built-in rule.
It must not disguise an existing Graph validator or recoverable producer
diagnostic as that first rule.

Deferred unless separately accepted:

- dynamic plugins, third-party rule SDK, scripting, remote acquisition, hot
  reload, mutable global registration, and filesystem discovery;
- user/project configuration grammar, environment precedence, persistent
  preferences, profiles, baselines, directives, protocol administration, and
  settings UI;
- new semantic facts, parsing, validation authority, graph mutation, source
  reads, source mutation, fixes, code actions, refactoring, and safe edits;
- new MCP tools, LSP capabilities, IDE UI, mutable documents, remote transport,
  authentication, telemetry, performance, security, and release claims;
- Git Change Adapter and Sprint 38 implementation.

## Decision readiness

The mandatory data and testability gate passes for ADR-0059 and the planned
implementation sequence. Canonical immutable inputs, consumers, integration
points, bounds, compatibility constraints, deterministic fixtures, focused
tests, cache/rebuild/lifecycle oracles, and public-process seams are present in
the repository. The remaining questions are architecture decisions, not
missing external data.

No accepted architecture conflicts with live source evidence. Production
behavior remains unchanged by this investigation.
