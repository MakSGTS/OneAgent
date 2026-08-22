# Sprint 23 LLM Provider Abstraction Integration Review

## Decision

`pass`

Sprint 23 satisfies ADR-0045 and the Roadmap completion gate. No blocking or
non-blocking findings and no missing acceptance evidence remain. Sprint 24
OpenAI-Compatible Provider is the unique next planning target.

## Reviewed baseline

- Sprint 22 review head: `3257ac2c92a11861ffc2baeedd07ce7cad910528`
- Reviewed range: `b24eae84^..277fb568`
- Committed Task 6 head: `277fb56893a0641d7b5e6530230d754c8485fe96`
- Review date: 2026-08-22

| Commit | Subject | Owned paths |
| --- | --- | --- |
| `b24eae8414cc2d794c4693843f7e0df23acb60a0` | `Add LLM provider task framework` | `docs/Roadmap.md`, `docs/codex/README.md`, and the LLM Provider profile/workflow/template |
| `01f51a5045938a4369c768ae5cba5eacec95cc65` | `Plan Sprint 23 LLM Provider Abstraction` | `docs/Roadmap.md` and the eight-file Sprint 23 prompt suite |
| `bb5df4ca18f25f42a255b8b18caefe7d6e593e7d` | `Investigate Sprint 23 LLM Provider boundary` | `docs/architecture/llm-provider-investigation.md` |
| `92821529c1f1ad938f283bde452d0730bddbd495` | `Define Sprint 23 LLM Provider abstraction` | `docs/adr/0045-llm-provider-abstraction.md` |
| `84e137e83f81e6158e33953f312e6ebd3301fadd` | `Implement Sprint 23 provider domain model` | workspace manifests and the provider-neutral crate domain modules |
| `141f668314813ac18618df6d060dd491e06a4fe1` | `Implement Sprint 23 capability-aware requests` | request construction, request-bound response construction, and public exports |
| `d98a6d9475eb7ef2bf09f03b725dfb84a3a9978a` | `Implement Sprint 23 provider execution boundary` | asynchronous provider/cancellation seam, unit fakes, and public exports |
| `277fb56893a0641d7b5e6530230d754c8485fe96` | `Complete Sprint 23 LLM Provider evidence` | public conformance target and current-state README/architecture documentation |

The range changes only the reusable LLM Provider framework and planning,
investigation and ADR evidence, one additive std-only `oneagent-llm` crate,
public provider-neutral conformance evidence, and current-state documentation.
It changes no graph, analysis, Runtime, CLI, protocol, source adapter, Coverage,
configuration source, provider wire implementation, or live-service behavior.

## Acceptance evidence matrix

| Criterion | Evidence | Result |
| --- | --- | --- |
| Framework and planning readiness | The committed framework stage adds the smallest reusable provider profile/workflow/template; the plan orders Tasks 1–7, preserves prerequisite gates, and records the exact Sprint 22 retirement inventory. | pass |
| Investigation | Repository-backed evidence distinguishes existing Context/Runtime patterns from absent provider authority and inventories ownership, dependencies, secrets, execution policy, fakes, platforms, consumers, and unresolved decisions before architecture selection. | pass |
| Accepted architecture | ADR-0045 fixes crate ownership, exact vocabulary and bounds, validation precedence, secret/redaction rules, asynchronous substitution, policy, cancellation, errors, conformance, compatibility, and deferred scope. | pass |
| Ownership and dependencies | The additive `oneagent-llm` crate is std-only and owns no graph, Context Engine, Runtime, executor, transport, wire schema, filesystem, environment, or global state. | pass |
| Provider/model identity | Separate bounded strong IDs preserve accepted bytes and exact validation precedence; model identity is provider-scoped and deterministically ordered. | pass |
| Discovery and capabilities | The single closed `TextGeneration` capability canonicalizes duplicate input; catalogs enforce count/provider/identity constraints, sort deterministically, and retain empty discovery as success. | pass |
| Public domain values | Model descriptors/catalogs, configuration/secrets, execution policy, request, response, usage, finish, errors, cancellation, futures, context, and provider trait are exported without provider wire values or Serde claims. | pass |
| Request validation and precedence | Empty text, UTF-8 input bound, zero output bound, output maximum, then missing capability are checked atomically in ADR order; accepted whitespace, line endings, Unicode, and byte order are preserved. | pass |
| Compatibility | Only a descriptor advertising `TextGeneration` can produce a request; provider mismatch is rejected before fake work and no fallback, alias, implicit feature, or partial request exists. | pass |
| Response, usage, and finish | Public response construction is tied to the originating validated request, enforces non-empty bounded output, computes checked local byte usage, retains exact identity, and exposes only `Completed` or `OutputLimit`. | pass |
| Configuration and secrets | Configuration contains only provider identity and an optional bounded opaque credential; the secret is non-cloneable and explicitly exposed only through its accessor. | pass |
| Redaction | Secret Debug is exact redaction; request/response/error Debug omits sensitive text; bounded diagnostics require explicit access and synthetic sentinel checks prove implicit formatting does not disclose content. | pass |
| Error taxonomy | All fourteen closed error kinds and exact retryable subset are publicly exercised; no unrestricted source error or provider body can escape through `Error::source`. | pass |
| Async substitution | Two independent fakes operate through `&dyn LlmProvider`; boxed standard-library futures borrow provider/request/context and return one owned terminal catalog or response. | pass |
| Timeout and retry | Optional timeout is a validated value only; `RetryPolicy::Never` exposes exactly one attempt and fake retryable failures prove no hidden clock, delay, backoff, or replay. | pass |
| Cancellation and cleanup | Receiver-only cancellation is checked before work and observed during pending work; explicit counters and drop guards prove cancellation returns one typed terminal error with no surviving fake operation. | pass |
| Public conformance corpus | Seven public integration tests use only exported APIs, exact Rust values, deterministic fakes, explicit state/wakers, and synthetic sentinels without network, credentials, environment, filesystem, or sleeps. | pass |
| Reordered and repeated equality | Capability duplicates, reordered catalogs, equivalent descriptors, repeated requests/catalogs, and repeated fresh provider executions produce canonical equal observations. | pass |
| Dependency impact | `cargo tree -p oneagent-llm --edges normal` and reverse normal dependency inspection contain only `oneagent-llm`; analysis and Runtime manifests remain independent. | pass |
| Context and Runtime compatibility | `oneagent-analysis` passes 27 unit and 11 public tests; `oneagent-runtime --lib` passes 78 tests; neither package or observable contract changed. | pass |
| Platforms | Review validation passed on `aarch64-apple-darwin` with Rust 1.97.1; the implementation is portable std-only Rust and CI retains macOS 14 and Windows targets. | pass |
| Documentation truth | README, Architecture, Semantic Model, Roadmap, investigation, ADR-0045, Rustdoc, and public tests agree on the implemented first slice and review-pending state before this decision. | pass |
| Scope containment | No concrete provider, HTTP/JSON/SSE, live configuration/credential source, tokenizer, prompt/tool policy, streaming, conversation, Runtime/CLI/protocol, MCP/IDE, or unsupported quality/security claim entered the baseline. | pass |

## Findings

No blocking or non-blocking findings.

## Missing evidence

None.

The review counted only non-zero focused targets: 22 provider-domain unit
tests, including 4 request and 5 provider/execution tests, plus 7 public
conformance tests. Filtered matches were reported separately from complete
package results.

## Validation

The review independently reran the focused and compatibility matrix:

- `cargo test -p oneagent-llm --lib -- --list` — 22 tests listed.
- `cargo test -p oneagent-llm --lib` — 22 passed.
- `cargo test -p oneagent-llm --lib request::tests -- --nocapture` — 4 passed.
- `cargo test -p oneagent-llm --lib provider::tests -- --nocapture` — 5 passed.
- `cargo test -p oneagent-llm --test provider_contract -- --list` — 7 tests listed.
- `cargo test -p oneagent-llm --test provider_contract -- --nocapture` — 7 passed.
- `cargo test -p oneagent-llm` — 22 unit and 7 public integration tests passed; doctests completed successfully.
- `cargo test -p oneagent-analysis` — 27 unit and 11 public integration tests passed; doctests completed successfully.
- `cargo test -p oneagent-runtime --lib` — 78 passed.
- `cargo tree -p oneagent-llm --edges normal` and `cargo tree -i oneagent-llm --edges normal` — only the std-only package itself.

The canonical complete gate also passed:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
- `git diff --check`

The managed sandbox denies the existing Runtime HTTP loopback binds without
additional local permission; the complete suite ran with bounded loopback
permission. No external network or service was used.

## Deferred scope

Concrete OpenAI-compatible, LM Studio, and Ollama adapters; provider HTTP/JSON/
SSE schemas and mappings; live configuration, credential sources, model
discovery, and execution; automatic retries and clock enforcement; streaming,
tokenization, prompt/tool policy, conversations, structured output and media;
Runtime/HTTP/CLI/protocol exposure; persistence, MCP, LSP, and IDE integration;
and latency, cost, quality, performance, security, or broad compatibility
claims remain deferred.

## Risk assessment

Residual risk is bounded to the provider-neutral first slice. Timeout is only a
represented value, retry is disabled, and cancellation is cooperative rather
than forceful. Every future concrete adapter must validate its provider wire
mapping, cancellation race and cleanup, secret redaction, response bounds, and
platform behavior through the public conformance contract. These accepted
limits do not block ADR-0045 or Sprint 24 adapter planning.

## Previous-suite retirement

After the `pass` decision, `git ls-files` and the filesystem both contained
exactly the eight planned Sprint 22 prompt files, with no symlink or untracked
addition. Repository search found no retained Markdown link dependency on an
individual deleted prompt. The exact suite is retired atomically with this
review; the complete Sprint 23 suite, `docs/codex/prompts/run-next-sprint.md`,
non-adjacent suites, and `.codex/` remain unchanged.
