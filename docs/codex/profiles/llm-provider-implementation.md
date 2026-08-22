# LLM Provider Implementation Profile

## Purpose

Use this profile for provider-independent LLM contracts and concrete provider
adapters. It covers model and capability discovery, request and response
compatibility, secrets, timeouts, retries, cancellation, error taxonomy, and
provider contract evidence.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/llm-provider.md`
- `docs/codex/workflows/runtime-service.md` when Runtime lifecycle, owned
  background work, transport exposure, or supported-client behavior changes
- `docs/codex/workflows/context-engine.md` when provider input is assembled
  from the accepted Context Engine boundary

## Task-family expectations

- Keep provider-neutral model, capability, request, response, usage, finish,
  and error contracts separate from provider-specific wire formats.
- Define discovery, configuration, secret handling, timeout, retry,
  cancellation, streaming, and compatibility behavior only from accepted
  architecture and repository evidence.
- Preserve request identity and expose enough stable evidence to classify every
  terminal outcome without leaking credentials or provider payloads.
- Prove provider conformance through deterministic repository-owned fakes,
  fixtures, or controlled local endpoints before claiming a provider supported.
- Keep prompt policy, conversations, tool authorization, MCP, IDE, and
  provider-specific features outside a task unless separately accepted and
  explicitly included.
