# Sprint 40 Refactoring Planner Evidence

## Status and scope

This document records Task 9 evidence executed on 2026-09-02 from committed
recovery head `3924acb37af4528f18dcaa0ee93c4358dae1730f`, whose parent is exact Task 8
head `3ab6002355c5dcf471630b2b507238edc9494724`. The implemented boundary is
governed by [ADR-0063](../adr/0063-refactoring-planner.md). Sprint 40 remains
active until the mandatory fresh-context Task 10 review, primary
reconciliation, artifact-consistency check, Sprint 41 hand-off, and conditional
Sprint 39 prompt-suite retirement.

Post-review remediation commit `36c28e48` restores Configuration-at-Workspace-
root support and adds one public Workspace regression plus one public MCP
process regression. The live inventory at remediation merge `5d16782a` was 9
Workspace tests, 19 MCP process tests, and 1,334 tests across 85 targets.

A later independent review found that the planner treated every non-unique
same-name occurrence in the Configuration as target-related. The current
implementation remediation retains and byte-validates the immediate lexical
owner of qualified calls, scopes local/declaration relevance by owner Module
ID, scopes qualified relevance by BSL-equivalent owner name, and adds paired
positive/negative planner regressions. This changes the private occurrence
manifest and advances cache semantic compatibility from `6` to `7`.
A fresh full inventory on 2026-09-05 remains 85 targets with 81 non-zero and
four expected zero-test binary entries; the four added Analysis regressions
increase the complete total to 1,338 tests.

The keyword-parser remediation makes declaration, scope, termination, and
trailing `Export` recognition token-exact, rejects a callable declaration
nested inside another callable scope, and proves the fail-closed behavior in
both production adapters. Six intentionally truncated Writes snippets are
closed only when composed into their integration-test module. The four new BSL
and adapter regressions increase the complete total to 1,342 tests without a
cache, Graph, Coverage, protocol, or public planner contract change.

The accepted first slice is exactly `bsl_callable_rename_v1`: one top-level BSL
Procedure or Function declaration and every supported unique local or exported
qualified direct-call identifier in one complete Configuration publication.
Planning and preview are deterministic read-only evidence. They do not modify
source or repository state, authorize an edit, create a transaction, or promise
that a later apply operation will succeed.

The committed implementation and recovery chain before Task 9 is:

| Slice | Commit |
| --- | --- |
| Prompt Contract v2 prerequisite | `5210f8ee` |
| Refactoring and Safe Edits prerequisite | `5c273da1` |
| Planning | `2bc6afb7` |
| Immutable-source-evidence amendment | `1319674f` |
| Investigation | `991dded6` |
| ADR-0063 | `50746ae3` |
| Immutable source evidence | `fe28eff5` |
| EDT/Designer source evidence | `ec896e20` |
| Refactoring plan domain | `933829ca` |
| Validated planner | `8e5e1432` |
| Workspace composition | `e66a1bbb` |
| Product projection | `3ab60023` |
| Diff-hygiene recovery | `3924acb3` |

## Requirement-to-evidence matrix

| ADR-0063 requirement | Repository-owned evidence | Result |
| --- | --- | --- |
| Graph remains the sole Configuration, Module, callable, ownership, `Calls`, and query authority | No Graph production path changed in the ADR-through-recovery range; 298 Graph package tests; Analysis uses `SemanticGraphQuery` and the BSL-owned identity/name helpers | pass |
| Analysis owns source-independent documents, requests, preconditions, operations, plans, previews, bounds, summaries, and failures | `oneagent-analysis::refactoring`, 12 source-evidence tests, 17 plan/planner tests, strict Rustdoc | pass |
| Adapters capture evidence, Runtime publishes it, and MCP only projects it through Tool Policy | EDT/Designer source-evidence targets, Workspace tests, MCP semantic/stdio/process targets, and dependency-direction audit | pass |
| The only family is one top-level BSL Procedure or Function rename in one Configuration | Planner Procedure, Function, English, Russian, unsupported-target, missing-target, and single-Configuration tests | pass |
| EDT supports exact declarations plus unique local and qualified calls; Designer supports accepted Object, Manager, and Common module roles with exported qualified calls | 5 EDT source-evidence tests, 5 Designer source-evidence tests, and the paired production conformance oracle | pass |
| Unsupported, unresolved, ambiguous, dynamic, string/comment, nested, multi-segment, or otherwise incomplete target-related evidence never produces a guessed operation, while unrelated same-name calls do not block the plan | Adapter complete-ledger negative tests plus planner unrelated-local, unrelated-qualified, target-owner-qualified, missing, ambiguous, incompatible, and incomplete tests | pass |
| One document is identified only by Configuration and Module IDs and binds format, role, confined relative path, exact raw bytes, content version, canonical occurrences, and completeness | 12 source-evidence tests cover identity, lexical owner, duplicate IDs/paths, format/role, confinement, raw bytes, canonical order, and complete sets | pass |
| Content version is exact raw length plus all 32 SHA-256 bytes from one canonical implementation | Common SHA-256 vectors, deterministic content-version tests, moved private Designer hash implementation, and manifest/source audit | pass |
| UTF-8, at most one BOM, CRLF/CR/LF preservation, UTF-8 scalar boundaries, and exact token bytes are enforced | BSL 44-test package, source-evidence range/token/BOM/encoding tests, and paired LF versus BOM+CRLF fixture | pass |
| Accepted regular non-symlink sources are captured before publication and are never reopened by planning or preview | EDT/Designer non-UTF-8/symlink/changed-during-capture tests; Workspace and MCP tests change, remove, and rename source after publication while repeated plans remain equal | pass |
| Every syntactically relevant direct-call candidate has one retained unique, unresolved, ambiguous, or unsupported outcome plus exact qualified-call lexical owner context | EDT and Designer complete-ledger/owner-context tests, Analysis byte-validation tests, and paired canonical occurrence projection | pass |
| `WorkspacePublicationId` is the one checked process-local publication sequence shared with Change Impact | The public alias, Runtime initial/successor/stale tests, File Watching, MCP live-publication, and fresh Workspace runs | pass |
| Target identity binds Configuration, pre-rename node, kind, one owner Module, declaration, source version, and the BSL-owned expected post-rename ID | Planner target/owner/source tests and Graph-backed production fixture evaluation | pass |
| Desired names use the accepted Unicode grammar, 256-byte bound, BSL lowercase equivalence, reserved set, no-op rule, and sibling/identity collision rules | Public name-bound/grammar/reserved/redaction test and planner no-op/name/identity collision tests | pass |
| Requests contain only family, publication, Configuration, target, and desired name; preconditions contain the complete ordered document/version set | Domain constructor tests, planner input API audit, and MCP exact schema audit | pass |
| Plan and operation identities use canonical length-prefixed SHA-256 input and fail closed on conflicting equal identities | Stable/different identity, reorder, repetition, conflict, and redaction tests | pass |
| The operation vocabulary is closed to declaration and direct-call identifier replacement with zero dependencies | Domain operation tests and forbidden-dependency failure evidence | pass |
| Operations have the exact document/range/version/token/replacement fields and canonical descending-range application order | Domain construction/order tests, mixed-line preview, paired production plans, and exact range assertions | pass |
| Exact duplicates collapse while unequal same-range, replacement, version, identity, or overlapping evidence rejects the whole request | Duplicate, same-anchor, version, identity, and overlap tests in both source-evidence and plan targets | pass |
| A successful plan is complete, has one declaration plus all accepted calls, and reconciles every checked summary count with zero internal omission | Domain summary tests, paired three-operation oracle, Workspace plans, and public MCP summary assertions | pass |
| Preview is a deterministic structured no-snippet projection with relative path, raw range, one-based scalar positions, and replacement only | Mixed BOM/line-ending preview tests, repetition tests, and MCP forbidden-field/redaction assertions | pass |
| Every exact inclusive bound is accepted and one-over fails atomically before partial retention or projection | Source-document/set, candidate/operation, identity/name, public limit, Tool Policy output, and MCP frame tests | pass |
| Closed failures and deterministic precedence preserve redaction and expose no partial result | Domain error-kind tests, Workspace cancellation/stale/missing tests, and MCP invalid/not-found/execution-failed/oversize/policy mappings | pass |
| Cancellation is checked through planning and Runtime joins owned work during shutdown | Planner cancellation checkpoints, Runtime library lifecycle tests, MCP cancellation projection, stdio cancellation, EOF, and process cleanup | pass |
| Every Workspace Configuration snapshot publishes one complete source-evidence set atomically | Workspace public positive/failure tests, Runtime 124-test unit suite, and adapter failure/duplicate identity validation | pass |
| Failed/cancelled/stale/incomplete builds publish nothing, consume no ID, retain the last valid snapshot, and recover normally | Runtime unit, Workspace, File Watching, Git-input, cache, and live MCP successor evidence | pass |
| Cache schema stays `1`, semantic compatibility is `7`, exact bytes stay in the private source envelope, and the semantic DTO stores only the canonical manifest | Cache source audit, exact envelope assertions, version-6 invalidation, 124 Runtime unit tests, and 4 public persistent-cache tests | pass |
| Cold and accepted warm snapshots expose equal source evidence and equal plans; plans and publication IDs are not persisted | Cache round-trip planner equality, public cold/warm/replacement tests, and serialized-envelope audit | pass |
| The MCP catalog contains exactly eight lexicographically ordered read-only tools for all three accepted revisions | MCP catalog/schema tests, 53 Protocol tests, 10 semantic-tool tests, 8 stdio tests, 19 public-process tests, and 62 VS Code unit tests | pass |
| `oneagent.refactor.plan` requires exact publication, Configuration, target, and desired-name fields, admits optional `1..=100` limit, and rejects unknown fields | Catalog schema assertions and positive/missing/extra/type/exact/one-over public tests | pass |
| Public output is complete, reconciled, bounded, redacted, `readOnly=true`, and `editAuthorization="none"` | In-memory and public-process projection tests, retained-source test, output-size tests, and sensitive-data scans | pass |
| The tool remains `ReadOnly`; deny, execution failure, and output overflow preserve existing Tool Policy behavior | 33 Tool Policy tests plus Runtime allow/deny/cancellation/output mapping tests | pass |
| Legacy tools, revisions, framing, channel purity, EOF, repeated sessions, and supported clients remain compatible | Protocol, MCP semantic/stdio/process, Graph Query, HTTP, LSP, CLI, and VS Code matrices | pass |
| No Graph/Coverage transition, third-party dependency, feature, license, unsafe code, protocol revision, or unrelated public-API removal is introduced | Path/manifest/lockfile/Coverage/API/unsafe/license audits, workspace check, strict Clippy, and Rustdoc | pass |
| No source/repository mutation, edit authorization, transaction, rollback, deferred family, UI, persistence/history, telemetry, or broad performance/security claim is introduced | Production diff audit, deferred-scope scan, public projection assertions, and tracked-artifact audit | pass |

No required row uses a zero-match test filter, and no required check is skipped or
reported as a pass while unavailable. The four zero-test all-target entries are
the expected public binary entry points `oneagent-cli`, `oneagent-runtime`,
`oneagent-mcp`, and `oneagent-lsp`; they are inventory, not acceptance evidence.

## Focused executable evidence

The following commands ran from the repository root and exited zero. Package
totals and explicit target reruns overlap and must not be added together.

| Command or exact suite | Tests passed | Failed / ignored / filtered |
| --- | ---: | --- |
| `cargo test -p oneagent-common --quiet` | 6 | 0 / 0 / 0 |
| `cargo test -p oneagent-bsl --quiet` | 44 | 0 / 0 / 0 |
| `cargo test -p oneagent-graph --quiet` | 298 | 0 / 0 / 0 |
| `cargo test -p oneagent-analysis --test refactoring_source_evidence --quiet` | 12 | 0 / 0 / 0 |
| `cargo test -p oneagent-analysis --test refactoring_plan --quiet` | 17 | 0 / 0 / 0 |
| `cargo test -p oneagent-analysis --quiet` | 158 | 0 / 0 / 0 |
| `cargo test -p oneagent-edt --test source_evidence --quiet` | 5 | 0 / 0 / 0 |
| `cargo test -p oneagent-designer-xml --test source_evidence --quiet` | 5 | 0 / 0 / 0 |
| `cargo test -p oneagent-designer-xml --test conformance --quiet` | 4 | 0 / 0 / 0 |
| `cargo test -p oneagent-edt --quiet` | 345 | 0 / 0 / 0 |
| `cargo test -p oneagent-designer-xml --quiet` | 40 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --lib --quiet` | 124 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test workspace_service --quiet` | 9 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test file_watching --quiet` | 2 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test git_change_workspace --quiet` | 3 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test persistent_cache --quiet` | 4 | 0 / 0 / 0 |
| `cargo test -p oneagent-protocol --quiet` | 53 | 0 / 0 / 0 |
| `cargo test -p oneagent-tool-policy --quiet` | 33 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_semantic_tools --quiet` | 10 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_stdio --quiet` | 8 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test mcp_process --quiet` | 19 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test graph_query_api --quiet` | 3 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test http_health --quiet` | 4 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test lsp_stdio --quiet` | 5 | 0 / 0 / 0 |
| `cargo test -p oneagent-runtime --test lsp_process --quiet` | 8 | 0 / 0 / 0 |
| `cargo test -p oneagent-cli --test runtime_client --quiet` | 2 | 0 / 0 / 0 |

The completed canonical `cargo test --workspace --all-targets --quiet` run
emitted 85 target summaries: 81 non-zero targets, the four expected zero-test
binaries, and 1,342 tests. It reported exactly 1,342 passed tests with zero
failures, ignored, measured, or filtered tests.

## Paired source and planner oracle

The tracked Sprint 14 conformance fixture represents the same exported
`FillSecurityCollection` Procedure, its caller, one local call, and one
qualified call in EDT and Designer XML layouts. EDT retains LF bytes; Designer
retains one UTF-8 BOM and CRLF bytes. Their paths, formats, raw bytes, content
versions, and raw ranges deliberately differ, while the canonical declaration,
local-call, qualified-call, lexical-owner, resolution, mapped-target,
owner-role, and semantic identity projection is equal.

Each production adapter plan contains exactly one declaration, one local-call,
and one qualified-call operation with no omission. Repeated evaluation over the
same publication is equal and leaves the retained evidence unchanged. The EDT
and Designer plan IDs deliberately differ because exact content versions,
ranges, formats, and operations are plan preconditions; cross-format evidence
equality does not erase those source facts.

## Workspace, cache, protocol, and client evidence

Workspace tests prove that plans remain byte-equal after the original EDT file
is changed and renamed and the Designer file is removed. A successor source
version produces a new publication and plan while rejecting the predecessor
request as stale; a retained predecessor `Arc` remains independently usable.
Cold and accepted semantic-version-`7` warm cache snapshots reconstruct equal
documents, occurrences, versions, and plans from the private source-state bytes.

The three accepted MCP revisions expose the same eight-name catalog and exact
schema. In-memory, stdio, and real `oneagent-mcp` process tests cover positive,
negative, stale, missing, malformed, bounded, truncated, repeated, reordered,
live-successor, policy, EOF, session, channel-purity, and shutdown behavior.
The seven legacy tools retain their schemas and results.

The affected VS Code consumer was checked with the bundled Node executable:
both production/test TypeScript no-emit checks, both compilation checks, and 62
unit tests passed with zero failures, cancellations, skips, or todos. The only
client source change is the synchronized eight-name MCP catalog assertion. No
VS Code command, code action, edit request, preview UI, or automatic planner
invocation was added. No Electron Extension Host, Eclipse, or EDT GUI launch is
claimed or required by this source/test-only catalog change.

The first convenience command, `pnpm --dir extensions/vscode run typecheck`,
did not start TypeScript because `node` was absent from `PATH`; it exited 1 with
`node: not found` and is not acceptance evidence. The same two type checks and
the compilation/unit matrix were then executed directly with the provided
bundled Node runtime and exited zero.

The target-relevance remediation repeated the same five TypeScript stages
sequentially on 2026-09-05. The chained command exited zero and again reported
62 passed unit tests with zero failures, cancellations, skips, or todos. The
Electron-as-Node wrapper emitted non-fatal macOS process-inspection diagnostics;
they did not launch an Extension Host or change any stage exit status.

The keyword-parser remediation repeated those five stages sequentially on the
same date with the same successful 62-test result and zero failure,
cancellation, skip, or todo counts. The same non-fatal process-inspection
diagnostics did not change any stage exit status.

## Canonical gate

| Command | Exact outcome |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo check --workspace --all-targets` | exit 0 |
| `cargo test --workspace --all-targets` | exit 0; 85 targets, 1,342 passed, 0 failed/ignored/measured/filtered |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | exit 0 |
| `git diff --check` | exit 0 for the final Task 9 and post-review remediation documentation diffs |

## API, dependency, cache, Coverage, and sensitive-data audits

- `oneagent-analysis::refactoring` is additive and publicly owns the complete
  source-evidence and planner domain. BSL additively exposes exact optional
  identifier ranges, one shared name-equivalence helper, and the existing
  callable-ID constructor. Common additively exposes the canonical SHA-256
  implementation. `SourceOccurrence::new` remains available for declarations
  and local calls; qualified calls use the additive explicit lexical-owner
  constructor. Existing call-resolution results remain available.
- EDT and Designer add source-evidence build results/accessors without changing
  existing graph results. Workspace Configuration snapshots add immutable
  evidence access, and `WorkspaceSnapshot::plan_refactoring` is additive. No
  removed public item was found in the ADR-through-recovery diff; Rustdoc with denied
  warnings passed.
- Cargo changes add only internal workspace edges: EDT and Designer depend on
  Analysis, and Analysis tests depend on Metadata. `Cargo.lock` mirrors those
  three internal edges. No third-party package, version, feature, workspace
  member, native library, license field, VS Code package, pnpm lock, or EDT
  package changed. The workspace continues to inherit Apache-2.0 and forbid
  unsafe Rust.
- Cache schema is exactly `1` and semantic compatibility is exactly `7`.
  Source bytes remain in the private source-state envelope; the semantic DTO
  adds only canonical document/occurrence/version/lexical-owner claims and
  validates them against recomputed bytes during decode. Version `6` entries
  cold-rebuild; publication IDs and plans are not serialized.
- No Graph, EDT Coverage, or Designer Coverage registry path changed. Existing
  graph facts and Coverage capabilities remain unchanged; the planner consumes
  them and retained source evidence without creating a competing fact or
  traversal authority.
- Changed production code contains no source-write, repository-write, shell,
  process, editor edit, code-action, or Git mutation primitive. New filesystem
  writes/removes/renames occur only in temporary tests that prove capture
  stability, retained-publication behavior, and successor staleness.
- Sensitive-data scans found no personal absolute path, username, private-key
  marker, token prefix, bearer value, credential, or secret in changed
  production output. The test-only `secret-source` sentinel proves rejected
  desired text is absent from errors. Public projections exclude raw content,
  expected tokens, content versions/digests, absolute paths, provenance, policy
  internals, and raw error chains.
- The ADR-through-recovery range contains no generated binary, cache, package, local log,
  credential, or unrelated artifact. Successful complete logs are ignored local
  artifacts rather than tracked evidence inputs.

## Deferred scope and no-mutation boundary

Sprint 40 does not implement any other refactoring family; nested, indirect,
dynamic, reflected, string, query, callback, event, extension, override,
metadata, Module, parameter, variable, path, or file renames; multi-target or
multi-Configuration planning; model-generated planning; new Graph traversal or
facts; source or repository mutation; editor edits or code actions; apply
authorization; staging, atomicity, rollback, reversibility, backup, recovery,
cleanup, durability, or post-edit semantic validation; Git mutation or remote
access; plan persistence/history; new HTTP/CLI/LSP/IDE UI; telemetry,
benchmarks, or broad performance/security claims.

Sprint 41 owns edit-transaction architecture and implementation. It must
recheck every publication, document version, range, expected token,
confinement, and authorization precondition immediately before any accepted
mutation. A Sprint 40 plan is evidence for that future decision, not permission
to write.

## Review hand-off

The complete planning-through-Task-9 implementation-branch range begins after
completed Sprint 39 version head
`8d28ba8acacd00efd902eb2aa4ab3194f1636c05`. Its terminal must be the unique
Task 9 commit whose first parent is recovery head
`3924acb37af4528f18dcaa0ee93c4358dae1730f` and whose subject is exactly
`Complete Sprint 40 Refactoring Planner evidence`; the Task 9 final report
records that terminal SHA.

After that commit is pushed and the sprint branch is merged with `--no-ff` into
`codex/v0.7`, Task 10 must resolve and review the immutable range
`8d28ba8acacd00efd902eb2aa4ab3194f1636c05..<Sprint 40 implementation merge>`.
It must start from a path inventory, map every ADR-0063 row to independent and
primary evidence, and rerun the complete focused, executable-enumeration,
canonical, compatibility, API/dependency/license, cache/Coverage,
sensitive-data, deferred-scope, tracked-artifact, and cleanliness matrix above.
Task 9 supplies no review decision and does not mark Sprint 40 completed.

## Retained command logs

Complete successful logs are retained only for reproducible enumeration and
the canonical test audit:

- `local-artifacts/codex-runs/sprint40-task9/test-list.log`
- `local-artifacts/codex-runs/sprint40-task9/workspace-tests.log`
- `local-artifacts/codex-runs/sprint40-target-relevance-remediation/workspace-tests.log`

They contain no credentials or external payloads and are ignored, untracked
local artifacts.
