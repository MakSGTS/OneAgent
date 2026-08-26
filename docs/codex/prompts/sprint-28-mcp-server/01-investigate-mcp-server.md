# Investigate the MCP Server Boundary

Continue OneAgent development.

## Reporting

- Prompt and repository changes: English.
- User-visible reports: Russian.

## Profile

`docs/codex/profiles/investigation.md`

## Template

`docs/codex/templates/investigation-task.md`

## Authoritative ADRs and architecture documents

- `docs/Roadmap.md`, Sprint 28 execution plan
- `docs/adr/0037-runtime-service-container.md`
- `docs/adr/0038-http-api-health.md`
- `docs/adr/0040-graph-query-api.md`
- `docs/adr/0043-cli-client.md`
- `docs/adr/0049-tool-execution-policy.md`
- `docs/architecture/semantic-model-2.md`
- `docs/reviews/sprint-27-tool-execution-policy.md`
- official Model Context Protocol specification, schema, transports, and
  versioning sources current at task execution

## Prerequisites / Required gate

The committed Sprint 28 planning baseline is current and the working tree has
no conflicting task-created change.

## Investigation objective

Create only `docs/architecture/mcp-server-investigation.md` with the complete
repository and authoritative specification evidence needed to decide ADR-0050
safely.

## Questions to answer

- Which exact official MCP revision and schema are authoritative, how are they
  pinned or cited, and what changed from the legacy handshake/session era?
- Which crate must own JSON-RPC/MCP values, validation, codec, server dispatch,
  transport adaptation, Runtime composition, and the public process boundary?
- Which existing and planned dependency edges are necessary, already locked,
  approved, or still gated?
- What request identifiers, per-request metadata, versions, messages,
  notifications, results, errors, validation precedence, and resource bounds
  form the smallest conforming first slice?
- What exact `server/discover`, empty capability, method registration, unknown-
  method, unsupported-version, and notification behavior can be proven before
  Sprint 29 semantic tools?
- What newline/UTF-8 framing, embedded-newline rejection, output serialization,
  channel-purity, ordering/concurrency, EOF, cancellation, reader/writer
  failure, shutdown, and cleanup contract applies to stdio?
- How should the public process compose with ADR-0037 Runtime services and
  preserve current HTTP, Workspace, Graph Query, CLI, Context, Tool Policy, and
  provider behavior?
- Which platform-neutral in-memory streams and real child-process pipes prove
  positive, malformed, missing, incompatible, duplicate, reordered, repeated,
  EOF, cancellation, failure, no-extra-output, and cleanup cases?
- Which semantic tools, legacy/other versions, HTTP transports, auth, remote
  access, external clients, packaging, and later MCP features remain deferred?

## Evidence scope and sources

- Workspace manifests, `Cargo.lock`, dependency graph, `crates/protocol`,
  `apps/runtime`, process entry points, service/HTTP/Workspace tests, and
  consumers.
- Official versioned MCP specification pages and source-of-truth schema for the
  selected revision; record URLs, retrieval date, revision, and normative role.
- Accepted ADRs, current architecture, Roadmap, reviews, Git history, and the
  committed MCP Protocol framework.
- Repository-owned Tokio duplex, async I/O, child-process pipe, lifecycle,
  cancellation, and controlled failure patterns.

## Excluded

Architecture acceptance, Rust/Cargo changes, semantic tools, live MCP client
execution, external network beyond read-only official specification retrieval,
credentials, remote transport, current-state docs, support claims, or Sprint
completion.

## Completion Criteria

- Confirmed facts, normative specification requirements, compatibility
  constraints, candidates, unknowns, and unsupported assumptions are separated.
- Every ADR-0050 decision input and deterministic oracle is documented, or the
  sprint stops with an exact evidence blocker.
- The selected protocol and public-process test matrix requires no live client,
  credential, remote service, real signal, or tool side effect.
- Only the named investigation document changes.

## Repository Safety

Preserve `.codex/`, Roadmap, prompt suites, ADRs, Rust/Cargo, current-state
documentation, source fixtures, and unrelated files.

## Task-specific Validation

- Re-run focused protocol/Runtime/process/dependency/test/history searches.
- Reopen the versioned official sources and reconcile the cited revision and
  schema with every recorded field and normative statement.
- Audit the document against every investigation question and accepted ADR.
- Verify internal links and run `git diff --check`.

## Suggested commit message

`Investigate Sprint 28 MCP server`

## Final report additions

Report official sources and revision, confirmed repository facts, accepted
constraints, unresolved decisions, deterministic oracle, exact created path,
validation, commit hash, and final Git state.
