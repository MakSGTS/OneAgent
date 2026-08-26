# Investigate the Ollama Integration Boundary

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative ADRs and architecture documents

- `docs/Roadmap.md`, Sprint 26 execution plan
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0046-openai-compatible-provider.md`
- `docs/adr/0047-lm-studio-integration.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-25-lm-studio-integration.md`

## Prerequisites / Required gate

- The Sprint 26 planning baseline is committed.
- The working tree contains no conflicting task-created change.
- Any live Ollama call has renewed current user authorization.
- Local observations remain supplementary and sanitized; do not start/stop the
  daemon, mutate models, send generation, use credentials, or contact a cloud
  model.

## Task

Create `docs/architecture/ollama-integration-investigation.md` with the exact
evidence needed for ADR-0048 and deterministic implementation planning.

Separate confirmed repository evidence, official Ollama documentation, mutable
sanitized local observations, accepted inherited contracts, and unresolved
architecture questions. Inventory:

- crate ownership, public `LlmProvider` values, consumers, dependencies, and
  compatible private transport seams;
- native and compatibility discovery/generation endpoints and exact bounded
  request/response fields relevant to the provider-neutral first slice;
- model identity, text-generation capability evidence, unknown capability
  handling, local versus cloud entries, and empty/duplicate/reordered cases;
- explicit/local construction, authentication behavior, status/protocol/error
  vocabulary, body/output bounds, redaction, timeout, cancellation, attempts,
  and cleanup;
- deterministic synthetic fixtures and controlled-loopback positive, negative,
  malformed, partial, ambiguous, maximum, repeated, and race oracles;
- exact decisions ADR-0048 must make and deferred scope.

Record official URLs and access date. Sanitize local evidence: retain only the
client/server version and structural catalog vocabulary needed for the decision;
do not retain digests, developer paths, server configuration, credentials,
unrestricted prompt/output, timing payloads, or availability claims.

## Scope

### Included

- Read-only repository and official-primary-source investigation.
- Authorized read-only local `/api/version` and `/api/tags` observations when
  still available and needed.
- One evidence document.

### Excluded

- ADR acceptance, Rust/Cargo changes, production behavior, daemon/model
  mutation, generation, cloud traffic, downloads, and response-quality claims.

## Acceptance Criteria

- Every retained field or behavior has a repository, official, or explicitly
  classified local source.
- The document resolves enough evidence to choose discovery and generation
  boundaries without inventing wire behavior.
- Capability mapping cannot silently advertise an unsupported text model.
- The proposed acceptance oracle requires no installed/running Ollama, model,
  credential, developer state, cloud service, or external network.
- Unknowns are explicit ADR questions or a blocker rather than assumptions.
- No sensitive or mutable local payload prohibited above is retained.

## Repository Safety

Modify only the investigation document. Preserve `.codex/`, Roadmap, prompt
suites, ADRs, code, manifests, lockfile, and unrelated files.

## Task-specific Validation

- Verify every cited repository path and official URL.
- Audit the document for prohibited local payload and unsupported claims.
- Verify every required investigation category is present.
- Run `git diff --check`.

## Suggested commit message

`Investigate Sprint 26 Ollama integration`

## Final report additions

Report evidence categories, local-call scope, unresolved ADR decisions,
sanitization result, exact created path, validation, commit hash, and final Git
state.
