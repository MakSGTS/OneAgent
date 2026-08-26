# MCP Protocol Task Template

## Purpose

Use this template for one accepted MCP protocol, server, capability,
transport, method-dispatch, or conformance implementation slice.

## Recommended profile

- `docs/codex/profiles/mcp-protocol-implementation.md`

## Required task-specific sections

- Authoritative MCP specification, schema, ADRs, and architecture documents
- Prerequisites / required gate
- Task
- Protocol revision and compatibility boundary
- Message, identifier, parameter, result, and error contract
- Capability and method-dispatch contract
- Transport framing and channel ownership, when applicable
- Lifecycle, cancellation, EOF/disconnect, failure, and shutdown policy
- Resource bounds and sensitive-data handling
- Domain, Runtime, and tool-policy ownership
- Conformance fixtures and public integration oracle
- Scope
- Included
- Excluded
- Acceptance Criteria
- Task-specific Validation
- Suggested commit message (recommendation only)

## Additional acceptance requirements

- Use one explicit authoritative protocol revision and record the provenance of
  every schema or fixture used as an oracle.
- Keep parsing and JSON-RPC validation, method dispatch, domain projection,
  transport framing, and Runtime service ownership separately testable.
- Define deterministic validation and error precedence and preserve request IDs
  without responding to notifications.
- Advertise only implemented methods and capabilities, with executable evidence
  for each advertised value and explicit rejection for unsupported behavior.
- Keep protocol output free from logs, banners, diagnostics, and unrelated
  bytes; keep sensitive request or result content out of implicit diagnostics.
- Prove transport claims through a non-zero public server matrix, including
  framing, concurrency or ordering where accepted, EOF/disconnect,
  cancellation, shutdown, cleanup, and repeated fresh execution.
- Treat external-client behavior as unsupported until an explicit client matrix
  is executed through the public boundary.

## Additional report sections

- Protocol authority and supported revision
- Wire, validation, and error behavior
- Capabilities and dispatch
- Transport and channel ownership
- Lifecycle and resource cleanup
- Conformance and public-entry-point evidence
- Domain, Runtime, tool-policy, and external-client impact
- Deferred compatibility and feature scope

## Additional validation

- Run non-zero focused message, schema, identifier, validation-precedence,
  version, discovery/capability, dispatch, error, and repetition tests
  applicable to the changed slice.
- Run non-zero public transport tests for framing, channel purity,
  EOF/disconnect, cancellation, failure, shutdown, and cleanup when claimed.
- Audit advertised capabilities and methods against implemented handlers and
  test coverage, and audit deferred capabilities for absence.
- Run affected Runtime, protocol, tool-policy, semantic, and client checks when
  their public or observable behavior changes.
- Run full workspace validation for production protocol behavior, public APIs,
  Cargo manifests, transport, lifecycle, or supported-client changes as
  required by `docs/codex/core/validation.md`.
