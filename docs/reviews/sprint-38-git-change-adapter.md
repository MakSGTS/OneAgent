# Sprint 38 Git Change Adapter Review

## Decision

`pass`

The effective decision matches the independent reviewer recommendation. Sprint
38 satisfies ADR-0060: Runtime owns one explicit bounded local Git reader, a
normalized repository-change domain, and a source-neutral Workspace rebuild
input; Git remains repository evidence rather than semantic authority; and
accepted inputs reuse the complete filesystem discovery, semantic build,
validation, cache, and immutable publication lifecycle.

This decision does not claim automatic Git discovery, remote or mutating Git
operations, separate staged and unstaged semantics, ignored or sparse content,
submodule content inspection, selective semantic mutation, product impact
analysis, refactoring, edits, a Git protocol or UI, telemetry, benchmarks, or
broad performance or security results.

## Reviewed baseline

- Sprint 38 planning commit:
  `e60b95c05d1977996d5468d87c1397ea8c9e17ae`.
- Pre-planning base: `580496eb415965feb4710e0c536e519a46180645`.
- Final production-code head:
  `129b69c81987112d07741f3bd0abf06114430816`.
- Task 6 and remediation head:
  `9e17cf3cc799c532517fda5045a608b7e57da752`.
- Reviewed commit set: `e60b95c0^..9e17cf3c`.
- Exact reviewed diff:
  `580496eb415965feb4710e0c536e519a46180645..9e17cf3cc799c532517fda5045a608b7e57da752`.
- Range size: 18 commits, 22 paths, 6,768 additions, 98 deletions.

The dependency-ordered commits are:

| Step | Commit | Subject | Result |
| --- | --- | --- | --- |
| Planning | `e60b95c0` | `Plan Sprint 38 Git Change Adapter` | pass |
| Investigation | `d82ba4cb` | `Investigate Sprint 38 Git Change Adapter` | pass |
| ADR-0060 | `3cff2f95` | `Define Sprint 38 Git Change Adapter` | pass |
| Domain | `e0aadfab` | `Implement Sprint 38 change-set domain` | pass |
| Reader | `3ed8990f` | `Implement Sprint 38 Git repository reader` | pass |
| Workspace | `175de804` | `Integrate Sprint 38 Workspace change inputs` | pass |
| Evidence | `550fa5df` | `Complete Sprint 38 Git Change Adapter evidence` | pass |
| Cleanup remediation | `3e13e523` | `Remediate Sprint 38 Git reader cleanup evidence` | pass |
| Evidence correction | `a2cb0641` | `Correct Sprint 38 review evidence` | pass |
| Order equivalence | `dffd2f1c` | `Prove Sprint 38 Workspace order equivalence` | pass |
| Failure evidence | `03281583` | `Complete Sprint 38 injected failure evidence` | pass |
| Timeout stabilization | `b1d551a1` | `Stabilize Sprint 38 injected timeout evidence` | pass |
| Evidence update | `8fbb12f1` | `Update Sprint 38 remediation evidence` | pass |
| Review remediation merge | `c852cf3b` | `Merge Sprint 38 review remediation` | pass |
| Cancellation and lazy fetch | `43165bfe` | `Close Sprint 38 cancellation and lazy-fetch gaps` | pass |
| Production merge | `129b69c8` | `Merge Sprint 38 remediation` | pass |
| Documentation remediation | `20679fd8` | `Remediate Sprint 38 evidence documentation` | pass |
| Documentation merge | `9e17cf3c` | `Merge Sprint 38 evidence remediation` | pass |

The final review head has parents `129b69c8` and `20679fd8`, and the production
head is its ancestor. Only `docs/Roadmap.md` and
`docs/architecture/git-change-adapter-evidence.md` differ between the final
production-code head and the immutable review head.

Independent reviewer `/root/sprint38_reviewer` received a guaranteed fresh
context containing only the repository root, immutable range, authorities,
criteria and exclusions, validation matrix, and output contract. The reviewer
began and ended on `codex/v0.7-sprint-38-review` at `9e17cf3c` with no staged,
unstaged, or untracked non-ignored path, remained read-only, used no network,
delegated no work, and made no file, Git-state, branch, configuration,
download, or remote mutation. Cargo created or refreshed only ordinary ignored
build outputs under `target/`.

## Findings and primary reconciliation

### Blocking findings

None.

### Non-blocking findings

None.

### Missing evidence

None.

The primary independently reproduced the baseline, code and documentation
claims, focused and full validation results, API/dependency/scope audits, and
repository cleanliness. There is no unresolved disagreement, and the effective
decision is not less severe than the reviewer recommendation.

## Acceptance evidence matrix

| Criterion | Independent and primary evidence | Result |
| --- | --- | --- |
| Authority and dependencies | Runtime exclusively owns the domain, reader, and input boundary; Graph and source adapters do not depend on Git; manifest and lockfile diffs contain zero paths | pass |
| Explicit/default split | No production call site invokes the reader outside its module and exports; default Runtime, CLI, HTTP, MCP, LSP, VS Code, and EDT paths do not start Git | pass |
| Repository and root | Existing directory canonicalization, exact top-level equality, normal/detached/linked/SHA-256 repositories, and closed bare/unborn/missing/non-repository outcomes are covered | pass |
| Endpoints and layers | Baseline is a pinned 40- or 64-character lowercase-hex `HEAD`; current state is final tracked worktree plus non-ignored untracked content; staged and unstaged state is intentionally folded | pass |
| Conflicts and gitlinks | Any unmerged index entry closes the read; changed gitlinks and unsupported modes return typed failure | pass |
| Stability | Exactly two complete passes compare root, baseline, modes, ordered changes, and completeness; drift returns `UnstableRepository` without retry | pass |
| Paths | Relative confined UTF-8 `/` paths preserve case and Unicode, enforce the 4,096-byte bound, and reject root, prefix, UNC, backslash, dot, traversal, NUL, and empty components | pass |
| Status, identity, and order | Closed Added/Modified/Deleted/TypeChanged/Untracked states, effective-path/status/old/new ordering, exact duplicate collapse, and atomic same-path conflict failure are covered | pass |
| Rename and copy | `--no-renames` preserves deterministic delete/add move evidence and ordinary copy additions without similarity identity | pass |
| Bounds | 10,000 changes, 4,096-byte paths, 16 MiB stdout, 64 KiB stderr, one child, two passes, and one 30-second deadline with cleanup reserve are enforced | pass |
| Process and environment | Fixed `git` argv only, null stdin, bounded concurrent output, no shell or remote/mutating command, credential/lazy-fetch/pager/color/quoting/rename/fsmonitor/untracked-cache controls, and repository-variable removal are covered | pass |
| Cancellation and cleanup | The read future owns the active child; cancellation, timeout, and drop kill and reap it; no production detached reader task exists | pass |
| Errors and redaction | Closed reader/domain failures do not format root, path, Git output, configuration, environment, error chain, or source values | pass |
| Workspace input | A preregistration cloneable capacity-one handle returns exact empty/accepted/backpressure/closed outcomes and preserves empty precedence | pass |
| Workspace scheduling | Each accepted input requests a complete rebuild, at most one follow-up remains pending, and source identity creates no semantic priority | pass |
| Rebuild and equivalence | Complete scan, discovery, EDT/Designer build, validation, stable rescan, cache, and atomic publication are reused; opposite operation orders yield equal complete snapshots and graphs | pass |
| Failure and recovery | Invalid semantic builds preserve the last valid snapshot and a later explicit input recovers publication | pass |
| Lifecycle | The change receiver closes before active owned work is joined; cleanup then clears the snapshot and closes observations in `Stopped` state | pass |
| Cache | Schema `1` and semantic compatibility `4` remain unchanged; Git baseline, paths, statuses, process state, and queue state are not serialized | pass |
| Graph, Analysis, and Coverage | Complete Graph and Analysis suites pass; Coverage has zero changed paths; repository paths and statuses are not graph operations or impact seeds | pass |
| Consumers and protocols | HTTP/CLI behavior, seven MCP tools and three revisions, LSP 3.17 capabilities, stdio framing, VS Code, and EDT surfaces remain compatible | pass |
| Platform and current evidence | Exact production head CI run `33399662895` passed all six macOS/Windows Rust, VS Code, and EDT jobs; the current 80-target/1,270-test inventory is separated from historical evidence | pass |
| Scope and exclusions | No remote/credential/repository mutation, automatic polling, semantic impact, rules/diagnostic inference, selective update, refactoring/edit flow, Git UI/protocol, telemetry, benchmark, dependency/schema/capability, or Coverage drift entered the range | pass |

## Exact independent validation

The reviewer ran the complete required focused and public-process matrix. Every
command exited zero; every required target contained tests; and no accepted
target reported a failed, ignored, measured, or filtered test:

| Area | Passed |
| --- | ---: |
| Runtime library | 121 |
| Repository-change domain / Git reader / Git Workspace | 6 / 8 / 3 |
| Workspace service / file watching / persistent cache | 6 / 2 / 4 |
| Graph-query API | 3 |
| Graph focused | 86 |
| Analysis / Protocol / Tool Policy | 117 / 53 / 33 |
| MCP semantic / stdio / process | 7 / 8 / 17 |
| LSP stdio / process | 5 / 8 |
| HTTP / CLI | 4 / 2 |

The reviewer then ran the canonical gate:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
git diff --check
git diff --check 580496eb415965feb4710e0c536e519a46180645..9e17cf3cc799c532517fda5045a608b7e57da752
```

Every command exited zero. Compiled executable enumeration with
`--list --format terse` found 80 test targets, 76 non-zero targets, four
expected zero-test binary entry points, and 1,270 tests. The zero-test entries
were `oneagent-cli`, `oneagent-lsp`, `oneagent-mcp`, and `oneagent-runtime`.
Automatic Analysis, Protocol, and Tool Policy zero-test doctest harnesses were
not accepted as evidence.

An additional real SHA-256 repository selector passed. The environment refused
creation of a real non-UTF-8 APFS filename with `Operation not permitted`; the
injected production parser test passed the `UnsupportedPathEncoding` branch.
Neither diagnostic selector replaced the complete eight-test reader target.

The reviewer discarded and corrected two audit attempts: four diffs used an
incorrectly expanded short hash and returned `Invalid revision range`, and one
zsh audit assigned to the read-only `status` variable. The corrected exact-hash
and `rg_exit` audits passed. A recursive `find` attempt was replaced by
`rg --files -g AGENTS.md`. No discarded, zero-match, or zero-test attempt is
counted as positive acceptance evidence.

## Exact primary validation

The primary independently ran the following complete focused and public matrix
with the same successful counts:

```bash
cargo test -p oneagent-runtime --lib
cargo test -p oneagent-runtime --test repository_change_domain
cargo test -p oneagent-runtime --test git_change_reader
cargo test -p oneagent-runtime --test git_change_workspace
cargo test -p oneagent-runtime --test workspace_service
cargo test -p oneagent-runtime --test file_watching
cargo test -p oneagent-runtime --test persistent_cache
cargo test -p oneagent-runtime --test graph_query_api
cargo test -p oneagent-graph --test validation --test report --test build_diff --test reference_request_build --test coverage
cargo test -p oneagent-analysis
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

The primary then ran the same canonical gate with every command exiting zero
and independently reconfirmed the 80/76/4/1,270 compiled inventory. The local
VS Code node-mode gate also passed TypeScript production and test typechecks,
production and test compilation, 62 unit tests, and two real MCP Runtime-process
tests. Non-fatal Electron code-signing warnings were recorded as environment
output rather than failures.

Local GUI-dependent VS Code Extension Host and EDT hosts were not launched.
GitHub Actions run `33399662895` was queried by the primary and reported
`completed/success`, exact production `headSha` `129b69c8`, and six successful
macOS/Windows Rust, VS Code, and EDT jobs. The immutable review head differs
from that production head only in the two documentation evidence paths named
above.

## Scope, API, dependency, configuration, and security audits

- The review range changes no Cargo manifest, lockfile, production dependency,
  feature, license, CI workflow, cache version, Coverage file, protocol, CLI,
  VS Code, EDT, HTTP, MCP, or LSP consumer implementation.
- Production changes after ADR-0060 are confined to Runtime exports, the
  normalized domain, the local Git reader, and Workspace integration. Public
  API changes are additive; no existing public API was removed.
- Source audits found no unsafe code, shell invocation, forbidden production
  Git command, detached production reader task, secret, credential, private
  key, user-specific path, tracked generated/binary/package artifact, or raw
  sensitive Git value.
- Every production Git process uses the fixed read-only command vocabulary and
  the complete environment policy, including `GIT_NO_LAZY_FETCH=1`.
- Git-derived paths and statuses do not enter Graph, Analysis, diagnostics,
  Rules, Coverage, cache, Workspace snapshots, protocols, or IDE payloads.

## Residual risks and Sprint 39 hand-off

The accepted residual limitations are deliberate first-slice boundaries:

- the adapter is explicit-demand only, requires a compatible system Git, and
  does not add automatic discovery, polling, or a public control protocol;
- the observed endpoint is pinned `HEAD` plus final tracked worktree and
  non-ignored untracked files, not separate staged/unstaged, ignored, sparse,
  submodule-content, empty-directory, or multi-repository state;
- conflicts and changed gitlinks fail closed, and rename/copy identity remains
  deterministic delete/add evidence;
- two complete passes detect in-read drift but cannot prevent a repository
  mutation after an accepted observation;
- one-slot backpressure leaves retry policy with the caller, and every accepted
  input performs a complete rather than selective rebuild;
- a real non-UTF-8 path could not be constructed on the local APFS environment,
  so that rejection branch relies on the injected production parser test;
- macOS and Windows GUI compatibility relies on immutable exact-production-head
  CI evidence;
- remote Git, repository mutation, product impact, refactoring, edits, and
  broad performance or security claims remain deferred.

Sprint 38 is `completed`. Sprint 39 — Change Impact Analysis is the unique
`next` target. Sprint 39 must derive product impact only from complete semantic
graphs and canonical `SemanticGraphDiff`; repository paths and statuses remain
input evidence and cannot become semantic impact seeds or authority.

## Artifact consistency

The same fresh-context reviewer inspected the complete uncommitted review,
Roadmap/current-state diff, and exact retirement diff after primary drafting
and before staging or commit. The reviewer confirmed that every finding,
missing-evidence result, decision, validation outcome, environment limit,
residual risk, Sprint 39 hand-off, and retirement path is preserved without
weakening. No second reviewer was launched, and the reviewer remained read-only
during the consistency pass.

## Prompt retirement and preserved paths

The completed transition deletes exactly these nine verified Sprint 37 prompt
files:

- `docs/codex/prompts/sprint-37-rules-engine/00-sprint-37-execution-loop.md`
- `docs/codex/prompts/sprint-37-rules-engine/01-investigate-rules-engine.md`
- `docs/codex/prompts/sprint-37-rules-engine/02-define-rules-engine.md`
- `docs/codex/prompts/sprint-37-rules-engine/03-implement-rule-registry.md`
- `docs/codex/prompts/sprint-37-rules-engine/04-implement-rule-planning.md`
- `docs/codex/prompts/sprint-37-rules-engine/05-implement-rule-execution.md`
- `docs/codex/prompts/sprint-37-rules-engine/06-integrate-rule-snapshots.md`
- `docs/codex/prompts/sprint-37-rules-engine/07-complete-rules-engine-evidence.md`
- `docs/codex/prompts/sprint-37-rules-engine/08-sprint-37-integration-review.md`

The complete Sprint 38 prompt suite remains tracked and unchanged. Production
code, tests, fixtures, manifests, lockfile, ADR-0060, investigation, Task 6
evidence, prior reviews, and every unrelated prompt suite are preserved.
