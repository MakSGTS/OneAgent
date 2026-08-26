# Sprint 28 MCP Server Review

## Decision

`pass with non-blocking follow-ups`

The effective decision matches the final independent reviewer recommendation.
Sprint 28 satisfies the accepted ADR-0050 discovery-first MCP boundary: one
explicit protocol revision, bounded JSON-RPC values, complete syntax and
validation precedence, truthful empty capabilities, deterministic discovery,
newline-framed stdio, structured process ownership, closed failures, and
repository-owned public conformance. No blocking finding or required missing
evidence remains.

This decision does not claim MCP semantic tools, a legacy initialization
session, remote transport, authentication, external-client compatibility,
packaging, real signals, or any graph, context, provider, or tool effect.

## Reviewed baseline

- Framework baseline: `53b1b0dff815b0345790119c53be817ded94112c`.
- Planning commit: `65a1220c058e36268a5e4b6a714fb514bb35b71a`.
- Review head: `c8fe7ac0916415a6057770bc155bf1a6478b6aea`.
- Exact reviewed range: `65a1220c^..c8fe7ac0`.
- Resolved range parent:
  `53b1b0dff815b0345790119c53be817ded94112c`.
- Initial/final independent review status: clean.
- Initial/final primary review status: clean.
- Range size: 14 commits, 28 paths, 5,811 additions, 34 deletions.

The dependency-ordered commits are:

| Step | Commit | Subject | Result |
| --- | --- | --- | --- |
| Planning | `65a1220c` | `Plan Sprint 28 MCP Server` | pass |
| Investigation | `1b27b609` | `Investigate Sprint 28 MCP server` | remediated |
| Investigation correction | `33ca3da4` | `Correct Sprint 28 MCP investigation` | pass |
| ADR-0050 | `6ab34b85` | `Define Sprint 28 MCP server` | pass |
| Protocol domain | `b3ae44c1` | `Implement Sprint 28 MCP protocol domain` | pass |
| Dispatch | `f7d941c4` | `Implement Sprint 28 MCP server dispatch` | pass |
| Stdio | `2b26604b` | `Implement Sprint 28 MCP stdio transport` | pass |
| Lifecycle | `dfc27a1d` | `Integrate Sprint 28 MCP server lifecycle` | pass |
| Public evidence | `657d1fe7` | `Complete Sprint 28 MCP server evidence` | remediated |
| Conformance correction | `6b42b0a1` | `Correct Sprint 28 MCP conformance` | remediated |
| Review correction | `f65ef016` | `Correct Sprint 28 MCP review findings` | pass |
| Numeric precedence | `646c1119` | `Correct Sprint 28 numeric error precedence` | pass |
| Arbitrary-number parsing | `7ebb0e71` | `Correct Sprint 28 arbitrary number parsing` | pass |
| Syntax precedence | `c8fe7ac0` | `Correct Sprint 28 JSON syntax precedence` | pass |

The range is limited to the committed Sprint 28 plan and prompts,
investigation, ADR-0050, protocol crate, separate Runtime MCP adapter and
binary, public tests, authorized dependency edges, and three current-state
documents. It does not change graph semantics, Context Engine, Tool Policy,
providers, source adapters, existing HTTP routes, Workspace semantics, CLI
production behavior, or Coverage Registries.

## Final independent reviewer handoff and report

- Reviewer task: `/root/sprint28_release_review`.
- Context: fresh, with `fork_turns: "none"`; no implementation conversation,
  prior finding, proposed decision, primary rationale, or expected outcome was
  inherited.
- Operating constraint: read-only, no delegation, and no repository mutation,
  staging, commit, or state transition.
- Authority supplied: repository root, exact range and review HEAD, committed
  Sprint objective/scope/criteria/matrix, all Task 8 authorities, and the exact
  official MCP `2026-07-28` sources accepted by ADR-0050.
- Recommendation: `pass with non-blocking follow-ups`.
- Blocking findings: none.
- Non-blocking findings: two, preserved below.
- Missing required evidence: none.
- Initial/final HEAD:
  `c8fe7ac0916415a6057770bc155bf1a6478b6aea`.
- Initial/final Git status: branch header only; porcelain empty.
- Working-tree discrepancy: none.
- Read-only/delegation confirmation: the reviewer reported no tracked,
  staged, or untracked repository change and no delegated work.

The reviewer inspected all 14 commits and 28 changed paths, resolved the range
parent to the framework baseline, reviewed every required repository authority,
compared the implementation with all five accepted official sources, ran the
focused and canonical matrices, and independently probed JSON grammar,
arbitrary-precision, marker-collision, error-precedence, bounds, framing, and
process behavior.

## Review remediation history

Four earlier fresh-context read-only reports blocked state transition. Each
confirmed issue was corrected in its own commit before a new independent
review; no reviewer modified the repository.

| Reviewer | Reviewed head | Result | Resolution |
| --- | --- | --- | --- |
| `/root/sprint28_independent_review` | `6b42b0a1` | blocked | `f65ef016` made `RequestId` opaque and bounded, accepted schema-valid empty implementation strings, closed public error ID/code construction, added exact outbound-depth evidence, and exercised controlled encode failure. |
| `/root/sprint28_remediation_review` | `f65ef016` | blocked | `646c1119` preserved arbitrary-precision JSON numbers so syntactically valid `id:1e400` maps to `InvalidRequest`, while schema-valid arbitrary-precision progress values remain accepted. |
| `/root/sprint28_final_clean_review` | `646c1119` | blocked | `7ebb0e71` distinguished genuine numeric tokens from literal or escaped serde private-marker objects and restored duplicate, schema, and nesting behavior. |
| `/root/sprint28_final_integration_review` | `7ebb0e71` | blocked | `c8fe7ac0` added iterative full-frame syntax validation before duplicate/depth classification, with domain, stdio, and process regressions. |

The final `/root/sprint28_release_review` independently reviewed the entire
remediated range and found no blocking defect or required evidence gap.

## Acceptance evidence matrix

| Criterion | Independent and primary evidence | Result |
| --- | --- | --- |
| Planning and order | The exact range contains planning, Tasks 1-7, and bounded correction commits in dependency order; subjects and changed paths match the committed manifest. | pass |
| Investigation | `docs/architecture/mcp-server-investigation.md` records official revision/schema provenance, ownership, dependencies, messages, errors, bounds, capabilities, stdio, lifecycle, compatibility, deferrals, and deterministic oracles. | pass |
| Accepted architecture | ADR-0050 is `Accepted` and fixes protocol authority, ownership, values, errors, bounds, discovery, dispatch, stdio, Runtime/process lifecycle, compatibility, evidence, and exclusions. | pass |
| Official revision/schema | The exact official MCP basic protocol, stdio transport, schema reference, TypeScript schema, and generated JSON Schema for `2026-07-28` match the implemented discovery-first slice. | pass |
| Dependencies | The current user authorizes exactly protocol-to-Serde, protocol-to-Serde-JSON, and Runtime-to-protocol. Existing locked `serde 1.0.228` and `serde_json 1.0.150` are reused; no new third-party package/version exists. | pass |
| Public surface | MCP protocol values and Runtime transport exports are additive. Request IDs and outbound responses have private representations and bounded constructors; no existing public API was removed. | pass |
| JSON grammar | An iterative, non-recursive full-frame syntax preflight covers containers, punctuation, strings, escapes, surrogate handling, literals, exact number grammar, whitespace, and trailing input before semantic validation. A 200,000-array probe terminated safely. | pass |
| Error precedence | Invalid JSON always yields ID-less `ParseError`; syntactically valid duplicate/depth and invalid IDs yield `InvalidRequest`; later params/method/version errors preserve a valid ID as accepted. | pass |
| Arbitrary precision | Numeric `progressToken:1e400` is accepted; exponent/out-of-representation request IDs are `InvalidRequest`; genuine numeric tokens remain distinct from literal/escaped private-marker objects. | pass |
| Bounds | Message 1 MiB, string ID 256 bytes, method 256 bytes, and aggregate JSON nesting 128 are enforced with exact and one-over evidence. | pass |
| Metadata/schema | Required version/capability metadata, optional client information/icons, progress token, log level, extension keys, and known capability shapes match the accepted schema. | pass |
| Discovery and dispatch | The sole production method is `server/discover`; the only version is `2026-07-28`; capabilities are `{}`; discovery returns complete/zero-TTL/public-cache metadata; unknown method/version and notifications are deterministic. | pass |
| Framing/channel purity | Injected stdio accepts LF and optional CR, bounds frames with fixed scratch storage, serializes one compact JSON response plus LF, flushes, suppresses notification output, and keeps diagnostics off stdout. | pass |
| Failures and cleanup | EOF/cancellation are successful terminal outcomes; read, UTF-8, size, incomplete frame, encode, write, flush, and shutdown failures are closed and redacted; no detached task/channel remains. | pass |
| Process lifecycle | The dedicated binary owns process streams and Ctrl-C composition, creates no Runtime `App`, exits zero on EOF/cancellation, and emits only one bounded stderr diagnostic on terminal failure. | pass |
| Public evidence | Protocol 7 unit + 15 domain + 3 dispatch, Runtime 2 MCP unit + 7 stdio + 5 process tests are non-zero and pass. | pass |
| Compatibility | Complete Runtime 117 and CLI 20 test matrices pass with approved local loopback access; all workspace consumers of unified Serde JSON features pass. | pass |
| No real effect | No live client, external network, credential, fixed port, real signal, filesystem/tool/graph/provider action, privileged operation, or third-party-visible effect is an acceptance oracle. | pass |
| Documentation/prompts | README, Architecture, and Semantic Model truthfully describe discovery-only current behavior. Sprint 28 is 9 tracked/9 filesystem/0 untracked; Sprint 27 is 8/8/0 before conditional retirement; `run-next-sprint.md` is intact. | pass |
| Complete validation | Independent and primary focused, compatibility, audit, and canonical workspace gates pass with no required zero-match target. | pass |

## Findings

### Blocking

None.

### Non-blocking follow-ups

1. Add explicit stdio tests for the retained-CR overlap: exactly
   `MAX_MESSAGE_BYTES` payload bytes plus `\r` then EOF remains
   `IncompleteFrame`, while the same prefix plus a non-LF byte becomes
   `FrameTooLarge`. Current behavior is consistent with ADR-0050, but the
   overlap is not pinned directly.
2. Replace the revision-directory GitHub `main` schema URLs with an immutable
   upstream commit or digest when stable upstream provenance is available.

Neither follow-up changes the accepted current behavior or blocks Sprint 28.

## Missing evidence

None for the accepted ADR-0050 discovery-first boundary.

Live MCP clients, credentials, remote transports, fixed ports, real signals,
platform-specific pipe APIs, authentication, packaging, semantic tools, and
real graph/context/provider/tool effects were intentionally not executed
because they are explicit deferrals, not missing completion evidence.

No ignored test was run separately because independent and primary static
audits found zero ignored tests.

## Independent validation ledger

The reviewer reported:

- Protocol enumeration/execution: 7 unit, 15 domain, and 3 dispatch tests;
  25 passed.
- Runtime MCP enumeration/execution: 2 unit, 7 stdio, and 5 process tests;
  14 passed.
- Focused total: 39 passed, zero failed or ignored.
- Runtime first sandbox run: 76 passed and four existing loopback tests failed
  with permission denial; approved-loopback rerun passed all 117 tests.
- CLI first sandbox run: 16 passed and two existing loopback tests failed with
  permission denial; approved-loopback rerun passed all 20 tests.
- `cargo fmt --all -- --check` — exit 0.
- `cargo check --workspace` — exit 0.
- `cargo test --workspace` — exit 0; 1,078 tests, zero failed/ignored.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` —
  exit 0.
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps` — exit 0;
  18 documentation outputs generated.
- `git diff --check` and `git diff --check 65a1220c^..c8fe7ac0` — exit 0.
- Required-document link audit — 254 local links, zero missing.
- Prompt inventories — Sprint 28 `9/9/0`, Sprint 27 `8/8/0`.

The reviewer additionally ran a 28-case public-process JSON grammar/schema
probe and a 200,000-array, 400,001-byte depth probe. Every expected response,
process exit, and channel-purity assertion passed.

## Primary validation and reconciliation

The primary review started only after the final independent report returned,
from the same clean `c8fe7ac0` HEAD, independently inspected the same 14
commits and 28 paths, and reproduced the reviewer conclusions:

- Protocol: 7 unit + 15 domain + 3 dispatch passed.
- Runtime MCP: 2 unit + 7 stdio + 5 process passed.
- Runtime: 117 tests passed with approved local loopback access.
- CLI: 20 tests passed with approved local loopback access.
- Workspace: 1,078 tests passed, zero failed or ignored.
- Direct/reverse/feature dependency trees match the three authorized edges and
  already-locked package versions.
- Public-surface, official revision/schema, capability/method/version, error,
  bounds, framing/channel, lifecycle/task, ignored-test, live/external-state,
  no-real-effect, changed-path, link, and prompt-inventory audits match the
  independent report.
- The canonical workspace `fmt`, `check`, `test`, `clippy`, Rustdoc, worktree
  diff, and exact-range diff checks all passed.

No criterion, finding, missing-evidence conclusion, command result, scope
conclusion, or risk conflicts with primary evidence. The effective decision is
therefore `pass with non-blocking follow-ups`, exactly as severe as the
independent recommendation.

Neither validation path contacted a live provider/client, used credentials,
exercised product-runtime network activity, executed a real signal or
semantic/tool effect, or performed a destructive/privileged operation. The
independent reviewer retrieved only the five mandated official MCP
specification/schema sources. Approved escalation was used only for
repository-owned loopback tests.

## Dependency, public-surface, and feature audits

- `Cargo.lock` adds only dependency references to existing workspace package
  entries; no new third-party package or version stanza appears.
- Protocol normal dependencies are only Serde and Serde JSON plus already
  locked transitives; Runtime is the sole direct reverse consumer.
- The user-authorized `arbitrary_precision` and `unbounded_depth` Serde JSON
  features unify into Runtime, Axum, provider adapters, and CLI dev consumers.
  Their complete tests pass.
- Request, notification, result, and error representations remain closed;
  construction is fallible where accepted bounds or ID/code combinations can
  fail. Implicit diagnostics do not expose frame, identifier, metadata, error
  data, or transport contents.
- Existing Runtime/CLI public contracts are unchanged; MCP additions are
  separate and additive.

## Specification, capability, framing, and no-real-effect audits

- The server advertises exactly one revision, one discovery method, and empty
  capabilities. No semantic, legacy initialization/session, progress,
  server-to-client, remote HTTP/SSE/WebSocket, or alternate-version path is
  present.
- Parsing accepts exactly one JSON value, rejects duplicates and excessive
  aggregate nesting, preserves valid arbitrary-precision values, and applies
  the accepted closed error/ID precedence.
- Stdio has one sequential reader/writer owner, no banner/log on stdout, no
  spawned work, and deterministic EOF/cancellation/failure cleanup.
- Production MCP modules perform no filesystem, network-client, subprocess,
  credential, graph, Context Engine, provider, Tool Policy, or semantic-tool
  effect.

## Scope and exclusion conformance

Included scope is complete: investigation, accepted ADR, protocol domain,
discovery/dispatch, injected stdio, dedicated Runtime/process composition,
public library/transport/process evidence, truthful current-state docs, and
independent review evidence are present.

Excluded scope remains absent: semantic MCP tools; prompts, resources,
completions, logging, subscriptions, sampling, elicitation, roots, tasks, and
extensions as server capabilities; legacy initialize/session behavior;
alternate revisions; remote/HTTP/SSE/socket transport; authentication/TLS;
external-client compatibility; packaging/supervision; real signals/effects;
graph, Context, provider, Tool Policy, adapter, HTTP-route, Workspace semantic,
Coverage Registry, IDE/LSP, security, performance, or compliance claims.

## Residual risks

- The retained-CR overlap behavior is accepted but not directly pinned by the
  two explicit tests described in the non-blocking follow-up.
- Official GitHub schema links are revision-directory-specific but follow
  mutable `main`; immutable provenance would improve future reproducibility.
- Workspace-wide Serde JSON feature unification is explicitly authorized and
  currently regression-free, but future JSON consumers inherit those features.
- External-client interoperability, semantic usefulness, remote/security
  posture, packaging, and process supervision remain deferred.

## Artifact-consistency check

The same `/root/sprint28_release_review` reviewer completed the required final
read-only consistency check before any Roadmap transition, prompt deletion,
staging, or commit. The first check correctly failed because the draft
overstated the absence of external network access: the reviewer had retrieved
the five mandated official MCP sources. After the sole wording correction
distinguished official specification retrieval from prohibited live product-
runtime/client network activity, the repeated check returned `pass`.

The reviewer confirmed that every finding, missing-evidence conclusion,
decision, validation result, scope conclusion, and risk is preserved without
weakening; that the remediation history is accurate; and that the exact range,
reviewer identity, read-only/no-delegation facts, and repository state remain
truthful. HEAD remained `c8fe7ac0`, there was no tracked/staged change, this
artifact was the sole untracked path, prompt inventories remained Sprint 28
`9/9/0` and Sprint 27 `8/8/0`, and `run-next-sprint.md` was intact. The reviewer
then authorized only the minimal Roadmap transition and exact eight-file
retirement recorded below.

## Previous-suite retirement

Before drafting and again after the passing consistency check, tracked and
filesystem inventories each contained exactly the eight authorized Sprint 27
prompt files and the untracked inventory was empty (`8/8/0`). The suite is
retired explicitly and atomically with this review after that successful
re-enumeration. The complete Sprint 28 suite (`9/9/0`),
`docs/codex/prompts/run-next-sprint.md`, non-adjacent suites, and `.codex/`
remain unchanged.

The exact conditionally retired paths are:

- `docs/codex/prompts/sprint-27-tool-execution-policy/00-sprint-27-execution-loop.md`
- `docs/codex/prompts/sprint-27-tool-execution-policy/01-investigate-tool-execution-policy.md`
- `docs/codex/prompts/sprint-27-tool-execution-policy/02-define-tool-execution-policy.md`
- `docs/codex/prompts/sprint-27-tool-execution-policy/03-implement-tool-request-domain.md`
- `docs/codex/prompts/sprint-27-tool-execution-policy/04-implement-authorization-policy.md`
- `docs/codex/prompts/sprint-27-tool-execution-policy/05-implement-confirmed-execution.md`
- `docs/codex/prompts/sprint-27-tool-execution-policy/06-complete-tool-policy-evidence.md`
- `docs/codex/prompts/sprint-27-tool-execution-policy/07-sprint-27-integration-review.md`

## Repository state and next action

Production and test code remain unchanged from committed head `c8fe7ac0`.
Review-owned changes are limited to this artifact, the minimal Roadmap hand-off,
and the exact eight Sprint 27 prompt deletions. Sprint 28 transitions from
`next` to `completed`; Sprint 29 MCP Semantic Tools becomes the unique `next`
planning target. The post-change complete validation, Markdown-link, and
inventory gates passed before these changes were committed atomically.
