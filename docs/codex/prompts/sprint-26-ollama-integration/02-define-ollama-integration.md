# Define the Ollama Provider Contract

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/architecture.md`

## Template

`docs/codex/templates/architecture-task.md`

## Authoritative ADRs and architecture documents

- `docs/Roadmap.md`, Sprint 26 execution plan
- `docs/architecture/ollama-integration-investigation.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0046-openai-compatible-provider.md`
- `docs/adr/0047-lm-studio-integration.md`
- `docs/architecture/semantic-model-2.md`

## Prerequisites / Required gate

The committed Task 1 investigation contains sufficient official, repository,
sanitized local, dependency, consumer, and deterministic-oracle evidence. Stop
if its unresolved questions cannot be decided safely.

## Task

Create and accept `docs/adr/0048-ollama-integration.md` for the smallest bounded
Ollama leaf behind `LlmProvider`.

Decide exact ownership and dependency direction, public surface, stable
provider identity, explicit and numeric-loopback construction, locality and
authentication contract, client policy, discovery endpoint and capability
mapping, generation endpoint and exact wire mapping, identity validation,
request/response/body bounds, status/protocol/error precedence, redaction,
timeout/cancellation races, one-attempt cleanup, deterministic conformance,
consumer compatibility, implementation prerequisites, and deferred scope.

Choose composition, bounded reuse, or independent native transport only from
Task 1 evidence. Do not weaken ADR-0045, ADR-0046, or ADR-0047. Identify the
exact direct dependency/features block, and state whether it introduces any new
repository production dependency or feature that requires explicit approval.

## Scope

### Included

- One accepted ADR with alternatives, decision, rejected alternatives,
  prerequisites, risks, conformance, and deferrals.

### Excluded

- Rust, Cargo, lockfile, current-state documentation, live provider calls,
  implementation, or support claims.

## Acceptance Criteria

- ADR-0048 is implementable from confirmed evidence and closes every Task 1
  decision required by the first slice.
- Provider-neutral values remain unchanged and provider metadata stays private.
- Discovery never assigns `TextGeneration` without exact accepted evidence.
- Local/cloud behavior, authentication, identity, bounds, failures, timeout,
  cancellation, retry/fallback, and cleanup are explicit.
- Acceptance uses synthetic fixtures and controlled loopback only.
- Any new dependency/feature approval gate is exact and enforceable before
  Task 3.
- Architecture acceptance alone does not claim Ollama support.

## Repository Safety

Modify only ADR-0048. Preserve `.codex/`, Roadmap, prompts, investigation,
existing ADRs, code, manifests, lockfile, and unrelated files.

## Task-specific Validation

- Audit ADR-0048 against every investigation question and inherited ADR.
- Verify links, dependency direction, decision/rejection consistency, and
  accepted-versus-deferred scope.
- Run `git diff --check`.

## Suggested commit message

`Define Sprint 26 Ollama integration`

## Final report additions

Report the accepted boundary, dependency approval result, rejected alternatives,
deferred scope, exact created path, validation, commit hash, and final Git state.
