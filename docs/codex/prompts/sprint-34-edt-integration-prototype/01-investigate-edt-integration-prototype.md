# Investigate Sprint 34 EDT Integration Prototype

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/investigation.md`
- `docs/codex/templates/investigation-task.md`

## Authoritative documents and sources

- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`
- `docs/reviews/sprint-33-ai-chat-context-panel.md`
- repository Runtime/MCP process code, tests, fixtures, and current IDE consumers
- immutable official Eclipse and 1C plug-in-development sources
- the exact user-authorized EDT/PDE applications and read-only p2 pool

## Prerequisites / required gate

- The committed Sprint 34 planning baseline is HEAD.
- Sprint 33 is completed and Sprint 34 is the unique eligible target.
- External access remains limited exactly as recorded by the master prompt.

## Task

Create `docs/architecture/edt-integration-prototype-investigation.md` and
update only the Sprint 34 Roadmap state needed to record Task 1 start. Produce
decision-ready evidence for ADR-0056 without production implementation.

## Required evidence

- Pin exact official repository revisions, pages, target files, manifests, and
  supported public APIs used as evidence.
- Record installed EDT, Eclipse, bundle, launcher, Maven, Java version and
  architecture facts; distinguish build JDK 25 from the EDT 2026.1 Java 17
  runtime and explain the x86_64/arm64 boundary.
- Record the exact EDT project nature, local project eligibility candidates,
  workbench selection APIs, job/UI threading APIs, command/handler extension
  points, OSGi lifecycle, feature/repository conventions, and public-versus-
  internal API boundary.
- Map current `oneagent-mcp` cwd, framing, discovery, catalog, timeout, stderr,
  EOF, shutdown, and existing VS Code lifecycle behavior to a dependency-free
  Java candidate without selecting the architecture.
- Evaluate official authenticated p2, public Eclipse p2, local read-only pool,
  Maven/Tycho, PDE host, disposable EDT host, CI, package, install, uninstall,
  and repeated-run oracles. Keep credentials and personal absolute paths out of
  tracked artifacts except as explicitly redacted provenance categories.
- Record positive, empty, missing, inaccessible, non-EDT, multiple selection,
  malformed, oversized, duplicate, incompatible, timeout, cancellation,
  process-exit, stderr, repeated invocation, configuration-change, deactivation,
  install/uninstall, and clean-host cases.
- State every decision ADR-0056 must make, every rejected candidate supported by
  evidence, dependency approval gates, residual environmental constraints, and
  exact first-slice/deferred boundaries.

## Excluded scope

Architecture acceptance, production Java/PDE files, Maven reactor, feature or
repository artifacts, CI changes, Runtime changes, credentials, p2-pool writes,
application-installation writes, and capability completion.

## Validation

Run source/provenance/version/architecture/bundle/nature/API/process/toolchain/
host-oracle/inventory/link audits, applicable existing focused Runtime process
tests, and `git diff --check`. Zero-match evidence is not sufficient.

## Suggested commit message

`Investigate Sprint 34 EDT integration prototype`

## Final report additions

Report pinned sources and versions, executable commands and outcomes, public
API and dependency findings, Java architecture boundary, p2/auth handling,
candidate workflow, unresolved ADR questions, and unchanged behavior.
