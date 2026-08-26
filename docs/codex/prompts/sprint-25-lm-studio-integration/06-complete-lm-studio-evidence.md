# Complete Sprint 25 LM Studio Provider Evidence

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/llm-provider-implementation.md`

## Template

`docs/codex/templates/llm-provider-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 25 execution plan
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0046-openai-compatible-provider.md`
- `docs/adr/0047-lm-studio-integration.md`
- `docs/architecture/lm-studio-integration-investigation.md`
- `docs/reviews/sprint-24-openai-compatible-provider.md`

## Prerequisites / Required gate

Require committed Task 5 generation, clean task-owned state, all focused LM
Studio tests passing, and all existing generic/provider-neutral regression
targets passing.

## Task

Complete Sprint 25 acceptance evidence: add a non-zero public LM Studio provider
conformance target using only exported APIs and controlled loopback, prove the
complete ADR-0047 matrix and ADR-0045 substitution, rerun generic adapter and
consumer compatibility, audit dependencies/redaction/no-live-state ownership,
and synchronize truthful current-state documentation.

## Provider-neutral ownership and API boundary

Exercise the LM Studio provider through `&dyn LlmProvider` and public
`oneagent-llm` values. Prove it is a leaf adapter and that provider-neutral,
Context, Runtime, protocol, CLI, graph, workspace, and source ownership remains
unchanged.

## Model identity, discovery, and capability contract

The public target must prove stable `lm-studio` identity, mixed LLM/embedding
filtering, canonical discovery ordering, accepted loaded/downloaded behavior,
fresh repeated calls, and exact invalid/ambiguous catalog failures without
exposing provider metadata in shared values.

## Request, response, usage, finish, and compatibility contract

Prove one exact non-streaming text request/response, identity preservation,
normal/output-limit finish, local UTF-8 byte usage, bounds, provider-addition
handling, and exact malformed/mismatch/unsupported terminal failures.

## Configuration and secret-handling contract

Prove explicit deterministic construction, optional synthetic authentication,
no implicit environment/file/CLI inputs, no redirect/proxy fallback, and
sentinel absence from all implicit formatting and diagnostics.

## Timeout, retry, cancellation, and cleanup contract

Prove existing and in-flight cancellation, total timeout, simultaneous-ready
precedence where applicable, exactly one request, no retry/fallback/cache/
background task, deterministic server joins, and zero surviving adapter state.

## Error taxonomy and provider mapping

Cover every ADR-0047 construction, status, transport, protocol, catalog,
request, response, timeout, cancellation, and internal class applicable to the
public boundary with bounded redacted diagnostics.

## Contract oracle

Use only repository-owned synthetic fixtures and controlled `127.0.0.1:0`
servers with explicit readiness and joins. Do not access installed LM Studio,
downloaded models, local configuration, credentials, external network, sleeps,
or ignored artifacts. Live planning observations remain supplementary only.

## Consumer and provider-adapter compatibility

Run and record non-zero public/focused targets for LM Studio, the complete
OpenAI-compatible adapter, `oneagent-llm`, `oneagent-analysis`, and affected
Runtime library behavior. Audit reverse dependencies and direct feature sets.

## Scope

### Included

- Public LM Studio conformance target and any repository-owned synthetic
  fixture/helpers required by it.
- Small test-only or production corrections required to close an observed
  acceptance gap, with the exact reason reported.
- Truthful implemented-state updates only in `README.md`,
  `docs/Architecture.md`, and `docs/architecture/semantic-model-2.md`.

### Excluded

- Live-provider tests or claims, model/server lifecycle, quality/performance,
  Runtime registration, configuration sources, chat/streaming/tools/MCP,
  prompt policy, Sprint completion, review artifact, and prompt retirement.

## Acceptance Criteria

- A non-zero public target proves the complete supported LM Studio contract
  through exported APIs and deterministic controlled loopback.
- Embedding false positives, malformed catalogs, identity fallback, bounds,
  redaction, timeout, cancellation, one-attempt behavior, and cleanup have exact
  public or focused evidence.
- Existing generic adapter and provider-neutral public contracts remain intact.
- Analysis and Runtime compatibility targets pass and dependency direction is
  unchanged.
- Documentation states only implemented behavior and preserves every deferral.
- No acceptance result depends on live LM Studio or developer-local state.

## Repository Safety

State exact test, fixture/helper, and documentation files before editing.
Preserve prompts, Roadmap state, review files, `.codex/`, local LM Studio state,
and unrelated paths. Do not contact a live provider.

## Task-specific Validation

- List and run every non-zero LM Studio unit and public conformance target.
- `cargo test -p oneagent-openai-compatible --lib --offline`
- `cargo test -p oneagent-openai-compatible --test conformance --offline`
- `cargo test -p oneagent-llm --offline`
- `cargo test -p oneagent-analysis --offline`
- `cargo test -p oneagent-runtime --lib --offline`
- Exact direct/reverse dependency and feature audit.
- Search repository tests/docs for live host, credential, local path, ignored
  fixture, retry/fallback, and sensitive sentinel leakage.
- Canonical full workspace validation from `docs/codex/core/validation.md`.

## Suggested commit message

`Complete Sprint 25 LM Studio evidence`

## Final report additions

Report public conformance counts/results, exact contract matrix, compatibility
and dependency audits, current-state docs, preserved deferrals, focused/full
validation, commit, and final Git state.
