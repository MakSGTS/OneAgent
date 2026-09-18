# Sprint 36 Diagnostics Engine Review

## Decision

`pass`

The effective decision matches the final independent reviewer recommendation.
Sprint 36 satisfies ADR-0058: one source-independent diagnostics engine
normalizes the two accepted Graph-owned evidence families into a complete,
bounded, deterministic report; Workspace publishes that report atomically;
the persistent cache reconstructs it from canonical inputs; and the existing
MCP and LSP adapters expose only their accepted bounded projections.

This decision does not claim Rules Engine registration or execution,
configurable or persisted suppression, a new diagnostic producer, diagnostic
UI, push or workspace diagnostics, mutable-document analysis, fixes or edits,
remote transport, authentication, telemetry, or a Coverage transition.

## Reviewed baseline

- Completed Sprint 35 prerequisite: `4a165109`.
- Sprint 36 planning anchor: `cc890879`.
- Task 7 implementation head: `8e6dbd7b`.
- Final remediation merge head: `9afd3026`.
- Exact reviewed range:
  `4a165109a37dc44371d81e49b1931c2d3a1de06c..9afd3026a98c900a7e8b606650d6bc056e92a3bc`.
- Range size: 18 commits, 35 paths, 5,769 additions, 153 deletions.

The dependency-ordered commits are:

| Step | Commit | Subject | Result |
| --- | --- | --- | --- |
| Planning | `cc890879` | `Plan Sprint 36 Diagnostics Engine` | pass |
| Investigation | `7fa34274` | `Investigate Sprint 36 diagnostics engine` | pass |
| Task framework | `3b28dcdd` | `Establish Diagnostics Engine task framework` | pass |
| ADR-0058 | `7ceec272` | `Define Sprint 36 diagnostics engine` | pass |
| Domain | `1cab3293` | `Implement Sprint 36 diagnostic domain` | pass |
| Orchestration | `0b0c5d60` | `Implement Sprint 36 diagnostic orchestration` | pass |
| Workspace/cache | `4f9ffeeb` | `Integrate Sprint 36 diagnostic snapshots` | remediated |
| MCP/LSP | `49e0c647` | `Integrate Sprint 36 diagnostic reporting` | pass |
| Evidence | `8e6dbd7b` | `Complete Sprint 36 diagnostics evidence` | remediated |
| Implementation merge | `daacec4c` | `Merge Sprint 36 diagnostics engine implementation` | pass |
| Review findings | `88295738` | `Remediate Sprint 36 diagnostics review findings` | remediated |
| Remediation merge | `48ac4c35` | `Merge Sprint 36 diagnostics remediation` | pass |
| Roadmap correction | `5573ef60` | `Complete Sprint 36 diagnostics documentation remediation` | pass |
| Roadmap merge | `bf59a320` | `Merge Sprint 36 diagnostics documentation remediation` | pass |
| Final findings | `60743f45` | `Remediate Sprint 36 final review findings` | remediated |
| Final remediation merge | `e07a191d` | `Merge Sprint 36 final review remediation` | pass |
| Regression evidence | `170b9e8f` | `Add Designer XML build validation regression evidence` | pass |
| Evidence merge | `9afd3026` | `Merge Sprint 36 validation evidence remediation` | pass |

The final reviewer began and ended at
`9afd3026a98c900a7e8b606650d6bc056e92a3bc` with a clean working tree and the
review branch equal to `origin/codex/v0.7-sprint-36-review`. The reviewer used
a fresh context, remained read-only, delegated no work, and created no file,
artifact, staging entry, commit, branch update, or remote update.

## Independent review and primary reconciliation

Final reviewer `/root/sprint36_validation_evidence_reviewer` received the
repository root, exact immutable range, authorities, acceptance and exclusion
criteria, validation matrix, and output contract. The reviewer recommended
`pass`, reported no finding at any severity, no missing required evidence, and
no optional follow-up.

Primary reconciliation reproduced every final reviewer claim:

| Reviewer item | Primary classification | Reconciliation |
| --- | --- | --- |
| Cache schema and canonical serialized evidence fields are stable while private semantic compatibility advances from `2` to `3`. | Accepted. | Code, README, Architecture, Roadmap, ADR-0058, and evidence agree; version `2` entries are intentionally rejected and rebuilt. |
| Designer XML uses complete build-result validation before publication. | Accepted. | Production and the negative regression use the same private helper over graph, diagnostics, request ledger, statistics, and report. |
| The negative Designer regression distinguishes complete validation from `graph.validate()`. | Accepted. | The graph-only assertion is valid, while the mismatched report produces `InconsistentReport`; the exact test passed independently and primarily. |
| MCP and LSP preserve their existing authority, lifecycle, policy, capability, confinement, and bounded-projection contracts. | Accepted. | Focused protocol, Tool Policy, semantic-tool, stdio, and public-process suites all passed. |
| No dependency, security, API, Coverage, or deferred-scope blocker entered the range. | Accepted. | Manifest, lockfile, source, payload, scope, credential, path, and generated-artifact audits are clean. |
| Seven Sprint 35 prompt files remain the exact retirement inventory until completion. | Accepted. | Tracked and filesystem inventories match exactly and contain no extra file. |

There is no unresolved disagreement. The effective decision is `pass`.

## Acceptance evidence matrix

| Criterion | Independent and primary evidence | Result |
| --- | --- | --- |
| Inputs and ownership | Exactly Graph-owned `SemanticDiagnostic` and caller-supplied complete `SemanticGraphValidationResult`; no source read, validation execution, or graph mutation in Analysis | pass |
| Closed vocabulary | Exhaustive family, severity, category, code, kind, and disposition mappings | pass |
| Identity and duplicates | Family-tagged typed identity, exact duplicate collapse, conflicting-content rejection independent of order | pass |
| Suppression | Default empty policy plus exact typed in-memory suppression with no configurable or persisted form | pass |
| Ordering and summary | Canonical total order, order-preserving filters, checked complete summary, active/suppressed reconciliation | pass |
| Bounds and failures | Exact input, finding, suppression, message, anchor, provenance, MCP, and LSP bounds; atomic redacted failures | pass |
| Orchestration | Deterministic complete mixed reports from Graph-owned semantic and validation evidence | pass |
| Workspace | Complete validation and diagnostic report constructed before immutable publication and watch replacement | pass |
| Designer validation | Cold build uses the complete helper; mismatched report is rejected where graph-only validation remains valid | pass |
| Cache | Schema `1`, stable canonical fields, private semantic version `3`, version `2` invalidation, deterministic derived recomputation | pass |
| MCP | Seven read-only Tool Policy-gated tools, exact filters, unfiltered summary, 100-result bound, explicit truncation, revision parity | pass |
| LSP | Existing 3.17 capability, pull-only full reports, active-only exact-one-span confinement, complete 100-result fail-closed bound | pass |
| Compatibility | Graph report/diff/validation, producers, adapters, Workspace lifecycle, HTTP, CLI, VS Code, EDT, and Coverage preserved | pass |
| API and dependencies | Additive Analysis and Workspace API; no Cargo manifest, lockfile, dependency, feature, license, or Graph API change | pass |
| Sensitive data | MCP remains path/source/reference/provenance-free; LSP URI requires existing confinement; errors remain redacted | pass |
| Deferred scope | Rules Engine, configurable suppression, new producers, UI, mutable documents, fixes, remote access, authentication, and telemetry absent | pass |

## Exact independent validation

The final reviewer ran the canonical gate in the required order at exact head
`9afd3026`:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
git diff --check
```

Every command exited zero. The full test run contained 73 targets and 1,177
passed tests with zero failed, ignored, measured, or filtered tests. The four
zero-test targets were the expected binary entry points, not acceptance
filters.

The independent focused results were:

| Area | Passed |
| --- | ---: |
| Analysis diagnostics domain | 25 |
| Analysis public engine | 3 |
| Graph validation/report/build-diff/reference-request/Coverage | 86 |
| Runtime library | 96 |
| Workspace service | 6 |
| Persistent cache | 4 |
| File watching | 2 |
| MCP semantic/stdio/process | 32 |
| LSP domain/stdio/process | 25 |
| Protocol total | 53 |
| Tool Policy | 33 |
| HTTP health | 4 |
| CLI public process | 2 |
| Exact negative Designer regression | 1 |

## Exact primary validation

After the independent report, the primary reran the complete non-zero focused
matrix at exact head `9afd3026`:

```bash
cargo test -p oneagent-analysis --lib diagnostics::
cargo test -p oneagent-analysis --test diagnostics_engine
cargo test -p oneagent-graph --test validation
cargo test -p oneagent-graph --test report
cargo test -p oneagent-graph --test build_diff
cargo test -p oneagent-graph --test reference_request_build
cargo test -p oneagent-graph --test coverage
cargo test -p oneagent-runtime --lib
cargo test -p oneagent-runtime --test workspace_service
cargo test -p oneagent-runtime --test persistent_cache
cargo test -p oneagent-runtime --test file_watching
cargo test -p oneagent-protocol
cargo test -p oneagent-tool-policy
cargo test -p oneagent-runtime --test mcp_semantic_tools
cargo test -p oneagent-runtime --test mcp_stdio
cargo test -p oneagent-runtime --test mcp_process
cargo test -p oneagent-runtime --test lsp_stdio
cargo test -p oneagent-runtime --test lsp_process
cargo test -p oneagent-runtime --test http_health
cargo test -p oneagent-cli --test runtime_client
```

The results were Analysis 25 and 3; Graph 55, 3, 2, 7, and 19; Runtime
96; Workspace/cache/watching 6, 4, and 2; Protocol 53; Tool Policy 33; MCP 7,
8, and 17; LSP 5 and 8; HTTP 4; and CLI 2. Every required target was non-zero
and every command exited zero.

The primary then repeated the canonical gate shown above. Every command exited
zero, including strict Clippy and Rustdoc. The full test result was again 73
targets and 1,177 passed tests with zero negative counters.

## Compatibility and host evidence

Task 7 committed the required unchanged-consumer host evidence:

- VS Code 1.134.0 TypeScript compilation/typecheck passed, followed by 62 unit
  and 2 real Runtime-process tests with zero failures, cancellations, skips, or
  todos.
- EDT Tycho/PDE completed with `BUILD SUCCESS`, 41 tests, zero failures,
  errors, or skips, and two public Runtime probes.
- ADR-0058 did not require another Codex or Cursor client launch because no
  repository-owned compatibility issue was found and Sprint 36 did not change
  negotiation, framing, catalog count, Tool Policy, or client configuration.

Fresh read-only reviewers did not rerun GUI-dependent VS Code or EDT hosts.
They inspected the committed evidence, the unchanged consumer source, and the
complete Rust public-process matrix. This is an environment boundary, not
missing mandatory evidence.

## Findings and missing evidence

### Blocking

None remain at `9afd3026`.

### Non-blocking follow-ups

None.

### Missing evidence

None.

## Remediation and rejected attempts

The review gate remained blocked until every confirmed item was fixed in a
separate commit and the updated immutable range received a new fresh review:

1. The first review found false cache-byte compatibility claims, one broken
   Task 5 ADR link, and stale Roadmap current-state text. Commit `88295738`
   corrected the claims and link.
2. The second review found two remaining line-wrapped Roadmap statements.
   Commit `5573ef60` corrected both.
3. One reviewer brief supplied an invalid expanded base hash. That attempt is
   not acceptance evidence. The reviewer additionally found ambiguous README
   cache wording and graph-only Designer cold-build validation. Commit
   `60743f45` corrected both, and all later reviews used the exact valid base.
4. The next review confirmed the production fix but found that its happy-path
   assertion could not distinguish complete validation from graph-only
   validation. Commit `170b9e8f` added the shared production helper and the
   negative `InconsistentReport` regression.
5. Final reviewer `/root/sprint36_validation_evidence_reviewer` reviewed the
   resulting exact range and returned `pass` with no finding or follow-up.

The implementation cycle also had one strict Clippy run with four mechanical
warnings; those were fixed before Task 7 acceptance and the complete cycle was
rerun successfully. One earlier concurrent primary MCP process attempt timed
out and was not used as evidence; subsequent isolated and canonical runs
passed 17/17. One auxiliary link-wording `rg` invocation used unescaped shell
backticks and was discarded; the safely quoted rerun and both link audits
passed. No failed or partial command above is counted as acceptance evidence.

## Scope, security, API, and dependency audit

- Graph remains authoritative for semantic facts, raw diagnostics, validation
  vocabulary and execution, provenance, typed locations, reports, and diffs.
- Analysis adds only source-independent diagnostic domain and orchestration.
- Workspace adds immutable accessors and constructs complete derived evidence
  before publication; current raw evidence access remains available.
- Cache serializes no derived validation or diagnostic report. Schema stays
  `1`; semantic compatibility advances to `3` and intentionally rejects
  version `2` entries.
- MCP remains an immutable, read-only, seven-tool, Tool Policy-gated surface.
  LSP retains its existing 3.17 capability and confined pull-diagnostic shape.
- No manifest, lockfile, dependency, feature, license, Graph producer, remote
  authority, filesystem mutation, source mutation, secret source, credential,
  personal absolute path, generated binary, cache, or raw trace entered the
  reviewed range.
- No Rules Engine, configurable or persisted suppression, diagnostics UI,
  push/workspace diagnostics, mutable-document analysis, fix/edit workflow,
  remote transport, authentication, telemetry, or unsupported Coverage claim
  entered the range.

## Residual risks and Sprint 37 hand-off

The accepted residual limitations are deliberate ADR-0058 boundaries:

- production suppression is empty and has no user configuration or storage;
- MCP returns at most 100 ordered findings while retaining the complete
  unfiltered summary;
- LSP omits findings without exactly one confined span and rejects a complete
  report above 100 findings;
- the engine normalizes only existing Graph evidence and does not register or
  execute rules.

Sprint 37 — Rules Engine is eligible to define deterministic rule
registration, dependencies, execution, configuration, ownership, and result
production against the accepted diagnostic identity/report boundary. This
review does not preselect that architecture.

## Retirement inventory and state transition

Before transition, tracked and filesystem inventories contain exactly these
seven Sprint 35 prompt files:

1. `docs/codex/prompts/sprint-35-external-ai-client-compatibility/00-sprint-35-execution-loop.md`
2. `docs/codex/prompts/sprint-35-external-ai-client-compatibility/01-investigate-external-ai-client-compatibility.md`
3. `docs/codex/prompts/sprint-35-external-ai-client-compatibility/02-define-external-ai-client-compatibility.md`
4. `docs/codex/prompts/sprint-35-external-ai-client-compatibility/03-implement-legacy-mcp-protocol.md`
5. `docs/codex/prompts/sprint-35-external-ai-client-compatibility/04-integrate-mcp-client-lifecycle.md`
6. `docs/codex/prompts/sprint-35-external-ai-client-compatibility/05-complete-external-client-evidence.md`
7. `docs/codex/prompts/sprint-35-external-ai-client-compatibility/06-sprint-35-integration-review.md`

The authorized atomic transition will mark Sprint 36 completed, make Sprint
37 the unique next target, preserve all nine Sprint 36 prompt files, and delete
exactly the seven paths above. No state transition or retirement occurs before
the same final reviewer accepts this draft and the exact proposed diffs.

## Artifact consistency

Final reviewer `/root/sprint36_validation_evidence_reviewer` performed the
mandatory read-only same-reviewer consistency check and approved this exact
draft, the proposed current-state and Roadmap transition, the Sprint 37
hand-off, preservation of all nine Sprint 36 prompt files, and deletion of
exactly the seven listed Sprint 35 prompt files as truthful, complete, and
non-weakening.
