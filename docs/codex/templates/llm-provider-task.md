# LLM Provider Task Template

## Purpose

Use this template for provider-independent LLM contracts, model and capability
discovery, provider adapters, or provider contract evidence.

## Recommended profile

- `docs/codex/profiles/llm-provider-implementation.md`

## Required base template

- `docs/codex/templates/task-prompt.md`

## Required task-specific sections

- Provider-neutral ownership and API boundary
- Model identity, discovery, and capability contract
- Request, response, usage, finish, and compatibility contract
- Configuration and secret-handling contract
- Timeout, retry, cancellation, and cleanup contract
- Error taxonomy and provider mapping
- Contract corpus, fake, fixture, or controlled-endpoint oracle
- Consumer and provider-adapter compatibility

## Additional acceptance requirements

- Keep shared domain contracts independent from provider wire schemas and
  validate requests against explicit model capabilities before provider I/O.
- Define deterministic configuration precedence, model ordering, compatibility,
  terminal outcomes, error mapping, and redaction for the task slice.
- Make timeout, retry, cancellation, streaming, and cleanup behavior explicit;
  absence from accepted scope means unsupported rather than an invented default.
- Ensure credentials and sensitive prompt or response content cannot enter
  source control, diagnostics, snapshots, debug output, or test fixtures.
- Prove supported behavior through repository-owned deterministic evidence that
  requires no live credentials or external network.
- Audit all consumers before changing a public provider-neutral request,
  response, capability, or error contract.

## Additional report sections

- Provider-neutral boundary
- Model and capability behavior
- Request/response compatibility
- Configuration and secret handling
- Timeout, retry, cancellation, and cleanup
- Error mapping and redaction
- Contract-test evidence
- Consumer and adapter impact
- Deferred provider and AI integration scope

## Additional validation

- Run focused model, capability, request, response, error, redaction, timeout,
  retry, cancellation, and contract tests applicable to the changed slice.
- Run non-zero public provider conformance tests for every adapter claimed as
  supported.
- Run affected Context Engine, Runtime, protocol, or consumer checks when their
  public or observable behavior changes.
- Run full workspace validation for production provider behavior, public APIs,
  Cargo manifests, or adapter changes as required by
  `docs/codex/core/validation.md`.
