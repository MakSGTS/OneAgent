# MCP Protocol Implementation Profile

## Purpose

Use this profile for implementing one accepted MCP protocol, server,
capability, transport, or conformance slice.

## Required Core modules

- `docs/codex/core/repository-safety.md`
- `docs/codex/core/repository-investigation.md`
- `docs/codex/core/change-contract.md`
- `docs/codex/core/validation.md`
- `docs/codex/core/final-report.md`

## Required Workflow modules

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/mcp-protocol.md`
- `docs/codex/workflows/runtime-service.md` when service lifecycle, transport
  ownership, cancellation, shutdown, or public process behavior changes
- `docs/codex/workflows/ai-tool-policy.md` when an MCP method maps to an
  executable tool request or side-effect boundary

## Task-family expectations

- Pin the authoritative protocol revision and implement only the accepted
  compatibility, method, capability, and transport surface.
- Keep JSON-RPC validation, method dispatch, domain projection, transport
  framing, and Runtime lifecycle in their accepted ownership layers.
- Advertise only methods and capabilities with complete executable evidence and
  reject unsupported versions, messages, methods, or parameters explicitly.
- Preserve protocol-channel purity and prove EOF/disconnect, cancellation,
  failure, shutdown, cleanup, and repeated execution through public entry
  points when transport behavior is included.
- Keep semantic tools, external-client support, authentication, remote
  transport, provider behavior, and IDE integration outside a task unless
  separately accepted and explicitly included.
