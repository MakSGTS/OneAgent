# Implement Ollama Model Discovery

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

The committed Task 3 foundation exactly matches ADR-0048 and leaves no
uncommitted task-created change.

## Task

Implement only fresh bounded Ollama model discovery. Send the exact accepted
wire request, parse only the required catalog/capability fields, preserve exact
valid model identities, assign `TextGeneration` only from accepted evidence,
and construct the result atomically through `ModelCatalog`.

Implement deterministic handling for empty and maximum catalogs, reordered
entries, unknown additions, local and cloud entries, missing/mistyped/malformed
fields, duplicate/invalid/over-count identities, ambiguous or unsupported
capabilities, statuses, redirects, partial/over-bound bodies, transport,
timeout, cancellation, repeated calls, and cleanup as ADR-0048 requires.

## Scope

### Included

- Private discovery wire mapping, operation implementation, and focused tests.

### Excluded

- Generation, shared-domain metadata expansion, catalog cache/refresh, model
  lifecycle, public conformance target, docs, and Runtime composition.

## Acceptance Criteria

- Every call is fresh, bounded, one-attempt, deterministic, and atomic.
- Only exactly accepted text-capable entries are advertised; unsupported or
  ambiguous entries follow ADR-0048 without silent capability inflation.
- Canonical ordering and exact provider/model identities are preserved.
- Error kinds, redaction, timeout/cancellation precedence, and cleanup match the
  accepted contract.
- No test uses live Ollama, a model, credential, cloud traffic, or external
  network.

## Repository Safety

Modify only discovery-owned files and necessary local exports within the
accepted Ollama package. Preserve `.codex/`, Roadmap, prompt suites, shared LLM,
existing concrete providers, consumers, and unrelated files.

## Task-specific Validation

- List and run non-zero focused discovery tests.
- Run provider-neutral tests and accepted foundation regressions.
- Audit exact wire, catalog matrix, no-live-state, redaction, one-attempt, and
  cleanup behavior.
- Run the canonical full workspace validation.

## Suggested commit message

`Implement Sprint 26 Ollama discovery`

## Final report additions

Report the discovery endpoint and capability rule, test matrix/count, validation,
modified paths, commit hash, and final Git state.
