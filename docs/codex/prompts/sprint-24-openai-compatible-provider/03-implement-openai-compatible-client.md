# Implement Sprint 24 OpenAI-Compatible Client Foundation

Continue OneAgent development.

## Reporting

- Prompt and repository content: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/llm-provider-implementation.md`

## Template

`docs/codex/templates/llm-provider-task.md`

## Authoritative documents

- `docs/Roadmap.md`, Sprint 24 execution plan
- `docs/architecture/openai-compatible-provider-investigation.md`
- `docs/adr/0045-llm-provider-abstraction.md`
- `docs/adr/0046-openai-compatible-provider.md`

## Prerequisites / Required gate

Require committed ADR-0046, clean task-owned state, and explicit current-user
approval for every selected production dependency and feature. Stop before
Cargo changes when approval is absent or narrower than the ADR set.

## Task

Implement only the concrete OpenAI-compatible adapter client foundation:
workspace/crate ownership, approved dependencies, validated construction,
secret-safe bounded HTTP client policy, and exact wire value definitions. Do
not perform discovery or generation I/O.

## Provider-neutral ownership and API boundary

Depend inward on `oneagent-llm`; do not add wire fields to its public types.
Expose only the construction and provider identity surface accepted by
ADR-0046. Keep Runtime, Context Engine, configuration sources, and global state
outside the adapter.

## Configuration and secret-handling contract

Implement exact base-URL scheme/credential/query/fragment/path validation and
normalization, optional bearer handling, redirect/proxy policy, bounded client
configuration, and Debug/error redaction. Never retain or expose an
unrestricted URL, header, secret, prompt, response, or provider body.

## Wire and execution foundation

Define the accepted discovery/completion request and response wire values,
field names, defaults/omissions, unknown-field policy, bounds, and private
helpers without issuing HTTP requests. Client construction must not access
environment/file credential sources or perform network I/O.

## Scope

### Included

Root workspace membership, new adapter manifest and source modules, mechanically
required `Cargo.lock`, Rustdoc, focused construction/URL/wire/redaction tests,
and only the approved dependencies/features.

### Excluded

Discovery/generation HTTP calls, `LlmProvider` behavior beyond construction,
live services, Runtime/Context/protocol/CLI changes, chat/Responses APIs,
streaming, tools, retry, configuration loading, public conformance docs, sprint
transition, and prompt retirement.

## Acceptance Criteria

- Construction is deterministic, performs no I/O, and enforces ADR-0046 exactly.
- Client policy disables every implicit behavior rejected by the ADR.
- Credentials and sensitive URL components cannot enter Debug, Display, errors,
  diagnostics, fixtures, or snapshots.
- Wire values encode/decode only accepted fields and bounds.
- No unapproved dependency or feature enters the workspace.
- Focused tests are non-zero and the complete workspace gate passes.

## Repository Safety

Modify only root Cargo files and the new adapter crate paths accepted by
ADR-0046. Preserve `oneagent-llm`, other adapters, Runtime, analysis, protocol,
docs, prompts, ignored artifacts, `.codex/`, and unrelated paths. Do not access
external services.

## Task-specific Validation

- List and run non-zero construction, URL, client-policy, wire, and redaction
  tests.
- Run the complete new adapter package tests.
- Audit `cargo tree` for exact dependencies/features.
- Run the canonical complete workspace validation.
- Verify diff scope and `git status --short`.

## Suggested commit message

`Implement Sprint 24 OpenAI-compatible client`

## Final report additions

Report dependency approval and exact tree, crate/API ownership, construction,
URL/auth/client policy, wire values, redaction evidence, focused/full
validation, changed paths, commit, and final Git state.
