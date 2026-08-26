# Sprint 25 LM Studio Integration Review

## Decision

`pass`

Sprint 25 satisfies the accepted ADR-0047 bounded provider slice and completion
gate. The implementation is additive, native discovery cannot advertise an
embedding-only entry as text generation, generation preserves the original LM
Studio identity through one unchanged generic adapter attempt, all focused and
complete validation passes, and no blocking or non-blocking defect was found.

## Reviewed baseline

- Planning baseline: `e6fb8e4c90ddb78e13e6991ecb170507648761a3`.
- Planning commit: `da5cdbd58b351b77f5ea7bc8075075edad61024c`.
- Review head: `1498111334de7fd8fd1f85f14656235f6540a33b`.
- Reviewed changes: planning through committed Task 6, relative to the planning
  baseline.
- Initial review Git status: clean.
- Platform: `aarch64-apple-darwin`.
- Toolchain: Rust `1.97.1` (`8bab26f4f`, 2026-07-14), Cargo `1.97.1`.

The dependency-ordered Sprint 25 commits are:

| Step | Commit | Result |
| --- | --- | --- |
| Planning | `da5cdbd5` | pass |
| Investigation | `552c2862` | pass |
| ADR-0047 | `26b33e4e` | pass |
| Commit-policy clarification | `a5731d28` | pass |
| Client foundation | `0e97b563` | pass |
| Native model discovery | `a1c1679a` | pass |
| Composed text generation | `9c17433c` | pass |
| Public evidence and current-state docs | `14981113` | pass |

The user explicitly approved the exact ADR-0047 normal and dev dependency
block before Task 3. The separate commit-policy clarification records the
user's standing instruction that every successfully validated Sprint 25 task
must create its own commit; subsequent tasks follow that requirement.

The reviewed range changes only the workspace/member and lock registration, the
new `oneagent-lm-studio` leaf and its tests, ADR-0047 and investigation, Sprint
25 planning prompts, Roadmap planning, and the three authorized current-state
documents. It does not change the generic adapter, shared LLM crate, Analysis,
Runtime, protocol, CLI, graph, workspace, or source-adapter implementation.

## Acceptance evidence matrix

| Criterion | Evidence | Result |
| --- | --- | --- |
| Planning and prerequisites | The plan orders Tasks 1-7, identifies exact gates and exclusions, and Tasks 1-6 plus the user-directed commit clarification are committed in dependency order from a clean baseline. | pass |
| Investigation | The investigation separates repository facts, official LM Studio documentation, sanitized authorized local observations, and deterministic acceptance evidence. Live observations remain supplementary and are not a test oracle. | pass |
| Accepted architecture | ADR-0047 fixes ownership, exact approved dependencies, construction, locality, auth, native discovery, composed generation, identity, bounds, failures, timeout/cancellation, cleanup, compatibility, and deferred scope. | pass |
| Ownership and public API | `oneagent-lm-studio` exports only `LmStudioProvider` with explicit `new`, `new_local`, `id`, and `LlmProvider`; clients, URLs, headers, DTOs, execution helpers, and the composed provider remain private. | pass |
| Dependency direction | The leaf depends on `oneagent-openai-compatible` and `oneagent-llm`; the generic adapter depends only on `oneagent-llm`; the shared crate remains std-only. Reverse inspection finds no LM Studio consumer and no Analysis or Runtime dependency. | pass |
| Exact dependencies and features | Direct normal dependencies are `oneagent-llm`, `oneagent-openai-compatible`, reqwest `0.13.4` with defaults disabled and `rustls`, serde `1.0.228` with `derive`, serde_json `1.0.150`, and Tokio `1.53.0` with `macros,time`; dev Tokio adds only the accepted I/O/runtime/sync features. | pass |
| Construction and locality | Focused and public tests prove exact `lm-studio` identity, explicit root validation, provider-mismatch precedence, no construction I/O, optional auth, invalid-header redaction, and the sole `http://127.0.0.1:1234` numeric-loopback default. | pass |
| Client policy and authentication | Native discovery uses no redirects or proxy, static LM Studio user agent, exact optional bearer auth, and bounded byte-before-JSON decoding. Generation retains the unchanged generic client policy and user agent. | pass |
| Native discovery | Each call sends one fresh `GET /api/v1/models`; mixed fixtures retain only loaded `llm` instance IDs, ignore embedding and unloaded entries, sort canonically, accept additions, and expose no LM Studio metadata. | pass |
| Catalog failures | Missing/mistyped/malformed values, unknown types, invalid or duplicate IDs, over-count results, statuses, redirects, advertised/streamed bounds, partial bodies, and transport failures are atomic and typed. | pass |
| Generation wire and identity | One non-streaming `/v1/completions` request sends exactly model, prompt, max tokens, and `stream=false`; success is rebound to the original `lm-studio` provider/model identity and local UTF-8 byte usage. | pass |
| Terminal mapping and fallback rejection | Only `stop` and `length` map to `Completed` and `OutputLimit`; model fallback, zero/multiple choices, nonzero index, unsupported/null finish, empty/over-bound output, malformed responses, status, redirect, partial, and body-bound failures are rejected. | pass |
| Timeout, cancellation, attempt, and cleanup | Existing and in-flight cancellation, total timeout, biased simultaneous-ready precedence, one request per terminal case, losing-future drop, deterministic server joins, fresh repetition, and no retained adapter work have exact tests. | pass |
| Redaction | Synthetic URL, credential, header, discovery body, prompt, completion body, and output sentinels are absent from implicit provider/error formatting and diagnostics. Provider and transport bodies or library sources are not attached to public errors. | pass |
| Public substitution | Six public tests use `&dyn LlmProvider` and exported provider-neutral values for exact construction, discovery, generation, failures, timeout, cancellation, redaction, repeated calls, and cleanup. | pass |
| Generic compatibility | The unchanged OpenAI-compatible adapter passes 18 unit and 6 public tests, including exact wires, bounds, errors, timeout/cancellation, redaction, one attempt, and cleanup. | pass |
| Provider-neutral and consumer compatibility | `oneagent-llm` passes 22 unit and 7 public tests; Analysis passes 27 unit and 11 public tests; Runtime library passes 78 tests. Their implementation and ownership remain unchanged. | pass |
| Deterministic oracle | Acceptance uses repository-owned synthetic fixtures and controlled `127.0.0.1:0` servers with explicit joins. There are no ignored tests, environment/credential inputs, developer paths, installed LM Studio calls, model downloads, or external network. | pass |
| Documentation and exclusions | README, Architecture, Semantic Model, Roadmap planning, investigation, and ADR-0047 describe only the implemented bounded leaf. Runtime integration, configuration sources, model/server lifecycle, live compatibility, chat/templates, streaming, tools, MCP, IDE, quality, and performance remain deferred. | pass |

## Findings

### Blocking

None.

### Non-blocking

None.

## Missing evidence

None for the accepted ADR-0047 boundary. Review validation deliberately does
not contact installed or running LM Studio, use a downloaded model or
credential, or judge response quality. ADR-0047 defines those as unsupported
claims rather than completion evidence.

## Focused validation

- `cargo test -p oneagent-lm-studio --lib --offline -- --list` — 19 tests
  listed.
- `cargo test -p oneagent-lm-studio --test conformance --offline -- --list` —
  6 tests listed.
- `cargo test -p oneagent-lm-studio --lib --offline` — 19 passed.
- `cargo test -p oneagent-lm-studio --test conformance --offline` — 6 passed.
- `cargo test -p oneagent-openai-compatible --lib --offline` — 18 passed.
- `cargo test -p oneagent-openai-compatible --test conformance --offline` — 6
  passed.
- `cargo test -p oneagent-llm --offline` — 22 unit and 7 public tests passed.
- `cargo test -p oneagent-analysis --offline` — 27 unit and 11 public tests
  passed.
- `cargo test -p oneagent-runtime --lib --offline` — 78 passed.

No selected target or filter matched zero tests.

## Dependency, scope, and sensitive-state audits

- `cargo tree -p oneagent-lm-studio --depth 1 --edges normal --offline` matches
  the exact approved direct dependency set.
- `cargo tree -p oneagent-lm-studio --depth 2 --edges features --offline`
  matches the accepted normal and dev feature ownership.
- Reverse normal trees show no consumer of `oneagent-lm-studio`, only the LM
  Studio leaf above `oneagent-openai-compatible`, and only both concrete leaves
  above std-only `oneagent-llm`.
- Public-surface inspection finds only `LmStudioProvider`, `new`, `new_local`,
  `id`, and its `LlmProvider` implementation.
- Range/path inspection finds no implementation change outside the new leaf;
  generic/provider-neutral/consumer packages are unchanged.
- Redaction, no-live-state, ignored-test, local-path, credential-input,
  retry/fallback/cache/background-state, prompt-inventory, and Markdown/link
  audits found no blocking or non-blocking finding.

## Complete validation

The canonical complete workspace gate passed on committed Task 6 review head
`1498111334de7fd8fd1f85f14656235f6540a33b` with sandbox permission limited to
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

The reviewed range creates one leaf adapter and deterministic evidence. It does
not change graph facts, Context Engine semantics, Analysis, Runtime, protocol,
CLI, workspace, source adapters, persistence, Coverage Registries, MCP, LSP,
IDE, or configuration ownership. Chat/templates, streaming, tools, embeddings,
structured output, prompt orchestration, retry/backoff, catalog caching, model
selection and lifecycle, live compatibility, and Sprint 26 implementation
remain excluded.

## Risk assessment

Residual risk is bounded to the accepted first slice: `/v1/completions` is a
legacy base-model endpoint and can be unsuitable for chat-tuned models; HTTP is
an explicit caller choice; HTTPS relies on platform verification; local byte
bounds are not provider token or cost authority; cancellation is cooperative;
and secret/input/output copies are not zeroized. The review makes no broad LM
Studio version, model quality, performance, cost, availability, or security
claim. None blocks the accepted provider boundary.

## Previous-suite retirement

Before deletion, filesystem and tracked inventories each contained exactly the
eight authorized Sprint 24 prompt files and the untracked inventory was empty
(`8/8/0`). The suite is retired atomically with this review. The complete
Sprint 25 suite, `docs/codex/prompts/run-next-sprint.md`, non-adjacent suites,
and `.codex/` remain unchanged.

## State transition and next action

Sprint 25 transitions from `next` to `completed`. Sprint 26 Ollama Integration
becomes the unique `next` planning target. Sprint 26 implementation has not
started; its next action is a separately committed dependency-ordered planning
task.
