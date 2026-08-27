# Define Sprint 34 EDT Integration Prototype

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/architecture.md`
- `docs/codex/templates/architecture-task.md`

## Authoritative documents

- `docs/architecture/edt-integration-prototype-investigation.md`
- `docs/Roadmap.md`
- `docs/Architecture.md`
- `docs/architecture/semantic-model-2.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`
- `docs/adr/0052-vscode-extension-foundation.md`

## Prerequisites / required gate

Task 1 is committed, its evidence has no blocking unknown, and any required
dependency approval is resolved before the decision accepts that dependency.

## Task

Create `docs/adr/0056-edt-integration-prototype.md` and synchronize only the
planning-level architecture text needed to accept the bounded native EDT
Runtime-probe workflow. Do not implement production behavior.

## Required decisions

Fix semantic and source authority, Java/Eclipse/Runtime ownership, dependency
direction, exact eligible selection and local-project rules, configuration and
validation, process cwd/environment, exact request/response projection,
protocol and byte bounds, timeout, stderr and exit handling, concurrency,
cancellation, process termination, redacted public errors, background/UI
threading, activation, configuration change, deactivation and disposal,
observability, public API policy, bundle/feature/category/repository structure,
Java build and execution environments, authenticated and local target handling,
host compatibility, deterministic unit/process/PDE/EDT test seams, CI boundary,
first slice, migration, rejected alternatives, and deferred scope.

## Acceptance criteria

- The first-slice user journey, exact visible success/failure states, and every
  owned resource have one unambiguous contract.
- Java performs no source parsing or semantic inference and accepts no
  proprietary EDT implementation API dependency without evidence and approval.
- No ITS credential, personal path, JRE, Runtime binary, or generated package is
  tracked or bundled.
- JDK 25 build/PDE behavior and Java 17 EDT 2026.1 compatibility are explicit
  and testable.
- Tasks 3-6 can implement and validate the decision without inventing policy.

## Excluded scope

Production implementation, new Rust/MCP semantics, editor semantic UI, source
reads, persistent connection, auto-start, remote/multi-project support,
Marketplace publication, and capability completion.

## Validation

Run ADR/investigation/source/API/dependency/consumer/scope/terminology/link
consistency checks and `git diff --check`.

## Suggested commit message

`Define Sprint 34 EDT integration prototype`

## Final report additions

Report accepted workflow, ownership, exact contracts and bounds, dependency and
Java policy, lifecycle, test seams, rejected alternatives, migration, and
deferred scope.
