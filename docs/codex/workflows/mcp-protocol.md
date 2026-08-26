# MCP Protocol Workflow

Use this workflow for Model Context Protocol server, transport, schema,
capability, and conformance boundaries.

## Protocol authority and versioning

- Identify the authoritative MCP specification revision, schema source, and
  repository-owned compatibility record before implementing wire behavior.
- Define the supported protocol revision set, negotiation or rejection rules,
  and backward-compatibility boundary explicitly; do not infer compatibility
  from an SDK or a newer draft.
- Keep MCP wire values separate from semantic, Runtime, tool-policy, provider,
  and client authorities. An adapter may project accepted domain behavior but
  must not redefine it.
- Treat specification drafts, examples, SDK behavior, and historical prompts as
  evidence with stated authority rather than silently combining them.

## Messages, schemas, and failures

- Define accepted JSON-RPC request, response, notification, identifier,
  parameter, result, and error shapes from the selected revision.
- State validation and error precedence for malformed JSON, invalid requests,
  unsupported versions, unknown methods, invalid parameters, notifications,
  duplicate or ambiguous fields, and unsupported message patterns.
- Preserve request identifiers exactly where the protocol permits them and
  never manufacture a response to a notification.
- Bound message, field, collection, schema, and result sizes when untrusted
  input or allocation is admitted. Do not fetch external schema references as
  an implicit validation side effect.

## Capabilities and method dispatch

- Make advertised capabilities a truthful projection of registered supported
  methods; absence or deferred support must not be advertised.
- Define capability negotiation, discovery, method registration, duplicate
  registration, deterministic ordering, and unknown-method behavior before
  implementation.
- Keep protocol dispatch independent from transport framing and long-lived
  Runtime ownership so the same accepted method behavior can be exercised in
  memory and through a public transport.
- Preserve tool-policy authorization, confirmation, and execution gates when a
  protocol method can trigger side effects; MCP request validity is not
  authorization.

## Transport and lifecycle

- Define framing, encoding, input/output ownership, startup acknowledgement,
  EOF or disconnect behavior, cancellation, graceful shutdown, failure, and
  resource cleanup for every included transport.
- Keep protocol data off diagnostic channels and keep diagnostics off protocol
  output channels when the transport requires channel purity.
- Inventory every reader, writer, task, listener, connection, channel, and
  cancellation source and keep it under an accepted structured owner.
- Do not claim another standard transport, authentication mode, session model,
  streaming behavior, retry, timeout, or process-supervision policy unless it
  is explicitly included and supported by deterministic evidence.

## Compatibility and conformance evidence

- Use repository-owned fixtures derived from the selected specification or an
  explicitly pinned schema, with provenance recorded in the task output.
- Cover positive, malformed, missing, unknown, duplicate, incompatible,
  reordered, repeated, EOF/disconnect, cancellation, and cleanup cases as
  applicable to the slice.
- Run non-zero in-memory protocol tests and public transport tests. Handler-only
  tests do not prove framing, channel purity, process lifecycle, or cleanup.
- Audit every advertised method and capability against executable evidence and
  every deferred capability against absence from the wire surface.
- Keep external-client compatibility as a separate claim unless the task runs
  an explicitly accepted client matrix against the public server entry point.

## Boundaries

This workflow does not select an MCP revision, crate owner, SDK or dependency,
serialization library, message-size bound, capability set, method catalog,
transport, authentication policy, Runtime composition, tool mapping, external
client, or first production slice. Those decisions belong to accepted ADRs or
the current task. It does not authorize network access, credentials, real tool
effects, or unsupported compatibility claims.
