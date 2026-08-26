# Implement Ollama Text Generation

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/llm-provider-implementation.md`

## Template

`docs/codex/templates/llm-provider-task.md`

## Authoritative ADRs and architecture documents

- `docs/adr/0048-ollama-integration.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/architecture/ollama-integration-investigation.md`
- `docs/architecture/semantic-model-2.md`

## Prerequisites / Required gate

The committed Task 4 discovery implementation exactly matches ADR-0048 and
leaves no uncommitted task-created change.

## Task

Implement only the accepted non-streaming Ollama text-generation path. Preserve
the exact validated request provider/model identity, input bytes, and output
bound; issue one accepted wire request; validate one terminal response; map only
accepted finish states; and construct `TextGenerationResponse` against the
originating request with local UTF-8 byte usage.

Cover provider/model mismatch, malformed or partial responses, response-model
mismatch, unsupported terminal states, empty and over-bound output, status and
body bounds, transport, timeout, cancellation races, one attempt, no fallback,
repeated calls, redaction, and complete cleanup as ADR-0048 requires.

## Scope

### Included

- Private generation wire mapping, `LlmProvider::generate`, and focused tests.

### Excluded

- Streaming, chat/history, tools, structured output, thinking/reasoning, vision,
  embeddings, prompt templates, keep-alive/model lifecycle ownership, retry,
  fallback, public conformance target, docs, and Runtime composition.

## Acceptance Criteria

- One request yields one valid terminal response or one typed failure with no
  partial success.
- Exact request identity survives success; provider fallback or response model
  mismatch is rejected.
- Output and body bounds, finish mapping, redaction, timeout/cancellation
  precedence, one-attempt behavior, and cleanup match ADR-0048.
- Existing discovery, provider-neutral, and concrete-provider contracts remain
  unchanged.
- No test or completion claim uses live Ollama, a model, credential, cloud
  traffic, output quality, or external network.

## Repository Safety

Modify only generation-owned files and necessary local exports within the
accepted Ollama package. Preserve `.codex/`, Roadmap, prompt suites, shared LLM,
existing providers, consumers, and unrelated files.

## Task-specific Validation

- List and run non-zero focused generation tests.
- Run complete Ollama unit tests plus relevant existing concrete-provider and
  provider-neutral regressions.
- Audit exact wire, terminal matrix, identity, bounds, redaction, no retry/
  fallback, timeout/cancellation, repetition, and cleanup.
- Run the canonical full workspace validation.

## Suggested commit message

`Implement Sprint 26 Ollama generation`

## Final report additions

Report exact generation wire/finish mapping, focused test matrix/count,
validation, modified paths, commit hash, and final Git state.
