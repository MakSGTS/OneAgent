# External MCP Client Compatibility Fixtures

These fixtures preserve the exact first JSON-RPC requests observed from the
pinned public clients during the Sprint 35 investigation:

- `codex-initialize.json`: Codex CLI `0.150.0-alpha.8`, executable SHA-256
  `4ff5e75f028e913cfeb53bd7319f87573cdce6538c1b1ccc44ce62d5ce51ca1d`;
- `cursor-initialize.json`: Cursor Agent `2026.08.25-3e8eec8`, executable
  SHA-256
  `2ccc9a8e167797641448b5e5c936f006ba137a2555f117f38c5eb76a5238a233`.

The requests were captured at the repository-owned server boundary before the
compatibility implementation. They are untrusted protocol inputs, not client
configuration or executable artifacts. Runtime public-process tests consume
the files directly and extend them with repository-owned lifecycle, list,
call, failure, notification, framing, EOF, repetition, and isolation frames.

The authoritative provenance and accepted projection rules are recorded in
the [investigation](../../../../docs/architecture/external-ai-client-compatibility-investigation.md)
and [ADR-0057](../../../../docs/adr/0057-external-ai-client-compatibility.md).
Live-client outcomes are recorded in the
[Sprint 35 evidence](../../../../docs/architecture/external-ai-client-compatibility-evidence.md).

These fixtures make protocol conformance platform-neutral. They do not claim
that a synthetic test executed the proprietary clients.
