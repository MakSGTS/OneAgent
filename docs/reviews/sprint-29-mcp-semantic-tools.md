# Sprint 29 MCP Semantic Tools Review

## Decision

`pass with non-blocking follow-up`

The effective decision matches the final independent reviewer recommendation.
Sprint 29 satisfies the accepted ADR-0051 boundary: one immutable six-tool
catalog, bounded list/call protocol values, exact read-only annotations and
schemas, Tool Policy authorization and execution, deterministic graph/query/
validation/diagnostics/impact/context projections, bounded failures, immutable
workspace ownership, and repository-owned public process evidence.

This decision does not claim mutation tools, confirmations, policy
administration, live external-client compatibility, remote transport,
authentication, concurrent calls, workspace reload, IDE/LSP integration, or
performance and security properties beyond the accepted bounded evidence.

## Reviewed baseline

- Planning commit: `0becccad`.
- Initial Task 7 review head: `050bc35d`.
- Functional remediation head: `844515d1`.
- Final evidence remediation and review head: `f24967d1`.
- Exact effective reviewed range: `0becccad^..f24967d1`.
- Initial/final independent final-review HEAD:
  `f24967d1638e496569b7576b587e6af605dc87ad`.
- Initial/final independent final-review status: clean.
- Initial/final primary final-review status: clean.
- Range size: 10 commits, 26 paths, 3,459 additions, 200 deletions.

The dependency-ordered commits are:

| Step | Commit | Subject | Result |
| --- | --- | --- | --- |
| Planning | `0becccad` | `Plan Sprint 29 MCP Semantic Tools` | pass |
| Investigation | `d41fd74b` | `Investigate Sprint 29 MCP semantic tools` | pass |
| ADR-0051 | `abb4478d` | `Define Sprint 29 MCP semantic tools` | pass |
| Protocol tools | `acfd2b9a` | `Implement Sprint 29 MCP tool protocol` | pass |
| Graph tools | `da9ae9b0` | `Implement Sprint 29 semantic graph tools` | remediated |
| Impact and Context | `9d16b4b7` | `Implement Sprint 29 impact and context tools` | remediated |
| Process integration | `f485d13b` | `Integrate Sprint 29 MCP semantic tools` | pass |
| Public evidence | `050bc35d` | `Complete Sprint 29 MCP semantic tool evidence` | remediated |
| Contract remediation | `844515d1` | `Remediate Sprint 29 MCP semantic tool contracts` | pass |
| Evidence remediation | `f24967d1` | `Complete Sprint 29 MCP semantic tool evidence remediation` | pass |

The range is limited to the Sprint 29 plan and prompts, investigation,
ADR-0051, the existing protocol and Runtime MCP boundaries, two authorized
local Runtime dependency edges, semantic tool composition, public tests, and
three current-state documents. It does not change graph or Context semantics,
Tool Policy semantics, source adapters, existing HTTP routes, CLI production
behavior, providers, or Coverage Registries.

## Independent reviewer handoff and report

- Reviewer task: `/root/sprint29_integration_review`.
- Initial context: fresh; no implementation conversation or primary conclusion
  was inherited.
- Operating constraint: strict read-only, no delegation, and no repository
  mutation, staging, commit, state transition, or prompt retirement.
- Authority supplied: repository root, exact committed range, Sprint 29
  authorities, acceptance criteria, exclusions, validation matrix, and output
  contract.
- Initial recommendation at `050bc35d`: `blocked`.
- First remediation recommendation at `844515d1`: `blocked` only by incomplete
  required reordered evidence.
- Final recommendation at `f24967d1`: `pass with non-blocking follow-up`.
- Final blocking findings: none.
- Final missing required evidence: none.
- Final non-blocking finding: one, preserved below.
- Initial/final final-review HEAD:
  `f24967d1638e496569b7576b587e6af605dc87ad`.
- Initial/final final-review `git status --short`: empty.
- Read-only/delegation confirmation: the reviewer reported no created,
  modified, deleted, staged, or committed repository state and no delegation.

The same reviewer retained the independent report across the two authorized
remediation re-reviews. No additional reviewer was launched.

## Review remediation history

The initial independent report identified five functional contract defects and
required evidence gaps. The user separately authorized remediation of blocking
contracts and missing tests. The reviewer never modified the repository.

| Reviewed head | Decision | Finding | Resolution |
| --- | --- | --- | --- |
| `050bc35d` | blocked | Query relations omitted stable edge identity/kind/related node; traversal omitted the reason edge. | `844515d1` projects `edgeId`, `edgeKind`, source/target, `relatedNode`, and `viaEdgeId`, with public fixture assertions. |
| `050bc35d` | blocked | Every Context Engine error mapped to `not_found`, including an existing seed with insufficient budget. | `844515d1` maps only `MissingSeed` to `not_found`, request/budget failures to `invalid_arguments`, and invariant failures to `execution_failed`. |
| `050bc35d` | blocked | Tool Policy output overflow was exposed as `execution_failed`; `result_too_large` was unreachable. | `844515d1` returns a fixed bounded `result_too_large` envelope and proves it with a generated large workspace. |
| `050bc35d` | blocked | `edgeKinds` advertised `uniqueItems` without a closed item enum while runtime silently deduplicated duplicates. | `844515d1` advertises the exact 11-value enum and rejects duplicates. `f24967d1` proves one successful all-11 call and retains duplicate rejection. |
| `050bc35d` | blocked | Impact returned only `reasonCount`; Context omitted typed reason and relation/path summaries. | `844515d1` projects deterministic Impact reasons and ordered Context reason/relation summaries. |
| `050bc35d` | missing evidence | Empty snapshot, policy denial/no-bypass, oversized output, exact/one-over bounds, complete projections/redaction, duplicates, and reordered cases were incomplete. | `844515d1` adds empty, denial/no-bypass, oversize, bounds, projections/redaction, and duplicate evidence. |
| `050bc35d` | missing evidence | Bounded protocol tool-definition schema and aggregate catalog construction were not proved against the complete encoded `tools/list` response. | `844515d1` validates individual definitions and the aggregate catalog through bounded response construction and adds oversized-schema, excessive-depth, and aggregate-catalog public tests. |
| `844515d1` | blocked | Reordered equivalence existed only for `oneagent.graph`, while ADR-0051 requires it for each tool. | `f24967d1` recursively reverses every JSON object member order and compares canonical/repeated/reordered results for all six tools and all three query operations. |
| `050bc35d` | non-blocking public-surface finding | Runtime publicly exported an unplanned graph-only semantic-server constructor. | `844515d1` removes `graph_semantic_server`; only the complete `semantic_server` remains public. |

## Acceptance evidence matrix

| Criterion | Independent and primary evidence | Result |
| --- | --- | --- |
| Planning and order | The exact range contains planning, Tasks 1-7, and two bounded remediation commits in dependency order; subjects and paths match their owned outcomes. | pass |
| Investigation and architecture | The investigation records official MCP sources, repository owners, dependencies, wire contracts, errors, bounds, lifecycle, exclusions, and deterministic oracles. ADR-0051 is accepted and remains consistent with the implementation. | pass |
| Catalog and discovery | The server advertises `capabilities.tools={}` and lists exactly context, diagnostics, graph, impact, query, and validation in canonical order with zero TTL/public cache metadata. | pass |
| Schemas and annotations | Every tool has a closed object-root schema and exactly the four accepted read-only annotations. Query advertises the complete closed 11-kind edge enum and unique items. | pass |
| Protocol boundary | Public asynchronous sequential handler dispatch, bounded definition/catalog construction, list/call parsing, notifications, versions, unknown tools, protocol errors, and tool errors are covered. | pass |
| Tool Policy | Every known call builds the accepted read-only request and passes through `execute_tool`. Completed, denied, and no-bypass behavior is covered; MCP annotations do not authorize execution. | pass |
| Immutable workspace | The process builds one workspace snapshot from its working directory before frame processing and reuses it for sequential calls. Empty and tracked mixed snapshots are covered. | pass |
| Graph and query | Summary, exact node, relations, and traversal preserve deterministic owners, bounds, stable edge identities/kinds, related-node projection, traversal reason edge, truncation, and redaction. | pass |
| Validation and diagnostics | Canonical ordered bounded issues and recoverable diagnostics preserve stable codes, severities, identifiers, totals, truncation, and path/provenance exclusion. | pass |
| Impact | Distinct immutable configurations feed canonical Graph diff/impact. Summary, affected nodes, typed deterministic reasons, bounds, and truncation are covered. | pass |
| Context | Canonical Context Engine selection returns rendered context, ordered item/relation summaries, typed reasons, accepted bounds, correct missing/budget errors, and no source/provenance path. | pass |
| Output and error bounds | Protocol values, arguments, catalog responses, semantic projections, and Tool Policy output are bounded. Oversized semantic output reaches stable `result_too_large`; strings are not silently truncated. | pass |
| Determinism matrix | Positive, negative, exact-bound, one-over, repeated, and reordered public cases cover every tool. The reordered serializer reverses all JSON object members without changing array order. | pass |
| Edge-kind matrix | The schema, runtime matcher, successful all-11 public call, and duplicate negative case agree on the complete closed vocabulary. | pass |
| Process and channel purity | Repository child-process tests cover discovery, list, all six tool families, errors, repetition, startup failure, EOF, shutdown, stdout protocol purity, and bounded stderr diagnostics. | pass |
| Compatibility and dependencies | Runtime adds only the authorized local Analysis and Tool Policy edges. No new third-party package/version exists; the complete workspace matrix passes. | pass |
| Public surface | MCP additions are additive except removal of the unplanned remediation-only graph constructor before completion. Existing Runtime, HTTP, CLI, graph, Context, provider, and adapter contracts remain covered. | pass |
| Exclusions | No mutation, confirmation UX, policy administration, live external client, remote/auth transport, concurrency, watcher/reload, provider effect, IDE/LSP work, or expanded performance/security claim is required or introduced. | pass |

## Findings

### Blocking

None remain after `844515d1` and `f24967d1`.

### Non-blocking follow-up

The investigation's version-specific generated-schema URL still uses the
mutable GitHub `main` branch. The `schema/2026-07-28` path fixes the accepted
revision, and the repository contract and tests are self-contained, so this
does not affect current correctness. Pinning the upstream URL to an immutable
commit would improve long-term provenance reproducibility.

## Missing evidence

None for the accepted ADR-0051 boundary.

Live external clients, credentials, remote transports, authentication,
concurrent calls, real signals, mutation effects, workspace reload, IDE/LSP,
and broad performance/security evidence are explicit deferrals rather than
missing Sprint 29 completion evidence.

## Independent validation ledger

### Initial review at `050bc35d`

The reviewer reported:

- `cargo fmt --all --check` — exit 0.
- `cargo test -p oneagent-protocol --test mcp_dispatch` — 5 passed.
- `cargo test -p oneagent-runtime --test mcp_semantic_tools` — 2 passed.
- `cargo test -p oneagent-runtime --test mcp_process` — 7 passed.
- The first sandboxed workspace run reached the existing CLI loopback tests and
  failed only because two tests could not bind a local port with
  `PermissionDenied`. The approved-loopback rerun passed the full 1,086-test
  inventory with zero failures or ignored tests.
- Full workspace check, Clippy, and Rustdoc — exit 0.
- Required-document link audit — 293 local links, zero missing.
- Required Sprint 29/Sprint 28 prompt inventories — 9 tracked files, 9
  filesystem files, and zero untracked additions for each verified suite.
- `git diff --check 0becccad^..050bc35d` — exit 0.
- Dependency, public-surface, capability, policy, effect, redaction, ignored-
  test, and deferred-scope audits completed. The public-surface audit identified
  the unplanned `graph_semantic_server` export; required zero-match searches
  otherwise reported no prohibited implementation or ignored test.

These successful checks did not override the five functional findings or the
missing-evidence matrix, so the initial recommendation remained `blocked`.

### Functional-remediation review at `844515d1`

The reviewer reported:

- `cargo fmt --all --check` — exit 0.
- `cargo check --workspace --all-targets` — exit 0, including an authoritative
  clean-target repeat.
- `cargo test -p oneagent-protocol --test mcp_dispatch` — 6 passed.
- `cargo test -p oneagent-runtime --test mcp_semantic_tools` — 4 passed.
- `cargo test -p oneagent-runtime --test mcp_process` — 7 passed.
- `cargo test --workspace --all-targets` with approved repository-owned
  loopback access — 1,090 passed, zero failed or ignored.
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
- `cargo doc --workspace --no-deps` — exit 0; 18 crate documentation outputs
  generated in the review target.
- `git diff --check 0becccad^..844515d1` — exit 0.
- Ignored-test and removed-public-constructor searches — zero matches;
  dependency diff for the remediation commit — empty; output-error, forbidden-
  effect, path/stat, schema, and evidence searches completed.

All functional blockers were resolved, but the required reordered case still
covered only `oneagent.graph`, so the recommendation remained `blocked`.

### Final evidence-remediation review at `f24967d1`

The reviewer reported:

- `cargo fmt --all --check` — exit 0.
- `cargo test -p oneagent-runtime --test mcp_semantic_tools` — 4 passed, zero
  failed or ignored.
- Focused Runtime semantic-test Clippy with `-D warnings` — exit 0.
- `cargo test --workspace --all-targets` with approved repository-owned
  loopback access — 1,090 passed, zero failed or ignored.
- `git diff --check 0becccad^..f24967d1` — exit 0.
- Rust ignored-test and removed-public-constructor audits — zero matches.
- Final-remediation manifest and lockfile diff — empty; commit range,
  path/stat, all-11 edge-kind, duplicate, and reordered-equality audits — exact.
- Full workspace check, full workspace Clippy, and Rustdoc were not rerun at
  this final head. They had passed at the immediately preceding production
  head; `f24967d1` changes only the compiled and executed public Runtime
  integration test.

No functional finding regressed and no mandatory evidence gap remained, so the
final recommendation was `pass with non-blocking follow-up`.

## Primary validation and reconciliation

The following outcomes come from the primary agent's separate execution
record; they are not attributed to the independent reviewer reports. The
primary independently inspected the same range and remediation diffs and
reproduced every final reviewer conclusion.

At the `844515d1` production head, primary validation included:

- protocol dispatch — 6 passed;
- Runtime semantic tools — 4 passed;
- Runtime MCP process — 7 passed;
- `cargo fmt --all -- --check`, workspace check, strict workspace Clippy, and
  Rustdoc with `-D warnings` — exit 0;
- the complete workspace suite with approved loopback access — exit 0.

After applying the final test-only remediation represented by `f24967d1`, the
primary reran:

- `cargo fmt --all -- --check` — exit 0.
- `cargo check --workspace --all-targets` — exit 0.
- `cargo test -p oneagent-runtime --test mcp_semantic_tools` — 4 passed.
- `cargo test --workspace` with approved repository-owned loopback access —
  exit 0; the current inventory is 1,090 passed, zero failed or ignored.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — exit
  0.
- `git diff --check 0becccad^..f24967d1` — exit 0.

Rustdoc was not rerun after the final test-only commit; the strict Rustdoc run
had passed at `844515d1`, and no production or documentation path changed in
`f24967d1`.

No criterion, finding, missing-evidence conclusion, command result, scope
conclusion, or risk conflicts with the final independent report. The effective
decision is therefore `pass with non-blocking follow-up`, exactly as severe as
the reviewer recommendation.

## Scope and exclusion conformance

Included scope is complete: investigation, ADR-0051, protocol tool catalog and
dispatch, Runtime semantic composition, Tool Policy gate, immutable startup
snapshot, six deterministic tool families, bounded/error/redaction behavior,
stdio/process integration, public evidence, documentation, and independent
review evidence are present.

Excluded scope remains absent: graph/Context/source semantics changes;
mutation/write tools; confirmation UX and policy administration; filesystem,
shell, Git, network-client, or provider effects; workspace watching/reload;
remote transport and authentication; external-client compatibility;
concurrent calls and progress/cancellation notifications; other MCP
capabilities; IDE/LSP; and broad performance/security claims.

## Residual risks

- The upstream generated-schema provenance URL follows mutable `main`; an
  immutable commit pin remains a documentation follow-up.
- External-client interoperability, remote transport/authentication, concurrent
  dispatch, mutable workspaces, and IDE integration remain intentionally
  deferred.

## Artifact-consistency check

The same `/root/sprint29_integration_review` reviewer completed the required
read-only artifact-consistency check before any Roadmap transition, Sprint 28
prompt-suite retirement, staging, or final review commit. Two initial checks
correctly failed because the draft compressed the historical validation ledger,
misattributed one sandbox failure, included reviewer-unconfirmed primary/spawn
details, and omitted the bounded protocol-catalog evidence-gap history. The
primary corrected only those provenance and completeness defects.

The final check passed against draft SHA-256
`c85ed7571ce8ee5eb2f3fd1d8c342a1787b811c3c61415fa37e46273d1c6c770`. The
reviewer confirmed that the artifact preserves all three decisions, every
functional and evidence finding and resolution, the final missing-evidence
disposition, the mutable-URL follow-up, exact validation and unexecuted-check
history, primary/independent provenance separation, scope, exclusions,
residual risks, commits, and range statistics without weakening or inventing
reviewer evidence. Initial/final HEAD remained `f24967d1`; initial/final status
contained only this expected untracked artifact, which remained byte-identical
during the check. This section is the sole post-check artifact update and
records the completed check without changing its reviewed content.
