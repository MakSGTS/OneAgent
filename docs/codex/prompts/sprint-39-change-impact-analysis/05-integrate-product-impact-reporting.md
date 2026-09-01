# Integrate Sprint 39 Product Impact Reporting

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profiles and template

- `docs/codex/profiles/mcp-protocol-implementation.md`
- `docs/codex/profiles/ai-tool-policy-implementation.md`
- `docs/codex/profiles/runtime-service-implementation.md`
- `docs/codex/templates/mcp-protocol-task.md`

## Required workflows

- `docs/codex/workflows/implementation.md`
- `docs/codex/workflows/mcp-protocol.md`
- `docs/codex/workflows/ai-tool-policy.md`
- `docs/codex/workflows/runtime-service.md`

## Authoritative documents

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/architecture/change-impact-analysis-investigation.md`
- `docs/architecture/mcp-semantic-tools-investigation.md`
- `docs/architecture/external-ai-client-compatibility-evidence.md`
- `docs/adr/0039-workspace-service.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0057-external-ai-client-compatibility.md`
- `docs/adr/0061-change-impact-analysis.md`
- `docs/reviews/sprint-29-mcp-semantic-tools.md`
- `docs/reviews/sprint-35-external-ai-client-compatibility.md`

## Prerequisite

Task 4 is committed, its complete validation passes, and Workspace publishes
the accepted immutable Change Impact evidence without partial state.

## Task

Integrate the ADR-0061 product-facing workflow through the existing
`oneagent.impact` MCP tool. Implement only the accepted compatible request,
projection, policy, error, bound, and public-process behavior.

## Required behavior and evidence

- Keep the pinned MCP protocol revision, JSON-RPC validation, static catalog,
  Runtime semantic projection, Tool Policy, and public stdio process in their
  existing owners. The handler projects one accepted immutable Workspace
  report and does not normalize or recompute product impact semantics.
- Implement exactly the ADR-0061 input migration or additive selection
  contract. Reject missing, extra, mutually incompatible, ambiguous, malformed,
  unavailable, and unsupported inputs with deterministic existing-precedence
  errors before domain execution.
- Project accepted report identity, previous/current Configuration identity,
  completeness, summary, affected items, reasons, statuses, availability,
  depth, omitted/returned counts, and truncation truthfully in canonical order.
  Never reconstruct complete summary values from a bounded subset.
- Enforce schema and handler bounds consistently for depth, item limit, reasons,
  identifier components, result size, and every ADR-accepted field. An output
  that cannot fit the policy/protocol bound fails closed; strings are not
  silently shortened.
- Preserve the exact read-only Tool Policy identity/effects, decision-to-
  execution gate, audit completion, default denial, result bound, and error
  mapping. Schema validity or read-only annotations are not authorization.
- Keep public output and implicit errors free from Workspace/source roots,
  repository baseline/paths/statuses/completeness, provenance paths, source
  content, credentials, rejected input values, policy internals, and Rust error
  chains.
- Preserve catalog/capability truth, request identifiers, notification
  behavior, channel purity, sequential immutable-process ownership, EOF,
  failure, shutdown, cleanup, and repeated fresh execution.
- Add non-zero in-memory and public-process evidence for positive/empty/
  unchanged/added/removed impact, every accepted input form, exact/one-over
  bounds, truncation/completeness/summary reconciliation, missing/unavailable/
  malformed/conflicting inputs, policy deny/failure, oversized output,
  reordered arguments, repeated output, EOF/channel purity, and compatibility.
- Audit the VS Code and validated external MCP clients for catalog/schema or
  request assumptions. Do not claim a new external-client matrix unless it is
  executed through the public process under an accepted task scope.

## Excluded scope

New MCP tool names or unrelated catalog entries, protocol revision change,
remote transport, authentication, streaming, session history, HTTP/CLI/LSP/IDE
UI, Graph/Workspace report semantics, new diagnostics/rules, source parsing,
scoring/risk prediction, refactoring, source edits, transactions, current-state
documentation, and Sprint completion.

## Validation

Run non-zero focused schema/argument/lookup/projection/order/summary/
completeness/bound/error/redaction/policy/repetition tests; MCP protocol, stdio,
public-process, Tool Policy, Runtime/Workspace, existing semantic-tool, VS Code
client-contract, and affected external-client compatibility checks; then the
canonical full Rust workspace gate and `git diff --check`.

## Suggested commit message

`Integrate Sprint 39 product impact reporting`

## Final report additions

Report protocol authority and revision, request/result migration, schema and
validation precedence, immutable report projection, ordering/completeness/
summary/bounds/errors, Tool Policy evidence, sensitive-data behavior,
public-process counts and lifecycle, client compatibility, API/dependency
impact, and full validation.
