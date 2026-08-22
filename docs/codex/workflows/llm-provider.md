# LLM Provider Workflow

Use this workflow for provider-independent LLM boundaries and concrete model
provider adapters.

## Provider-neutral boundary

- Identify the owner of provider configuration, client construction, requests,
  responses, capability discovery, and execution state.
- Define closed model, capability, request, response, usage, finish, and error
  contracts without embedding a provider wire schema in the shared domain.
- State which fields are required, optional, defaulted, rejected, preserved, or
  deliberately unsupported, including bounds and deterministic validation.
- Keep Context Engine output, prompt/tool policy, Runtime transport, and
  provider adapters in their accepted ownership layers.

## Capabilities and compatibility

- Define how a model identity is scoped to a provider and how available models
  and capabilities are discovered, cached, refreshed, ordered, and reported.
- Validate every request against the selected model capabilities before
  provider I/O and classify incompatible combinations explicitly.
- Define provider-neutral compatibility for text, streaming, usage, finish
  reasons, structured output, tools, and other features only when repository
  evidence and accepted scope support them.
- Preserve unknown provider additions at the adapter boundary or reject them
  explicitly; do not silently reinterpret them as accepted domain values.

## Configuration and secrets

- Inventory every configuration source and establish deterministic precedence,
  missing/invalid behavior, and whether discovery can run without credentials.
- Use typed secret-bearing inputs that avoid accidental copying, serialization,
  display, debug output, diagnostics, fixtures, snapshots, and source control.
- Redact provider URLs, headers, bodies, and errors wherever the accepted
  contract says they may contain secrets or sensitive user content.
- Do not require live credentials, remote services, or developer-local state for
  repository validation.

## Execution, timeout, retry, and cancellation

- Define connect, request, idle/stream, and total timeout ownership and
  distinguish timeout from provider, transport, protocol, and cancellation
  failures.
- Define retry eligibility, attempt limits, ordering, delay/backoff inputs, and
  request replay safety before implementation. Do not invent automatic retries.
- Propagate cancellation through every owned task, connection, stream, and
  retry wait; define the terminal outcome and cleanup evidence.
- Keep concurrency bounds, connection reuse, rate limiting, and shutdown under
  explicit owners and accepted policy.

## Error and response mapping

- Define one stable provider-neutral error taxonomy and the exact mapping from
  provider, transport, protocol, validation, timeout, cancellation, and
  compatibility failures.
- Preserve useful provider diagnostics only within accepted redaction and size
  bounds; never expose credentials or unrestricted response bodies.
- Define partial, empty, malformed, duplicated, reordered, interrupted-stream,
  and unknown terminal response behavior before implementing adapters.
- Keep successful response ordering and aggregation deterministic where the
  provider protocol permits it.

## Contract evidence

- Use repository-owned fake providers, fixtures, or controlled loopback servers
  with exact request and response oracles; live network calls are supplementary,
  never the only acceptance evidence.
- Cover supported, unsupported, missing, invalid, malformed, partial, duplicate,
  reordered, timeout, retryable, non-retryable, cancelled, redacted, and
  repeated cases as applicable.
- Prove that every supported adapter satisfies the same provider-neutral
  contract and that provider-specific extensions do not alter shared semantics.
- Treat latency, cost, quality, security, and broad compatibility claims as
  unsupported unless checked-in evidence and thresholds make them reproducible.

## Boundaries

This workflow does not select a provider SDK, HTTP stack, async trait strategy,
configuration source, credential store, retry algorithm, tokenizer, streaming
protocol, or concrete request schema. Those decisions belong to accepted ADRs
or the current task. It does not authorize live-provider access or secrets.
