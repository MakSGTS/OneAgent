# Implement Sprint 32 LSP Navigation and Symbols

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/implementation.md`

## Template

`docs/codex/templates/implementation-task.md`

## Authoritative documents

- `docs/adr/0054-lsp-adapter.md`
- `docs/architecture/lsp-adapter-investigation.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0053-navigation-symbol-search.md`

## Prerequisites / Required gate

Task 4 is committed; its LSP lifecycle/public-process matrix and full Rust gate
pass; the immutable Workspace and typed source-location baseline is available.

## Task

Implement the exact navigation and symbol capabilities accepted by ADR-0054.

## Scope

### Included

Accepted LSP navigation/symbol requests and truthful capabilities; URI/root/
document validation; source-location confinement and coordinate conversion;
symbol-kind mapping; deterministic matching, filtering, ordering, limits and
null/empty behavior; stable request failures; transport-independent handlers;
positive, missing, ambiguous, conflicting, escaping, malformed, reordered,
repeated and bounded in-memory/public-process evidence across accepted EDT and
Designer fixtures; and compatibility with the existing MCP symbol projection.

### Excluded

Any method not accepted by ADR-0054, source text reads, opaque-provenance
decoding, mutable-document synchronization, fuzzy matching changes, references,
completion/hover/rename/edits, TypeScript/VS Code UI changes, MCP schema changes,
external-client claims, and new dependencies unless approved.

## Acceptance Criteria

- Advertised navigation/symbol capabilities exactly match executable handlers
  and immutable canonical graph evidence.
- Paths remain workspace-confined file URIs; positions/ranges follow negotiated
  ADR-0054 semantics and unsupported/multiple locations fail or omit exactly as
  accepted without guessing.
- Results are deterministic under reordered graph/fixture/request evidence and
  remain bounded.
- Non-zero public process tests cover each accepted method and regression-test
  lifecycle, channel purity, MCP symbols, and existing Runtime behavior.

## Task-specific Validation

- Run focused Common/Graph/adapter/Workspace location tests, protocol LSP tests,
  Runtime semantic handler tests, and public `oneagent-lsp` process tests.
- Audit capability/handler/kind/URI/range agreement and MCP regression behavior.
- Run the canonical Rust workspace gate and `git diff --check`.

## Suggested commit message

`Implement Sprint 32 LSP navigation and symbols`

## Final report additions

Report accepted methods/capabilities, symbol mapping/order/bounds, URI/path and
position safety, public test counts, MCP compatibility, and deferred methods.
