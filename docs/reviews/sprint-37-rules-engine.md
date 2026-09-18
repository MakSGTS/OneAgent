# Sprint 37 Rules Engine Review

## Decision

`pass with non-blocking follow-ups`

The effective decision matches the independent reviewer recommendation. Sprint
37 satisfies ADR-0059: Analysis owns one bounded source-independent rule
registry, configuration, dependency plan, synchronous execution boundary,
terminal report, and Rule diagnostic input; Workspace composes the complete
result before immutable publication and recomputes it after cache decode; and
MCP and LSP retain their existing bounded read-only surfaces.

This decision does not claim a product rule, external rule configuration,
dynamic plugins, scripts, remote rules, hot reload, mutable-document analysis,
fixes or edits, a rule-management protocol or UI, telemetry, performance or
security results, or a Coverage transition.

## Reviewed baseline

- Completed Sprint 36 prerequisite: `8240ed1a1e56bac4e6fef985cce31c56ec7233ce`.
- Sprint 37 planning anchor: `3c0222dce613cb38eef3959b3a7379ca4af42726`.
- Task 7 head: `d82b9d12e25b6fea737656f8803c03bb1d06a82e`.
- Exact reviewed range:
  `8240ed1a1e56bac4e6fef985cce31c56ec7233ce..d82b9d12e25b6fea737656f8803c03bb1d06a82e`.
- Range size: 9 commits, 38 paths, 6,490 additions, 53 deletions.

The dependency-ordered commits are:

| Step | Commit | Subject | Result |
| --- | --- | --- | --- |
| Framework | `68045f0c` | `Establish Rules Engine task framework` | pass |
| Planning | `3c0222dc` | `Plan Sprint 37 Rules Engine` | pass |
| Investigation | `bab5cade` | `Investigate Sprint 37 rules engine` | pass |
| ADR-0059 | `6518c6c1` | `Define Sprint 37 rules engine` | pass |
| Registry | `f8589f2c` | `Implement Sprint 37 rule registry` | pass |
| Planning and configuration | `241b8350` | `Implement Sprint 37 rule planning` | pass |
| Execution and diagnostics | `84313b6e` | `Implement Sprint 37 rule execution` | pass |
| Workspace/cache/projections | `ca054770` | `Integrate Sprint 37 rule snapshots` | pass |
| Evidence | `d82b9d12` | `Complete Sprint 37 rules engine evidence` | pass |

Independent reviewer `/root/sprint37_rules_engine_reviewer` received a fresh
context with the exact immutable range, governing authorities, acceptance and
exclusion criteria, required test matrix, and structured output contract. The
reviewer began and ended at Task 7 head with a clean working tree, remained
read-only, delegated no work, and made no source, documentation, staging,
commit, branch, configuration, download, or remote mutation. Required Cargo
validation could create or refresh ordinary ignored outputs under `target/`;
those build outputs are not repository-source mutations or review artifacts.
The reviewer used no network and executed no path outside the repository.

## Findings and primary reconciliation

### Blocking findings

None.

### Non-blocking follow-up

The reviewer found one Low source-compatibility documentation issue at
`docs/architecture/rules-engine-evidence.md:184`. The evidence states that
existing APIs remain available and no consumer requires migration, but
`DiagnosticFinding::code()` changed from returning `DiagnosticCode` by value to
returning `&DiagnosticCode` at `crates/analysis/src/diagnostics/mod.rs:607`.
`DiagnosticCode` also stopped implementing `Copy` when the owned Rule-local
code variant was added. Existing repository consumers were migrated and all
tests pass, but external Rust source such as
`let code: DiagnosticCode = finding.code();` now requires an explicit clone or
borrowed binding.

Primary classification: **accepted**. The baseline and current definitions,
derive list, accessor signature, and every repository consumer reproduce the
claim. ADR-0059 permits internal API migration and the sprint does not promise
public API stability before Sprint 42, so this does not block the accepted
first slice. The correct interpretation is narrower than the Task 7 sentence:
all existing repository consumers remain compatible after their in-range
migration; arbitrary external Rust source compatibility is not established.
Restoring the by-value accessor or correcting the evidence claim requires a
separate authorized change and fresh review of a new immutable range, so Task 8
does not silently alter production code or Task 7 evidence.

### Missing evidence

None. There is no unresolved disagreement, and the effective decision is not
less severe than the reviewer recommendation.

## Acceptance evidence matrix

| Criterion | Independent and primary evidence | Result |
| --- | --- | --- |
| Inputs and authority | Rules borrow one immutable Graph, complete validation, and base Semantic/Validation report; Graph, validation, provenance, locations, reports, and diffs remain authoritative | pass |
| Identity and registry | Validated global rule identity, local diagnostic/failure codes, canonical definitions, bounded immutable registration, deterministic duplicate/conflict rejection | pass |
| Dependencies and order | Missing, self, cyclic, and unknown inputs fail atomically; smallest ready complete rule ID is the stable tie-breaker | pass |
| Configuration and applicability | Bounded in-memory Enabled/Disabled settings, Enabled default, distinct NotApplicable result, no external grammar or persistence | pass |
| Execution and cancellation | Synchronous sequential execution, Completed-only dependency satisfaction, independent failure continuation, cooperative before/after cancellation, no partial aggregate on engine failures | pass |
| Failures, results, and bounds | Closed terminal vocabulary, redacted failures, exact/one-over registration, dependency, output, message, anchor, provenance, and aggregate limits | pass |
| Diagnostics | Rule identity maps through ADR-0058 conflict, suppression, order, summary, provenance, location, and completeness contracts | pass |
| Workspace and lifecycle | Complete base report, plan, execution report, and final report precede atomic cold/watched publication and cleanup | pass |
| Cache | Schema `1`, private semantic compatibility `4`, no executable/configuration/result serialization, deterministic recomputation, clean recovery from version `3` | pass |
| MCP | Exactly seven lexicographic read-only Tool Policy-gated tools; only family `rule` and Rule-only `ruleId` are additive | pass |
| LSP | Exact LSP 3.17 capability and payload shape; only active one-anchor Rule findings with one confined typed span project | pass |
| Compatibility | Graph, adapters, HTTP, CLI, VS Code, EDT, Coverage, Runtime ownership, protocol lifecycle, and public processes remain compatible | pass |
| Dependencies and sensitive data | No manifest, lockfile, dependency, feature, license, secret, credential, source text, raw trace, generated binary, or cache artifact entered the range | pass |
| Deferred scope | No product rule, plugin/script/remote rule, external configuration, mutable document, source mutation, fix/edit flow, management surface, telemetry, performance/security, or Git Change Adapter work | pass |

## Exact independent validation

The reviewer ran the required focused and public-process matrix at exact Task 7
head. All commands exited zero and every required target was non-zero:

| Area | Passed |
| --- | ---: |
| Rules unit / registry / planning / execution / diagnostics | 19 / 5 / 6 / 15 / 6 |
| Diagnostics unit / public | 25 / 3 |
| Graph focused | 86 |
| Runtime library | 99 |
| Workspace / cache / watching | 6 / 4 / 2 |
| Service container / graph-query API | 6 / 3 |
| Protocol / Tool Policy | 53 / 33 |
| MCP semantic / stdio / process | 7 / 8 / 17 |
| LSP stdio / process | 5 / 8 |
| HTTP / CLI | 4 / 2 |

The reviewer also ran the canonical gate in order:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps
git diff --check
```

Every command exited zero. The compiled inventory contained 77 test targets,
73 non-zero targets, four expected zero-test binary entry points, and 1,231
passed tests with zero failed, ignored, measured, or filtered tests.

The focused Rules unit selector reported 19 accepted tests and 52 unrelated
filtered tests. The focused Diagnostics unit selector reported 25 accepted
tests and 46 unrelated filtered tests. These filtered counters are selector
scope, not the unfiltered canonical result above. Automatic Protocol and Tool
Policy doctest targets each contained zero tests and were not accepted as
evidence. The reviewer additionally ran
`git diff --check 8240ed1a1e56bac4e6fef985cce31c56ec7233ce..d82b9d12e25b6fea737656f8803c03bb1d06a82e`
with exit 0.

Three discarded reviewer audit attempts are preserved explicitly. One `rg`
used a nonexistent configuration path and exited 2; the corrected repository
path audit passed. A separate `rg` returned exit 1 because it found zero
matches; that result was not treated as evidence and the intended absence was
verified with a command whose zero-match semantics were handled explicitly.
The first inventory aggregation algorithm incorrectly printed `targets=0`; it
was rejected and replaced by executable enumeration, which produced the
accepted 77/73/4/1,231 inventory. No failed, zero-match, zero-test, or corrected
attempt is counted as positive acceptance evidence.

## Exact primary validation

After receiving the independent report, the primary reproduced the finding and
ran the complete focused/public matrix. The successful commands were:

```bash
cargo test -p oneagent-analysis --lib rules::
cargo test -p oneagent-analysis --test rule_registry
cargo test -p oneagent-analysis --test rule_planning
cargo test -p oneagent-analysis --test rule_execution
cargo test -p oneagent-analysis --test rule_diagnostics
cargo test -p oneagent-analysis --lib diagnostics::
cargo test -p oneagent-analysis --test diagnostics_engine
cargo test -p oneagent-graph --test validation --test report --test build_diff --test reference_request_build --test coverage
cargo test -p oneagent-runtime --lib
cargo test -p oneagent-runtime --test workspace_service
cargo test -p oneagent-runtime --test persistent_cache
cargo test -p oneagent-runtime --test file_watching
cargo test -p oneagent-runtime --test service_container
cargo test -p oneagent-runtime --test graph_query_api
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

The counts equal the independent matrix above. An additional full Graph library
run passed 162 tests. The first primary batch stopped after the 19 rule unit
tests because `rule_registry_public` was not a live target. Two later batches
stopped after already successful preceding targets because `workspace_service`
was assigned to the wrong package and `watching`/`tool_policy` used stale target
names. Cargo metadata identified the exact live package/target names; the
corrected commands above all passed. These failed selectors are not acceptance
evidence and exposed no repository defect or zero-match filtered test.

The primary then reran the canonical gate shown above. Every command exited
zero, including strict Clippy and Rustdoc. Executable enumeration plus
`--list --format terse` independently reconfirmed 77 targets, 73 non-zero, four
expected zero-test binaries, and 1,231 tests.

## Compatibility and host evidence

Exact code head `ca054770` passed all six cross-platform CI jobs in run
`33299869526`. Task 7 head differs from that code head only in README and
architecture/Roadmap evidence files.

- macOS and Windows VS Code jobs passed typecheck, compilation, 62 unit tests,
  18 Extension Host scenarios, 2 real Runtime-process tests, and package/scope
  audits.
- macOS and Windows EDT jobs used JDK 25 and passed 41 tests with zero failures,
  errors, or skips plus the p2 package audit.
- macOS and Windows Rust jobs passed their required workspace gates.

The earlier local EDT attempt used ambient Java 17 and failed in Tycho before
tests because the current reactor requires a newer Java. It is recorded as
non-evidence, caused no tracked change, and does not weaken the exact-code-head
CI result. The fresh reviewer and primary did not relaunch GUI-dependent hosts;
they inspected the committed immutable CI evidence, unchanged consumer source,
and complete Rust public-process matrix.

## Scope, API, dependency, configuration, and security audits

- Registry and configuration are immutable, bounded, in-memory, deterministic,
  source-independent, and empty/default in production. No second authority or
  unsupported configuration surface exists.
- Dependency planning is complete before execution; results are reconciled to
  the plan and diagnostics to canonical Graph evidence. Cancellation is
  cooperative and cannot preempt a rule that never returns.
- Cache bytes do not own derived rule semantics. Version `3` entries rebuild;
  version `4` decode recomputes the complete empty production result.
- MCP catalog count, lexicographic order, schema closure, read-only annotations,
  Tool Policy gate, revision parity, request isolation, and stdout purity remain
  intact. LSP capabilities, framing, confinement, pull-only diagnostics, and
  complete bound remain intact.
- The range changes no Cargo manifest or lockfile and adds no production
  dependency, feature, license, unsafe code, remote input, credential source,
  authentication claim, filesystem mutation, source mutation, or generated
  artifact.
- The one accepted public Rust source migration is recorded above; no broader
  public API stability claim is made before Sprint 42.

## Residual risks and Sprint 38 hand-off

The accepted residual limitations are deliberate first-slice boundaries:

- production registration is empty, so behavior proves composition but not a
  product-rule catalog;
- cancellation is cooperative and cannot preempt a stuck or panicking rule;
- rule messages are trusted bounded values rather than sanitized source text;
- cache semantic version `3` becomes a deterministic cold rebuild;
- arbitrary external Rust source compatibility is not guaranteed;
- GUI compatibility relies on immutable exact-code-head CI evidence.

The non-blocking API/evidence wording issue remains an explicit follow-up. It
does not authorize a production or evidence edit in this review task.

Sprint 37 is `completed`. Sprint 38 — Git Change Adapter is the unique `next`
target. Sprint 38 must preserve Git as an input-evidence adapter rather than a
semantic authority and must not reinterpret the Rules Engine boundary.

## Artifact consistency

The same reviewer inspected the first uncommitted review, Roadmap/current-state
diff, and retirement diff and blocked transition on four documentation-only
inconsistencies: one stale Roadmap current-state sentence, one inaccurate
README uniqueness phrase, omitted reviewer validation/environment details, and
an overbroad claim that Cargo validation created no artifacts. The corrected
draft updates the two state statements, preserves the filtered/zero-test and
discarded-audit results plus the no-network/no-external-path limit, and
distinguishes ignored Cargo outputs from repository-source mutation.

The same reviewer then rechecked the complete corrected draft read-only and
confirmed that the decision, Low finding, absence of blocking findings and
missing evidence, exact validation results, environment limits, six residual
risks, Sprint 38 hand-off, and retirement inventory are preserved without
weakening. No second reviewer was launched, and the reviewer made no source,
documentation, or Git-state mutation during either consistency pass.

## Prompt retirement and preserved paths

The completed transition deletes exactly these nine verified Sprint 36 prompt
files:

- `docs/codex/prompts/sprint-36-diagnostics-engine/00-sprint-36-execution-loop.md`
- `docs/codex/prompts/sprint-36-diagnostics-engine/01-investigate-diagnostics-engine.md`
- `docs/codex/prompts/sprint-36-diagnostics-engine/02-define-diagnostics-engine.md`
- `docs/codex/prompts/sprint-36-diagnostics-engine/03-implement-diagnostic-domain.md`
- `docs/codex/prompts/sprint-36-diagnostics-engine/04-implement-diagnostic-orchestration.md`
- `docs/codex/prompts/sprint-36-diagnostics-engine/05-integrate-diagnostic-snapshots.md`
- `docs/codex/prompts/sprint-36-diagnostics-engine/06-integrate-diagnostic-reporting.md`
- `docs/codex/prompts/sprint-36-diagnostics-engine/07-complete-diagnostics-evidence.md`
- `docs/codex/prompts/sprint-36-diagnostics-engine/08-sprint-36-integration-review.md`

The complete Sprint 37 prompt suite remains tracked and unchanged. Production
code, tests, fixtures, manifests, lockfile, ADR-0059, investigation, Task 7
evidence, prior reviews, and every unrelated prompt suite are preserved.
