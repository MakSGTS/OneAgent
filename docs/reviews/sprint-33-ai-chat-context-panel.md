# Sprint 33 AI Chat and Context Panel Review

## Decision

`pass with non-blocking follow-ups`

The effective decision matches the final fresh-context independent reviewer
recommendation. Sprint 33 satisfies the accepted ADR-0055 boundary: one
explicitly selected, immutable Runtime Context is rendered in a static
inspectable panel and supplied with the current bounded prompt as exactly two
user messages to the request-selected VS Code model. The extension retains no
model history, provider credential, hidden Context, source-reading authority,
tool surface, edit surface, or additional semantic authority.

The initial review gates were correctly blocked. A clean GitHub macOS runner
first supplied the missing independent 18/18 pinned Extension Host evidence.
The repeated reviewer then found that an already-cancelled VS Code request
could reach model token counting before the production cancellation adapter
observed the deferred event. Commit `4368a7c5` closes that defect with a
race-safe shared production helper and exact zero-model-call regression
evidence. The final reviewer found no remaining production blocker.

This decision does not claim automatic Context collection, editor-to-node
inference, source reads, model tools or edits, Runtime model-provider wiring,
provider discovery or secrets in the extension, conversation persistence,
custom model history, webview scripts, remote/web/multi-root or EDT support,
diagnostics UI, Marketplace publication, telemetry, live external-model
determinism, or broad performance or security properties.

## Reviewed baseline

- Completed Sprint 32 prerequisite: `8e33c95c`.
- Planning commit: `39c229aa`.
- Task 7 evidence head: `5d738c0e`.
- Cancellation remediation head: `4368a7c5`.
- Exact reviewed range:
  `8e33c95c15ef1c92b1749243b3b5376ba61901bf..4368a7c5db3a5d71707baa2aa73e5eaa40b0eb2f`.
- Range size: 9 commits, 35 paths, 4,667 additions, 47 deletions.

The dependency-ordered commits are:

| Step | Commit | Subject | Result |
| --- | --- | --- | --- |
| Planning | `39c229aa` | `Plan Sprint 33 AI Chat and Context Panel` | pass |
| Investigation | `1f69a6e2` | `Investigate Sprint 33 AI chat and context panel` | pass |
| ADR-0055 | `f0e5f3d2` | `Define Sprint 33 AI chat and context panel` | pass |
| Context Runtime client | `24730f6e` | `Implement Sprint 33 context Runtime client` | pass |
| Context panel | `9e0f8116` | `Implement Sprint 33 context panel` | pass |
| AI chat participant | `79fb2119` | `Implement Sprint 33 AI chat participant` | remediated |
| Extension integration | `73709897` | `Integrate Sprint 33 chat and context extension` | remediated |
| Completion evidence | `5d738c0e` | `Complete Sprint 33 chat and context evidence` | remediated |
| Cancellation remediation | `4368a7c5` | `Fix Sprint 33 cancellation propagation` | pass |

The range is bounded to Sprint 33 planning and prompts, pinned VS Code API
investigation, ADR-0055, the strict TypeScript Context client, static Context
panel, chat controller and participant, extension lifecycle integration,
tests, package and CI evidence, and synchronized current-state documentation.
It changes no Rust source, Cargo manifest, Cargo lockfile, Node lockfile, MCP
catalog, or Coverage Registry entry. The extension manifest changes only the
accepted command, panel, participant, metadata, and package inventory surface;
it adds no production dependency.

The final reviewer began and ended at
`4368a7c5db3a5d71707baa2aa73e5eaa40b0eb2f`. It observed the same four
pre-existing user-owned unstaged paths before and after review:

```text
 M .codex/config.toml
 M docs/Roadmap.md
 M docs/architecture/mcp-semantic-tools-investigation.md
 M docs/reviews/sprint-29-mcp-semantic-tools.md
```

The reviewer inspected committed snapshots, used a fresh context, remained
read-only, delegated no work, and made no edit, creation, deletion, staging,
commit, push, Roadmap transition, or prompt retirement.

## Independent reviewer handoff and report

Final reviewer `/root/sprint33_post_remediation_gate` received no inherited
conversation turns. It received the repository root, exact range,
authoritative documents, objective, criteria, exclusions, CI evidence,
required output contract, and strict read-only/no-delegation boundary. It did
not receive an expected decision.

The reviewer recommended `pass with non-blocking follow-ups` and independently
confirmed:

- the exact linear planning-through-remediation range and unchanged user-owned
  working-tree paths;
- strict Context result decoding, one shared semantic FIFO, immutable selected
  Context generations, and stale-result suppression;
- complete escaping and a script-, form-, command-, and resource-free Context
  panel under a strict content security policy;
- exact prompt, assembled-input, token, output, concurrency, cancellation,
  stream, error, and disposal boundaries;
- production use of the shared cancellation helper and a regression proving
  zero `countTokens` and zero `sendRequest` calls for an already-cancelled
  parent;
- trusted local single-workspace activation, Runtime reconnect invalidation,
  registration ownership, and repeatable cleanup;
- macOS and Windows 62/62 unit, 18/18 Host, and 2/2 public-process results at
  the exact remediation head;
- complete macOS package, two-clean-VSIX, and audit evidence; and
- no Rust, Cargo, lockfile, dependency, MCP catalog, Coverage Registry,
  secret, hidden-context, source-access, tool, edit, or excluded production
  surface drift.

It found no actionable production finding. It classified the Windows `vsce`
launcher failure and clean-checkout Rust fixture failures as pre-existing CI
infrastructure defects rather than Sprint 33 regressions.

## Review remediation history

| Reviewed head | Finding | Resolution |
| --- | --- | --- |
| `5d738c0e` | The local macOS Host installation exited with `SIGABRT`, so the review lacked an independent complete Host result. | GitHub Actions run `33082339858` supplied 18/18 Host checks with all six runner exits zero on both macOS and Windows. |
| `5d738c0e` | The production adapter subscribed to parent cancellation without synchronously inheriting an already-cancelled state, permitting model use before VS Code delivered the deferred callback. | `4368a7c5` subscribes first, synchronously checks the parent, cancels the child, and tests the exact helper with zero token-count and request calls. |
| `4368a7c5` | Committed Roadmap Task 7 evidence still reports the pre-remediation count of 61 unit tests. | The Task 8 state transition records the current 62/62 unit result and cancellation remediation. |

## Acceptance evidence matrix

| Criterion | Independent and primary evidence | Result |
| --- | --- | --- |
| Planning, investigation, and architecture | Nine commits preserve strict dependency order. The pinned investigation, ADR-0055, Roadmap, architecture, and semantic model agree on ownership, first slice, compatibility, and exclusions. | pass |
| Context transport | The TypeScript client sends exact fixed `both`, depth 2, 32-candidate, and 16,384-byte inputs over the unchanged `oneagent.context` tool, strictly decodes the complete bounded result, and serializes semantic operations over one FIFO. | pass |
| Context selection and panel | One immutable generation owns the visible model-eligible Context. Replacement, close, disconnect, reconnect, and deactivation invalidate it; late results cannot revive it. Complete escaping and strict static CSP prevent script, form, command, or resource behavior. | pass |
| Chat input and model admission | The participant rejects commands, references, and tools; requires a current selected Context and a 1–8,192-byte prompt; caps assembled input at 32,768 bytes; validates safe token counts against the request-selected model; and sends exactly two user messages. | pass |
| Streaming and output safety | Only string fragments are accepted, every fragment is escaped as untrusted non-HTML Markdown, raw output is capped atomically at 65,536 UTF-8 bytes, and model/provider errors remain bounded and redacted. | pass |
| Cancellation and concurrency | One active request is allowed. The linked source subscribes before synchronously inheriting parent state, covers every cancellation timing window, is disposed in `finally`, and prevents all model use for pre-cancelled requests. | pass |
| Runtime and lifecycle compatibility | Existing trusted local single-workspace connection, status, symbol navigation, unsupported-workspace rejection, Runtime reconnect, process ownership, and deactivation behavior remain covered. | pass |
| Protocol and semantic authority | The seven-tool MCP catalog and Rust protocol are unchanged. Runtime and Graph remain semantic authorities; the extension consumes only canonical Context and exposes no new source-reading or provider surface. | pass |
| Extension execution | At remediation HEAD, macOS and Windows each pass 62/62 unit tests, 18/18 pinned VS Code 1.134.0 Host checks, and 2/2 public Runtime process tests. All six Host runner exits are zero on each OS. | pass |
| Package and dependency evidence | macOS validates 12 packaged files, two clean 14-file VSIX archives, and an audit of 43 tracked extension files, 18 license groups, and 3 linked documents. No production dependency or Node lockfile changes. | pass with Windows launcher follow-up |
| Rust compatibility | The range changes no Rust or Cargo surface. Primary local full workspace fmt/check/test/Clippy/Rustdoc and focused Context 11/11, semantic MCP 6/6, and public MCP process 9/9 evidence pass. | pass with clean-checkout fixture follow-up |
| Documentation and scope | Current documents describe the bounded Context/chat behavior and deferrals. No secret, personal path, generated tracked artifact, hidden Context, source read, tool, edit, provider wiring, or deferred host surface was added. | pass |

## Findings

### Blocking

None remain at `4368a7c5`.

### Non-blocking follow-ups

1. Replace the direct Windows `spawnSync("vsce.cmd")` package-audit launcher
   with a proven cross-platform invocation and include bounded spawn-error
   diagnostics. The launcher predates Sprint 33; Windows reaches and passes
   typecheck, 62 unit tests, 18 Host tests, and 2 process tests before this
   failure.
2. Make Rust CI self-contained by adding or generating the required EDT source
   fixtures, or by explicitly separating live-fixture tests. Clean checkout
   currently lacks untracked `OneAgent_EDTproject` content used by 11 tests.
3. Keep the CI focused Context/MCP compatibility step independently runnable
   even when the full workspace test encounters a fixture-infrastructure
   failure.

The stale Roadmap unit count is reconciled inside the Task 8 state transition
and is not deferred.

## Missing evidence and unexecuted checks

No mandatory Sprint 33 product evidence is missing after primary
reconciliation.

The final reviewer preserved strict read-only operation and did not regenerate
build, package, VSIX, or Host artifacts. It inspected the committed range,
production and test paths, authoritative documents, Git state, and the exact
GitHub Actions logs. It explicitly relied on the independent CI run and
separately identified primary-only local validation.

The primary local pinned Host command did not execute assertions because the
installed macOS Electron process exited with `SIGABRT`. This is superseded for
the completion gate by clean GitHub macOS and Windows runners, both of which
executed all 18 Host checks with six zero exits. Windows package/VSIX/audit
completion remains unexecuted after the pre-existing `.cmd` spawn failure.
The focused Rust CI step remains unexecuted because it follows the failing full
workspace step. These limitations are recorded as residual infrastructure
risks, not hidden product passes.

No accepted focused filter matched zero tests. Zero-test binary and doc-test
targets in the full Rust workspace are not used as functional evidence.

## Independent CI evidence

[GitHub Actions run 33086744079](https://github.com/MakSGTS/OneAgent/actions/runs/33086744079)
targets exact remediation commit
`4368a7c5db3a5d71707baa2aa73e5eaa40b0eb2f`.

| OS / job | Unit | Extension Host | Process | Package / VSIX / audit | Result classification |
| --- | ---: | ---: | ---: | --- | --- |
| macOS 14 / `98568337451` | 62/62 | 7 + 7 + 1 + 1 + 1 + 1 = 18/18; six exits 0 | 2/2 | 12 files; two clean 14-file archives; 43/18/3 audit | complete Sprint 33 extension pass |
| Windows / `98568337870` | 62/62 | 7 + 7 + 1 + 1 + 1 + 1 = 18/18; six exits 0 | 2/2 | stopped at pre-existing `vsce.cmd` spawn with `status: null` | product gates pass; infrastructure follow-up |

The overall workflow is red for two unrelated infrastructure classes. The
macOS Rust job passes formatting, checking, and public Runtime build, then
reports 247 passing and 11 failing tests, all requiring absent untracked EDT
fixture paths in a clean checkout. Windows Rust reaches formatting and checking
but its Runtime-process build is cancelled by matrix fail-fast. Neither job
contains a Sprint 33 Rust or Cargo diff.

## Primary validation and reconciliation

The primary independently inspected the exact range and ran:

- `pnpm --dir extensions/vscode run typecheck` — pass;
- `pnpm --dir extensions/vscode run compile` — pass;
- focused AI chat controller tests — 11/11 pass;
- complete extension unit tests — 62/62 pass;
- public Runtime process tests with the built `oneagent-mcp` — 2/2 pass;
- package inventory — 12 files pass;
- two clean VSIX builds — 14 files each, pass;
- extension audit — 43 tracked files, 18 license groups, and 3 documents,
  pass;
- `cargo fmt --all --check` — pass;
- `cargo check --workspace --all-targets` — pass;
- `cargo test --workspace --all-targets` — pass;
- `cargo clippy --workspace --all-targets -- -D warnings` — pass;
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` — pass; and
- range and worktree `git diff --check` — pass.

Task 7 also records focused Rust compatibility passes for 11 Context Engine,
6 semantic MCP, and 9 public MCP process tests. The remediation changes only
three TypeScript files, and the complete local Rust matrix was repeated after
that change.

The primary and final reviewer agree on implementation correctness, scope,
compatibility, cancellation safety, evidence classification, and the effective
`pass with non-blocking follow-ups` decision. The primary supplies the complete
local build/package/Rust matrix unavailable to the read-only reviewer; the
reviewer supplies independent code inspection and clean-host CI assessment.
No unresolved evidence disagreement remains.

## Scope and exclusion conformance

The reviewed range adds no Rust semantic or protocol capability, MCP tool,
Runtime provider composition, provider/model discovery, extension credential,
arbitrary source read, inferred editor-to-node resolution, automatic Context
collection, model tool or edit, webview script, retained executable state,
conversation persistence, custom history, remote/web/multi-root or EDT host,
diagnostics UI, Marketplace publication, telemetry, or broad quality claim.

Runtime and Graph remain semantic authorities. The extension owns only strict
transport decoding, explicit selection, presentation, model-message assembly,
bounded text streaming, lifecycle invalidation, and cleanup.

## Residual risks

- Windows package inventory, clean VSIX comparison, and dependency audit await
  a portable `vsce` subprocess launcher.
- Clean-checkout Rust CI remains dependent on fixture content that is not
  tracked or generated by the workflow.
- The focused Rust compatibility step is currently ordered after the full
  workspace test and therefore does not execute when that broader fixture gate
  fails.
- Live external model output remains intentionally nondeterministic and was
  not used as acceptance evidence.

## Artifact consistency

The same final reviewer `/root/sprint33_post_remediation_gate` performed a
read-only consistency check of this artifact against its independent report
for
`8e33c95c15ef1c92b1749243b3b5376ba61901bf..4368a7c5db3a5d71707baa2aa73e5eaa40b0eb2f`
and returned `pass`. The reviewer confirmed that the artifact preserves without
weakening every finding, missing-evidence item, decision, validation result,
scope/exclusion conclusion, residual risk, observed HEAD/status, and
fresh-context/read-only/no-delegation evidence. Primary-only evidence remains
explicitly separated from independent evidence. The consistency check made no
repository change, state transition, prompt retirement, staging, commit, or
delegation.
