# Sprint 34 EDT Integration Prototype Review

## Decision

`pass with non-blocking follow-ups`

The effective decision matches the final independent reviewer recommendation.
Sprint 34 satisfies the accepted ADR-0056 boundary: one native EDT command is
available only for one eligible local EDT configuration project, starts one
explicitly configured public `oneagent-mcp` process, performs one bounded
compatibility probe off the UI thread, publishes one fixed redacted result, and
releases every owned process, stream, job, listener, and UI callback.

The review initially exposed incomplete cancellation and post-frame process
termination, unsafe disposable p2 assumptions, stale package evidence, and
cross-platform CI infrastructure defects. The dependency-ordered corrective
commits close those findings. The final exact-head GitHub Actions run completes
all Rust, VS Code, and EDT jobs on macOS and Windows, and the authorized
disposable full-EDT host evidence completes positive, repeated, invalid-
configuration, timeout, cancellation, stop, and clean lifecycle scenarios.

This decision does not claim Java semantic authority, proprietary EDT API use,
editor navigation, symbol search, Context, chat, diagnostics, edits, automatic
startup, a persistent connection, multiple Runtime processes, remote or
multi-project support, Marketplace publication, signing, telemetry, bundled
Runtime or JRE content, stored ITS credentials, or compatibility beyond the
explicitly validated first slice.

## Reviewed baseline

- Completed Sprint 33 prerequisite: `19ba2671`.
- Planning commit: `793ad400`.
- Task 6 evidence head: `4f88e6c2`.
- Final corrective head: `4e7cfa34`.
- Exact reviewed range:
  `19ba267188da76c0abed433a374aab2d25cc622a..4e7cfa34f7ad6f5f4856474304b11a03df9bfe66`.
- Range size: 20 commits, 90 paths, 7,523 additions, 65 deletions.

The dependency-ordered commits are:

| Step | Commit | Subject | Result |
| --- | --- | --- | --- |
| Planning | `793ad400` | `Plan Sprint 34 EDT Integration Prototype` | pass |
| Investigation | `85740005` | `Investigate Sprint 34 EDT integration prototype` | pass |
| ADR-0056 | `e9ce6bed` | `Define Sprint 34 EDT integration prototype` | pass |
| Runtime client | `19c6b2ae` | `Implement Sprint 34 EDT Runtime client` | remediated |
| Command lifecycle | `580cbddf` | `Implement Sprint 34 EDT command lifecycle` | remediated |
| Feature and p2 package | `5d30c332` | `Package Sprint 34 EDT integration prototype` | remediated |
| Completion evidence | `4f88e6c2` | `Complete Sprint 34 EDT integration evidence` | remediated |
| Runtime deadline | `3a2aaac0` | `Fix Sprint 34 Runtime cancellation deadline` | pass |
| Package evidence | `69d0b988` | `Correct Sprint 34 package evidence` | pass |
| Cancellation completion | `58c5727c` | `Fix Sprint 34 EDT cancellation completion` | pass |
| Safe p2 validation | `81e76d4e` | `Add safe Sprint 34 p2 validation` | pass |
| Post-frame termination | `0feef697` | `Fix Sprint 34 post-frame process termination` | pass |
| Complete CI matrix | `0026b29c` | `Preserve Sprint 34 CI matrix evidence` | pass |
| Repository-owned corpus gate | `4edc0f0d` | `Make EDT corpus tests explicitly opt-in` | pass |
| BSL fixture line endings | `69a02008` | `Normalize BSL query fixtures across platforms` | pass |
| Windows VS Code commands | `3872b18d` | `Fix Windows VS Code package validation` | pass |
| Rust 1.98 Clippy | `c1d0222a` | `Fix Rust 1.98 Clippy compatibility` | pass |
| EDT fixture mutations | `424f4cc7` | `Fix cross-platform EDT fixture mutations` | pass |
| Package inventory order | `2cdc83f0` | `Normalize VS Code package inventories` | pass |
| Windows enum layout | `4e7cfa34` | `Reduce EDT writes resolution outcome size` | pass |

The final reviewer began and ended at
`4e7cfa34f7ad6f5f4856474304b11a03df9bfe66`. It observed the same five
pre-existing user-owned unstaged paths before and after review:

```text
 M .codex/config.toml
 M AGENTS.md
 M docs/Roadmap.md
 M docs/architecture/mcp-semantic-tools-investigation.md
 M docs/reviews/sprint-29-mcp-semantic-tools.md
```

The reviewer inspected committed snapshots and repository-local evidence,
remained read-only, delegated no work, and made no edit, creation, deletion,
staging, commit, push, Roadmap transition, prompt retirement, application-
bundle change, or p2-pool change.

## Independent reviewer handoff and report

Final reviewer Carver, `/root/sprint34_endpoint_final_review`, retained the
mandatory independent clean-context reviewer role. The reviewer received the
repository root, exact endpoint, authoritative scope, exact CI run, current
corrective commits, read-only local host evidence, required output contract,
and strict no-edit/no-delegation boundary. It was not given an expected
decision.

The reviewer recommended `pass with non-blocking follow-ups` and independently
confirmed:

- the exact linear 20-commit range and unchanged user-owned worktree paths;
- bounded protocol parsing, post-frame stderr handling, cancellation,
  deadline, termination, and cleanup behavior;
- exact project, executable, command, preference, job, UI-thread, replacement,
  and bundle-stop ownership;
- 41/41 Tycho/PDE/real-Runtime tests and the exact JavaSE-17 p2 package;
- deterministic feature, bundle, and normalized repository metadata evidence;
- disposable p2 lifecycle and full EDT 2026.1 workbench evidence;
- successful macOS and Windows Rust, VS Code, and EDT CI jobs at the exact
  reviewed head;
- preserved Rust/MCP semantic authority, seven-tool catalog, and existing IDE
  client behavior; and
- no tracked credential, private-p2 metadata, personal path, generated package,
  unauthorized dependency, proprietary Java import, bundled Runtime/JRE/
  JavaFX/native artifact, or excluded production surface.

It found no blocking defect and no missing mandatory evidence. Its one
non-blocking finding is that the root README still describes Sprint 33 as
active pending review. The authorized Task 7 state transition accepts that
finding and synchronizes the README with the completed Sprint 34 and unique
Sprint 35 `next` state in the final review commit.

## Review remediation history

| Gate | Finding | Resolution |
| --- | --- | --- |
| Runtime lifecycle | Cancellation and termination did not share one bounded deadline across every post-frame path. | `3a2aaac0`, `58c5727c`, and `0feef697` enforce bounded cancellation, late post-frame termination, forced cleanup, and regression coverage. |
| Package evidence | Endpoint bundle bytes and tracked evidence disagreed after Runtime remediation. | `69d0b988` records the rebuilt exact package evidence; final hashes and package audit agree. |
| Disposable p2 | A prior launcher could retain an external install area and mutate the application configuration. | `81e76d4e` makes validation fail closed unless install, configuration, p2 data, profile, bundle pool, and workspace are disposable and repository-local. The repository owner reported the full EDT reinstallation, and the primary verified its full-product identity before final host evidence was accepted. |
| CI observability | Matrix fail-fast concealed later platform failures. | `0026b29c` preserves all six job results. |
| Clean checkout | Default Rust tests depended on an external sibling EDT corpus. | `4edc0f0d` makes live corpus tests explicitly feature- and environment-gated; default CI uses repository-owned fixtures. |
| Windows fixtures | Git checkout line endings changed exact byte offsets and multiline mutation fragments. | `69a02008` fixes query fixture LF and `424f4cc7` preserves each copied fixture's CRLF/LF convention. |
| Windows Node launch | Direct `.cmd` spawning returned no process status. | `3872b18d` uses `ComSpec` for Windows command shims with bounded failure handling. |
| Package inventory | `vsce` returned the same exact files in OS-specific order. | `2cdc83f0` canonicalizes order while preserving exact cardinality, names, duplicate detection, and repeated-build equality. |
| Rust 1.98 | New platform-sensitive Clippy lints rejected prior compatible source. | `c1d0222a` removes constant `chunks_exact` patterns without raising MSRV; `4e7cfa34` boxes one private large outcome field without semantic change. |

## Acceptance evidence matrix

| Criterion | Independent and primary evidence | Result |
| --- | --- | --- |
| Planning, investigation, and architecture | The exact commits preserve dependency order. The pinned investigation, ADR-0056, Roadmap, Architecture, semantic model, prompts, and implementation agree on the public-Eclipse first slice and exclusions. | pass |
| Runtime protocol and process | Dependency-free Java sends the exact discovery request, validates the closed compatible response, bounds UTF-8 JSON depth/frame/stderr/time, and classifies malformed, duplicate, incompatible, timeout, cancellation, exit, and cleanup paths without disclosure. | pass |
| Project and configuration gate | Exactly one open, accessible, local, non-linked, non-virtual project with the EDT configuration nature and readable file location is eligible. Executable validation remains bounded and explicit. | pass |
| Command, UI, and lifecycle | One owned background job serializes probes, publishes one fixed redacted information/error result on the UI thread, suppresses stale callbacks, and joins cancellation, configuration replacement, and bundle stop. | pass |
| Package and dependencies | The repository contains exactly the accepted bundle, feature, and category. The production bundle is JavaSE-17/class major 61 with only accepted public Eclipse/OSGi imports and no production dependency. | pass |
| Package determinism | Two clean repositories have equal seven-file inventories and normalized metadata. Feature SHA-256 is `9078608b97dc1a8c04ca0bacdc77a23948668a621e051c7924281923bde1015a`; bundle SHA-256 is `c85e3e45bb476354743bbe1a3fb98317909b83bdb898f1dd4f8b2f95d3645ddd`. | pass |
| Disposable p2 lifecycle | Safe validation installs, lists, removes, and confirms absence without permitting a non-local install/configuration/profile/bundle-pool boundary. | pass |
| Full EDT host | The full EDT 2026.1.2.2 x86_64 workbench on JDK 17.0.20.1 and OpenJFX records compatible results twice, invalid configuration, timeout, cancellation, joined stop, clean project shutdown, and final `PASS`. | pass |
| JavaFX and host cleanup | The final log contains no JavaFX missing-class incident, OneAgent failure, retained OneAgent job, or failed evidence. Remaining NLS, navigator, and Cocoa shutdown warnings are platform-owned. | pass |
| Rust compatibility | Primary full fmt/check/test/strict-Clippy/strict-Rustdoc passes. Exact-head CI repeats those gates on macOS and Windows, including focused Context/MCP compatibility. | pass |
| VS Code compatibility | Primary headless validation passes typecheck, compile, 62 unit tests, 2 public process tests, 12-file inventory, two 14-file VSIX inventories, and the 43-file/18-license-group/3-document audit. Exact-head CI additionally passes every pinned Extension Host profile on both operating systems. | pass |
| EDT CI | Exact-head macOS and Windows jobs build the public Runtime, execute Tycho/PDE/real-process verification on JDK 25, and pass the Java p2 auditor. | pass |
| Protocol and semantic authority | No new Rust/MCP capability exists. Runtime and Graph remain semantic authorities; Java performs only strict process compatibility and native Eclipse workflow ownership. | pass |
| Scope, security, and external access | No credential, personal path, generated package, proprietary EDT import, private-p2 requirement, external publication/signing, or excluded feature is tracked. Authorized application and p2 paths remain unmodified after the corrected validation boundary. | pass |
| Current-state documentation | Roadmap and architecture are current. The root README inconsistency is accepted for correction in the final Task 7 state-transition commit. | pass with correction |

## Findings

### Blocking

None remain at `4e7cfa34`.

### Non-blocking follow-ups

1. Migrate deprecated GitHub actions before their upstream removal. The primary
   observed eight deprecation warnings in the exact-head Actions UI for Node
   20-based `actions/checkout@v4`, `actions/setup-node@v4`,
   `actions/setup-java@v4`, and the setup-java v4 lifecycle, while the reviewer
   independently confirmed through the Actions API that every current job and
   required step succeeds.
2. Keep the full EDT validation bound to its documented x86_64 JDK 17 and
   OpenJFX environment until the vendor host publishes a different supported
   architecture/runtime contract.

The stale root README state is not deferred: it is corrected atomically with
the Task 7 state transition.

## Missing evidence and unexecuted checks

No mandatory Sprint 34 product evidence is missing after final primary and
reviewer reconciliation.

The independent reviewer regenerated the 41-test Tycho/PDE/real-Runtime
matrix, late-stderr and late-cancellation regressions, package audit,
deterministic package comparison, disposable p2 lifecycle, and corrective EDT
host validation at `0feef697`. No later commit changes the EDT extension,
package, ADR-0056, or its architecture contract, so that independent executed
evidence remains applicable at the final head. The final endpoint
reconciliation additionally inspected the later corrective range, exact CI
result, current Git state, and retained repository-local host evidence without
creating or modifying artifacts. Primary-only observations and independent
reviewer execution are kept distinct below.

The primary did not rerun a local Electron Extension Host after the final Rust-
only commit. Exact-head macOS and Windows CI executed every extension profile,
so no extension-host acceptance gap remains. Zero-test binary and doc-test
targets in the full Rust workspace are not used as functional evidence.

## Independent CI evidence

[GitHub Actions run 33147520423](https://github.com/MakSGTS/OneAgent/actions/runs/33147520423)
targets exact commit
`4e7cfa34f7ad6f5f4856474304b11a03df9bfe66`, branch
`codex/sprint34-final-review`, run number 7, attempt 1. GitHub reports
`status=completed`, `conclusion=success`, and six successful jobs:

| Job | Result | Covered gates |
| --- | --- | --- |
| `rust (macos-14)` | success | format, check, Runtime binaries, workspace tests, focused Context/MCP tests, Clippy, Rustdoc |
| `rust (windows-latest)` | success | format, check, Runtime binaries, workspace tests, focused Context/MCP tests, Clippy, Rustdoc |
| `vscode (macos-14)` | success | frozen install, typecheck, extension tests, Runtime process, package, two VSIX builds, audit |
| `vscode (windows-latest)` | success | frozen install, typecheck, extension tests, Runtime process, package, two VSIX builds, audit |
| `edt (macos-14)` | success | JDK 25, Runtime build, Tycho/PDE/real process, p2 audit |
| `edt (windows-latest)` | success | JDK 25, Runtime build, Tycho/PDE/real process, p2 audit |

## Primary validation and reconciliation

The primary independently inspected the exact range and ran:

- `cargo fmt --all -- --check` — pass;
- `cargo check --workspace` — pass;
- `cargo test --workspace` — pass;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — pass;
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps` — pass;
- focused Writes resolution tests — 10/10 pass;
- end-to-end Writes tests — 6/6 pass;
- VS Code typecheck and compile — pass;
- complete VS Code unit tests — 62/62 pass;
- public Runtime process tests — 2/2 pass;
- package inventory — 12 files, pass;
- two clean VSIX inventories — 14 files each, pass;
- extension audit — 43 tracked files, 18 license groups, and 3 documents,
  pass; and
- range/worktree `git diff --check` — pass.

No commit after `0feef697` changes `extensions/edt`, ADR-0056, Architecture, or
its package definition. Task 6 and the primary provide two clean
Tycho/PDE/real-Runtime runs with 41/41 tests each. At `0feef697`, the independent
reviewer separately ran one clean 41/41 Tycho/PDE/real-Runtime matrix, the Java
package auditor, late-stderr and late-cancellation regressions, disposable p2
lifecycle, and corrective EDT host validation. The reviewer also independently
compared the two preserved current-build repositories, confirming equal
feature and bundle bytes and metadata equality after only the accepted
normalization.

The primary and reviewer agree on implementation correctness, package
contents, process/lifecycle safety, cross-platform compatibility, external
read-only compliance, scope, and effective decision. No unresolved evidence
disagreement remains.

## External boundary and scope conformance

The accepted validation reads installed application metadata and the local p2
pool only under explicit authorization. Corrected disposable configurations,
profiles, bundle pools, Maven state, workspaces, logs, and evidence remain under
ignored repository-owned `local-artifacts/`. No application bundle or p2-pool
file is modified, removed, re-signed, or used as an install destination.

The reviewed range adds no Java parser or semantic inference, proprietary EDT
implementation API, Rust/MCP capability, semantic editor UI, persistent
connection, automatic startup, multiple-process orchestration, remote or
multi-project aggregation, publication, signing, telemetry, bundled Runtime/
JRE, or stored credential.

## Residual risks

- The vendor EDT host remains an x86_64 application with a documented JDK 17
  and OpenJFX runtime boundary, while Maven/Tycho and CI build on JDK 25.
- The primary-observed GitHub Actions deprecation warnings require a future
  bounded infrastructure update before the referenced action majors are
  removed upstream.
- Proprietary EDT startup and shutdown produce platform warnings that are not
  OneAgent failures; the acceptance oracle remains the exact OneAgent evidence
  file plus clean process exit.
- Broader external MCP client compatibility remains intentionally deferred to
  Sprint 35.

## Next action

After the same reviewer passes artifact consistency, mark Sprint 34
`completed`, make Sprint 35 External AI Client Compatibility the unique `next`
target, synchronize the root README current-state sentence, retire exactly the
nine tracked Sprint 33 prompt files, and commit those changes atomically with
this review artifact.

## Artifact consistency

The same independent clean-context reviewer performed a final read-only
consistency check at
`4e7cfa34f7ad6f5f4856474304b11a03df9bfe66` and returned `pass`. The artifact
accurately separates Task 6 and primary two-run evidence from the reviewer's
independent one-run 41/41 Tycho validation and preserved two-repository
comparison. It correctly attributes the GitHub Actions deprecation warnings to
the primary and the EDT reinstallation to the repository owner with primary
verification. It preserves the exact range, HEAD and worktree status, decision,
findings, closed missing-evidence items, validation results, scope and
exclusions, residual risks, and clean-context/read-only/no-delegation record
without weakening the final reviewer report. No unresolved evidence
disagreement remains.
