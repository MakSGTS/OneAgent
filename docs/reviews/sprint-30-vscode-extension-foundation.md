# Sprint 30 VS Code Extension Foundation Review

## Decision

`pass with non-blocking follow-ups`

The effective decision matches the final independent reviewer recommendation.
Sprint 30 satisfies the accepted ADR-0052 boundary: one reproducible desktop
VS Code workspace extension, demand-only activation, bounded configuration,
one owned `oneagent-mcp` stdio process, exact MCP compatibility checks, a
closed connection lifecycle, deterministic status presentation, awaited
cleanup, reproducible packaging, and repository-owned public evidence.

This decision does not claim navigation or symbol search, LSP, diagnostics UI,
chat or context UI, EDT integration, remote or web hosts, multi-root fan-out,
concurrent MCP dispatch, workspace watching or reload, Runtime installation,
Marketplace publication or signing, telemetry, authentication, live external-
client compatibility, or broader performance and security properties.

## Reviewed baseline

- Framework prerequisite: `90695c74`.
- Planning commit: `f0958292`.
- Initial Task 7 review head: `99eef30c`.
- First remediation head: `0e4ccd3c`.
- Lifecycle and evidence remediation head: `b3abae08`.
- Extension Host evidence remediation head: `d3b56765`.
- Final production-guard and review head: `70bc5ff0`.
- Exact reviewed range:
  `90695c745da878a4222827ce1311ba495fe29c6e..70bc5ff0f73f00d5e8a2af6c3389e626ce794a6b`.
- Final full review HEAD:
  `70bc5ff0f73f00d5e8a2af6c3389e626ce794a6b`.
- Range size: 11 commits, 50 paths, 7,786 additions, 7 deletions.

The dependency-ordered commits are:

| Step | Commit | Subject | Result |
| --- | --- | --- | --- |
| Planning | `f0958292` | `Plan Sprint 30 VS Code Extension Foundation` | pass |
| Investigation | `1799fe7a` | `Investigate Sprint 30 VS Code extension foundation` | remediated |
| ADR-0052 | `792fa95f` | `Define Sprint 30 VS Code extension foundation` | pass |
| Extension package | `bb460b79` | `Establish Sprint 30 VS Code extension package` | pass |
| MCP client | `8bd1e751` | `Implement Sprint 30 MCP runtime client` | remediated |
| Runtime lifecycle | `fd3fd9db` | `Integrate Sprint 30 VS Code runtime lifecycle` | remediated |
| Production evidence | `99eef30c` | `Complete Sprint 30 VS Code extension evidence` | remediated |
| Contract remediation | `0e4ccd3c` | `Remediate Sprint 30 integration review blockers` | remediated |
| Lifecycle evidence remediation | `b3abae08` | `Complete Sprint 30 lifecycle evidence remediation` | remediated |
| Host evidence remediation | `d3b56765` | `Complete Sprint 30 Extension Host evidence remediation` | remediated |
| Production guard | `70bc5ff0` | `Restrict Sprint 30 Host evidence API` | pass |

The range is bounded to Sprint 30 planning and prompts, the investigation,
ADR-0052, extension package and CI configuration, TypeScript implementation,
tracked fixtures and tests, package evidence, and synchronized current-state
documentation. It changes no Rust or Cargo path and introduces no production
Node dependency.

## Independent reviewer handoff and reports

The initial independent review used `/root/sprint30_final_review`. It received
a fresh context, the exact committed range and authorities, and a strict read-
only/no-delegation contract. It reviewed `90695c74..99eef30c` and recommended
`blocked` for two High findings: startup failure cleanup could retain a child
under the wrong terminal classification, and mutable upstream references did
not preserve the claimed pinned platform provenance.

The separately authorized remediation review used
`/root/sprint30_remediation_review`. It also began with a fresh context and the
same read-only/no-delegation constraint. The same reviewer retained its
independent report through all later remediation re-reviews:

| Reviewed head | Recommendation | Remaining blocker |
| --- | --- | --- |
| `0e4ccd3c` | `blocked` | A late stderr/process error could replace terminal `shutdown_failed`; mandatory Host, real-process, cleanup, safe-ID, and platform evidence remained incomplete. |
| `b3abae08` | `blocked` | Committed Host tests still did not observe actual status-item mapping/disposal or a second successful supported-host lifecycle. |
| `d3b56765` | `blocked` | The Host observation API was guarded only by an inherited environment marker and could conditionally expand Production extension exports. |
| `70bc5ff0` | `pass with non-blocking follow-ups` | None. |

At final review start and finish, HEAD was the same final SHA above. Tracked
status contained only three pre-existing user changes:

```text
 M docs/Roadmap.md
 M docs/architecture/mcp-semantic-tools-investigation.md
 M docs/reviews/sprint-29-mcp-semantic-tools.md
```

The reviewer ignored those changes as committed evidence. The index remained
empty. The reviewer reported no tracked edit, staging, commit, Git-state
mutation, or delegation; only ordinary ignored validation artifacts were
created and cleaned or retained under ignored paths.

## Review remediation history

| Reviewed head | Finding or gap | Resolution |
| --- | --- | --- |
| `99eef30c` | Startup timeout/protocol failure published its original failure before cleanup; failed graceful and forced shutdown could retain the child/listeners without terminal `shutdown_failed`. | `0e4ccd3c` awaits cleanup, gives `shutdown_failed` terminal precedence, retains ownership only until late exit, and adds the combined startup-timeout/failed-shutdown test. |
| `99eef30c` | The investigation called mutable stable-release/documentation endpoints pinned historical evidence. | `0e4ccd3c` separates dated mutable release inventory from immutable VS Code `1.134.0` source tag and commit `474a349a...`. |
| `0e4ccd3c` | Late stderr or process `error` after terminal `shutdown_failed` could replace the failure with `stderr_overflow` or another failure and start a new timer. | `b3abae08` ignores late stderr/error outside connecting or connected state, keeps the terminal state stable, and releases all listeners after late exit. |
| `0e4ccd3c` | Pure-client evidence did not completely cover timers, stdout/stderr/error/exit/close listeners, or safe-integer request-ID exhaustion. | `b3abae08` adds one complete release assertion and exact `MAX_SAFE_INTEGER`/one-over fail-closed cases. |
| `0e4ccd3c` | Real-process evidence did not prove the exact selected working directory and a real Runtime startup failure. | `b3abae08` starts the public binary in a tracked conflicting-project fixture, observes the real redacted `process_exited` failure, and retains repeated successful handshake/EOF runs. |
| `0e4ccd3c` | Host evidence omitted actual Restricted Mode, empty, virtual, and multi-root windows, no-spawn proof, configuration precedence, and broader disposal/repetition behavior. | `b3abae08` adds isolated pinned hosts, a custom Workspace Trust launcher, cross-platform spawn markers, exact workspace fixtures, precedence/replacement checks, and repeatable deactivation. |
| `b3abae08` | Actual `StatusBarItem` text/tooltip/command, status disposal, configuration-listener disposal, and a second supported lifecycle were still unobserved. | `d3b56765` runs two isolated trusted profiles and observes all five actual status presentations plus status, connect, disconnect, and configuration disposal in each five-test cycle. |
| `d3b56765` | An inherited `ONEAGENT_HOST_CASE` value alone enabled the observation API, so a Production host could expose an undocumented extension API. | `70bc5ff0` adds a closed equality allowlist for two non-production profiles. Unit evidence denies both trusted markers in Production and explicitly denies `undefined`, `""`, `untrusted`, `empty`, `virtual`, and `multi-root` in both modes. Every other value is rejected by the production equality check. Production `activate()` returns `undefined`. |

## Acceptance evidence matrix

| Criterion | Independent and primary evidence | Result |
| --- | --- | --- |
| Framework ancestry and task order | `90695c74` is an ancestor; 11 linear planning-through-remediation commits preserve dependency order. | pass |
| Investigation and architecture | Immutable platform/toolchain provenance, repository owners, alternatives, lifecycle, failures, bounds, packaging, and exclusions agree with accepted ADR-0052. | pass |
| Toolchain and dependencies | VS Code `^1.134.0`, pinned host `1.134.0`, Node 24, pnpm `11.19.0`, locked exact development dependencies, and zero production dependencies agree. | pass |
| Manifest and activation | One desktop workspace extension has demand-only contributed-command activation, two commands, one window-scoped setting, Workspace Trust limitation, and no eager process startup. | pass |
| Configuration and workspace gate | Executable byte bounds, exactly one trusted file workspace, workspace-over-global precedence, and untrusted/empty/virtual/multi-root rejection before spawn are covered. | pass |
| MCP compatibility | Sequential discover/list initialization, exact protocol version and six-tool catalog, bounded framing/depth/stderr/request IDs, duplicate and malformed input, and redacted failures are covered. | pass |
| Process lifecycle | Success, timeout, startup failure, unexpected exit, graceful/forced/failed shutdown, late events, late exit, replacement, retry, and repetition are deterministic. | pass |
| Resource ownership | Timers and stdout/stderr/error/exit/close listeners release on terminal cleanup; ownership is retained only until a late owned child exit. | pass |
| Status UI | All five exact actual status text/tooltip/command presentations are observed in two pinned trusted hosts. | pass |
| Disposal and repetition | Status item, two commands, and configuration listener are disposed; commands reject afterward; two isolated supported activation/deactivation cycles pass. | pass |
| Test observation scope | Production code uses a closed equality allowlist for the two tracked non-production profiles. Unit tests deny both trusted markers in Production and explicitly deny `undefined`, `""`, `untrusted`, `empty`, `virtual`, and `multi-root` in both modes. | pass |
| Real public Runtime | Two fresh successful public-process handshakes/EOF exits plus a tracked exact-cwd real startup failure are covered without path disclosure. | pass |
| Packaging | Exact nine-file payload and exact eleven-entry VSIX archive are reproduced across two clean builds. | pass |
| Dependency/license audit | The audit covers 37 tracked extension files, 18 license groups, and three current documents. | pass |
| Cross-platform design | CI declares Node/Rust jobs on `macos-14` and `windows-latest`; `.exe` and `.cmd` branches and platform-neutral tests agree. | pass with residual execution risk |
| Compatibility and ownership | The range contains zero Rust/Cargo changes; protocol and semantic authority remain in Rust and the extension remains an editor/process adapter. | pass |
| Deferred scope | All named navigation/LSP/UI/EDT/remote/concurrency/watching/install/publication/telemetry/authentication deferrals remain absent. | pass |

## Findings

### Blocking

None remain at `70bc5ff0`.

### Non-blocking follow-ups

1. Both trusted Extension Host cycles emit `Error: Unexpected SIGPIPE` from VS
   Code `bootstrap-fork.js`. All ten trusted assertions pass, both hosts exit
   zero, and disposal evidence completes, so no leak or lost assertion was
   found. A later diagnostic task should localize the source if the message
   remains reproducible.
2. The actual `windows-latest` job was not executed in either local review
   environment. The repository has no configured Git remote. Static CI,
   `.exe`/`.cmd`, packaging, and platform-neutral evidence agree, but the
   immutable final commit should be associated with a real Windows CI result
   when remote execution becomes available.

## Missing evidence and unexecuted checks

No mandatory evidence is missing for the locally accepted ADR-0052 boundary.

The final independent reviewer did not perform a new frozen install because
its read-only contract prohibited installing or updating dependencies; it used
the existing ignored dependency tree. The reviewer also could not run an
actual Windows CI worker. Marketplace publication/signing and every other
deferred area are outside Sprint 30 rather than missing completion evidence.

## Independent validation ledger

### Initial review at `99eef30c`

The initial review began at `2026-08-26T23:41:49+07:00` and ended at
`2026-08-26T23:57:00+07:00`. Its full unchanged HEAD was
`99eef30c9e1ea048058ec8a7c6dd76838d92d60d`.

The first reviewer reported:

- exact ancestry, seven Task 1-7 commits, range and `git diff --check` — pass;
- initial `node --version` — exit 127; the first typecheck — exit 1 with
  `node: not found`;
- typecheck and compile using cached Electron Node `24.18.1` — exit 0; the
  investigation named bundled Node `24.19.0`, which was unavailable in the
  reviewer shell;
- unit tests — 24 passed;
- public extension real-process tests — 2 passed;
- two sandboxed Extension Host attempts — `SIGABRT`; the permitted host rerun
  — 4 passed;
- package list — exact nine files;
- `package:check` — exit 1 only because an Electron codesign diagnostic on
  stderr violated the assertion; the two underlying nine-file inventories
  were equal;
- first `package:verify` — exit 1 because `npm` was unavailable; a temporary
  ignored npm-to-pnpm shim produced exact eleven-entry inventories across two
  clean builds and was then removed;
- first `cargo test --workspace` — exit 101 because two existing CLI tests
  could not bind loopback; the permitted rerun — 1,090 passed with zero failed
  or ignored tests;
- fmt, workspace check, strict Clippy, focused seven-process tests, and Rustdoc
  — pass;
- 280 required local links — no missing targets;
- Sprint 30 prompt inventory — exactly 8; Sprint 29 — exactly 9;
- all 13 cited official VS Code URLs were reachable, while the mutable stable
  inventory had already advanced to `1.135.0`;
- zero prohibited/deferred APIs, ignored tests, generated tracked artifacts,
  secret/path disclosures, production dependency declarations, and Sprint
  range Rust/Cargo changes.

The reviewer did not perform a clean install under the read-only contract.
`audit` and `pnpm list --prod` failed with `ERR_SQLITE_ERROR` because the pnpm
database could not be opened, and the external-database escalation was denied.
`pnpm licenses list --json` failed with
`ERR_PNPM_MISSING_PACKAGE_INDEX_FILE`. Windows was not run. These environment
limitations did not override the two High findings, so the recommendation was
`blocked`.

### Remediation reviews through `b3abae08`

The remediation reviewer independently reproduced the late-terminal-state
defect at `0e4ccd3c`, audited all missing-evidence categories, and kept the
decision `blocked`. At `b3abae08`, it reported:

- 26 unit tests;
- 2 real-process tests;
- 9 pinned Extension Host tests across trusted, Restricted Mode, empty,
  virtual, and multi-root windows;
- exact nine-file package and eleven-entry two-build VSIX inventories;
- audit coverage of 37 tracked extension files, 18 license groups, and three
  documents;
- the full 1,090-test Rust gate, focused MCP process tests, Clippy, and Rustdoc;
- 281 committed links and exact prompt inventories;
- zero Rust/Cargo changes, production dependencies, ignored tests, generated
  tracked artifacts, personal absolute paths, or excluded feature additions.

It confirmed the lifecycle, late-event, real-cwd, safe-ID, Host-shape, no-spawn,
and package blockers closed, but correctly kept `blocked` because actual status
and full supported-host repetition/disposal evidence were incomplete.

### Final Host and production-guard reviews

At `d3b56765`, both isolated trusted hosts passed five tests each and observed
all five status snapshots and four disposal flags. The reviewer nevertheless
kept `blocked` because the environment-only test API gate could affect a
Production extension.

At `70bc5ff0`, the reviewer reported:

- typecheck and compile — exit 0;
- unit tests — 27 passed, zero failed, skipped, or todo;
- two trusted hosts — 5 passed each, exit 0;
- empty, virtual, multi-root, and actual Restricted Mode hosts — 1 passed each;
- public Runtime process tests — 2 passed;
- package check — exact nine files;
- VSIX verification — exact eleven entries in two clean builds;
- audit — 37 tracked files, 18 license groups, three documents;
- canonical unchanged Rust tree — fmt/check/workspace 1,090/focused MCP
  process 7/Clippy/Rustdoc all pass;
- 281 committed links in 20 authoritative files — no missing targets;
- exact eight-file Sprint 30 prompt inventory and zero prohibited test,
  artifact, dependency, path, scope, or ownership matches;
- range, worktree, index, and `git diff --check` — pass.

The final recommendation became `pass with non-blocking follow-ups`.

The reviewer recorded local validation anomalies separately from repository
findings:

- the aggregate `b3abae08` Host run passed the first four labels with 8 tests,
  but the untrusted GUI child did not start because a validation-only Electron-
  as-Node wrapper propagated `ELECTRON_RUN_AS_NODE=1`; a separate clean-
  environment Restricted Mode rerun passed 1 test;
- the virtual Host printed the expected VS Code `ENOPRO` diagnostic because no
  filesystem provider existed for its synthetic URI; its assertion and Host
  still passed;
- the first focused `d3b56765` trusted-host attempt inside the GUI sandbox
  ended `SIGABRT`; the permitted rerun passed 5 + 5 tests;
- package false starts included codesign stderr, absent `npm`, an incomplete
  `vsce` PATH, and one invocation from the wrong working directory; corrected
  reruns passed;
- the first sandboxed audit could not open pnpm's SQLite database; the
  permitted read-only rerun passed.

After the reviewer reported the `d3b56765` Production-gate finding, the primary
created four unstaged remediation changes in the shared worktree. The reviewer
switched to committed `git show` evidence, excluded package runs that might
have compiled those unstaged files, did not modify them, and repeated the
content review only after commit `70bc5ff0`.

## Primary validation and reconciliation

The primary independently inspected the implementation and reproduced the
reviewer findings before each remediation. On the final tree it ran:

- `pnpm install --frozen-lockfile --force` — exit 0, lockfile unchanged;
- extension typecheck and compile — exit 0;
- unit tests — 27 passed;
- public process tests — 2 passed;
- the complete pinned Extension Host command — 14 passed across six isolated
  hosts: trusted 5 twice, empty 1, virtual 1, multi-root 1, Restricted Mode 1;
- package check — nine packaged files;
- package verification — eleven VSIX entries across two clean builds;
- extension audit — 37 tracked files, 18 license groups, three documents;
- `cargo fmt --all -- --check` and workspace check — exit 0;
- `cargo test --workspace --all-targets` with approved loopback access — exit
  0, 1,090 tests;
- focused Runtime MCP stdio tests — 7 passed;
- strict workspace Clippy and Rustdoc with `-D warnings` — exit 0;
- all 222 tracked Markdown files — 388 local links, zero missing;
- worktree/index diff checks and explicit staged-path audits — pass.

The first primary link command did not start because system `node` was absent
from PATH (exit 127); the identical read-only audit rerun with the bundled Node
24 executable passed. The first primary audit at `b3abae08` similarly reached
pnpm's sandbox-inaccessible SQLite database and failed before repository audit
assertions; the permitted rerun passed. An early Host evidence attempt returned
no test API because it assumed Development rather than the actual non-
Production test mode; no assertion reached status evidence. The corrected
committed profiles passed. These failures are retained as validation history,
not successful evidence.

The primary and final reviewer agree on every acceptance criterion, resolved
finding, remaining follow-up, missing-evidence disposition, scope boundary, and
risk. The effective decision is therefore `pass with non-blocking follow-ups`,
exactly as severe as the final independent recommendation.

## Scope and exclusion conformance

Included scope is complete: planning, pinned investigation, ADR-0052, locked
extension package, manifest, commands, configuration, pure MCP client, process
and editor lifecycle, status UI, cleanup, real Runtime integration, pinned Host
matrix, deterministic VSIX, dependency/license audit, CI definition,
documentation, and independent review evidence.

Excluded scope remains absent: navigation and search, LSP, diagnostics UI,
chat/context UI, EDT integration, remote/web hosts, multi-root fan-out,
concurrent MCP, workspace watching/reload, Runtime download or update,
Marketplace publication/signing, telemetry, authentication, external-client
compatibility, semantic behavior changes, and broad performance/security
claims.

## Residual risks

- Actual Windows execution remains unavailable locally despite complete static
  CI and platform-branch evidence.
- The successful trusted macOS Extension Host runs retain an unexplained VS
  Code `bootstrap-fork.js` SIGPIPE diagnostic on stderr.

## Artifact-consistency check

The same `/root/sprint30_remediation_review` reviewer completed the required
read-only artifact-consistency check before any Roadmap transition, Sprint 29
prompt retirement, staging, or final review commit. The first check correctly
requested exact historical validation details that the draft had compressed.
The primary added the initial Node, package, pnpm, loopback, URL, prompt-count,
Host, virtual-filesystem, and shared-worktree outcomes without changing the
decision or finding history.

The corrected draft passed at SHA-256
`d4caf68a30f7cb9601f21e1d761d5f43148a7cb98ac743e072d5eba92be5e353`
and 333 lines. The reviewer confirmed that it preserves every decision,
finding, evidence gap, remediation, exact result and anomaly, reviewer/primary
provenance boundary, scope, exclusion, follow-up, and residual risk. HEAD
remained `70bc5ff0f73f00d5e8a2af6c3389e626ce794a6b`; the index remained empty;
the draft was byte-identical; and the reviewer made no file, Git-state, or
delegation change. This section is the sole post-check artifact update and
records the completed check without altering the reviewed content.
