# Implement Sprint 35 Legacy MCP Protocol Compatibility

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/mcp-protocol-implementation.md`
- `docs/codex/templates/mcp-protocol-task.md`

## Required workflows

- `docs/codex/workflows/mcp-protocol.md`
- `docs/codex/workflows/implementation.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/external-ai-client-compatibility-investigation.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0057-external-ai-client-compatibility.md`

## Prerequisite

Task 2 is committed and ADR-0057 is accepted with no blocking dependency or
migration question.

## Task

Implement only the ADR-0057 protocol-domain compatibility boundary: negotiated
version/session state, lifecycle-aware dispatch, and version-correct response
projection. Preserve modern protocol behavior and semantic-tool authority.

## Required behavior and evidence

- Represent the accepted connection/session state and transitions as a bounded,
  deterministic protocol-domain abstraction with no global mutable authority.
- Parse and validate initialize requests, negotiate only accepted versions,
  record only accepted capability/client facts, and produce the exact accepted
  initialize result or deterministic JSON-RPC failure.
- Enforce pre-initialize, initialized-notification, duplicate initialize,
  post-shutdown, notification, request-ID, unknown-method, invalid-params, and
  malformed-message rules with the accepted error precedence.
- Project tools/list and tools/call responses exactly for each accepted version;
  omit unsupported fields rather than leaking a newer schema. Keep the exact
  seven-tool definitions, ordering, Tool Policy gate, semantic results, domain
  errors, and modern 2026-era fields unchanged where ADR-0057 requires them.
- Preserve or migrate every public protocol consumer identified by Task 1.
  Keep stdio read/write loops, process spawning, real client execution, and
  current-state documentation outside this task.
- Add focused unit/integration tests for every accepted version; compatible and
  unsupported negotiation; each lifecycle/order/error state; list/call success
  and domain failure; version-specific shape absence/presence; request-ID
  echoing; malformed input; repeat calls; two independent sessions; and exact
  modern regression projections. Every filter must match non-zero tests.

## Excluded scope

Runtime stdio lifecycle integration, client download/configuration, external
client execution, new transport, new tool, semantic result changes, catalog
reordering, authentication, production dependency, current-state docs, and
Sprint completion.

## Validation

Run the focused protocol library/integration targets named by the changed code,
existing MCP semantic-tool and public-process regression tests that compile at
this boundary, explicit version/shape/error/isolation filters with non-zero
counts, then:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo doc --workspace --no-deps
git diff --check
```

## Suggested commit message

`Implement Sprint 35 legacy MCP protocol`

## Final report additions

Report changed APIs and consumers, negotiated versions, state and response
behavior, exact focused test names/counts, modern/tool-semantic preservation,
dependency impact, full-gate outcomes, and deferred Runtime integration.
