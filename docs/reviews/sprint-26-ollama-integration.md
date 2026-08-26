# Sprint 26 Ollama Integration Review

## Decision

`pass`

Sprint 26 satisfies the accepted ADR-0048 bounded local provider slice and
completion gate. The implementation is additive, excludes remote-backed
entries before provider-specific inspection, advertises text generation only
from exact native capability evidence, performs one strict native generation
attempt, and preserves the provider-neutral request identity. All focused and
complete validation passes, and no blocking or non-blocking defect was found.

## Reviewed baseline

- Planning baseline: `5f903043679309d1939d4ee750d745fc8c6d6bb5`.
- Planning commit: `68d8574bd1b3a91ef63b3614c95a370c52a8afb6`.
- Review head: `68022493b2fac6de56a825fe41b6d33a7fabde22`.
- Exact reviewed range:
  `68d8574bd1b3a91ef63b3614c95a370c52a8afb6^..68022493b2fac6de56a825fe41b6d33a7fabde22`.
- Initial review Git status: clean.
- Platform: `aarch64-apple-darwin`.
- Toolchain: Rust `1.97.1` (`8bab26f4f`, 2026-07-14), Cargo `1.97.1`.

The dependency-ordered Sprint 26 commits are:

| Step | Commit | Result |
| --- | --- | --- |
| Planning | `68d8574b` | pass |
| Investigation | `97eaccc7` | pass |
| ADR-0048 | `a22dcfe0` | pass |
| Client foundation | `366a6cbd` | pass |
| Native model discovery | `32fe1b56` | pass |
| Native text generation | `37920533` | pass |
| Public evidence and current-state docs | `68022493` | pass |

The user explicitly approved the exact ADR-0048 normal and dev dependency
block before Task 3. The reviewed range changes only workspace/member and lock
registration, the new `oneagent-ollama` leaf and its tests, ADR-0048 and its
investigation, Sprint 26 planning prompts, Roadmap planning, and the three
authorized current-state documents. It does not change the existing concrete
adapters, shared LLM crate, Analysis, Runtime, protocol, CLI, graph, workspace,
or source adapters.

## Acceptance evidence matrix

| Criterion | Evidence | Result |
| --- | --- | --- |
| Planning and prerequisites | The committed plan orders Tasks 1-7, fixes exact gates and exclusions, and Tasks 1-6 are committed in dependency order from the clean Sprint 25 review baseline. | pass |
| Investigation | Repository facts, official Ollama documentation, sanitized authorized local observations, and deterministic acceptance evidence are separated. Mutable live observations remain supplementary and are not a test oracle. | pass |
| Accepted architecture | ADR-0048 fixes ownership, exact approved dependencies, construction, locality, native discovery and generation wires, identity, bounds, failures, timeout/cancellation, cleanup, compatibility, and deferred scope. | pass |
| Ownership and public API | `oneagent-ollama` exports only `OllamaProvider` with explicit `new`, `new_local`, `id`, and `LlmProvider`; clients, URLs, DTOs, execution helpers, and test support remain private. | pass |
| Dependency direction | The leaf depends on `oneagent-llm`; the shared crate remains std-only. Reverse inspection finds no Ollama consumer and no Analysis or Runtime dependency. | pass |
| Exact dependencies and features | Direct normal dependencies are `oneagent-llm`, reqwest `0.13.4` with defaults disabled, serde `1.0.228` with `derive`, serde_json `1.0.150`, and Tokio `1.53.0` with `macros,time`; dev Tokio adds only the approved I/O/runtime/sync features. No TLS feature or other direct edge is present. | pass |
| Construction and locality | Tests prove exact `ollama` identity, provider-mismatch precedence, credential rejection, no construction I/O, the fixed numeric-loopback default, and explicit lowercase HTTP `127.0.0.1` root validation. | pass |
| Client policy | The private client disables redirects and implicit proxies, uses the static Ollama user agent, sends no authentication, and performs byte-bounded decoding. Environment and files do not configure provider behavior. | pass |
| Tags discovery | Each call sends one fresh exact `GET /api/tags`; identities and remote markers validate strictly, remote-backed entries are excluded before Show, local IDs are sorted, and duplicate or over-count input is rejected atomically. | pass |
| Show capability projection | Sequential canonical `POST /api/show` calls preserve exact IDs and fields. Only exact lowercase `completion` contributes `TextGeneration`; missing, null, mistyped, malformed, or later-failing responses reject the entire catalog. | pass |
| Discovery bounds | Tags and each Show response are bounded to 1 MiB, each Show request to 4 KiB, and source entries, Show requests, and projected models to 1,024. Empty, remote-only, maximum, reordered, and repeated cases have exact tests. | pass |
| Generation wire and identity | One `POST /api/generate` sends exactly model, prompt, `stream=false`, `raw=true`, `think=false`, and `options.num_predict` equal to the validated output-byte bound. Success remains bound to the original `ollama` provider/model request. | pass |
| Terminal mapping and fallback rejection | Only exact terminal model, `done=true`, absent/empty thinking, and `stop` or `length` map to `Completed` or `OutputLimit`; mismatch, nonterminal, unsupported, malformed, empty, and over-bound results are rejected. | pass |
| Timeout, cancellation, attempt, and cleanup | Existing and in-flight cancellation, total timeout, biased simultaneous-ready precedence, one request sequence per operation, losing-future drop, deterministic server joins, repetition, and no retained adapter work have exact evidence. | pass |
| Redaction | Synthetic URL, secret, header, Tags, Show, prompt, response, thinking, output, and transport sentinels are absent from implicit provider/error formatting and diagnostics. Error bodies and library sources are not attached to public errors. | pass |
| Public substitution | Six public tests use `&dyn LlmProvider` and exported provider-neutral values for construction, discovery, generation, failures, timeout, cancellation, redaction, repetition, and cleanup. | pass |
| Existing-provider compatibility | The unchanged OpenAI-compatible adapter passes 18 unit and 6 public tests; the unchanged LM Studio adapter passes 19 unit and 6 public tests. | pass |
| Provider-neutral and consumer compatibility | `oneagent-llm` passes 22 unit and 7 public tests; Analysis passes 27 unit and 11 public tests; all Runtime package targets pass. Their implementations and ownership remain unchanged. | pass |
| Deterministic oracle | Acceptance uses synthetic fixtures and controlled `127.0.0.1:0` servers with explicit joins. There are no ignored tests, credential/environment inputs, developer paths, installed Ollama calls, model downloads, or external network. | pass |
| Documentation and exclusions | README, Architecture, Semantic Model, Roadmap planning, investigation, and ADR-0048 describe only the implemented bounded leaf. Runtime integration, live compatibility, daemon/model lifecycle, cloud/auth, chat/templates, streaming, tools, MCP, IDE, quality, and performance remain deferred. | pass |

## Findings

### Blocking

None.

### Non-blocking

None.

## Missing evidence

None for the accepted ADR-0048 boundary. Review validation deliberately does
not contact installed or running Ollama, use a local or cloud model or
credential, or judge response quality. ADR-0048 defines those as unsupported
claims rather than completion evidence.

## Focused validation

- `cargo test -p oneagent-ollama --lib --offline -- --list` — 25 tests listed.
- `cargo test -p oneagent-ollama --test conformance --offline -- --list` — 6
  tests listed.
- `cargo test -p oneagent-ollama --lib --offline` — 25 passed.
- `cargo test -p oneagent-ollama --test conformance --offline` — 6 passed.
- `cargo test -p oneagent-openai-compatible --offline` — 18 unit and 6 public
  tests passed.
- `cargo test -p oneagent-lm-studio --offline` — 19 unit and 6 public tests
  passed.
- `cargo test -p oneagent-llm --offline` — 22 unit and 7 public tests passed.
- `cargo test -p oneagent-analysis --offline` — 27 unit and 11 public tests
  passed.
- `cargo test -p oneagent-runtime --offline` — all library and integration
  targets passed, including 78 library tests.

No selected target or filter matched zero tests.

## Dependency, public-surface, scope, and sensitive-state audits

- `cargo tree -p oneagent-ollama --depth 1 --edges normal --offline` and the
  feature tree match the exact approved direct dependency and feature set.
- Reverse normal trees show no consumer of `oneagent-ollama`, and only concrete
  adapters above std-only `oneagent-llm`.
- Public-surface inspection finds only `OllamaProvider`, `new`, `new_local`,
  `id`, and its `LlmProvider` implementation.
- Range/path inspection finds no implementation change outside the new leaf;
  existing provider, provider-neutral, and consumer packages are unchanged.
- Redaction, no-live-state, ignored-test, local-path, credential-input,
  retry/fallback/cache/background-state, documentation-link, prompt-inventory,
  and deletion-boundary audits found no blocking or non-blocking finding.

## Complete validation

The canonical complete workspace gate passed on committed Task 6 review head
`68022493b2fac6de56a825fe41b6d33a7fabde22` with sandbox permission limited to
required local loopback binds. The same gate passed again after the review
artifact, state transition, hand-off synchronization, and authorized suite
retirement:

- `cargo fmt --all -- --check`
- `cargo check --workspace --offline`
- `cargo test --workspace --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --offline`
- `git diff --check`

No external network, live provider, credential, environment configuration,
ignored artifact, or response-quality observation was used by review
validation.

## Scope and exclusion conformance

The reviewed range creates one local-only leaf adapter and deterministic
evidence. It does not change graph facts, Context Engine semantics, Analysis,
Runtime, protocol, CLI, workspace, source adapters, persistence, Coverage
Registries, MCP, LSP, IDE, or configuration ownership. Runtime registration,
live compatibility, daemon/model lifecycle, cloud/authentication, chat,
templates, streaming, tools, embeddings, structured output, retry/backoff,
catalog caching, quality, performance, and Sprint 27 implementation remain
excluded.

## Risk assessment

Residual risk is bounded to the accepted first slice: Ollama wire additions may
vary across versions; numeric-loopback HTTP is intentionally unauthenticated;
sequential inspection of 1,024 candidates can be expensive although bounded by
one total timeout; `num_predict` is provider token control rather than local
byte authority; cancellation is cooperative; and sensitive buffers are not
zeroized. The review makes no broad Ollama version, model quality, performance,
cost, availability, cloud, or security claim. None blocks the accepted local
provider boundary.

## Previous-suite retirement

Before deletion, filesystem and tracked inventories each contained exactly the
eight authorized Sprint 25 prompt files and the untracked inventory was empty
(`8/8/0`). The suite is retired atomically with this review. The complete
Sprint 26 suite, `docs/codex/prompts/run-next-sprint.md`, non-adjacent suites,
and `.codex/` remain unchanged.

## Repository state

Review-owned changes are limited to this artifact, minimal Roadmap/current-
state hand-off text, and the exact eight authorized Sprint 25 prompt deletions.
Production code, manifests, lock state, current/non-adjacent prompt suites,
Coverage Registries, and `.codex/` remain unchanged from the committed Task 6
head.

## State transition and next action

Sprint 26 transitions from `next` to `completed`. Sprint 27 Tool Execution
Policy becomes the unique `next` planning target. Sprint 27 implementation has
not started; its next action is a separately committed dependency-ordered
planning task.
