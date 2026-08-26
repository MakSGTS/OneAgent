# Sprint 27 Tool Execution Policy Review

## Decision

`pass`

The effective decision matches the independent reviewer recommendation. Sprint
27 satisfies the accepted ADR-0049 source-independent fail-closed library
boundary: bounded requests and effects, deterministic authorization, exact
one-use confirmation, cancellation-aware one-attempt execution, closed terminal
results, and content-free audit evidence. No blocking or non-blocking defect or
missing evidence was found for the accepted slice.

This decision does not claim a concrete tool, real side effect, Runtime or MCP
integration, authenticated confirmer, clock-enforced timeout, persistence,
sandbox, rollback, or cross-process replay prevention.

## Reviewed baseline

- Framework baseline: `d31e9a43420223329578e0890c3da361b1270366`.
- Planning commit: `1bfaf2d4a48935a8b01b7ad6acac6b120d91c8c9`.
- Review head: `64954f4965d74c2df1c807a2f2b78967fe1f6af5`.
- Exact reviewed range: `1bfaf2d4^..64954f49`.
- Resolved range parent:
  `d31e9a43420223329578e0890c3da361b1270366`.
- Initial independent review status: clean.
- Final independent review status: clean.
- Initial primary review status: clean.
- Range size: 7 commits, 25 paths, 4,866 additions, 20 deletions.

The dependency-ordered commits are:

| Step | Commit | Subject | Result |
| --- | --- | --- | --- |
| Planning | `1bfaf2d4` | `Plan Sprint 27 Tool Execution Policy` | pass |
| Investigation | `3bfe0a1f` | `Investigate Sprint 27 tool execution policy` | pass |
| ADR-0049 | `6b71bb72` | `Define Sprint 27 tool execution policy` | pass |
| Request domain | `4173778d` | `Implement Sprint 27 tool request domain` | pass |
| Authorization | `feb35363` | `Implement Sprint 27 authorization policy` | pass |
| Confirmed execution | `eac8b841` | `Implement Sprint 27 confirmed execution` | pass |
| Public evidence and current-state docs | `64954f49` | `Complete Sprint 27 tool policy evidence` | pass |

The range is limited to the committed Sprint 27 plan, investigation, ADR,
additive `oneagent-tool-policy` crate and tests, workspace member/lock
registration, and the three authorized current-state documents. It does not
change LLM, Analysis, Runtime, protocol, CLI, graph, metadata, BSL, workspace,
source adapters, or provider adapters.

## Independent reviewer handoff and report

- Reviewer task: `/root/sprint27_reviewer`.
- Context: fresh, with `fork_turns: "none"`; no implementation conversation,
  expected decision, proposed finding, or primary rationale was inherited.
- Authority supplied: repository root, exact range and observed HEAD, committed
  Sprint 27 objective/scope/criteria/matrix, and the authoritative review,
  Roadmap, ADR, investigation, Runtime, Context, LLM, semantic-model, and prior
  review documents.
- Operating constraint: read-only, no delegation, no edit/create/delete,
  staging, commit, or state transition.
- Recommendation: `pass`.
- Blocking findings: none.
- Non-blocking findings: none.
- Missing evidence for the accepted boundary: none.
- Initial/final HEAD:
  `64954f4965d74c2df1c807a2f2b78967fe1f6af5`.
- Initial/final `git status --short`: empty.
- Working-tree discrepancy: none.
- Read-only/delegation confirmation: the reviewer reported no repository-owned
  mutation, staging, commit, state change, or delegated work; only ordinary
  ignored Cargo build artifacts were touched by validation.

The independent reviewer inspected all seven commits and every changed path,
resolved the range parent to the framework baseline, matched commit subjects and
task ownership to the Roadmap manifest, and reported complete evidence for every
criterion below.

## Acceptance evidence matrix

| Criterion | Independent and primary evidence | Result |
| --- | --- | --- |
| Planning and order | The exact range contains planning plus Tasks 1-6 in dependency order; subjects and task-owned paths match `docs/Roadmap.md`. | pass |
| Investigation | `docs/architecture/tool-execution-policy-investigation.md` records ownership, dependency, identities, sensitivity, effects, rules, confirmation, execution, terminal/audit, consumers, portability, rejected assumptions, and deterministic fake evidence without selecting unsupported production behavior. | pass |
| Accepted architecture | ADR-0049 is `Accepted` and fixes ownership, values, effects, policy precedence, confirmation, execution, outcomes, audit, errors, conformance, compatibility, and deferrals before Rust implementation. | pass |
| Owner and dependencies | `oneagent-tool-policy` is one additive std-only crate with no normal/dev dependency or feature; direct and reverse normal trees contain only itself and there is no external consumer. | pass |
| Public surface | `crates/tool-policy/src/lib.rs` exports the required ADR concepts while implementation modules, confirmation binding, audit construction, and executor orchestration details remain private; no serialization contract exists. | pass |
| Identities and bounds | Tool, actor, request, and policy-revision strong identities use exact 128-byte limits and accepted validation precedence without echoing rejected input. | pass |
| Requests and effects | Opaque arguments accept at most 65,536 UTF-8 bytes through explicit access; request effects canonicalize all six closed variants and reject empty or contradictory `ReadOnly` combinations. | pass |
| Policy construction | The 4,096 input-rule bound precedes sort/dedup; canonical order is stable, identical rules deduplicate, and conflicting actions remain for conservative precedence. | pass |
| Authorization | Exact scopes plus global deny, uncovered-effect default deny, confirmation-over-allow, and complete allow coverage produce a request-owning non-cloneable decision without execution. | pass |
| Confirmation | Only `RequireConfirmation` can issue one challenge; non-cloneable consuming evidence privately binds revision, request ID, actor, tool, canonical effects, and exact argument bytes and redacts content. | pass |
| Execution gate | Deny, missing, mismatched, stale, and unexpected confirmation terminate before executor construction; pre-cancellation also produces zero calls. Accepted paths construct exactly one future and never retry, fall back, sleep, or spawn work. | pass |
| Cancellation and cleanup | Cancellation is polled before an executor outcome in the same poll; dropping the losing future is the cleanup boundary and counter/drop-guard fakes prove zero retained active work. | pass |
| Terminal result and timeout | Completed, Partial, Failed, executor-reported TimedOut, and Cancelled map to one typed result. The crate contains no clock or timeout-duration enforcement and makes no rollback claim. | pass |
| Audit and redaction | Audit contains exactly safe identities/revision, canonical effects, argument/output byte counts, authorization/confirmation states, zero-or-one attempts, and terminal class; implicit formatting omits arguments, output, and diagnostics. | pass |
| Unit evidence | 26 non-zero unit tests cover construction, Unicode bounds, precedence, canonicalization, confirmation, zero/one calls, result matrix, cancellation, cleanup, redaction, and repetition. | pass |
| Public evidence | 7 non-zero exported-API tests use only standard-library fakes and cover bounded construction, deny/default deny, exact/stale confirmation, zero/one attempts, terminal outcomes, audit ordering, redaction, cancellation, cleanup, and repetition. | pass |
| Compatibility | `oneagent-llm` passes 22 unit + 7 public tests; `oneagent-analysis` passes 27 unit + 11 public tests; `oneagent-runtime` passes 78 library + 25 integration tests. These packages are unchanged and have no Tool Policy dependency. | pass |
| Documentation and prompt ownership | README, Architecture, and Semantic Model describe the implemented library and explicit deferrals without a premature completion claim. Sprint 27 and Sprint 26 suites each have exactly 8 tracked/filesystem files with no untracked addition; the current suite and `run-next-sprint.md` are intact. | pass |
| Complete validation | Independent and primary focused, compatibility, audit, and canonical workspace gates pass; no required Tool Policy target or filter is zero-match. | pass |

## Findings

### Blocking

None.

### Non-blocking

None.

## Missing evidence

None for the accepted ADR-0049 first slice.

The independent and primary reviews intentionally did not require a live
provider or MCP/IDE client, credential or external network, real filesystem/
shell/Git/database/tool effect, destructive or privileged action,
third-party-visible mutation, authenticated confirmer, confirmation UX, policy
administration, persistence or audit export, clock-based timeout enforcement,
sandbox, rollback, transaction, idempotency, atomicity, performance, security,
compliance, or cross-process/durable replay prevention. These are explicit
exclusions, not missing completion evidence.

No ignored test was executed separately because both static audits found zero
`#[ignore]` or `ignore =` matches, and all executed targets reported zero
ignored tests.

## Independent validation ledger

The reviewer reported these exact outcomes:

- `cargo test -p oneagent-tool-policy --lib -- --list` — 26 tests, exit 0.
- `cargo test -p oneagent-tool-policy --test conformance -- --list` — 7 tests,
  exit 0.
- `cargo test -p oneagent-tool-policy --lib` — 26 passed, exit 0.
- `cargo test -p oneagent-tool-policy --test conformance` — 7 passed, exit 0.
- `cargo test -p oneagent-llm` — 22 unit + 7 public passed, exit 0.
- `cargo test -p oneagent-analysis` — 27 unit + 11 public passed, exit 0.
- Initial sandbox `cargo test -p oneagent-runtime` — exit 101: 74 library
  tests passed and four loopback-bind tests failed with
  `PermissionDenied: Operation not permitted`.
- Escalated `cargo test -p oneagent-runtime` — 78 library + 25 integration
  tests passed, exit 0.
- `cargo fmt --all -- --check` — exit 0.
- `cargo check --workspace` — exit 0.
- Initial sandbox `cargo test --workspace` — exit 101 after two existing CLI
  loopback tests failed with `PermissionDenied: Operation not permitted`.
- Escalated `cargo test --workspace` — every workspace unit, integration, and
  doc-test target passed, exit 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — exit
  0.
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps` — exit 0.
- `git diff --check` and `git diff --check 1bfaf2d4^..64954f49` — exit 0.

The four Runtime sandbox failures were
`http_bind_failure_is_a_named_service_start_failure`,
`http_service_enforces_get_only_routes_and_default_fallback`,
`http_service_publishes_address_serves_health_and_releases_listener`, and
`query_enabled_http_serves_all_routes_through_the_owned_listener`. The two
workspace sandbox failures were
`executor_classifies_unreachable_and_malformed_responses_and_repeats` and
`executor_sends_exact_request_and_preserves_success_and_server_bodies`. All six
passed on the approved loopback-enabled retries; they are environment
restrictions, not Sprint defects.

Independent zero-match audits returned `rg` exit 1, as expected, for ignored
tests, environment/live/network/process/filesystem/credential use, real-effect/
clock/background operations, Serde, `unsafe`, external Rust/Cargo consumers,
and production incomplete markers. A broad incomplete-marker scan found only
two deliberate `panic!` assertions in `#[cfg(test)]` manual polling helpers.
The reviewer also checked 278 local Markdown links across 14 relevant documents
with zero missing targets and verified both prompt inventories as `8/8/0`.

## Primary validation and reconciliation

The primary review started from the same clean HEAD, independently inspected the
same seven commits and 25 paths, and reproduced the reviewer conclusions:

- Tool Policy enumeration: 26 unit and 7 public tests, both non-zero.
- Tool Policy execution: 26 unit and 7 public tests passed.
- LLM: 22 unit and 7 public tests passed.
- Analysis: 27 unit and 11 public tests passed.
- Runtime: 78 library and 25 integration tests passed with approved local
  loopback access.
- Direct and reverse normal dependency trees contain only
  `oneagent-tool-policy`; no external consumer was found.
- Public-surface, sensitive-trait, scope/path, ignored-test, live/environment,
  real-effect, incomplete-marker, link, and prompt-inventory audits matched the
  independent report.
- The canonical workspace `fmt`, `check`, `test`, `clippy`, Rustdoc, and diff
  checks all passed.

No reviewer criterion, command result, scope conclusion, risk, finding, or
missing-evidence conclusion conflicts with primary evidence. The effective
decision therefore remains `pass`; it is not less severe than the independent
recommendation.

Neither validation path contacted a live provider, used a credential or
external network, or executed a real tool effect. Approved escalation was used
only for repository-owned controlled loopback tests.

After the consistency check, Roadmap/current-state transition, and exact prompt
retirement, the canonical complete gate passed again:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`
- `git diff --check`

## Dependency, public-surface, sensitive-state, and no-real-effect audits

- `Cargo.toml` and `Cargo.lock` add only the workspace member and one package
  entry; the new crate has no dependency or feature.
- The strong identity macros derive only safe value traits. Sensitive
  `ToolArguments`, `ToolOutput`, `ToolDiagnostic`, `ToolAuthorization`,
  `ToolConfirmationChallenge`, `ToolConfirmation`, and `ToolExecutionResult`
  are not cloneable or displayable; custom `Debug` implementations expose only
  safe fields, byte lengths, or presence.
- Confirmation binding and audit construction remain private. No public API can
  manufacture authorization, confirmation, or audit evidence.
- There is no filesystem, network, subprocess, environment, credential,
  serialization, unsafe, clock, sleep, random, global mutable state, or
  concrete-tool production path in `crates/tool-policy`.
- Tests use counters, atomics, explicit polling, and drop guards only; no real,
  destructive, privileged, credentialed, or third-party-visible operation is
  an acceptance oracle.

## Scope and exclusion conformance

Included scope is complete: investigation, accepted architecture, additive
std-only domain owner, bounded values, conservative effects, canonical
authorization, exact confirmation, gated one-attempt execution, closed terminal
and audit evidence, deterministic unit/public tests, unchanged consumer
compatibility, truthful current-state docs, and independent review evidence are
present.

Excluded scope remains absent: concrete tools and schemas; Runtime lifecycle or
registration; HTTP/protocol/MCP/provider/CLI/IDE integration; graph, metadata,
BSL, workspace, source-adapter, or Coverage Registry changes; policy storage or
configuration; authentication/UX/audit sink; model tool-call wires, prompt or
conversation orchestration, automatic selection, chains or loops; clock,
retry/fallback, rollback, sandbox, OS permission, transaction, idempotency,
atomicity, security, performance, or compliance guarantees; Sprint 28
implementation; and the v0.5 release review.

## Residual risks

The independent and primary reviews agree on these deferred risks:

- truthful effect classification remains the future concrete executor/caller's
  responsibility;
- identifiers are safe correlation labels only when callers honor the accepted
  value contract;
- the confirmation boundary does not authenticate a human;
- re-evaluation and cross-process/durable replay protection are absent;
- timeout is executor-reported and has no built-in clock;
- partial effects are not rolled back;
- sandbox, OS permissions, atomicity/idempotency, and external security
  guarantees are not claimed;
- each future concrete executor must separately prove effect declarations,
  redaction, and real cleanup behavior.

These risks are explicit ADR-0049 deferrals and do not block the accepted
library boundary.

## Artifact-consistency check

The same `/root/sprint27_reviewer` completed the required final read-only check
with `pass` before any Roadmap transition, hand-off update, previous-suite
deletion, staging, or commit. The reviewer confirmed that this artifact
preserves every finding, missing-evidence item, decision, validation result,
scope conclusion, and residual risk without weakening. The reviewer also
confirmed unchanged HEAD, no staged/tracked change, the draft as the sole
review-owned untracked file, both prompt inventories at `8/8/0`, and authorized
the bounded transition and exact eight-file retirement without performing it.

## Previous-suite retirement

Before and after the consistency check, tracked and filesystem inventories each
contained exactly the eight authorized Sprint 26 prompt files and the untracked
inventory was empty (`8/8/0`). The suite is retired explicitly and atomically
with this review after that successful re-enumeration. The complete Sprint 27
suite, `docs/codex/prompts/run-next-sprint.md`, non-adjacent suites, and
`.codex/` remain unchanged.

The exact retired paths are:

- `docs/codex/prompts/sprint-26-ollama-integration/00-sprint-26-execution-loop.md`
- `docs/codex/prompts/sprint-26-ollama-integration/01-investigate-ollama-integration.md`
- `docs/codex/prompts/sprint-26-ollama-integration/02-define-ollama-integration.md`
- `docs/codex/prompts/sprint-26-ollama-integration/03-implement-ollama-client.md`
- `docs/codex/prompts/sprint-26-ollama-integration/04-implement-ollama-discovery.md`
- `docs/codex/prompts/sprint-26-ollama-integration/05-implement-ollama-generation.md`
- `docs/codex/prompts/sprint-26-ollama-integration/06-complete-ollama-evidence.md`
- `docs/codex/prompts/sprint-26-ollama-integration/07-sprint-26-integration-review.md`

## Repository state and next action

Production and test code remain unchanged from committed Task 6. Review-owned
changes are limited to this artifact, minimal Roadmap/current-state hand-off,
and the exact eight Sprint 26 prompt deletions. Sprint 27 transitions from
`next` to `completed`; Sprint 28 MCP Server becomes the unique `next` planning
target. The post-change complete validation, Markdown-link, and inventory gates
passed; these changes are committed atomically.
