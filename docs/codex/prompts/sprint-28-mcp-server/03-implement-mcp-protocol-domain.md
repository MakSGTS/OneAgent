# Implement the MCP Protocol Domain

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/mcp-protocol-implementation.md`

## Template

`docs/codex/templates/mcp-protocol-task.md`

## Authoritative MCP specification, schema, ADRs, and architecture documents

- `docs/Roadmap.md`, Sprint 28 execution plan
- `docs/adr/0050-mcp-server.md`
- `docs/architecture/mcp-server-investigation.md`
- `docs/architecture/semantic-model-2.md`
- the exact versioned official MCP sources accepted by ADR-0050

## Prerequisites / Required gate

- Task 2 and accepted ADR-0050 are committed.
- The current user has explicitly approved every production dependency edge
  required by ADR-0050, including any `serde`, `serde_json`, or internal
  `oneagent-protocol` consumer edge.
- The working tree has no conflicting task-created change.

## Task

Implement only the accepted bounded JSON-RPC/MCP value, validation, parsing,
and serialization foundation in the ADR-assigned protocol owner.

## Scope

### Included

- Exact supported revision constant/value, request IDs, per-request metadata,
  request/notification/result/error envelopes, and accepted discovery-domain
  values needed by later dispatch.
- Deterministic parse/validate/serialize behavior, exact error precedence,
  resource bounds, sensitive-data-safe diagnostics, and focused tests.
- Only approved manifest/lockfile changes required for this domain.

### Excluded

Method registry or dispatch, stdio framing, Runtime/service/process I/O,
semantic tools/capabilities, external-client support, HTTP transport, auth,
current-state docs, or Sprint completion.

## Acceptance Criteria

- Public values expose only ADR-0050-approved fields and preserve accepted
  string/integer request identifiers exactly.
- Malformed JSON, invalid request shape, missing/unsupported metadata/version,
  invalid identifiers/params, response/error conflicts, duplicates, bounds, and
  notification distinctions follow accepted precedence.
- Serialization emits one deterministic valid JSON value with no embedded log,
  source-chain, secret, unbounded payload, or Rust type prose.
- No notification can be converted into a response-producing request.
- Focused tests are non-zero and cover positive, negative, boundary,
  reordered, duplicate, Unicode, and repeated cases.
- No dispatch, transport, service, or semantic behavior enters the task.

## Repository Safety

Before editing, enumerate exact protocol source/test/manifest/lock paths from
the accepted ADR and live tree. Preserve `.codex/`, docs except task-owned API
Rustdoc, Runtime implementation, other crates, prompt suites, and unrelated
files.

## Task-specific Validation

- List and run non-zero protocol-domain unit and public tests.
- Audit public exports, dependency edges, bounds, error constants/precedence,
  request-ID preservation, notification separation, and redaction.
- Run `cargo test -p oneagent-protocol` and the canonical full workspace gate.

## Suggested commit message

`Implement Sprint 28 MCP protocol domain`

## Final report additions

Report implemented wire values and bounds, validation precedence, exact
dependency changes/approval, tests, public surface, exclusions, commit hash,
and final Git state.
