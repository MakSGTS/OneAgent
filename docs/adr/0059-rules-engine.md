# ADR-0059: Rules Engine

## Status

Accepted.

## Context

Sprint 36 established a source-independent Diagnostics Engine in
`oneagent-analysis`. Graph remains authoritative for semantic facts,
recoverable diagnostics, validation, provenance, source locations, reports, and
diffs. The Diagnostics Engine normalizes exactly Semantic and Validation
evidence into a bounded deterministic `DiagnosticReport`, and Runtime publishes
that report with each immutable Workspace Configuration snapshot.

Sprint 37 must define deterministic rule identity, registration, dependencies,
configuration, execution, and results without reclassifying Graph validators,
parser checks, Tool Policy rules, or producer diagnostics as general analysis
rules. The repository has no existing Rules Engine, registry, rule dependency
graph, rule configuration source, or aggregate rule result.

The decision evidence is recorded in the
[Rules Engine investigation](../architecture/rules-engine-investigation.md).
The dedicated reusable task contract is recorded by the Rules Engine
[profile](../codex/profiles/rules-engine-implementation.md),
[workflow](../codex/workflows/rules-engine.md), and
[template](../codex/templates/rules-engine-task.md).

## Decision

OneAgent adds a source-independent deterministic Rules Engine owned by
`oneagent-analysis::rules`. It evaluates an immutable validated registry over
borrowed canonical snapshot evidence and returns one complete bounded
`RuleExecutionReport`.

The Rules Engine does not parse source, discover files, mutate Graph, invoke
Graph validation, change build validity, create Graph facts, suppress
diagnostics, project protocols, apply fixes, or own Runtime lifecycle.

## Ownership and dependency direction

The accepted direction is:

```text
oneagent-graph
    -> oneagent-analysis::diagnostics
    -> oneagent-analysis::rules
        -> oneagent-runtime Workspace composition
            -> existing MCP and LSP diagnostic projections
```

`oneagent-analysis` already depends on Graph and owns the source-independent
Context and Diagnostics engines. Runtime already depends on Analysis. No Cargo
manifest, external dependency, feature, or license change is required.

Graph remains authoritative for semantic evidence and validation. Diagnostics
remains authoritative for normalized finding identity, suppression, ordering,
summary, and complete reports. Rules owns only rule domain, registry,
configuration, dependency planning, execution, terminal results, and
rule-produced evidence before diagnostic normalization.

Runtime owns production registry selection, immutable snapshot composition,
cache reconstruction, watching, lifecycle, cancellation integration, and
protocol adapters.

## Canonical immutable execution context

One rule evaluation borrows one `RuleContext` containing exactly:

- one `&SemanticGraph`;
- the complete caller-supplied `&SemanticGraphValidationResult` for that same
  build; and
- the base `&DiagnosticReport` containing only ADR-0058 Semantic and
  Validation evidence.

The context contains no Workspace root, source path, source content, mutable
graph handle, parser/adapter value, cache handle, Runtime service, protocol
request, credential, or clock.

The engine verifies that the registry, plan, configuration, context, and every
returned anchor satisfy the accepted contracts. It does not rerun validation or
reconstruct the base report. Rule-produced diagnostics are normalized only
after execution, so a rule never consumes its own output or another
rule-produced finding in this first slice.

## Rule identity and codes

`RuleId` and `RuleDiagnosticCode` are distinct validated owned UTF-8 values.
Each accepts 1 through 128 ASCII bytes.

The grammar is:

- the first and last byte are a lowercase ASCII letter or digit;
- interior bytes may also contain `.`, `-`, or `_`;
- two separator bytes may not be adjacent; and
- uppercase, whitespace, control characters, non-ASCII, leading/trailing
  separators, and empty segments are rejected.

Rule IDs are globally unique within one registry. Codes are local to one rule.
Stable public strings are the exact validated bytes. IDs and codes have total
lexicographic order and never depend on registration order, collection index,
process identity, path, source location, or hash iteration.

The first slice defines no UUID, numeric ID, editable alias, version suffix,
localization, display name, description, category catalog, or persistence key.

## Definition, registration, and registry

One immutable `RuleDefinition` contains:

- one `RuleId`; and
- zero or more required dependency `RuleId` values.

Definition construction sorts dependencies and collapses exact repeated
dependency IDs. A self-dependency is retained for registry validation so the
error is classified at the complete-graph boundary.

One object-safe `Rule` is `Send + Sync` and exposes:

- an immutable `&RuleDefinition`; and
- one synchronous bounded evaluation operation over `&RuleContext` and
  `&dyn RuleCancellationSignal`.

`RuleRegistry` owns immutable shared rule objects, validates the complete set,
and exposes them in ascending complete `RuleId` order. The registry is created
once and is immutable after construction. Empty registries are valid.

Two registrations with the same ID always reject the registry:

- equal definitions are `DuplicateRule`;
- different definitions are `ConflictingRule`.

The engine never selects a duplicate by insertion, pointer, hash, or source
order. Executable behavior is not compared, serialized, hashed, or treated as
data.

The hard registry limit is 4,096 input registrations. The limit applies before
duplicate collapse or rejection.

## Dependencies and deterministic plan

A dependency means both:

1. the dependency rule must appear earlier in the canonical plan; and
2. it must terminate as `Completed` before the dependent rule may execute.

Dependencies do not transfer arbitrary result values in the first slice.
Rules consume only the common immutable context.

Plan construction validates the complete registry and configuration before
execution:

- a missing dependency is `MissingDependency`;
- a self-dependency is `SelfDependency`;
- a dependency cycle is `DependencyCycle`;
- the same exact dependency listed repeatedly is one dependency;
- every dependency relationship uses exact `RuleId` equality; and
- no partial plan is returned after failure.

Each rule accepts at most 256 unique dependencies, and the complete registry
accepts at most 65,536 unique dependency relationships.

The plan is a deterministic topological order. When multiple rules are ready,
the smallest complete `RuleId` runs first. Equivalent definitions,
registrations, dependencies, and configuration produce an equal ordered plan
independently from input order.

## First-slice configuration

`RuleConfiguration` is one immutable source-independent in-memory value.
It contains zero or one `RuleSetting` per exact registered `RuleId`.

The only setting is `Enabled` or `Disabled`. Absence means `Enabled`.
There are no parameters, severity/category overrides, dependency overrides, or
execution-limit overrides.

Construction from input settings:

- rejects an input count above 4,096;
- rejects any repeated exact rule ID as `DuplicateSetting`, including equal
  repeated values;
- sorts accepted settings by `RuleId`; and
- exposes deterministic equality and iteration.

Plan construction rejects a setting for an unregistered rule as
`UnknownConfiguredRule`. Disabled rules remain in the plan and produce an
observable `Disabled` terminal result. A rule that depends on a Disabled rule
becomes `Blocked`.

Production Workspace uses an empty default configuration. Sprint 37 adds no
file grammar, environment variable, provider field, preference, persistence,
precedence, profile, baseline, directive, protocol setting, or UI.

## Applicability and evaluation

The synchronous rule operation returns one closed `RuleEvaluation`:

- `Completed(Vec<RuleDiagnostic>)`;
- `NotApplicable`; or
- `Failed(RuleFailureCode)`.

`RuleFailureCode` uses the same 1–128 byte grammar as a rule diagnostic code.
It is stable bounded metadata, not a message or error chain.

Rules may inspect only the borrowed context. They must be deterministic,
side-effect-free, bounded, and panic-free for equal context and configuration.
They may not read source, filesystem, environment, network, time, randomness,
global mutable state, cache, Runtime, transports, or another rule's result.

`NotApplicable` is a successful evaluation decision but is not
`Completed`; dependents are therefore blocked. A completed rule may return
zero diagnostics.

## Cancellation

`oneagent-analysis::rules` owns a minimal source-independent
`RuleCancellationSignal: Send + Sync` with
`is_cancelled(&self) -> bool`, plus `NeverCancelled`.

The engine checks cancellation:

1. before each rule;
2. immediately after each rule returns; and
3. while transitioning remaining plan entries.

The signal is also supplied to the rule for cooperative checks during bounded
synchronous work. If cancellation is observed after evaluation, that rule's
unpublished diagnostics are discarded, it becomes `Cancelled`, and every
remaining rule becomes `Cancelled`. No new rule begins.

Sprint 37 does not preempt a synchronous rule. A rule that ignores cancellation
must still terminate within its accepted bounded operation.

Production Workspace uses `NeverCancelled` inside the existing ADR-0039
non-interruptible blocking snapshot build. Runtime cancellation remains
observed after that build joins, exactly as before. The Rules Engine signal
establishes a reusable in-process contract without changing current startup or
watch rebuild cancellation semantics.

## Failure containment and terminal results

One valid plan produces exactly one `RuleResult` per registered rule in plan
order. The closed `RuleStatus` is:

- `Disabled`;
- `NotApplicable`;
- `Completed`;
- `Blocked`;
- `Failed`; or
- `Cancelled`.

Before evaluation:

- a disabled rule becomes `Disabled`;
- a rule with any dependency not `Completed` becomes `Blocked`;
- otherwise it is evaluated.

A rule-level `Failed` result blocks its dependents but does not stop
independent rules. The same is true for Disabled, NotApplicable, Blocked, and
Cancelled dependencies. Independent rules continue after a rule failure until
cancellation.

`RuleExecutionReport` owns canonical plan-ordered results, the canonical
ordered rule diagnostics from Completed results, and checked counts by terminal
status. Every registered rule appears exactly once and every count reconciles
with the total. Disabled, NotApplicable, Blocked, Failed, and Cancelled outcomes
are complete observable results, not engine errors.

Registry, plan, configuration, bound, duplicate/conflict, invalid anchor,
invalid output, or aggregate reconciliation failures are closed
`RuleEngineError` values and return no report.

Rule panics violate the `Rule` contract and are not caught by Analysis in the
first slice. No untrusted or dynamically loaded rule exists. At the production
Workspace boundary, an unexpected panic remains contained by the existing
owned blocking-build join failure and cannot publish a partial snapshot.

## Rule diagnostic evidence

`RuleDiagnostic` is typed source-independent derived evidence containing:

- the producing `RuleId`;
- one local `RuleDiagnosticCode`;
- normalized `DiagnosticSeverity`;
- one existing normalized `DiagnosticCategory`;
- a bounded message;
- zero or more canonical graph node anchors.

It contains no path, source content, raw reference, edge/request anchor,
credential, arbitrary properties, fix, edit, or protocol payload.

The Rules Engine validates each returned diagnostic before accepting the rule
as Completed:

- message length is at most 4,096 UTF-8 bytes;
- node anchors are sorted, unique, and limited to 256;
- every anchor exists in the supplied `SemanticGraph`;
- observed provenance count is derived from the anchored Graph nodes and
  limited to 256;
- one rule returns at most 4,096 input diagnostics; and
- the complete report accepts at most 65,536 rule diagnostics.

Exact equal rule diagnostics collapse. Equal typed identity with different
severity, category, message, or derived provenance count is invalid output and
fails that rule with the closed engine-owned failure code
`invalid_rule_output`; no diagnostics from that rule are retained.

## Additive Diagnostics Engine migration

ADR-0058 is extended additively with a third `DiagnosticFamily::Rule`.

The additive typed values are:

- `DiagnosticCode::Rule(RuleDiagnosticCode)`;
- `DiagnosticKind::Rule` with stable string `rule_finding`;
- `DiagnosticIdentity::Rule { rule_id, code, node_anchors }`; and
- `DiagnosticEvidence::Rule(RuleDiagnostic)`.

Rule identity and diagnostic identity remain distinct. Two different rules
producing equal code, anchors, severity, category, and message remain two
findings because `rule_id` participates in identity. One rule producing equal
code and anchors with different observable content is a conflict.

Rule findings participate in the unchanged complete report order:
disposition, severity, category, family, typed identity, then observable
content. They use the same exact-identity suppression policy, checked summary,
filtering, message/anchor/provenance bounds, and complete 65,536-finding limit.

The existing `DiagnosticEngine::build` signature and behavior remain
available and supply no Rule inputs. A new additive
`build_with_rules` entry point accepts the same Semantic and Validation
inputs plus a bounded immutable Rule diagnostic slice. It preserves one
complete collision and summary boundary.

No Rule finding changes Graph validation, graph reports, build diffs, raw
recoverable diagnostics, or rule terminal status.

## Production registry and first rule boundary

No unimplemented semantic rule is promoted without its own evidence. Production
Workspace uses a committed empty `RuleRegistry` and empty default
`RuleConfiguration` in Sprint 37.

Repository-owned test conformance rules exercise positive, dependency,
configuration, applicability, failure, cancellation, diagnostic, conflict,
bound, reorder, and repetition behavior through the public Rules Engine seam.
They are test fixtures, not hidden production semantics.

An empty production registry produces one complete empty
`RuleExecutionReport` and no Rule diagnostics. This is an implemented engine
contract, not a claim that a product rule exists. Adding the first non-empty
production rule requires a later evidence-backed task and does not reopen the
engine architecture unless it needs a new canonical input or result contract.

## Workspace composition and atomicity

`WorkspaceConfigurationSnapshot` gains a read-only
`rule_execution_report()` accessor. Snapshot construction order is:

1. construct and validate canonical graph and producer evidence;
2. construct complete Graph validation;
3. build the base ADR-0058 Semantic/Validation report;
4. construct the production registry, default configuration, and canonical
   plan;
5. execute rules over graph, validation, and the base report;
6. build the final DiagnosticReport with accepted Rule diagnostics; and
7. publish only the complete mutually consistent snapshot.

Any engine, diagnostic, bound, or consistency error follows the existing
semantic-build failure policy. No partial graph, rule report, or diagnostic
report is published. Rebuilds complete the same sequence privately before
atomic replacement.

The snapshot does not expose executable rule objects or mutable configuration.
The empty production registry and default configuration are composition inputs,
not snapshot state.

## Persistent cache and invalidation

Cache schema remains `1`. Rule definitions, executable objects, configuration,
plans, execution reports, and Rule diagnostics are not serialized.

After decoding canonical Graph and producer evidence, Runtime reruns the same
validation, base diagnostic, empty production registry, default configuration,
rule execution, and final diagnostic composition. Cold, warm, rebuilt, and
repeated snapshots expose equal validation, rule reports, and diagnostic
reports.

The private semantic compatibility version advances from `3` to `4` in the
same Task 6 change because equal source bytes now produce an additive rule
execution report and Rule-capable final diagnostic semantics. Version `3`
entries are intentionally invalidated. Byte-level compatibility is not claimed.

Any future non-empty registry or non-default production configuration must
participate in deterministic cache invalidation or cause another semantic
compatibility advance before use.

## MCP projection

The MCP catalog remains exactly seven lexicographically ordered tools.
`oneagent.diagnostics` remains read-only and Tool Policy gated.

Its optional `families` vocabulary adds `rule` to
`semantic|validation`. A Rule item uses:

- `family: "rule"`;
- `code`: the local stable Rule diagnostic code;
- `kind: "rule_finding"`;
- existing severity, category, message, disposition, and `nodeIds`; and
- additive `ruleId`.

`ruleId` is absent from Semantic and Validation items. Existing fields,
complete unfiltered summary, default and maximum limits, ordering, truncation,
argument validation, modern/legacy revision envelopes, catalog, annotations,
Tool Policy, lifecycle, framing, and error precedence remain unchanged.

The schema truthfully supports an empty Rule result in Sprint 37. No new tool,
rule catalog, configuration, enable/disable, execute-now, result-history,
explanation, fix, or edit surface is advertised.

MCP exposes no root, path, source content, raw reference, opaque provenance,
failure code, rejected value, or internal chain.

## LSP projection

LSP retains the exact 3.17 pull-diagnostic capability and payload shape.

An Active Rule finding is projectable only when it has exactly one canonical
node anchor and existing Workspace confinement resolves that node to one exact
span for the requested URI. Missing, zero, multiple, conflicting, span-less,
escaping, incompatible, suppressed, or different-document evidence is omitted.

The LSP code is the local Rule diagnostic code, source remains `oneagent`, and
severity/message retain current normalized mapping. Rule ID, family, category,
identity, provenance, terminal status, and configuration are not added to the
LSP payload. The complete located active-result limit remains 100 and over-bound
requests remain fail-closed.

No LSP capability, synchronization, push/workspace diagnostic, result ID,
refresh, code action, fix, or edit is added.

## Bounds

The first-slice hard bounds are:

| Item | Limit |
|---|---:|
| Rule ID bytes | 128 |
| Rule diagnostic or failure code bytes | 128 |
| Input registrations | 4,096 |
| Unique dependencies per rule | 256 |
| Unique dependencies per registry | 65,536 |
| Input configuration settings | 4,096 |
| Rule diagnostics returned by one rule | 4,096 |
| Total Rule diagnostics | 65,536 |
| Diagnostic message bytes | 4,096 |
| Diagnostic node anchors | 256 |
| Observed provenance records | 256 |
| Total final DiagnosticReport findings | 65,536 |

Counts use checked arithmetic. Exact bounds are accepted and one-over is
rejected before unbounded cloning or publication.

## Errors and sensitive data

Domain and engine errors use closed kinds plus bounded actual/maximum counts or
stable IDs only when the public contract explicitly requires an ID. They do
not echo:

- diagnostic messages;
- source content, paths, roots, spans, references, or candidates;
- rejected configuration;
- graph payloads or provenance records;
- credentials, environment, cache bytes, protocol input, or executor state; or
- internal error chains from a rule.

`Debug` and `Display` obey the same boundary. Rule failure codes are
observable only in the in-process RuleExecutionReport; they are not projected
through MCP or LSP in Sprint 37.

## Public API and compatibility impact

- Graph public APIs, validation, facts, reports, diffs, producers, adapters,
  and Coverage remain unchanged.
- Analysis gains additive `rules` APIs and additive Rule variants in its
  Diagnostics domain.
- Existing `DiagnosticEngine::build` remains source compatible; exhaustive
  consumers of public diagnostic enums must add the Rule variant.
- Workspace Configuration snapshots gain an additive immutable rule-report
  accessor and final diagnostic reports become Rule-capable.
- Cache schema and canonical serialized fields remain unchanged; semantic
  compatibility advances to `4` and derived values are recomputed.
- MCP keeps seven tools and adds the `rule` family vocabulary plus optional
  Rule item `ruleId`.
- LSP capability and payload shape remain unchanged.
- HTTP, CLI, VS Code, EDT, Context, Impact, adapters, graph query, and Tool
  Policy behavior remain unchanged.

No stable API versioning claim is made. All affected exhaustive matches,
schemas, fixtures, documentation, and public-process tests must migrate in the
same implementation sequence.

## Deterministic evidence

Completion requires non-zero tests for:

- ID/code grammar, exact/over bounds, equality, order, and redaction;
- empty, single, reordered, duplicate, and conflicting registries;
- missing, self, repeated, chain, diamond, cycle, independent, exact/over
  dependency plans and canonical ready-set ordering;
- default, explicit, duplicate, unknown, disabled, and exact/over configuration;
- Disabled, NotApplicable, Completed, Blocked, Failed, and Cancelled results;
- independent continuation, dependency blocking, cancellation before/after
  evaluation, invalid output, panic containment at Workspace, repetition, and
  cleanup;
- Rule diagnostic family/code/kind/identity/evidence, duplicates, conflicts,
  suppression, summaries, filters, bounds, anchors, provenance, missing
  location, and mixed-family order;
- Workspace empty-production report, atomic construction, rebuild, watcher,
  observer, shutdown, and failure behavior;
- cache version-3 invalidation, cold/warm equality, corruption, write failure,
  clean recovery, and repeated fresh-process reuse;
- MCP schema/family/item, Tool Policy, seven-tool catalog, revision parity,
  bounds, malformed input, public processes, EOF, cancellation, channel purity,
  and cleanup;
- LSP empty Rule family compatibility, located/unlocated Rule findings through
  controlled in-process evidence, exact/over bounds, confinement, lifecycle,
  and public processes; and
- Graph, Diagnostics, Workspace, Runtime, cache, HTTP, CLI, MCP/LSP, VS Code,
  EDT, adapter, Coverage, dependency, API, sensitive-data, documentation, and
  complete workspace regressions.

A zero-match filter, skipped required row, timing-dependent oracle, live
service, external network, credential, or developer-local state is not
acceptance evidence.

## Implementation sequence

Task 3 implements Rule IDs/codes, definitions, registry, bounds, errors, public
exports, Rustdoc, and focused registry evidence only.

Task 4 implements configuration, dependency validation, canonical planning,
applicability state, bounds, and focused planning evidence only.

Task 5 implements the rule trait, context, cancellation signal, sequential
execution, terminal results, diagnostic evidence, additive Diagnostics Engine
migration, and focused public conformance evidence.

Task 6 integrates the empty production registry and default configuration into
Workspace snapshots, recomputes derived values after cache decode, advances
semantic compatibility to `4`, migrates MCP diagnostics, preserves LSP
capability/payload behavior, and proves lifecycle/public-process compatibility.

Task 7 completes the requirement matrix, focused/public/full validation,
consumer/dependency/sensitive-data/scope audits, and current-state
documentation without changing accepted behavior.

## Coverage impact

Sprint 37 changes no Semantic Coverage Registry capability or count. The Rules
Engine consumes existing facts and evidence and adds no graph node, edge,
parser, source producer, or supported semantic capability. An empty production
registry does not claim a product rule.

## Rejected alternatives

- Graph-owned Rules Engine is rejected because Graph already owns facts and
  validation, not configurable higher-level execution.
- Runtime-owned rule domain is rejected because Runtime owns composition,
  lifecycle, cache, and transports rather than reusable analysis semantics.
- A new crate is rejected because current dependencies have no cycle and
  Analysis already owns the appropriate source-independent layer.
- Reusing `ToolRule` is rejected because Tool Policy owns external side-effect
  authorization and intentionally does not depend on Graph or Analysis.
- Repackaging Graph validators or producer diagnostics as rules is rejected
  because it duplicates or falsifies canonical authority.
- Reusing Semantic or Validation diagnostic variants is rejected because rule
  evidence has a different producer and identity.
- Keeping rule diagnostics permanently separate is rejected because Sprint 37
  requires integration with accepted diagnostic evidence and existing bounded
  projections provide the deterministic product seam.
- Result-consuming dependencies are deferred; the common immutable context is
  sufficient for the first slice.
- Concurrent rule execution is rejected because no performance evidence
  requires nondeterministic scheduling complexity.
- Fail-all on one rule failure is rejected; contained failure with dependent
  blocking preserves deterministic independent evidence.
- Catching panics inside Analysis is rejected for the trusted built-in first
  slice; Workspace already contains blocking-task panics atomically.
- External configuration and persisted results are rejected because no
  repository-owned grammar, lifecycle, or compatibility contract exists.
- A synthetic production rule is rejected because it would advertise behavior
  without semantic meaning; test-only conformance rules prove the engine.

## Deferred scope

Deferred beyond Sprint 37:

- non-empty production rules and their individual semantic decisions;
- dynamic plugins, third-party SDK, scripts, remote rules, hot reload, mutable
  global registration, filesystem discovery, and untrusted code;
- rule parameters, project/user configuration grammar, environment precedence,
  preferences, profiles, baselines, directives, persistence, protocol
  administration, and UI;
- result-consuming dependencies, concurrency, scheduling policy, timeouts,
  retries, quotas, performance, and security claims;
- new Graph facts, parser/adapter inputs, validation authority, source reads,
  mutable documents, source mutation, fixes, code actions, refactoring, safe
  edits, and Git Change Adapter behavior;
- new MCP tools, LSP capabilities, IDE UI, remote transport, authentication,
  telemetry, publication, and stable API guarantees.

## Consequences

OneAgent gains a deterministic bounded Rules Engine contract with a complete
typed result even when production has no registered rules. The engine can add
future evidence-backed built-in rules without changing registration,
dependency, configuration, execution, or diagnostic semantics.

The additive Rule diagnostic family expands exhaustive Analysis and MCP
consumers and advances cache semantic compatibility. This is deliberate:
rule-produced evidence remains distinguishable from Graph and validation
evidence while sharing one accepted suppression, ordering, summary, location,
bound, and projection boundary.
