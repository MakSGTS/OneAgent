# Sprint 24 OpenAI-Compatible Provider Integration Review

## Decision

`pass with non-blocking follow-ups`

Sprint 24 satisfies the accepted ADR-0046 first slice and completion gate. The
implementation is additive, provider-neutral ownership remains unchanged, the
focused and complete validation gates pass, and the only confirmed finding is
one stale crate-level Rustdoc sentence that does not affect the public API,
runtime behavior, acceptance evidence, or current-state documentation.

## Reviewed baseline

- Planning baseline: `c9166974a66bc8f8de419cec821f966e0d68db80`.
- Review head: `fd7ebc411a6950e057fee1253d2bb5294839e608`.
- Reviewed range: Sprint 24 planning through Task 6.
- Platform: `aarch64-apple-darwin`.
- Toolchain: Rust `1.97.1` (`8bab26f4f`, 2026-07-14).

The dependency-ordered Sprint 24 commits are:

| Task | Commit | Result |
| --- | --- | --- |
| Planning | `f9124372` | pass |
| Investigation | `b5700fe3` | pass |
| ADR-0046 | `612881d8` | pass |
| Client foundation | `36f3b451` | pass |
| Model discovery | `a78e208a` | pass |
| Text generation | `ce75ed94` | pass |
| Public evidence and current-state docs | `fd7ebc41` | pass |

The user explicitly approved the exact ADR-0046 dependency set before Task 3.
The manifest and lock state retain those approved versions and features.

## Acceptance evidence matrix

| Criterion | Evidence | Result |
| --- | --- | --- |
| Planning and prerequisites | The plan orders Tasks 1-7, preserves the architecture and dependency gates, and identifies the exact previous-suite inventory. All Tasks 1-6 are committed in order. | pass |
| Investigation and pinned source | The investigation separates repository facts, pinned llama.cpp build 10485 at commit `1511ce3bc3f087376c8526b4ad07100bfabb277f`, sanitized authorized live observations, and deterministic acceptance evidence. | pass |
| Accepted architecture | ADR-0046 fixes ownership, exact dependencies, construction, transport, authentication, wire mappings, bounds, failures, timeout/cancellation, cleanup, compatibility, and deferred scope. | pass |
| Ownership and public API | `oneagent-openai-compatible` exposes only `OpenAiCompatibleProvider`; private URLs, client, header, and DTOs do not escape. It depends inward on `oneagent-llm`. | pass |
| Provider-neutral compatibility | `oneagent-llm` remains std-only. Reverse normal dependency inspection finds only the concrete adapter; analysis and Runtime do not depend on the adapter or shared LLM crate. | pass |
| Exact dependencies | Direct normal dependencies are `oneagent-llm`, reqwest `0.13.4`, serde `1.0.228`, serde_json `1.0.150`, and Tokio `1.53.0`. Reqwest defaults are disabled and only `rustls` is enabled; Tokio production features are only `macros` and `time`. | pass |
| Construction and client policy | Focused construction tests prove provider identity, URL byte/scheme/host/port/root rules, endpoint joining, no construction I/O, optional/invalid bearer behavior, redaction, no proxy, no redirect, and the static user agent. | pass |
| Discovery | Every call performs a fresh bounded `GET /v1/models`; empty, maximum, reordered, unknown-field, missing, malformed, invalid, duplicate, over-count, status, redirect, partial, body-bound, transport, timeout, and cancellation cases have explicit terminal oracles. | pass |
| Generation | One bounded non-streaming `POST /v1/completions` sends exactly `model`, `prompt`, `max_tokens`, and `stream=false`; Unicode, maximum escaping, repeated calls, local byte usage, and `stop`/`length` mapping pass. | pass |
| Identity and fallback rejection | Provider mismatch fails before I/O. Response model mismatch, zero/multiple choices, nonzero index, unknown/null finish, empty output, and over-bound output are rejected atomically. | pass |
| Status, protocol, and transport | 408/429/5xx availability, other status rejection, malformed/missing/mistyped/trailing JSON, partial bodies, advertised and streamed body bounds, and closed-loopback transport failures map to the accepted error kinds. | pass |
| Retry, redirect, proxy, and cache absence | Controlled servers observe one exact request per terminal case; redirects are terminal, operations are fresh, and code inspection finds no retry, fallback, cache, refresh, proxy, or background operation owner. | pass |
| Redaction | Synthetic URL, credential, prompt, response, header, and provider-body sentinels are absent from `Display`, `Debug`, and diagnostics. Provider/Serde/reqwest sources and bodies are not attached to public errors. | pass |
| Timeout, cancellation, and cleanup | Pre-cancellation and in-flight cancellation are typed; total timeout covers pending work; biased precedence is explicit; losing futures are dropped; deterministic server joins prove completion without surviving adapter work. | pass |
| Public conformance | Six non-zero integration tests use only exported adapter and `oneagent-llm` APIs with controlled `127.0.0.1:0` servers, synthetic fixtures, no credentials, no external network, no environment inputs, and no ignored tests. | pass |
| Consumer compatibility | `oneagent-analysis` passes 27 unit and 11 public tests; `oneagent-runtime --lib` passes 78 tests. Neither consumer behavior nor dependency ownership changed. | pass |
| Documentation and exclusions | README, Architecture, Semantic Model, Roadmap, investigation, and ADR-0046 agree on the bounded first slice. Runtime registration, configuration sources, chat/Responses APIs, streaming, tools, prompt policy, additional providers, MCP, IDE, and live acceptance remain deferred. | pass |

## Findings

### Non-blocking

1. `adapters/openai-compatible/src/lib.rs` still says discovery and generation
   remain to be implemented. Both operations are implemented and validated.
   Correct the crate-level Rustdoc in a future implementation/documentation
   task; review policy does not permit changing a production file here.

### Blocking

None.

## Missing evidence

None blocking. Review validation deliberately did not reconnect to the live
llama.cpp host: ADR-0046 makes the pinned and sanitized investigation record
supplementary evidence and requires repository acceptance to remain controlled
loopback only.

## Focused validation

- `cargo test -p oneagent-openai-compatible --lib -- --list` — 18 tests listed.
- `cargo test -p oneagent-openai-compatible --lib --offline` — 18 passed.
- `cargo test -p oneagent-openai-compatible --test conformance -- --list` — 6 tests listed.
- `cargo test -p oneagent-openai-compatible --test conformance --offline` — 6 passed.
- `cargo test -p oneagent-llm --offline` — 22 unit and 7 public tests passed.
- `cargo test -p oneagent-analysis --offline` — 27 unit and 11 public tests passed.
- `cargo test -p oneagent-runtime --lib --offline` — 78 passed.
- `cargo tree -p oneagent-openai-compatible --depth 1 --edges normal --offline` — exact approved direct normal dependency set.
- `cargo tree -p oneagent-llm --depth 1 --edges normal --offline` — std-only package.
- `cargo tree -i oneagent-llm --edges normal --offline` — only `oneagent-openai-compatible` depends on it.

No focused filter or named target matched zero tests.

## Complete validation

The canonical complete workspace gate passed on the committed Task 6 review
head with sandbox permission limited to required local loopback binds:

- `cargo fmt --all -- --check`
- `cargo check --workspace --offline`
- `cargo test --workspace --offline`
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --offline`
- `git diff --check`

No external network, live provider, credential, environment configuration, or
ignored artifact was used by review validation.

## Scope and exclusion conformance

The reviewed range creates one leaf adapter and its deterministic evidence. It
does not change graph facts, Context Engine semantics, analysis, Runtime,
protocol, CLI, source adapters, persistence, Coverage Registries, MCP, LSP,
IDE, or configuration ownership. Chat/Responses APIs, streaming, tools,
structured output, provider usage authority, retry/backoff, catalog caching,
model selection, additional providers, and live compatibility claims remain
excluded.

## Risk assessment

Residual risk is bounded to the accepted first slice: HTTP is explicitly
caller-selected for local endpoints, HTTPS uses platform roots, provider token
cost is not equivalent to the local byte ceiling, cancellation is cooperative,
and credentials may have unavoidable transport-buffer copies. The review makes
no broad compatibility, performance, cost, security, or live-availability
claim. None of these accepted limits blocks completion.

## Previous-suite retirement

Before deletion, filesystem and tracked inventories each contained exactly the
eight authorized Sprint 23 prompt files and the untracked inventory was empty
(`8/8/0`). The suite is retired atomically with this review. The complete Sprint
24 suite, `docs/codex/prompts/run-next-sprint.md`, non-adjacent suites, and
`.codex/` remain unchanged.

## State transition and next action

Sprint 24 transitions from `next` to `completed`. Sprint 25 LM Studio
Integration becomes the unique `next` planning target. The next implementation
task should follow the Sprint 25 plan when created; the Rustdoc wording follow-up
may be handled separately without reopening ADR-0046 or Sprint 24.
