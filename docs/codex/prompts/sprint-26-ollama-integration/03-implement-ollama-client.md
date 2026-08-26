# Implement the Ollama Client Foundation

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
- `docs/adr/0046-openai-compatible-provider.md`
- `docs/adr/0047-lm-studio-integration.md`
- `docs/architecture/ollama-integration-investigation.md`
- `docs/architecture/semantic-model-2.md`

## Prerequisites / Required gate

- ADR-0048 is accepted and committed.
- Explicit user approval exists for every new repository production dependency
  or feature identified by ADR-0048. Stop before edits if approval is required
  and absent.
- The working tree contains no conflicting task-created change.

## Task

Implement only the ADR-0048 Ollama client foundation: workspace/package
registration when required, the stable concrete provider type and identity,
validated explicit and numeric-loopback construction, accepted authentication
and locality policy, bounded HTTP client/execution foundation, private wire
types/helpers, and deterministic construction/redaction tests.

Do not implement a discovery or generation operation in this task. Reuse or
compose existing adapters only through the exact accepted dependency direction;
do not broaden their public surface or observable contracts.

## Scope

### Included

- Exact ADR-0048 foundation code, manifest/lock changes, and focused tests.

### Excluded

- Discovery, generation, public conformance target, current-state docs, Runtime
  integration, daemon/model lifecycle, and unrelated refactors.

## Acceptance Criteria

- Public surface, provider ID, constructors, client policy, authentication, and
  dependency graph exactly match ADR-0048.
- Construction performs no I/O and exposes no transport, URL, header, secret,
  or DTO state.
- Invalid configuration and sensitive sentinels map to typed static redacted
  failures with deterministic precedence.
- Existing provider-neutral and concrete-adapter contracts remain unchanged.
- Focused tests are non-zero and deterministic.

## Repository Safety

Enumerate exact task-owned paths from ADR-0048 before editing. Preserve
`.codex/`, Roadmap, prompt suites, unrelated code/docs, and existing providers
except for an explicitly accepted minimal reuse seam.

## Task-specific Validation

- List and run non-zero focused foundation tests.
- Audit direct dependencies/features, reverse dependencies, public surface,
  no-construction-I/O, and redaction.
- Run relevant existing provider-neutral/concrete-adapter regressions.
- Run the canonical full workspace validation from
  `docs/codex/core/validation.md`.

## Suggested commit message

`Implement Sprint 26 Ollama client`

## Final report additions

Report dependency approval evidence, exact dependencies/features, public
surface, focused test count, full validation, created/modified paths, commit
hash, and final Git state.
