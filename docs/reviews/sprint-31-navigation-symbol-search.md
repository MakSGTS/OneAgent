# Sprint 31 Navigation and Symbol Search Review

## Decision

`pass with non-blocking follow-ups`

The effective decision matches the final fresh-context independent reviewer
recommendation. Sprint 31 satisfies the accepted ADR-0053 boundary: typed
source locations, truthful EDT and Designer XML producers, immutable Workspace
ownership, deterministic bounded symbol search, the seventh read-only MCP tool,
strict public-process and TypeScript clients, one-at-a-time symbol requests,
stale-safe Quick Pick replacement, exact source navigation, reproducible
packaging, and repository-owned public evidence.

The initial review was correctly blocked. Commit `e00540fb` remediates both
confirmed defects and adds regression evidence at the private handler, public
MCP process, TypeScript client/controller, and Extension Host boundaries. The
final reviewer found no remaining blocker. Its only non-blocking documentation
finding, stale test counts, is resolved by the review state-transition change:
extension unit tests are 38, Runtime MCP-process tests are 9, and Runtime
semantic-tool tests are 6.

This decision does not claim source-content disclosure, fuzzy ranking beyond
the accepted Unicode-lowercase substring contract, filesystem search, LSP or
VS Code provider APIs, reference search UI, diagnostics, chat/context UI,
workspace reload/watch behavior, remote/web/multi-root support, external-client
compatibility, Marketplace publication, telemetry, Runtime installation,
edits/refactoring, symlink-target containment, or broad performance/security
properties.

## Reviewed baseline

- Completed Sprint 30 prerequisite: `4b3198d1`.
- Planning commit: `6ac7a073`.
- Initial Task 7 review head: `e74d89e7`.
- Remediation head: `e00540fb`.
- Exact reviewed range:
  `4b3198d1eaf519d804a30ae95dfb79b74f2bf1ae..e00540fb1737195809ba846a52963ced8b590a90`.
- Range size: 8 commits, 46 paths, 4,789 additions, 140 deletions.

The dependency-ordered commits are:

| Step | Commit | Subject | Result |
| --- | --- | --- | --- |
| Planning | `6ac7a073` | `Plan Sprint 31 Navigation and Symbol Search` | pass |
| Investigation | `dbe3c618` | `Investigate Sprint 31 navigation and symbol search` | pass |
| ADR-0053 | `241f70cf` | `Define Sprint 31 navigation and symbol search` | pass |
| Source locations | `21bf1b84` | `Implement Sprint 31 source location model` | pass |
| MCP symbol tool | `a52d9d3d` | `Implement Sprint 31 navigation MCP tools` | remediated |
| VS Code integration | `444a75f8` | `Integrate Sprint 31 VS Code navigation and search` | remediated |
| Completion evidence | `e74d89e7` | `Complete Sprint 31 navigation and search evidence` | remediated |
| Review remediation | `e00540fb` | `Remediate Sprint 31 integration review blockers` | pass |

The range is bounded to Sprint 31 planning and prompts, the pinned
investigation, ADR-0053, additive Common/Graph location data, EDT and Designer
XML location producers, Runtime/Workspace/MCP projection, the VS Code command
and client, fixtures/tests, CI/package evidence, and synchronized current-state
documentation. It adds no Cargo or production Node dependency.

At both independent review boundaries, `HEAD` remained the reviewed committed
SHA. The worktree contained only these pre-existing user changes:

```text
 M docs/Roadmap.md
 M docs/architecture/mcp-semantic-tools-investigation.md
 M docs/reviews/sprint-29-mcp-semantic-tools.md
```

Reviewers used committed snapshots for those paths. They reported no edit,
staging, commit, deletion, or delegation, and the final status was identical to
the initial status.

## Independent reviewer handoff and reports

The initial reviewer `/root/sprint31_final_review` received a fresh context,
the exact `4b3198d1..e74d89e7` range, authorities, criteria, exclusions,
validation matrix, required report schema, and a strict read-only/no-delegation
contract. It recommended `blocked` for two confirmed defects:

1. `oneagent.symbols` performed Configuration lookup before validating `limit`
   and kinds, while the Tool Policy decision also preceded symbol semantic
   validation. The public combination `configurationId="missing", limit=0`
   returned `not_found` instead of `invalid_arguments`.
2. Replacing a Quick Pick while its symbol request remained in flight created
   a second request against the single pending JSON-RPC slot. The TypeScript
   client classified that as `protocol_failure`, moved to `failed`, and stopped
   the Runtime. A new valid query also left old selectable items visible until
   its response arrived.

The separately authorized remediation reviewer
`/root/sprint31_remediation_review` also received a new fresh context and the
strict read-only/no-delegation contract. It reviewed the complete
`4b3198d1..e00540fb` range and recommended
`pass with non-blocking follow-ups`. It independently confirmed both blockers
closed and found no replacement defect. Its only finding was that the committed
Roadmap still recorded the pre-remediation `36/8/5` counts instead of
`38/9/6`.

The final reviewer began and ended at
`e00540fb1737195809ba846a52963ced8b590a90`, with the same three user-owned
unstaged paths shown above. It confirmed fresh context, read-only operation,
and no delegation.

## Review remediation history

| Reviewed head | Finding | Resolution |
| --- | --- | --- |
| `e74d89e7` | Symbol argument validation did not precede Configuration lookup and Tool Policy outcome; combined invalid/missing input returned the wrong stable code. | `e00540fb` centralizes complete symbol argument decoding before lookup, prevalidates the seventh handler before policy, retains policy denial for valid input, and adds exact handler, semantic, and public-process regression cases. |
| `e74d89e7` | A replacement Quick Pick could collide with an old in-flight request, abort the shared Runtime client, and retain stale selectable results. | `e00540fb` serializes symbol operations over the existing one-request protocol slot, immediately clears presented items for a new searchable query, and adds unit plus public Extension Host replacement evidence. |
| `e00540fb` | Roadmap completion evidence undercounted the new regression tests. | The final state transition updates the counts from `36/8/5` to `38/9/6`. |

## Acceptance evidence matrix

| Criterion | Independent and primary evidence | Result |
| --- | --- | --- |
| Planning, investigation, and architecture | Eight commits preserve strict task order. The pinned investigation, ADR-0053, Roadmap, Architecture, and semantic model agree on ownership, first slice, compatibility, and exclusions. | pass |
| Common source locations | `SourcePath` is normalized and bounded to 4,096 UTF-8 bytes; positions are one-based; spans are ordered half-open values; locations have optional spans. Common 5/5 passed. | pass |
| Graph provenance and identity | Opaque source identity and typed location remain separate. The compatibility constructor is preserved; location participates in provenance equality without changing node or edge identity. Graph 162/162 passed. | pass |
| EDT and Designer XML producers | Locations derive from actual module paths and parser declaration/query lines. EDT covers Module/Procedure/Function/Query; Designer covers Module/Procedure/Function and truthfully omits Query. EDT 258/258 and Designer XML 31/31 passed. | pass |
| Workspace and cache | Immutable startup and Configuration roots are retained; locations round-trip through the bumped semantic cache version. Runtime 83/83 and persistent-cache 4/4 passed. | pass |
| Matching, ordering, ambiguity, and confinement | Search uses bounded Unicode-lowercase substring matching, exact kind/configuration filters, stable five-part ordering, total-before-limit truncation, unique-location projection, and lexical confinement to Configuration plus Workspace roots. | pass |
| Error precedence and Tool Policy | Invalid symbol arguments precede Configuration lookup and policy outcome; valid calls remain Tool Policy gated. Combined invalid/missing, valid/missing, and denied-policy cases pass at private, semantic-handler, and public-process boundaries. | pass |
| Protocol and public process | Discovery, schema, handler, annotations, policy rule, public JSON lines, stable errors, bounds, channel purity, EOF, and all seven tool families agree. Protocol 7/7, Tool Policy 26/26, semantic tools 6/6, MCP process 9/9 passed. | pass |
| TypeScript client | The exact seven-tool catalog and strict symbol result/path/span decoder are preserved. Repeated calls retain at most one request in flight and keep the connection `connected`. | pass |
| Quick Pick replacement | Input coalescing, generation invalidation, immediate stale-item clearing, replacement result presentation, navigation, and disposal are covered by controller and Host evidence. | pass |
| Navigation safety | Selection repeats relative-path validation and root confinement, opens only the exact workspace document, converts one-based coordinates to zero-based VS Code selections, and has no source-reading fallback. | pass |
| Lifecycle and compatibility | Existing connect/disconnect/configuration/deactivation behavior, six previous tools, HTTP/CLI Runtime surfaces, framing, redaction, and cleanup remain covered. | pass |
| Extension package | Frozen Node 24/pnpm 11.19.0 install, typecheck/compile, 38 unit tests, 2 public process tests, and 16 pinned VS Code 1.134.0 Host tests passed. Package inventory is 10 files; two clean VSIX builds each contain the exact same 12 archive files. | pass |
| Dependency/license/generated audit | The audit covers 39 tracked extension files, 18 license groups, three current documents, zero production dependencies, expected ignored artifacts, and no tracked generated output. Cargo and pnpm lockfiles are unchanged. | pass |
| Cross-platform CI | Repository CI declares the complete Rust and extension boundaries on `macos-14` and `windows-latest`; platform branches and package contracts are present. A fresh Windows worker was not executed locally. | pass with residual execution risk |
| Scope and documentation | Current docs describe the seven-tool/four-kind boundary and deferrals. Repository-wide local Markdown links resolve, and no excluded production surface was found. | pass |

## Findings

### Blocking

None remain at `e00540fb`.

### Non-blocking follow-ups

1. Associate the immutable final commit with a real `windows-latest` CI result
   when remote execution is available. The local reviews verified the workflow,
   `.exe`/`.cmd` paths, package inventory, and platform-neutral contracts but
   did not execute a Windows worker.
2. A portable non-UTF-8 filesystem-path case is not represented by a dedicated
   Sprint 31 test. The accepted public contract is UTF-8 relative paths and
   fail-closed omission, so this is not a correctness blocker; add a
   platform-conditional regression only if a portable repository-owned oracle
   becomes available.
3. Pinned trusted Extension Host runs may print VS Code bootstrap `SIGPIPE`
   diagnostics while all assertions and host processes still exit zero. Track
   the upstream noise separately if it remains reproducible.

## Missing evidence and unexecuted checks

No mandatory local evidence is missing after primary reconciliation.

The initial reviewer could not independently run TypeScript, Extension Host,
or package commands because its shell had no usable `node`; it reported those
checks as unexecuted rather than passed. It also did not fetch the external
VS Code references named by the pinned investigation. These limitations did
not hide its two independently reproduced blockers.

The final reviewer maintained read-only operation and therefore did not run
commands that would create or replace `target`, `dist`, `.vscode-test`, or VSIX
artifacts. It used current repository-local Rust binaries, pinned TypeScript
tools through the available VS Code Node, the existing VSIX, and static package
evidence. It independently executed current unit and real-process tests but not
a fresh Extension Host matrix, clean install, two VSIX rebuilds, or remote
Windows CI. The standard `pnpm run typecheck` path failed because the pnpm
wrapper could not find `node`; the reviewer then ran equivalent read-only
`--noEmit` checks with pinned TypeScript and the available VS Code Node. Its
`scripts/audit.mjs` run failed with `ERR_SQLITE_ERROR: unable to open database
file`; escalation to the global pnpm database was rejected under the repository
boundary, so the reviewer did not execute a fresh license-group audit. The
primary independently executed every required clean/generated artifact and
license gate with the necessary local permissions.

Marketplace publication/signing, remote/web hosts, external MCP clients,
symlink-target resolution, and every other deferred area are outside Sprint 31
rather than missing completion evidence.

## Initial independent validation ledger

At `e74d89e7`, the initial reviewer reported:

- exact range, seven ordered commits, eight Sprint 31 prompts, clean range
  diff, empty index, and unchanged three-path user status;
- canonical Rust format, workspace check/test, strict Clippy, and
  warning-denying Rustdoc — all passed;
- focused Common, BSL, Graph, EDT, Designer XML, protocol, Tool Policy,
  Runtime semantic, MCP process, and permitted persistent-cache rerun — passed;
- production dependency list empty and 18 license groups;
- 303 authoritative/current-state local links with zero missing targets;
- exact eight-file Sprint 30 prompt inventory and zero prohibited/deferred,
  secret, personal-path, generated, or dependency matches;
- extension typecheck/build/unit/process/Host/VSIX commands — unexecuted because
  `node` was unavailable in that review shell;
- the exact public precedence reproduction — exit zero and returned
  `not_found`, confirming the first blocker.

It also identified the missing portable non-UTF-8 path case and the absence of
a fresh Windows CI run as evidence gaps, not additional production defects.

## Final independent validation ledger

At `e00540fb`, the final reviewer reported:

- unchanged exact HEAD/status, `git diff --check <range>` exit 0, and
  `git diff --check` exit 0;
- `cargo fmt --all -- --check` — exit 0. Workspace check/test, strict Clippy,
  and warning-denying Rustdoc were not invoked through Cargo because they could
  write `target`; current repository-local test binaries were used instead;
- Common 5/5, Graph 162/162, BSL 37/37, EDT 258/258, Designer XML 31/31,
  Runtime 83/83, persistent cache 4/4, Tool Policy 26/26, protocol 7/7,
  CLI 18/18, semantic tools 6/6, and MCP process 9/9 — pass;
- first sandboxed Runtime and CLI runs failed only on loopback bind with
  `PermissionDenied`; the same binaries outside that sandbox passed 83/83 and
  18/18;
- standard `pnpm run typecheck` — failed before `tsc` with
  `node_modules/.bin/tsc: exec: node: not found`; equivalent read-only
  `--noEmit` checks for both configurations passed using pinned TypeScript
  7.0.2 and VS Code/Electron Node 24.18.1;
- compiled unit tests 38/38 and real-process tests 2/2 — pass;
- the source/configuration matrix contains 16 non-zero Extension Host tests:
  trusted 6 twice, three unsupported-workspace cases, and one Restricted Mode
  case. The reviewer did not execute a fresh Host process;
- direct package inventory — exact 10 files;
- existing VSIX inventory — exact 12 files, with all six packaged JavaScript
  hashes equal to current `dist`;
- 39 tracked extension files, 6/6 ignore probes, zero production dependency,
  lockfile, generated, secret, personal-path, filesystem-search,
  opaque-provenance-decoding, and deferred production API additions. One raw
  `exec(` match was inspected and proved to be a RegExp-parser method call, not
  process execution or another deferred production API;
- 394 repository-wide and 303 authoritative/current-state local links — zero
  missing targets;
- fresh Cargo workspace test/Clippy/Rustdoc, Host, clean install, two-VSIX
  rebuild, and Windows execution — explicitly unexecuted under its
  read-only/environment limits;
- `scripts/audit.mjs` — failed with `ERR_SQLITE_ERROR: unable to open database
  file`; escalation was rejected under the repository boundary, and no fresh
  reviewer license-group audit was executed.

Its sole finding was the stale `36/8/5` Roadmap count. It recommended updating
the values to `38/9/6` before the final review commit.

## Primary validation and reconciliation

The primary independently inspected the exact range, reproduced both initial
findings, implemented them in the separate `e00540fb` remediation commit, and
then ran:

- `cargo fmt --all -- --check` — pass;
- `cargo check --workspace` — pass;
- `cargo test --workspace` with permitted loopback access — pass, zero failed;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` —
  pass;
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` — pass;
- exact focused handler, semantic-tool, and public MCP-process precedence
  regressions — pass;
- frozen offline pnpm install, clean, both TypeScript typechecks, and compile —
  pass;
- extension unit tests — 38 passed, zero failed/skipped/todo;
- public extension real-process tests — 2 passed;
- complete pinned Extension Host matrix — 16 passed: trusted 6 twice, empty 1,
  virtual 1, multi-root 1, Restricted Mode 1;
- package list/check — exact 10 files;
- package verification — exact 12 VSIX entries across two clean builds;
- extension audit — 39 tracked files, 18 license groups, three documents;
- repository-wide Markdown audit — 244 tracked Markdown files, 394 local
  targets, zero missing;
- exact eight-file Sprint 30 prompt inventory, range dependency/lockfile/scope
  checks, `git diff --check`, and explicit staged-path audit — pass.

Validation-only environment failures were kept separate from repository
results. The first sandboxed Host command ended in `SIGABRT`; the permitted
rerun passed all 16 tests. The first sandboxed Rust workspace run failed only
because two existing CLI tests could not bind loopback; the identical permitted
full rerun passed. Real-process tests were rerun with the required absolute
`ONEAGENT_MCP_BIN`. `vsce` required a temporary ignored repo-local npm-to-pnpm
shim for its prepublish subprocess; the shim was removed after two clean VSIX
builds. The first sandboxed audit could not open pnpm's SQLite database; the
permitted rerun passed. None of these attempts changed tracked scope.

The primary and final reviewer agree on implementation correctness, scope,
compatibility, error precedence, one-request Quick Pick replacement, and the
effective `pass with non-blocking follow-ups` decision. The primary provides
the complete clean/generated-artifact matrix unavailable to the read-only
reviewer. The reviewer supplies independent static and executable confirmation
of both remediation contracts. No unresolved evidence disagreement remains.

## Scope and exclusion conformance

The final range adds no source-content API, fuzzy/relevance ranking,
filesystem-search fallback, LSP/definition/reference/document/workspace-symbol
provider, reference UI, diagnostics UI, chat/context/webview UI, reload/watch
behavior, remote/web/multi-root support, external-client claim, Marketplace
surface, telemetry, Runtime installer, edit/refactoring surface, or new
dependency. Matches for remote, multi-root, diagnostics, and telemetry are
negative tests or audit vocabulary rather than production capabilities.

Node/edge identities, opaque provenance ownership, existing six-tool behavior,
Runtime HTTP/CLI, Sprint 30 lifecycle, MCP channel purity, failure cleanup,
redaction, package scope, and explicit deferrals remain preserved.

## Residual risks

- The final reviewer did not execute fresh Windows, Extension Host, clean
  install, Cargo generated-artifact, or two-build VSIX checks. It relied on
  repository-local Rust binaries, the existing VSIX, static Host inventory, and
  current compiled TypeScript tests. The primary independently closed the full
  clean/generated-artifact matrix.
- A real Windows worker remains unexecuted locally; CI configuration and
  platform branches are static evidence until remote CI runs.
- Symlink-target containment remains explicitly outside ADR-0053. The public
  path contract is lexical and fail-closed.
- The portable non-UTF-8 filesystem-path and passing-host `SIGPIPE` diagnostics
  remain the non-blocking follow-ups listed above.
- The three original user changes remain unstaged and untouched throughout the
  review workflow.

## Next action

The draft details identified by the same reviewer were corrected and the
mandatory consistency check was repeated successfully. Execute only the
already-declared documentation transition, validate links/state/staged paths,
commit it atomically, and make Sprint 32 — LSP Adapter eligible as the next
planning target. Associate that immutable commit with a real Windows CI result
when remote execution becomes available.

## Artifact consistency

The first consistency check by `/root/sprint31_remediation_review` returned
`fail` because the draft compressed several command outcomes and environment
limits too aggressively. The primary corrected only the review artifact: exact
Git/fmt results, the standard pnpm typecheck failure and equivalent read-only
fallback, the denied pnpm SQLite audit escalation and missing reviewer license
run, the static non-zero 16-test Host matrix, the inspected RegExp `exec(`
match, residual risks, and the ordered next action.

The same reviewer repeated the check in the same context and returned `pass`.
It confirmed that every prior correction was exact; the initial blocked
reviewer, final remediation reviewer, and primary evidence remained clearly
separated; the complete preservation contract held; and recording this result
plus the declared `38/9/6` Roadmap/state/prompt transition could not weaken its
report. Both checks were strictly read-only, with no repository mutation and no
delegation.

## Final state transition

After a passing artifact-consistency check, the authorized review change:

- records this review artifact;
- synchronizes the Roadmap counts to 38 extension unit, 9 MCP-process, and 6
  semantic-tool tests;
- marks Sprint 31 `completed`;
- marks Sprint 32 — LSP Adapter as the unique `next` sprint;
- retires exactly the eight tracked Sprint 30 prompt files enumerated by the
  Sprint 31 Task 7 contract.

No production or test file changes in this final review commit.
