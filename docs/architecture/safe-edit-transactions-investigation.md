# Sprint 41 Safe Edit Transactions Investigation

## Status and admission

Decision readiness: `ready_for_architecture`; no transaction is implemented or
approved by this investigation. Task 1 starts from the unique committed
`Plan Sprint 41 Safe Edit Transactions` boundary
`078b258e150842da0a79da96ea09395887080cfc` on `codex/v0.7-sprint-41`, with an
empty modified/staged/untracked inventory. Sprint 40.1's accepted
[review](../reviews/sprint-40-1-refactoring-planner-remediation.md#decision)
is the prerequisite.

The fresh child admitted its investigation Profile, base/specialized Templates,
required Core modules, Architecture Workflow, and only the named Task 1
authorities and bounded consumer/fixture lookups. Effective context window and
measured token telemetry are unknown/unavailable. Admission is `warning`:
narrowed symbol/section reads preserve working and final-response capacity;
this is an admission judgement, not a measured percentage or token count.

## Accepted constraints

[ADR-0063](../adr/0063-refactoring-planner.md#authority-owner-and-dependency-direction)
keeps semantic identity and queries in Graph, pure plan construction in
Analysis, source capture in EDT/Designer adapters, and publication/lifecycle in
Runtime. The first family is exactly `bsl_callable_rename_v1`: one target in
one Configuration in one local Workspace, with a complete declaration/local
call/qualified call plan. A plan is evidence, never authorization.

Publication identity is the existing process-local
`oneagent_analysis::publication::WorkspacePublicationId`; a new service starts
at 1. Numeric equality cannot authenticate another service lifetime. Preserve
the `ChangeImpactPublicationId` alias and all existing planner APIs, Graph facts,
diagnostics, rules, protocol projections, and cache compatibility semantics.
No dependency addition, protocol/IDE mutation surface, source-format invention,
other rename family, Git mutation, cross-Workspace edit, durable cross-process
plan/undo history, or crash-atomic multi-file claim is admitted.

## Verified production owners and consumers

Paths below are repository-relative; symbols describe the inspected baseline.

| Owner/path | Confirmed responsibility and consequence |
|---|---|
| `crates/analysis/src/refactoring.rs`: `SourceEvidenceSet`, `SourceDocument` | Canonically ordered complete captured BSL documents, exact raw bytes, versions, occurrences and confined relative paths. This is not the complete metadata/discovery source set. |
| Same file: `RefactoringPlan`, `RefactoringPreconditionSet`, `RefactoringOperation`, `build_refactoring_plan` | Constructors validate structural relationships, conflicts and canonical IDs. `build_refactoring_plan` collects source preconditions from selected occurrences: only documents contributing operations. Public construction and a matching PlanId do not prove a caller supplied the current production planner result. |
| `apps/runtime/src/workspace/mod.rs`: `WorkspaceSnapshot::plan_refactoring` | Pure evaluation over one immutable snapshot; no disk read, mutation, service authentication or freshness check against current disk. A retained old snapshot still plans successfully after source files change or disappear. |
| Same file: `WorkspaceService::start` / `initialize_workspace` | The only production initial snapshot publication, after cold build or accepted cache hit. Creates watcher and enters the update loop afterwards. |
| Same file: `run_workspace_updates` | The only production successor snapshot writer. Serializes watcher-triggered and explicit `WorkspaceChangeInputHandle` rebuilds, checks predecessor Arc identity and next publication counter. No transaction input or mutation lock exists. |
| Same file: `finish_workspace_updates` | The production clearing writer on shutdown/source termination. Closes current observation while previously cloned snapshots remain immutable. |
| Same file: `WorkspaceSnapshotBuilder::build`, `build_edt`, `build_designer_xml`, `compose_change_impact` | Complete production discovery, adapter/source evidence construction, graph validation, rules/diagnostics composition and adjacent publication impact. Designer Runtime builds explicitly request `Complete`. A builder returns a candidate; it does not publish into a running service. |
| `apps/runtime/src/workspace/change.rs`: `WorkspaceFileState::scan`, `WorkspaceChangeSource` | Sorted paths, entry kinds and exact regular-file bytes; polling observation, controlled test ticks, failure/recovery and revision coalescing. Excludes root `.oneagent` and descendants of accepted ignored directories. Scan is an observation, not a filesystem lock. |
| `apps/runtime/src/workspace/cache.rs`: `WorkspaceCacheStore`, `WorkspaceCacheStorage` | Private source envelope and semantic cache; cache hits initialize a fresh service. Writes occur inside initialization/rebuild work before publication. Cache storage is not a source transaction journal. |
| `apps/runtime/src/mcp_tools.rs`: `Handler::call`, `refactoring`, `ToolRefactoringCancellation` | Production planner consumer through `execute_tool`, using a retained snapshot. Existing tool remains read-only with no edit authorization. |
| `crates/tool-policy/src/confirmation.rs`, `execution.rs`, `request.rs` | Non-cloneable one-use challenge/confirmation, authorization binding and execution; existing `ToolEffect::LocalMutation` is available. No service-lifetime or plan field exists in generic confirmation binding. |

The production snapshot sender search found exactly three mutation sites:
initial `Some`, successor `Some`, and shutdown `None`, all in Workspace's owner
module. Test-only graph-query senders are not additional production owners.
Watcher observation/status and cache-status senders cannot publish snapshots.
Explicit Git change input only requests a full rebuild through that same loop.

Runtime entry points `src/main.rs` and `src/bin/oneagent-mcp.rs` create the
service and borrow observers; GraphQuery and MCP read those observers. The LSP
binary builds a standalone immutable snapshot through the builder. Public
Workspace tests, cache tests, MCP tests and Designer paired conformance tests
are additional Rust consumers. No consumer should need migration for an
additive Runtime transaction handle. Standalone snapshots/builders must not
gain authority to mutate a running service.

## Freshness and publication gaps

**Confirmed:** `initialize_workspace` observes around cache acceptance/build;
`rebuild_workspace` observes before and after build. Unequal source observations
skip a cache write, but still return a candidate snapshot. The successor loop
checks the predecessor and publication counter, not equality of those source
observations. Startup schedules a follow-up for instability. Consequently the
existing cache stability check must not be cited as a transaction publication
guard. Reusing `rebuild_workspace` unchanged would also write the cache before
the transaction's final success decision.

**Required architecture decision:** retain a source baseline belonging to the
accepted publication, and bind transactions to it. Recheck complete paths,
entry kinds and bytes, including untouched BSL modules, Configuration/module
metadata, discovery markers, added/deleted files and other discovered roots.
Comparing only operation versions misses a new call or collision in another
module and metadata changes which alter ownership or discovery. Comparing two
fresh scans to each other also misses changes since the original publication.

`WorkspaceFileState` supplies a reusable conservative observation oracle, but
its ignored paths and source-set agreement with discovery/adapters must be
reconciled explicitly. Preserve the public SourceEvidence/Plan contract; attach
the broader source baseline privately to Runtime publication state. Reject
unstable or unprovable baselines before authorizing edits. After writing, the
only permitted state difference is the exact planned replacement set; after
reversal, exact original bytes must be restored. Final complete semantic
validation must precede the single successor publication and cache update.

## Authorization and cancellation boundaries

**Confirmed:** generic confirmation binds policy revision, request ID, actor,
tool, effects and exact argument bytes. It contains neither a Workspace owner
nonce nor a publication/plan field. A second independently created identical
authorization can have the same logical binding; one-use confirmation is not a
global replay registry. Plan IDs intentionally omit paths and process IDs.

**Candidate for Task 2:** a Runtime-owned, non-cloneable capability, privately
bound to the live service instance, exact retained publication, complete
structured production plan, source baseline, operation direction and one
attempt. Issue it only after mandatory policy/explicit confirmation, and
consume it in the service's serialized mutation path. Regenerate the plan from
the retained production snapshot and compare complete structure, not just hash
or caller-supplied operations. Apply and undo require separate explicit
authorization. Recheck live owner and all preconditions before first write;
queued, consumed, foreign, expired and stopped-service capabilities fail closed.
Task 2 must decide the exact policy/capability API and replay lifetime.

**Confirmed:** `execute_tool` stops polling and drops its executor future when
cancellation wins; cancellation can win simultaneously with ready completion.
This is suitable for existing read-only tools, not proof of completed source
recovery. Workspace already awaits its blocking build during shutdown.

**Candidate:** the service owns any accepted mutation through completion or
recovery even if the requester drops its response future. Cancellation before
mutation writes nothing; after mutation begins it requests recovery and waits
for the terminal result. Shutdown must join this work and close requests.
Cancellation cannot interrupt required rollback. No detached mutation task or
generic dropped executor may outlive the service's recovery owner.

## Filesystem alternatives and honest limits

**Confirmed primitives:** Analysis `ConfinedSourcePath` validates relative
representation and Configuration descent. Adapter `ConfinedRoots` canonicalizes
Workspace/Configuration/source paths. EDT and Designer module readers inspect
regular files, reject symlinks and detect metadata/length changes during capture.
These read-time checks are not a write-time capability or race-free directory
handle. Runtime scans classify nonregular entries as `Other` and do not follow
child symlink directories; root metadata alone may follow a root symlink.

The cache's `write_inner` demonstrates `create_new`, complete write, `sync_all`,
read-back, and rename with six injected failure points. It removes the previous
cache file before rename; its own failure test intentionally permits losing
that cache candidate. This is rebuildable-cache behavior, not acceptable source
rollback evidence and not a reusable source replacement algorithm unchanged.

| Alternative | Readiness assessment; Task 2 owns acceptance |
|---|---|
| Write directly from a snapshot or client handler | Reject: bypasses lifetime, freshness and every publication writer. |
| Separate transaction mutex around writes only | Reject: watcher, cache rebuild and shutdown remain competing owners unless all publication paths participate. |
| Service command queue/shared coordinator governing all writers | Achievable with existing Tokio channels and service ownership; preferred candidate. Must serialize startup readiness, transaction, rebuild and shutdown, including builds already running. |
| In-place truncate/write per source | Exposes partial files and recovery hazards; no multi-file atomicity. Requires stronger evidence than cache writes and offers no clear first-slice advantage. |
| Stage exact result bytes, verify, replace per file, retain exact originals | Achievable with existing standard-library I/O, with checked recovery and full semantic rebuild. Needs explicit private staging namespace, bounds, failure seams and conflict handling. |
| Persisted journal / OS-specific directory-handle implementation / external locking service | Outside the bounded dependency-free first slice; do not imply these guarantees from portable path checks. |

Any portable sequence of per-file operations has intervals in which external
readers can see a mixed source set; semantic publication can be atomic while
disk changes are not atomic across files. Path/canonicalization rechecks cannot
exclude a hostile concurrent path swap between check and operation. An ordinary
external writer can also change bytes after the final observation. Task 2 must
accept an explicit cooperative/exclusive source-ownership assumption or stop
for a stronger required guarantee. No cross-process exclusion, power-loss
durability, rollback after process termination, or multi-file crash atomicity is
established here. Filesystem rename/sync semantics beyond the inspected code
are not an experimentally validated guarantee in this task.

Rollback must compare current bytes to transaction-owned expected bytes before
restoration, never overwrite an unrelated external change. On recovery conflict
or I/O failure, retain explicit bounded recovery evidence and inhibit further
successful transaction/publication claims until recovery is resolved. Task 2
must decide whether current observation is cleared, quarantined or otherwise
made unavailable; retaining a last-good Arc is not proof that disk matches it.

## Deterministic oracle inventory

The tracked [Sprint 14 paired corpus](../../adapters/designer-xml/tests/fixtures/sprint14_conformance/README.md)
contains exact EDT LF and Designer UTF-8 BOM/CRLF module bytes. Its adapter
conformance test evaluates three operations per format: declaration, local call
and qualified call, zero omitted operations, stable repeated plans, different
format-specific PlanIds and equal canonical occurrence mapping. Both call sites
are in the same module file: this is not existing multi-file transaction proof.
Designer has no added Graph `Calls` facts in this slice; compare its source
occurrence resolution rather than demanding an unsupported Graph edge.

The paired test requests Designer `Partial`. Independently, the tracked
[Runtime fixture](../../apps/runtime/tests/fixtures/workspace_service/README.md)
and `workspace_service` public tests prove complete production Workspace
construction for EDT and Designer. The two paired roots share a Configuration
identity and must be exercised separately, not combined into one Workspace.
For future mutation tests, use repository-local temporary copies and exact
tracked source bytes; use a known complete Runtime root when testing `Complete`
publication. Fixture composition and multi-document variants must state their
provenance and be validated through production builders before claiming a pass.

| Requirement | Existing evidence / deterministic next oracle |
|---|---|
| Apply | Existing 3-operation paired plan. Future apply must compare every byte against replacement at original raw ranges, in reverse offset order per document; preserve all surrounding bytes/BOM/line endings and compare complete rebuilt occurrence mappings/expected target identity. |
| Stale rejection | Existing Analysis publication/cancellation tests and retained-snapshot Runtime tests. Future tests change untouched module, metadata, file inventory, root/path kind and publication while keeping touched bytes unchanged; assert zero writes and no publication increment. |
| Authorization | Existing policy missing/mismatched/unexpected confirmation, one-use and cancellation tests. Future tests use another live service at numeric publication 1, forged structured plan, changed policy binding, replay, stopped owner and separately authorized undo; assert zero writes. |
| Every write failure | Existing cache six-point injection proves an injection pattern only. Add production mutation seams for confinement/read, staging create/write/sync/read-back, each replacement including later files, final source scan, semantic build/validation, recovery and cleanup; assert exact disk/publication outcome for each ordinal. |
| Rollback | Existing Workspace invalid-build recovery and cache cleanup are component evidence, not source rollback. Fail after each applied file; compare original complete bytes, absent partial publication and safe staging cleanup. Inject restoration failure and external overwrite to require explicit recovery-required state. |
| Undo | No existing undo implementation. Retain bounded in-memory original/result bytes bound to one successful service publication; require fresh reverse authorization; assert exact original bytes plus complete rebuilt semantics. Test replay, stale result, intervening edits/publication, and undo recovery failure. |
| Semantic rebuild | `WorkspaceSnapshotBuilder::build` and graph validation plus rules/diagnostics composition are production oracles. Require expected new declaration identity, unchanged owner, all planned occurrences resolved to it and no unintended source changes; old target disappears. Failed post-edit validation rolls back and publishes nothing. |
| Cancellation | `GatedDetector`, controlled watcher ticks and service cancellation tests prove joining/coalescing without scheduler sleeps. Add barriers before first write, between files, during validation and during recovery; assert terminal response follows recovery/join and no detached work. |
| Concurrency | Existing explicit-input bounded follow-up, watcher coalescing and predecessor checks. Add two callers, queued watcher/rebuild, cache recovery and shutdown at deterministic barriers; assert one mutation owner, no mixed publication, no skipped/duplicate counter and unchanged retained Arcs. |
| Bounds/retention | Existing Analysis admission bounds do not bound original/result/staged copies or queued attempts. Task 2 must set pre-retention byte/path/operation/queue/undo limits and exact/one-over oracles; reject before allocation/write and avoid sensitive bytes in Debug/errors. |

No production transaction/rollback/undo symbol exists in the bounded Workspace
search. Missing transaction tests are implementation work, not fabricated
passing evidence. Existing fixture bytes, complete builder, controlled ticks,
gated detector, injected cache I/O pattern and exact-byte comparison supply an
achievable harness without new source formats or production dependencies.

## Decision readiness and first API slice

Task 2 can accept an additive Runtime Rust API: obtain a service-owned handle,
prepare an exact current production plan and confirmation opportunity, submit a
one-use authorized apply, and explicitly authorize reversal of its bounded
in-memory result. Keep all mutable state and publication coordination in
`apps/runtime/src/workspace/`; expose only transport-neutral immutable results
and a bounded command handle through `apps/runtime/src/lib.rs`. Exact types,
module split, fail-closed result vocabulary and policy flow remain ADR decisions.

Before implementation, Task 2 must settle all-writer serialization and stable
baseline retention; service/plan authorization; staging/confinement and race
assumptions; cancellation/recovery/poisoning; undo retention/expiry; semantic
postconditions; bounds; cache timing and watcher coalescing. Task 3 then maps
each accepted invariant to production guards and negative tests; Task 4 must
independently approve that committed design. No essential source-format oracle
is unavailable; no architecture decision is silently accepted by this note.

## Executable evidence

All commands run from the repository root. Tests use
`TMPDIR="$PWD/local-artifacts/codex-runs/sprint-41/task-1/tmp"`; tracked fixtures
are unchanged. Logs are retained under
`local-artifacts/codex-runs/sprint-41/task-1/`.

| Command | Outcome | Log |
|---|---|---|
| `cargo test -p oneagent-analysis --test refactoring_plan` | exit 0; 17 passed | `analysis-plan.log` |
| `cargo test -p oneagent-designer-xml --test conformance` | exit 0; 4 passed | `paired-conformance.log` |
| `cargo test -p oneagent-runtime --test workspace_service` | exit 0; 9 passed | `workspace-public.log` |
| `cargo test -p oneagent-runtime --lib workspace::` | exit 0; 75 passed, 49 filtered out | `workspace-unit.log` |
| `cargo test -p oneagent-tool-policy` | exit 0; 26 unit + 7 conformance passed; 0 doc-tests separately | `tool-policy.log` |

Zero-match searches are distinct from passing tests: the bounded Workspace
`transaction|rollback|undo|safe_edit` query returned 0 matches (`rg` exit 1).
The `.github`/`scripts` documentation-linter/link-checker query also returned 0
matches; the existing prompt validator and manual Markdown path/section checks
therefore supply the documentation gate. Preliminary wrong-path lookups were
corrected using the discovered template/adapter layout; no validation pass is
claimed for those failed reads.

This documentation-only task does not trigger the canonical Rust workspace
gate. Exact documentation outcomes are recorded in the Task 1
master ledger and retained validation log; no tests or production files change.
