# Investigate Sprint 24 OpenAI-Compatible Provider

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 24 execution plan
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/architecture/llm-provider-investigation.md`
- `docs/reviews/sprint-23-llm-provider-abstraction.md`
- `docs/codex/workflows/llm-provider.md`
- pinned llama.cpp source commit and sanitized live observations recorded by the
  current Sprint 24 launch investigation

## Prerequisites / Required gate

Require the committed Sprint 24 planning baseline, clean task-owned state, and
current user authorization before any renewed access to `192.168.0.176`.
Existing verified observations may be documented without another live call.

## Investigation objective

Create `docs/architecture/openai-compatible-provider-investigation.md` with the
complete evidence needed for ADR-0046. Separate repository facts, pinned
llama.cpp facts, sanitized live observations, accepted ADR-0045 constraints,
implementation choices, and unresolved decisions.

## Questions to answer

- Which crate owns the concrete adapter and what dependency direction is valid?
- Which exact production dependencies are required and which need user approval?
- What base-URL, scheme, normalization, redirect, proxy, TLS, and credential
  behavior is implementable without implicit configuration?
- What exact `/v1/models` and `/v1/completions` request/response fields are
  required, optional, ignored, rejected, or bounded?
- How do model IDs, capabilities, response identity, choice count/index,
  `stop`/`length`, output bytes, token ceiling, unknown fields, and provider
  usage map to ADR-0045?
- How do HTTP status, malformed JSON, missing fields, error payloads, transport,
  timeout, cancellation, response-size limits, cleanup, and redaction map?
- Which deterministic fixtures and controlled-loopback cases prove the adapter
  without live credentials or network?
- Which Context, Runtime, protocol, and provider-neutral consumers remain
  unchanged?

## Evidence scope

Inspect current `oneagent-llm`, workspace dependencies, consumers, relevant
HTTP/testing patterns, pinned llama.cpp build 10485 commit
`1511ce3bc3f087376c8526b4ad07100bfabb277f`, and the exact sanitized live
observations already gathered for health, discovery, successful completion,
malformed JSON, missing prompt, and unknown-model fallback.

Record source paths and versions, not unrestricted source copies. Do not record
credentials, system configuration, private model content, unrestricted prompt
or completion text, timings, logs, or personal paths.

## Excluded

ADR acceptance, Rust/Cargo changes, dependency installation, production or test
implementation, Runtime/configuration integration, chat/Responses APIs,
streaming, tools, prompt policy, retry, live-provider acceptance, docs
synchronization, sprint transition, and prompt retirement.

## Completion Criteria

- The evidence document closes every investigation question with confirmed,
  accepted, unresolved, or deferred status.
- Exact positive and negative wire shapes and the live model-identity mismatch
  are recorded in bounded sanitized form.
- The dependency candidates and approval requirement are explicit.
- A complete deterministic loopback oracle and affected-consumer inventory are
  defined.
- ADR-0046 can decide the first slice without inventing fields or behavior.

## Repository Safety

Create only the investigation document. Do not inspect parent/sibling projects
or external paths not explicitly authorized by the current user. Preserve Rust,
Cargo, prompts, Roadmap, ignored artifacts, `.codex/`, and unrelated paths.

## Task-specific Validation

- Recheck exact local definitions, consumers, dependencies, and source citations.
- Validate document links and field/value agreement with recorded evidence.
- `git diff --check`
- Verify diff scope and `git status --short`.

## Suggested commit message

`Investigate Sprint 24 OpenAI-compatible provider`

## Final report additions

Report confirmed repository/pinned/live findings, accepted constraints,
dependency candidates, mapping and error evidence, unresolved decisions,
oracle design, external access performed, validation, commit, and final Git
state.
