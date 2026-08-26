# ADR-0049: Source-Independent Tool Execution Policy

## Status

Accepted

## Context

Sprint 27 must define and enforce a safe AI tool execution boundary before MCP,
IDE chat, or a concrete tool catalog exists. The boundary must fail closed,
distinguish side effects, require exact confirmation when policy says so, and
prove that denied work never reaches an executor. Acceptance cannot depend on a
live provider, credential, external service, privileged operation, or real
destructive or externally visible action.

The [Tool Execution Policy investigation](../architecture/tool-execution-policy-investigation.md)
confirms that the repository has no tool identity, request, effect, rule,
decision, confirmation, executor, outcome, or audit domain. ADR-0037 gives
Runtime ownership of Tokio task lifecycle; ADR-0044 keeps Context Engine output
independent from execution; ADR-0045 keeps tool authorization outside the
provider-neutral LLM crate and exposes no tool-capable model. Runtime,
`oneagent-llm`, Analysis, protocol, CLI, and concrete providers have no current
tool-policy consumer.

The same investigation proves that Rust 1.97.1 can express the first slice with
owned bounded values, redacted formatting, canonical `BTreeSet` values, boxed
borrowed futures, receiver-only cancellation, and deterministic fake executors
using only the standard library.

## Decision

### Canonical statement

A tool request is executable only when one current `ToolPolicy` evaluation owns
that exact validated request and yields `Allow`, or yields
`RequireConfirmation` and the execution gate consumes the one confirmation
issued for that same authorization. `Deny`, no matching rule, missing or
mismatched confirmation, stale policy binding, and cancellation before an
attempt perform no executor call. A request, model output, tool declaration,
prior decision, or audit record is never authorization by itself.

### Ownership and dependency direction

Create the workspace library crate `crates/tool-policy` with package name
`oneagent-tool-policy`. It owns source-independent tool identities, sensitive
arguments, side effects, requests, authorization policies and decisions,
confirmation values, executor substitution, cancellation input, terminal
results, errors, and audit evidence.

The crate has no production dependency, including no dependency on common,
graph, Analysis, Context Engine, LLM, Runtime, protocol, Tokio, Serde, an MCP
library, a clock, an executor SDK, or a concrete tool. The root workspace adds
the local member; `Cargo.lock` may change only mechanically for that package.

Dependency direction is:

```text
future provider/MCP/IDE/CLI/Runtime callers ──▶ oneagent-tool-policy
future concrete tool executors ──────────────▶ oneagent-tool-policy
oneagent-tool-policy ──X──▶ callers, transports, providers, Runtime, tools
```

No external production dependency or Cargo feature is approved or required for
Sprint 27. Task 3 therefore needs no additional dependency approval unless the
implementation discovers a contradiction and stops before changing Cargo.

### Public module and value boundary

The crate may split implementation into private modules, but its public boundary
contains these concepts with the stated names unless a Rust constraint requires
an equivalent documented name:

- `ToolId`, `ActorId`, `ToolRequestId`, and `PolicyRevision`;
- `ToolArguments`, `ToolEffect`, and `ToolRequest`;
- `ActorScope`, `ToolScope`, `RuleAction`, `ToolRule`, `ToolPolicy`,
  `AuthorizationDecisionKind`, `AuthorizationDecisionReason`, and
  `ToolAuthorization`;
- `ToolConfirmationChallenge` and `ToolConfirmation`;
- `ToolFuture`, `ToolCancellationSignal`, `NeverCancelled`, `ToolExecutor`,
  `ToolExecutorOutcome`, `ToolOutput`, and `ToolDiagnostic`;
- `ToolExecutionResult`, `ToolAuditRecord`, `ConfirmationState`,
  `ToolTerminalOutcome`, and `ToolDenialReason`;
- `ToolPolicyErrorKind` and `ToolPolicyError`.

No public value implements Serde in Sprint 27. There is no wire format, stable
serialization, ABI, policy-file, or transport claim.

### Stable bounds

All string limits count UTF-8 bytes. Accepted inclusive maxima are:

| Value | Maximum UTF-8 bytes |
|---|---:|
| tool identifier | 128 |
| actor identifier | 128 |
| request identifier | 128 |
| policy revision | 128 |
| tool arguments | 65,536 |
| tool output | 65,536 |
| retained tool diagnostic | 512 |

One policy contains at most 4,096 input rules before duplicate
canonicalization. Numeric accounting uses checked arithmetic. Bounds are local
safety and testability contracts, not a statement about MCP, a concrete tool,
an operating system, context capacity, or external service limits.

### Identity validation

`ToolId`, `ActorId`, `ToolRequestId`, and `PolicyRevision` are separate strong
types. Each owns one case-sensitive string, preserves accepted bytes without
normalization, and rejects in this precedence:

1. empty or all-whitespace;
2. value over its exact UTF-8 byte maximum;
3. leading or trailing Unicode whitespace;
4. any Unicode control character.

They implement `Debug`, `Display`, `Clone`, equality, total order, and hash.
They are safe correlation labels, not secret containers; callers must not place
credentials, commands, file contents, URLs with secrets, or personal data in
them. Rejected raw values never enter errors or diagnostics.

`ToolRequestId` uniqueness is a caller obligation within the future execution
scope. The crate does not create randomness, a global sequence, durable nonce,
or cross-process replay registry.

### Sensitive arguments and request construction

`ToolArguments` owns zero to 65,536 UTF-8 bytes and preserves them exactly.
Empty arguments are valid for a zero-argument tool. It has one explicit
`expose()` accessor and a byte-length accessor. It does not implement `Clone`,
`Copy`, `Display`, equality, order, hash, or serialization. `Debug` reports only
the byte count.

`ToolRequest` owns one validated request ID, actor ID, tool ID, argument value,
and canonical non-empty effect set. It is not cloneable or comparable. Its
explicit accessors expose the exact arguments and other accepted components;
its `Debug` exposes safe identities, canonical effects, and argument byte count
only.

Request construction performs no I/O, policy evaluation, confirmation, tool
classification inference, or execution. The caller is responsible for
supplying a truthful effect declaration; future concrete tool conformance must
prove that declaration against real behavior.

### Side-effect vocabulary and validation

The closed Sprint 27 `ToolEffect` enum is:

- `ReadOnly` — observes state without intentionally changing it;
- `LocalMutation` — changes local process or workspace state;
- `ExternalMutation` — changes a remote or third-party-visible system;
- `Destructive` — can delete, overwrite, irreversibly replace, or otherwise
  make recovery materially difficult;
- `Privileged` — requires elevated or specially trusted authority;
- `SensitiveDataExposure` — can disclose credential, private, or otherwise
  sensitive content to another boundary.

A request holds a `BTreeSet<ToolEffect>` in enum order. Duplicate input is
deduplicated. Empty input is invalid. `ReadOnly` combined with any other effect
is contradictory and invalid. Every other combination is allowed so mixed
effects remain visible. Unknown serialized values are not representable and
must be rejected by future adapters rather than mapped to `ReadOnly`.

This vocabulary classifies declared risk; it does not prove sandboxing,
reversibility, idempotency, atomicity, operating-system permission, or the
truthfulness of a concrete tool declaration.

### Rule scopes, canonical policy, and matching

`ActorScope` and `ToolScope` each contain `Any` and `Exact` with the matching
strong identity. `RuleAction` is `Deny`, `RequireConfirmation`, or `Allow`.
One `ToolRule` contains one actor scope, one tool scope, one exact
`ToolEffect`, and one action. There is no path, host, argument, time, role,
group, network, environment, or custom expression language in Sprint 27.

`ToolPolicy::new` accepts one policy revision and zero to 4,096 rules. It:

1. rejects input over the rule-count maximum before allocation-dependent
   canonical work;
2. sorts rules by actor scope, tool scope, effect, then action;
3. deduplicates only fully identical rules;
4. retains conflicting actions so evaluation can apply conservative precedence.

An empty policy is valid and denies every request by default. Rules are
immutable after construction. The crate reads no environment, file, database,
Runtime configuration, or global state.

For every effect declared by the request, evaluation considers every rule whose
actor scope and tool scope match and whose effect equals that effect. Wildcards
have no override priority. The request-wide decision precedence is:

1. if any matching rule is `Deny`, return `Deny` with `ExplicitDeny`;
2. otherwise, if any request effect has no matching rule, return `Deny` with
   `NoMatchingRule`;
3. otherwise, if any matching rule is `RequireConfirmation`, return
   `RequireConfirmation` with `ConfirmationRequired`;
4. otherwise every effect has at least one matching `Allow`, so return `Allow`
   with `Allowed`.

Caller rule order, duplicate rules, a more-specific allow, a prior success, or
an audit entry cannot override a matching deny. Evaluation performs no I/O,
clock read, logging, confirmation, or executor call.

### Authorization ownership and binding

`ToolPolicy::evaluate` consumes one `ToolRequest` and returns one
`ToolAuthorization` that owns the exact request plus an inspectable decision
bound to the policy revision. Consuming the request prevents a caller from
substituting different arguments after evaluation. `ToolAuthorization` does not
implement `Clone` or public construction.

The decision exposes kind, reason, policy revision, safe request identities,
canonical effects, and argument byte count. It never exposes argument content
through implicit formatting. A decision is evidence for the execution gate,
not permission to invoke a concrete executor directly.

### Confirmation boundary and replay behavior

Only a `RequireConfirmation` authorization can issue a
`ToolConfirmationChallenge`. `ToolAuthorization::take_confirmation_challenge`
accepts mutable access and succeeds at most once. The challenge privately binds
the policy revision and the exact request ID, actor, tool, effects, and argument
bytes. Its `Debug` never reveals argument content.

`ToolConfirmationChallenge::confirm(self)` consumes the challenge and produces
one non-cloneable `ToolConfirmation`. This call represents the trusted future
caller's explicit confirmation boundary; the crate does not authenticate a
human, display UX, or determine who may call it.

The execution gate consumes the authorization and optional confirmation. A
confirmation from another request, actor, tool, effect set, argument payload,
or policy revision is rejected before executor construction. Missing
confirmation is rejected. A confirmation supplied to `Allow` or `Deny` is also
rejected conservatively. Safe Rust move semantics prevent reuse of the same
confirmation, and one authorization cannot issue a second challenge.

Re-evaluating the same caller-supplied request ID can issue another challenge;
cross-evaluation, cross-process, durable, authenticated, and time-based replay
prevention are deferred. There is no trusted clock, so Sprint 27 defines no
wall-clock expiry. A different policy revision or any different bound request
value is stale by exact mismatch.

### Executor and cancellation boundary

`ToolFuture<'a, T>` is a boxed borrowed `Send` future expressed with `std`.
`ToolCancellationSignal` is a receiver-only interface with an immediate state
query and a future that becomes ready when cancellation is requested.
`NeverCancelled` is the stateless default test value.

`ToolExecutor` is object-safe and exposes one operation that borrows the exact
validated `ToolRequest` plus the cancellation signal and returns one
`ToolExecutorOutcome`. The executor receives no policy rules or confirmation
value and cannot convert an untrusted request into an authorization.

The public execution gate consumes `ToolAuthorization` and an optional
`ToolConfirmation`, then borrows one executor and cancellation signal. Terminal
precedence is:

1. authorization denial;
2. confirmation missing, mismatch, or unexpected confirmation;
3. cancellation already requested;
4. one executor attempt;
5. in-flight cancellation before the executor outcome when both are ready in
   the same poll;
6. the executor outcome.

Denied, invalid-confirmation, and pre-cancelled paths do not call
`ToolExecutor::execute`. An accepted path calls it exactly once. The gate never
retries, falls back, sleeps, spawns background work, or retains the executor
future after a terminal result. Dropping the losing future is the cleanup
boundary that deterministic fakes must prove.

### Timeout, retry, and failure containment

The std-only Sprint 27 gate has no clock and does not represent or enforce a
duration. `ToolExecutorOutcome::TimedOut` lets a future concrete executor report
that its accepted owner-enforced timeout elapsed. The gate maps it exactly and
does not retry. Runtime or a concrete executor must later define any duration,
clock, timer, and simultaneous timeout/cancellation precedence before claiming
timeout enforcement.

`ToolExecutorOutcome` contains:

- `Completed(ToolOutput)`;
- `Partial(ToolOutput)`;
- `Failed(Option<ToolDiagnostic>)`;
- `TimedOut`.

`ToolOutput` owns zero to 65,536 UTF-8 bytes, exposes content only explicitly,
is not cloneable or implicitly displayable, and has content-free `Debug`.
`ToolDiagnostic` owns one non-empty redacted string up to 512 bytes, is
explicitly accessible, and is omitted from implicit error/result formatting.

`Partial` states that the executor reports some effect before failure; the gate
does not infer what changed or attempt rollback. Panics, process termination,
untruthful executor classifications, and external atomicity are outside the
domain contract.

### Terminal result and audit evidence

Every gate call returns one owned `ToolExecutionResult` containing one safe
`ToolAuditRecord`, optional explicitly accessible `ToolOutput`, and optional
explicitly accessible `ToolDiagnostic`.

`ToolTerminalOutcome` contains `Denied`, `Completed`, `Partial`, `Failed`,
`TimedOut`, and `Cancelled`. `ToolDenialReason` contains
`AuthorizationDenied`, `ConfirmationMissing`, `ConfirmationMismatch`, and
`UnexpectedConfirmation`. `ConfirmationState` contains `NotRequired`,
`Missing`, `Confirmed`, `Rejected`, and `NotApplicable`.

`ToolAuditRecord` contains exactly:

- request ID, actor ID, tool ID, and policy revision;
- canonical effect set and argument byte count;
- authorization kind and reason;
- confirmation state;
- attempt count, constrained to zero or one;
- terminal outcome and output byte count.

It contains no arguments, output content, diagnostic text, timestamp, duration,
credential, URL, command, path, provider body, source error, or arbitrary
metadata. It is an observation and cannot be supplied to evaluation or
execution as authorization. One record is already in deterministic field order;
there is no global log, mutable audit sink, retention policy, persistence, or
export in Sprint 27.

### Error taxonomy and validation failures

`ToolPolicyErrorKind` contains distinct invalid-identity kinds, invalid
arguments, invalid effect set, invalid policy, invalid confirmation operation,
invalid output, invalid diagnostic, and internal contract failure. Construction
errors use static diagnostics and never echo rejected input. Policy evaluation
and execution terminal states are data, not exceptions.

Identity validation uses the precedence defined above. `ToolArguments` and
`ToolOutput` validate only their byte maximum because empty and arbitrary UTF-8
content are accepted. Effect-set validation canonicalizes duplicates, then
rejects empty, then contradictory `ReadOnly`. Policy construction checks input
rule count before sorting/deduplication. Confirmation issuance checks decision
kind before already-issued state.

No public error retains an unrestricted source error. `Display` uses only its
stable kind; `Debug` exposes kind and bounded diagnostic presence/length, never
diagnostic content.

### Deterministic conformance and compatibility

The package test corpus must contain non-zero unit and public integration
targets. Public conformance uses only exported values and at least two
substitutable fake executors or modes. Fakes use counters and drop guards, can
complete, fail, report partial/timeout, or wait for cancellation, and perform no
filesystem, shell, Git, network, database, provider, privileged, destructive,
or external action.

Tests cover exact bounds and Unicode bytes, identifier precedence, empty and
mixed effects, canonical rules, duplicates, conflicts, all decision outcomes,
default deny, reordering, policy revisions, confirmation success/missing/
mismatch/one-use, zero-call denial, exactly one accepted attempt, pre-existing
and in-flight cancellation, completed/partial/failed/timeout outcomes, output
and diagnostic bounds, audit completeness, redaction, cleanup, and repeated
operations.

Complete `oneagent-llm`, `oneagent-analysis`, and affected
`oneagent-runtime` targets remain compatibility evidence. Those crates and all
provider adapters remain unchanged. The full workspace validation is required
for Tasks 3-6 and integration review.

## Rejected alternatives

### Put Tool Policy in `oneagent-llm`

Rejected because authorization applies beyond provider tool-call wires and
ADR-0045 explicitly defers tool policy outside the LLM domain.

### Put Tool Policy in Runtime

Rejected because it would couple reusable domain values to Tokio and
application lifecycle before Runtime composition is in scope.

### Put Tool Policy in protocol or MCP

Rejected because no current wire schema exists and transport values must not
become authorization authority.

### Use first-match or specificity-over-deny rules

Rejected because input ordering or a narrower allow could weaken a matching
deny. Global deny and confirmation precedence is deterministic and
conservative.

### Store a cryptographic argument digest

Rejected because the repository has no accepted hash algorithm or dependency
for this domain. Exact request ownership and private confirmation binding avoid
a false collision-resistance claim.

### Use a boolean authorization callback

Rejected because it cannot represent default denial, confirmation, policy
revision, exact request ownership, terminal outcomes, or audit evidence.

### Claim human confirmation or clock expiry

Rejected because Sprint 27 has no authenticated confirmer, UX, trusted clock,
or durable replay store. The first slice proves only explicit API-boundary
confirmation, exact matching, single issuance, and safe-Rust one-use.

### Enforce timeout, rollback, or sandboxing in the policy crate

Rejected because those require concrete runtime and executor authority. The
first slice maps executor-reported timeout and partial completion without
inventing reversibility or isolation.

## Deferred scope

- provider model capability changes and tool-call wire formats;
- concrete filesystem, shell, Git, network, database, 1C, MCP, IDE, CLI,
  browser, or other tool schemas and implementations;
- Runtime composition, task spawning, service lifecycle, timeout clock,
  concurrency limits, configuration, persistence, and audit delivery;
- authenticated actors or confirmers, roles/groups, policy administration,
  policy files, remote policy, UX, time expiry, durable nonce/replay protection,
  and cross-process execution;
- argument-aware rules, path/host/resource scopes, effect inference, dynamic
  risk scoring, tool catalogs, aliases, policy merge, caching, and refresh;
- automatic model tool selection, prompts, conversations, chains, loops,
  retries, fallback, rollback, transactions, sandboxing, OS permission,
  idempotency, atomicity, and compensation;
- real effects, external compatibility, security/compliance certification,
  latency, throughput, benchmarks, and performance claims;
- protocol, MCP server/tools, IDE/CLI UI, Semantic Coverage Registry, graph,
  metadata, BSL, workspace, and source-adapter changes.

## Implementation prerequisites and completion criteria

1. Task 3 creates only the accepted std-only package foundation, domain values,
   errors, and focused evidence; it does not implement policy or execution.
2. Task 4 implements canonical fail-closed evaluation and request-owning
   authorization with no executor call.
3. Task 5 implements confirmation issuance/binding, the cancellation-aware
   one-attempt gate, terminal result, audit record, and deterministic fakes.
4. Task 6 adds public conformance and current-state documentation without a real
   tool or new consumer.
5. No external production dependency or feature is required. A discovered need
   for one is a blocker requiring explicit user approval and architecture
   reconciliation before Cargo changes.
6. Sprint 27 may claim the bounded Tool Execution Policy library only after
   non-zero focused/public tests, unchanged LLM/Analysis/Runtime compatibility,
   the full workspace gate, and the independent Sprint 27 integration-review
   procedure pass.

Architecture acceptance alone does not mark Sprint 27 completed or make any
concrete tool, Runtime, provider, MCP, IDE, CLI, security, or real-effect claim.
