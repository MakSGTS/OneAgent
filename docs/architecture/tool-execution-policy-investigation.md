# Tool Execution Policy Investigation

## Status

Complete repository investigation for Sprint 27 architecture planning.

This document records evidence and decision inputs. It does not accept an
architecture or claim Tool Execution Policy support.

## Investigation baseline

- Repository root: `/Users/maxim_tomshin/Development/oneagent`.
- Planning prerequisite: `1bfaf2d495823685b889c1c17aa6aa9b31e94a75`.
- Framework prerequisite: `d31e9a43420223329578e0890c3da361b1270366`.
- Rust: `1.97.1 (8bab26f4f 2026-07-14)`.
- Cargo: `1.97.1 (c980f4866 2026-06-30)`.
- Initial working tree: clean.
- Network, live providers, credentials, and real tool execution were not used.

## Authoritative constraints

- [ADR-0037](../adr/0037-runtime-service-container.md) owns Runtime service
  lifecycle, Tokio task supervision, startup rollback, cancellation, shutdown,
  and cleanup. A source-independent policy crate must not become a second
  Runtime or own application services.
- [ADR-0044](../adr/0044-context-engine.md) keeps Context Engine output
  deterministic, derived, and independent from provider or tool execution.
  Context selection is not authorization.
- [ADR-0045](../adr/0045-llm-provider-abstraction.md) owns provider-neutral model
  discovery and text generation. Its only accepted model capability is
  `TextGeneration`, and tool policy, tool calls, Runtime composition, and
  concrete provider wires are deferred. A provider request or model output is
  not authorization.
- The [Sprint 23 review](../reviews/sprint-23-llm-provider-abstraction.md)
  confirms that `oneagent-llm` is std-only and that Analysis and Runtime do not
  depend on it. The [Sprint 26 review](../reviews/sprint-26-ollama-integration.md)
  confirms that provider adapters still do not integrate tools or Runtime.
- The committed [AI Tool Policy workflow](../codex/workflows/ai-tool-policy.md)
  requires fail-closed authorization, conservative side effects, exact
  confirmation binding, decision-to-execution gating, bounded redacted audit
  evidence, failure containment, and repository-owned fake executors.

## Confirmed repository evidence

### Existing ownership and dependency graph

The root workspace contains semantic, source-adapter, Analysis, Runtime,
protocol, CLI, LLM, and concrete-provider packages. Repository searches found
no existing Rust definition or Cargo package for a tool-policy request,
side-effect class, authorization decision, confirmation challenge or receipt,
tool executor, or tool audit record.

`oneagent-llm` has no normal dependency. Its public boundary owns provider and
model identities, text requests and responses, provider configuration and
secrets, represented timeout/no-retry policy, cooperative cancellation, boxed
borrowed futures, typed failures, and `LlmProvider`. Reverse normal dependency
inspection finds only the three concrete provider adapters. Analysis and
Runtime remain independent.

`oneagent-runtime` owns Tokio-based application lifecycle and service
cancellation. Its `Cancellation` handle is receiver-only but is built on a
Runtime-private Tokio watch channel. Reusing it would force a source-independent
policy library to depend on the application and Tokio. `oneagent-llm` exposes a
std-only receiver interface, but depending on the LLM domain would incorrectly
make provider semantics authoritative for all tools.

`oneagent-protocol` is an empty std-only package with no tool wire contract.
Future Sprint 28 and Sprint 29 MCP work therefore has no current schema that can
serve as Tool Execution Policy authority.

### Reusable implementation patterns

Repository code proves these language and test patterns on Rust 1.97.1:

- owned bounded strong identifiers with exact validation precedence and total
  ordering;
- canonical ordered sets using `BTreeSet` and deterministic duplicate handling;
- sensitive owned strings with explicit access and custom redacted `Debug`;
- request and response `Debug` implementations that expose byte counts rather
  than content;
- stable closed error kinds with bounded explicitly accessed diagnostics;
- object-safe substitution with
  `Pin<Box<dyn Future<Output = T> + Send + 'a>>` and no async-trait dependency;
- receiver-only cooperative cancellation and deterministic manual polling;
- fake-operation counters and drop guards that prove zero invocation, one
  attempt, cancellation, and cleanup without sleeps or external state;
- public integration tests that use only exported domain values and fakes.

The CI matrix runs stable Rust on `macos-14` and `windows-latest`. A std-only
crate and fake corpus avoid platform-specific filesystem, shell, permission,
clock, and networking behavior.

### Current compatibility constraints

- `ModelCapability` must remain unchanged in Sprint 27; adding a tool capability
  would claim provider wire support that no adapter implements.
- `TextGenerationRequest`, provider input/output, and Context Engine output must
  not be embedded in a tool-policy request. A future caller may derive a tool
  request, but it must cross an explicit source-independent boundary.
- Runtime, protocol, CLI, provider adapters, graph, workspace, and source
  adapters have no current tool-policy consumer. The first slice can therefore
  be additive and prove substitution without changing those packages.
- Existing Runtime and provider cancellation types are evidence for behavior,
  not reusable identity or authorization authority.
- No Semantic Coverage Registry entry governs AI tools. Sprint completion must
  use package conformance and current-state documentation rather than inventing
  a graph/EDT coverage transition.

## Required domain inputs

ADR-0049 needs separate owned identities for at least:

- the requested tool;
- the actor or initiating principal presented to policy evaluation;
- the request/correlation instance;
- the policy revision used to make a decision.

Each identity needs an explicit UTF-8 byte maximum, empty/whitespace/control
validation, case and normalization behavior, safe formatting, ordering, and
error precedence. Existing 128-byte provider identifiers are evidence that the
pattern works, but they do not decide Tool Policy limits or permit type reuse.

The executor needs exact arguments, while policy and audit generally need only
bounded metadata. Raw arguments may contain prompts, file contents, commands,
URLs, credentials, or third-party data. The architecture must choose between an
opaque bounded argument payload and a bounded digest/summary. Standard-library
Rust has no repository-owned cryptographic hash contract, so a digest must not
be invented or treated as collision-resistant identity. An opaque owned payload
with explicit access, content-free `Debug`, byte length, and request-instance
identity is implementable without a dependency; audit evidence can retain only
the request identity and byte count.

## Side-effect classification decision inputs

The first slice needs a closed conservative vocabulary that distinguishes at
least observation from mutation and identifies effects that cannot be silently
auto-allowed. Repository evidence does not select exact names or whether the
domain is a severity hierarchy or an ordered set.

Candidate A is one maximum-severity class such as read-only, local mutation,
external mutation, destructive, or privileged. It is simple and totally
ordered, but a maximum loses independent dimensions such as external plus
destructive.

Candidate B is a canonical non-empty set of orthogonal flags such as reads
state, mutates local state, changes an external system, destructive or
irreversible, privileged, or exposes sensitive data. It preserves mixed effects
but requires explicit invalid combinations and deterministic policy matching.

Unknown, empty, malformed, mixed, or understated classifications cannot be
treated as read-only. Either construction must reject them before evaluation or
policy must deterministically deny them. Concrete tool classification and proof
that a declaration matches real behavior remain future executor obligations.

## Actor, scope, rule, and decision inputs

The repository has no authenticated AI actor, role assignment, policy store,
rule language, or configuration precedence. ADR-0049 can safely define an owned
actor identity presented by a future trusted caller, but must not claim that it
authenticates a person or model.

The smallest deterministic policy can use immutable validated rules over exact
actor, exact tool, and accepted effect scope, with explicitly bounded wildcard
forms only when necessary. Decisions need three non-success-equivalent states:
explicit deny, allow, and confirmation required. No matching rule must be a
distinct default-deny reason, not an allow.

Order-sensitive first-match semantics are unsafe because caller reordering can
change authorization. Evidence supports canonical rule sorting followed by a
closed conflict rule where deny dominates confirmation required, which
dominates allow, or an exact-specificity model with the same conservative tie
handling. ADR-0049 must select one and define duplicates, conflicting matches,
unknown effects, policy revisions, and validation precedence.

A decision must be an inspectable value bound to the exact request identity,
tool, actor, effect classification, and policy revision. It must not be a
boolean, and it must not itself invoke a tool. Reusing an old decision after any
bound value changes must fail closed.

## Confirmation trust and replay inputs

There is no current user interface, authenticated confirmer, trusted clock,
durable nonce store, or confirmation transport. The first slice can prove
structural binding and one-use ownership but cannot prove that a human approved
the action.

An implementable candidate is an opaque confirmation value created from one
confirmation-required decision and consumed by value by the execution gate. It
copies no arguments and retains only safe bound identities. Moving rather than
borrowing the confirmation prevents ordinary reuse in safe Rust. The gate must
still reject a confirmation for another request, actor, tool, effect set, or
policy revision.

Time-based expiry is not deterministic without a trusted clock. ADR-0049 should
either defer expiry and state that a changed policy revision makes evidence
stale, or accept an injected monotonic time source with exact semantics. The
repository has no reason to introduce a clock for the first slice. Persistence,
cross-process replay prevention, confirmer authentication, and confirmation UX
remain deferred.

## Execution, timeout, cancellation, and outcome inputs

A source-independent executor must be substitutable and must receive a validated
request only after the gate accepts the current decision and any required
confirmation. A boxed borrowed future can support later asynchronous executors
without Tokio or an async-trait dependency and matches proven repository style.

The gate can prove:

- denied and unconfirmed paths create zero executor future/call attempts;
- an accepted path invokes the executor at most once;
- pre-existing and in-flight cooperative cancellation win according to an
  explicit precedence;
- an executor reports completed, failed, partial, timeout, or cancelled terminal
  evidence through a closed bounded result;
- the losing future/drop guard releases operation state;
- no automatic retry, fallback, rollback, or background work exists.

A std-only domain cannot enforce wall-clock timeout without an injected clock or
executor/runtime implementation. ADR-0049 can represent a bounded timeout and
require concrete executors to report `Timeout`, or omit timeout configuration
and retain timeout only as a terminal executor failure. It must not claim clock
enforcement by the policy crate.

Partial completion cannot be rolled back generically. It needs a distinct
terminal classification with no unrestricted result body and must not be
reported as denial or success. A concrete executor remains responsible for
truthfully mapping its effects.

## Audit evidence inputs

Audit history cannot authorize future work. The first slice can return one
owned execution report that contains only bounded stable facts such as request
identity, actor, tool, policy revision, decision kind/reason, confirmation state,
attempt count, and terminal outcome. It may include argument and result byte
counts when the accepted domain exposes them, but not unrestricted arguments,
output, secret values, source errors, URLs, commands, or provider payloads.

One report per gated operation is simpler and safer than a mutable global audit
sink. Internal phase records may be kept in a canonical fixed order only when
every attempted operation ends with exactly one terminal record. Persistence,
delivery, retention period, timestamps, external correlation IDs, and audit
export belong to future Runtime/protocol work.

## Alternatives for ADR-0049

### Add a dedicated std-only Tool Policy crate

This preserves source independence, keeps Runtime and provider dependencies
outward, supports future MCP/IDE consumers, and uses already proven Rust
patterns. It is the smallest evidence-backed candidate.

### Add Tool Policy to `oneagent-llm`

Rejected as a default candidate. Tool authorization applies beyond provider
tool-call wires, while ADR-0045 explicitly keeps tool policy outside the LLM
domain. It would also force non-LLM tools to depend on provider concepts.

### Add Tool Policy to Runtime

Rejected as a default candidate. It would couple reusable policy values and
tests to Tokio and application lifecycle before any Runtime composition is in
scope.

### Add Tool Policy to protocol or future MCP packages

Rejected as a default candidate. No current wire schema exists, and transport
values must not become authorization authority.

### Implement only a boolean callback

Rejected. A boolean cannot preserve denial reason, confirmation required,
policy revision, exact request binding, terminal outcome, or auditable evidence.

## Deterministic acceptance oracle

Repository acceptance can use a std-only public fake executor with explicit
atomic counters, captured safe identities, configurable terminal mode, a
pending-until-cancelled mode, and a drop guard. No fake performs filesystem,
shell, Git, network, database, provider, or external action.

The minimum matrix must cover, as applicable after ADR-0049 selects exact
values:

- maximum and over-maximum identifiers and arguments, Unicode byte boundaries,
  empty, boundary-whitespace, control-character, malformed-effect, and
  redaction cases;
- canonical rule order, duplicates, multiple matches, conflicts, explicit deny,
  default deny, confirmation required, allow, unknown/missing scope, policy
  revision changes, and repeated evaluation;
- exact confirmation success plus absent, wrong request, wrong actor/tool/effect,
  stale revision, duplicate, and compile-time/runtime replay prevention;
- zero executor calls for all denied/unconfirmed paths and exactly one attempt
  for accepted paths;
- completed, failed, partial, invalid, pre-cancelled, in-flight cancelled, and
  timeout-reported outcomes, one terminal report, redaction, drop cleanup, and
  repeated operations;
- public conformance through exported values and a substitutable executor;
- unchanged complete `oneagent-llm`, `oneagent-analysis`, and affected
  `oneagent-runtime` targets plus the full workspace gate.

Zero matched filters are not evidence. Tests must use no sleeps, environment,
credentials, ignored cases, external network, local services, developer paths,
or real side effects.

## Decision readiness

Repository evidence is sufficient for ADR-0049. The ADR must decide:

1. exact crate/package name and std-only dependency boundary;
2. public identities, bounds, validation precedence, and sensitive argument
   representation;
3. side-effect set or hierarchy and conservative malformed handling;
4. actor/scope/rule vocabulary, canonicalization, conflicts, and precedence;
5. decision binding and stable reasons;
6. confirmation construction, one-use/replay/staleness behavior, and explicit
   non-claim about human authentication;
7. executor future, cancellation, timeout, no-retry, cleanup, and failure
   precedence;
8. terminal outcome and bounded audit report vocabulary;
9. deterministic public conformance and compatibility matrix;
10. deferred Runtime, provider, MCP, IDE/CLI, persistence, concrete-tool, real-
    effect, security, rollback, and performance scope.

No external artifact, live service, credential, or user-provided source data is
missing for that decision. Exact tool classifications and real execution safety
must be proven later by the concrete tool owners and cannot be inferred here.
