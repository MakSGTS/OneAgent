# Sprint 32 LSP Adapter Review

## Decision

`pass with non-blocking follow-ups`

The effective decision matches the final fresh-context independent reviewer
recommendation. Sprint 32 satisfies the accepted ADR-0054 boundary: one bounded
editor-neutral LSP 3.17 stdio process owns a single immutable Workspace startup
snapshot, advertises only UTF-16 positions, no document synchronization,
workspace symbols, and pull diagnostics, and projects only existing canonical
Graph and Workspace facts.

Three separate pre-review remediation commits close every blocking finding
reported by the preceding independent reviews. The final reviewer found no
remaining blocker. Its sole non-blocking finding was stale Roadmap completion
evidence: the committed text still recorded six public-process tests and did
not describe the notification-suppression remediation. The Task 8 state
transition reconciles that text to seven tests and records the third
remediation.

This decision does not claim mutable document analysis, post-start source
reads, definition or another deferred LSP method, IDE provider migration,
changed MCP semantics, remote transport, external-client compatibility,
diagnostic rules or configuration, edits, telemetry, or broad performance or
security properties.

## Reviewed baseline

- Completed Sprint 31 prerequisite: `daab0ecf`.
- Planning commit: `49b7d02b`.
- Task 7 evidence head: `1307110d`.
- Final remediation head: `12e7f21a`.
- Exact reviewed range:
  `daab0ecf192a6a860ffc5e297657fccea0582efa..12e7f21a094a465ab5b619fd6da7f254b86d024a`.
- Range size: 11 commits, 26 paths, 5,392 additions, 42 deletions.

The dependency-ordered commits are:

| Step | Commit | Subject | Result |
| --- | --- | --- | --- |
| Planning | `49b7d02b` | `Plan Sprint 32 LSP Adapter` | pass |
| Investigation | `9fcc930c` | `Investigate Sprint 32 LSP adapter` | pass |
| ADR-0054 | `62bf16fb` | `Define Sprint 32 LSP adapter` | pass |
| Protocol core | `c8cc5445` | `Implement Sprint 32 LSP protocol core` | remediated |
| Runtime lifecycle | `ee506b18` | `Implement Sprint 32 LSP runtime lifecycle` | remediated |
| Symbols and navigation | `0deefeb8` | `Implement Sprint 32 LSP navigation and symbols` | pass |
| Pull diagnostics | `b08e9399` | `Implement Sprint 32 LSP diagnostics` | remediated |
| Completion evidence | `1307110d` | `Complete Sprint 32 LSP evidence` | remediated |
| Numeric and root URI remediation | `d52a7cba` | `Fix Sprint 32 LSP conformance gaps` | pass |
| Projection and document URI remediation | `d199f454` | `Fix Sprint 32 LSP projection confinement` | pass |
| Notification remediation | `12e7f21a` | `Fix Sprint 32 notification suppression` | pass |

The range is bounded to Sprint 32 planning and prompts, pinned investigation,
ADR-0054, the additive protocol LSP module, Runtime process and projections,
public tests, CI, and synchronized current-state documentation. Graph, Common,
adapters, CLI, HTTP, and the VS Code extension production surfaces are
unchanged. No Cargo or Node manifest or lockfile changes.

The final reviewer began and ended at
`12e7f21a094a465ab5b619fd6da7f254b86d024a`. Its initial and final status were
identical:

```text
 M .codex/config.toml
 M docs/Roadmap.md
 M docs/architecture/mcp-semantic-tools-investigation.md
 M docs/reviews/sprint-29-mcp-semantic-tools.md
```

It used the committed Roadmap endpoint, confirmed an empty index, and made no
edit, creation, deletion, staging, commit, Roadmap transition, prompt
retirement, or delegation. The context was fresh.

## Independent reviewer handoff and report

Reviewer `/root/sprint32_final_review_4` received no inherited conversation
turns or expected decision. It received only the repository root, exact range,
authorities, objective, criteria, exclusions, validation matrix, output schema,
and strict read-only/no-delegation contract.

The reviewer recommended `pass with non-blocking follow-ups`. It independently
confirmed:

- 11 linear planning-through-remediation commits and the exact 9-file Sprint 32
  and 8-file Sprint 31 prompt inventories;
- protocol 12/12, Runtime LSP 8/8, stdio 5/5, public process 7/7, MCP domain
  15/15, MCP dispatch 6/6, semantic tools 6/6, and MCP process 9/9;
- exact lifecycle, capability, numeric, byte, position, URI, notification,
  symbol, diagnostic, framing, channel, and handler contracts;
- all 11 mixed EDT/Designer fixture hashes;
- no dependency, lockfile, generated-artifact, secret, personal-path, deferred
  production surface, or scope violation; and
- 307 changed-document local links with zero missing targets.

It found no blocking defect. Its only non-blocking finding was that committed
Roadmap lines still said six public-process tests and ended with the second
remediation. The review state transition corrects both facts.

## Review remediation history

| Reviewed head | Finding | Resolution |
| --- | --- | --- |
| `1307110d` | LSP numeric fields used broader JSON number domains, and Windows canonical roots could produce non-standard extended-path file URIs. | `d52a7cba` enforces the exact signed 32-bit LSP `integer` and non-negative signed-32-bit `uinteger` domains, adds complete exact/one-over projection evidence, and normalizes drive/UNC extended paths with independent oracles. |
| `d52a7cba` | Runtime silently omitted present over-bound canonical positions, and document URI validation accepted percent-encoded Windows separators such as `%5C`. | `d199f454` propagates invalid canonical coordinates as `Internal error` through symbols and diagnostics, while rejecting raw or encoded backslashes before containment. |
| `d199f454` | A notification with a method longer than 256 bytes produced an `InvalidRequest` response with `id: null`, violating notification silence. | `12e7f21a` suppresses method-bound errors after notification classification and adds exact 256/257-byte protocol and public-process regressions. |
| `12e7f21a` | Roadmap evidence still recorded six process tests and omitted the third remediation. | The Task 8 state transition updates the count to seven and records the notification remediation. |

## Acceptance evidence matrix

| Criterion | Independent and primary evidence | Result |
| --- | --- | --- |
| Planning, investigation, and architecture | Eleven commits preserve strict dependency order. The pinned investigation, ADR-0054, Roadmap, Architecture, and semantic model agree on semantic ownership, first slice, compatibility, and exclusions. | pass |
| Pinned LSP authority | The final reviewer verified the immutable pin from committed evidence. The primary obtained HTTP 200 for all eight exact Microsoft sources at commit `8be2e191506ced923953b94b985c4a1831757b39`. | pass |
| Protocol and lifecycle | Exact JSON-RPC shapes, duplicate/depth/body bounds, signed-32-bit IDs and fields, lifecycle states, capability truth, notification suppression, stable errors, and handler result validation are covered by 12/12 protocol tests. | pass |
| Framing and process ownership | The 8,192-byte header and 1 MiB body boundaries, CRLF framing, fragmentation, coalescing, malformed input, UTF-8, EOF, cancellation, I/O, flush, stderr redaction, exit status, and cleanup pass in 5/5 stdio tests and 7/7 public-process tests. | pass |
| Workspace roots and document URIs | Canonical Unix, Windows drive, UNC, and extended-path forms; uppercase percent encoding; root equality; lexical confinement; and raw/encoded separator and traversal rejection have independent unit and process evidence. | pass with residual Windows execution risk |
| Position projection | One-based canonical points convert to zero-based LSP `uinteger`; exact maximum passes, one-over fails as `Internal error`, and missing/ambiguous/conflicting/span-less/escaping evidence remains omission. Both symbol and diagnostic paths are covered. | pass |
| Workspace symbols | Only Procedure, Function, and EDT Query nodes with one confined span are projected, using kinds 12/19, ADR-0053 matching and five-part ordering, complete 100/101 bounds, and empty-array semantics. The public fixture returns four exact EDT/Designer symbols. | pass |
| Pull diagnostics | Only same-Configuration diagnostics with one exact matching source-node span are projected. Stable code, severity, message, order, full/empty reports, URI rejection, 100/101 bounds, and over-bound position propagation are covered. The public EDT document returns three exact diagnostics. | pass |
| Immutable authority | Runtime handlers own one `WorkspaceSnapshot`; request handlers query it and do not read or parse source. Protocol owns wire validation and has no Graph or adapter dependency. | pass |
| MCP and Workspace compatibility | MCP domain 15/15, dispatch 6/6, stdio 7/7, semantic tools 6/6, process 9/9, Graph Query 3/3, and Workspace 6/6 pass. The full workspace test gate has zero failures. | pass |
| Dependencies and public inventory | Both Runtime binaries build. No Cargo/Node manifest or lockfile changed; production LSP capabilities and exports agree with docs and handlers. | pass |
| CI and extension boundary | CI builds both Runtime binaries and runs Rust and unchanged extension jobs on `macos-14` and `windows-latest`. The range does not change extension production files. A fresh local extension run was unavailable because `node` is absent. | pass with residual execution risk |
| Documentation and scope | The primary repository-wide audit covers 248 Markdown files and 398 local links with zero missing. No excluded method, remote transport, source reanalysis, secret, personal path, or generated artifact was added. | pass |

## Findings

### Blocking

None remain at `12e7f21a`.

### Non-blocking follow-ups

1. Associate the immutable completion commit with a real `windows-latest` CI
   result when remote execution is available. The local review verified the
   cross-platform code, independent drive/UNC oracles, Windows-only assertion,
   and CI declaration but did not execute a Windows worker.
2. Repeat the unchanged extension typecheck/unit/Host/package matrix when a
   repository-compatible Node runtime is available. The primary typecheck
   stopped before `tsc` with `exec: node: not found`; the range changes no
   extension production file, package manifest, or lockfile.

The stale Roadmap count and missing third-remediation narrative are reconciled
inside the review state transition and are not deferred.

## Missing evidence and unexecuted checks

No mandatory local Rust evidence is missing after primary reconciliation.

The final reviewer maintained strict read-only operation and therefore did not
run Cargo commands that could update `target`. It instead executed current
endpoint test binaries for protocol 12/12, Runtime LSP 8/8, stdio 5/5, process
7/7, and the MCP focused suites. Its attempts to refetch the eight immutable
Microsoft sources were unavailable in its environment. It did not execute a
fresh Windows worker, full Graph/adapter/Workspace/HTTP/CLI regression matrix,
or extension suite.

The primary independently ran the complete canonical Rust gate, every focused
LSP suite, the MCP and Workspace compatibility matrix, public binary build,
fixture/link/dependency/scope audits, and obtained HTTP 200 for all eight
pinned sources. The extension typecheck was attempted but could not start
because `node` is unavailable. A real Windows CI worker was not available
locally. These limitations are residual execution risks, not hidden production
defects or unsupported completion claims.

No accepted focused filter matched zero tests. Workspace binary and doc-test
targets with zero tests were recorded but not used as functional evidence.

## Independent validation ledger

The final reviewer reported:

- `cargo fmt --all -- --check` — pass;
- protocol 12/12, Runtime LSP 8/8 with 83 filtered, stdio 5/5, process 7/7 —
  pass using current endpoint binaries;
- MCP domain 15/15, dispatch 6/6, semantic tools 6/6, process 9/9 — pass;
- `git diff --check <range>` and worktree `git diff --check` — pass;
- exact prompt inventories, dependency/lockfile, secret/path/generated-artifact,
  fixture 11/11, API/capability/handler, scope, and 307-link audits — pass;
- Cargo workspace check/test/Clippy/Rustdoc, fresh Windows, full compatibility,
  and extension execution — explicitly unexecuted under the read-only or local
  environment limits; and
- exact pinned-source refetch — unavailable because web open missed cache and
  direct fetch returned HTTP 403 in its environment.

## Primary validation and reconciliation

The primary independently inspected the same exact range and ran:

- `cargo fmt --all -- --check` — pass;
- `cargo check --workspace` — pass;
- `cargo test --workspace --quiet` — pass, zero failures;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` —
  pass;
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` — pass;
- protocol LSP 12/12, Runtime LSP 8/8, stdio 5/5, and public process 7/7 —
  pass;
- protocol MCP 6/6 and 15/15; Graph Query 3/3, MCP process 9/9, semantic tools
  6/6, MCP stdio 7/7, and Workspace 6/6 — pass;
- both public Runtime binaries — build pass;
- all eight pinned Microsoft URLs — HTTP 200;
- all 11 fixture hashes — exact;
- 248 Markdown files and 398 local links — zero missing;
- exact 9-file Sprint 32 and 8-file Sprint 31 inventories, empty index,
  dependency/lockfile, compatibility-surface, secret/path, generated-artifact,
  excluded-scope, range and worktree diff audits — pass; and
- extension typecheck — attempted and stopped before TypeScript execution with
  `node_modules/.bin/tsc: ... exec: node: not found`.

The primary and final reviewer agree on implementation correctness, scope,
compatibility, notification silence, and the effective
`pass with non-blocking follow-ups` decision. The primary supplies the complete
Cargo and compatibility matrix unavailable to the read-only reviewer. The
reviewer supplies independent executable confirmation of the final protocol,
Runtime, transport, process, and MCP boundaries. No unresolved evidence
disagreement remains.

## Scope and exclusion conformance

The reviewed range adds no mutable document store or synchronization,
post-start source analysis, definition, references, completion, hover, rename,
code actions, formatting, semantic tokens, edits, dynamic registration,
workspace or push diagnostics, remote transport, multiple roots, VS Code
provider migration, changed MCP semantics, external-client compatibility
claim, diagnostic rule/configuration, telemetry, or broad performance/security
claim.

Graph and Workspace remain semantic authorities, Runtime remains projection and
process owner, and protocol remains the bounded wire owner. Existing MCP, HTTP,
CLI, cache, adapter, and extension boundaries are preserved.

## Residual risks

- A real Windows worker has not run locally for the final immutable completion
  commit; CI and platform-specific tests are static/local cross-platform
  evidence until that run exists.
- The final reviewer did not regenerate Cargo artifacts or execute the complete
  compatibility matrix. Primary validation closes the local Rust evidence gap.
- The unchanged extension suite could not be repeated in the primary shell
  because no Node runtime is available. The reviewed range changes no extension
  or package dependency surface.

## Artifact consistency

The same final reviewer `/root/sprint32_final_review_4` performed a read-only
consistency check of this untracked artifact against its independent report for
`daab0ecf192a6a860ffc5e297657fccea0582efa..12e7f21a094a465ab5b619fd6da7f254b86d024a`
and returned `pass`. The reviewer confirmed that the artifact preserves without
weakening every finding, missing-evidence and unexecuted item, command result,
decision, scope/exclusion conclusion, residual risk, observed independent-review
HEAD/status, fresh-context/read-only/no-delegation evidence, and recommended
next action. Primary-only evidence remains explicitly separated from and does
not overwrite the independent evidence ledger. The consistency check did not
edit, create, delete, stage, or commit repository content, change Roadmap state,
retire prompts, or delegate work; HEAD remained
`12e7f21a094a465ab5b619fd6da7f254b86d024a`, and the only status delta relative
to the independent review was this expected untracked artifact.
