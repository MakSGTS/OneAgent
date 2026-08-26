# Investigate Sprint 25 LM Studio Integration

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 25 execution plan
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0046-openai-compatible-provider.md`
- `docs/architecture/openai-compatible-provider-investigation.md`
- `docs/reviews/sprint-24-openai-compatible-provider.md`
- `docs/codex/workflows/llm-provider.md`
- official LM Studio REST, model-list, OpenAI-compatible model-list, legacy
  completions, authentication, server, and CLI documentation discovered from
  current `https://lmstudio.ai/docs/`
- sanitized local macOS observations recorded in the Sprint 25 plan

## Prerequisites / Required gate

Require the committed Sprint 25 planning baseline and clean task-owned state.
Treat recorded local observations as mutable context. Do not invoke `lms`,
start/stop LM Studio, inspect its local files, or contact its endpoint unless
the current user instruction explicitly authorizes that live local access.

## Investigation objective

Create `docs/architecture/lm-studio-integration-investigation.md` with the
complete evidence needed for ADR-0047. Separate repository facts, official LM
Studio facts, sanitized authorized local observations, accepted ADR-0045/0046
constraints, implementation choices, and unresolved decisions.

## Questions to answer

- Which package owns the LM Studio adapter and what dependency direction keeps
  `oneagent-llm`, Context, Runtime, and the generic adapter boundaries intact?
- Can the LM Studio provider safely compose over `OpenAiCompatibleProvider`, or
  does its hard-coded `openai-compatible` identity and `/v1/models` mapping
  require a minimal reusable seam or independent adapter?
- Which exact production dependencies/features are needed, which are already
  approved by ADR-0046, and which require new user approval?
- What stable provider ID, default/explicit server root, locality, HTTP/HTTPS,
  authentication, redirect, proxy, and configuration behavior is appropriate?
- Which LM Studio API generation/version boundary is supported, and which
  official version or compatibility claims can be made without guessing?
- How must `/api/v1/models` fields such as `models`, `type`, `key`, and
  `loaded_instances` map to ADR-0045 model identity and `TextGeneration` while
  excluding embedding models and avoiding unsupported metadata?
- Does discovery include every downloaded LLM or only loaded instances, and how
  do empty, duplicate, variant, multiple-instance, unknown-type, missing,
  malformed, reordered, and over-limit shapes behave?
- Which non-streaming LM Studio endpoint can represent ADR-0045's one raw text
  input without importing chat history, roles, prompt policy, state, streaming,
  reasoning, or tool semantics? Record the official legacy-completions warning
  and the observed chat-tuned output limitation.
- How do request/response identities, output limits, finish values, status,
  malformed/provider errors, body bounds, timeout, cancellation, cleanup,
  authentication, and redaction map?
- Which deterministic fixtures and controlled-loopback cases prove the adapter
  without installed LM Studio, local models, credentials, external network, or
  quality assertions?
- Which existing OpenAI-compatible, provider-neutral, Context, Runtime,
  protocol, and CLI consumers must remain unchanged?

## Evidence scope

Inspect current `oneagent-llm`, `oneagent-openai-compatible`, workspace
dependencies, reverse dependencies, public tests, consumers, relevant local
HTTP test harnesses, official LM Studio documentation, and only currently
authorized sanitized live observations.

The planning audit observed on macOS:

- `lms` CLI commit `71bd99c`, server root `http://127.0.0.1:1234`;
- loaded LLM `qwen/qwen3-4b` and available embedding model
  `text-embedding-nomic-embed-text-v1.5`;
- `/v1/models` listed both models without a type discriminator;
- `/api/v1/models` distinguished `type: "llm"` and `type: "embedding"` and
  exposed loaded instances;
- `/api/v0/models` provided a legacy typed list;
- `/v1/completions` returned a valid `text_completion` envelope, exact model,
  one choice, `finish_reason: "length"`, usage, and provider-specific `stats`;
- the output itself illustrated the official warning that legacy completions do
  not apply a prompt template to chat-tuned models.

Record bounded sanitized shapes only. Do not record credentials, unrestricted
prompt/output, timings, model paths, server settings, process data, or personal
filesystem paths. Live model quality and availability are never acceptance
evidence.

## Excluded

ADR acceptance, Rust/Cargo changes, dependency changes, production or test
implementation, Runtime/configuration integration, server/model lifecycle,
chat/Responses/streaming/tools/MCP/embeddings, docs synchronization, sprint
transition, and prompt retirement.

## Completion Criteria

- The evidence document closes every investigation question with confirmed,
  accepted, unresolved, or deferred status.
- It proves the generic discovery misclassification risk and records exact
  native type-aware source evidence.
- It resolves whether repository evidence is sufficient for ADR-0047 to choose
  composition, a reusable seam, or an independent adapter without weakening
  ADR-0046.
- Exact dependency candidates and approval requirements are explicit.
- The supported generation candidates and their semantic limitations are
  compared without selecting an unproven wire contract.
- A complete deterministic loopback oracle and affected-consumer inventory are
  defined.

## Repository Safety

Create only the investigation document. Preserve Rust, Cargo, prompts, Roadmap,
ignored artifacts, `.codex/`, and unrelated paths. Do not inspect LM Studio's
local files or another external filesystem path.

## Task-specific Validation

- Recheck exact definitions, consumers, reverse dependencies, tests, official
  citations, and any currently authorized sanitized wire observations.
- `cargo test -p oneagent-openai-compatible --lib --offline`
- `cargo test -p oneagent-openai-compatible --test conformance --offline`
- `cargo test -p oneagent-llm --offline`
- Validate document links and field/value agreement with recorded evidence.
- `git diff --check`
- Verify diff scope and `git status --short`.

## Suggested commit message

`Investigate Sprint 25 LM Studio integration`

## Final report additions

Report confirmed repository/official/local findings, accepted constraints,
dependency candidates, discovery and generation evidence, unresolved decisions,
oracle design, live access performed or avoided, validation, commit, and final
Git state.
