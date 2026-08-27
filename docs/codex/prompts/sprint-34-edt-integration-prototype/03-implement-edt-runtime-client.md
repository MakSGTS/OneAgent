# Implement Sprint 34 EDT Runtime Client

Continue OneAgent development.

## Reporting

- Prompt and repository artifacts: English.
- User-visible reports: Russian.

## Profile and template

- `docs/codex/profiles/ide-extension-implementation.md`
- `docs/codex/templates/ide-extension-task.md`

## Authoritative documents

- `docs/adr/0056-edt-integration-prototype.md`
- `docs/architecture/edt-integration-prototype-investigation.md`
- `docs/adr/0050-mcp-server.md`
- `docs/adr/0051-mcp-semantic-tools.md`

## Prerequisites / required gate

Task 2 and accepted ADR-0056 are committed. Protocol projection, bounds,
dependency policy, failure vocabulary, and lifecycle ownership are fixed.

## Task

Create the accepted Maven/Tycho reactor boundary needed by this task and
implement only the dependency-free Java Runtime compatibility-probe domain,
process adapter, and tests.

## Included scope

Exact immutable request/result/failure types; bounded newline-framed request;
strict compatible response validation; UTF-8, size, nesting or structural,
duplicate, missing, incompatible and trailing-content rejection as accepted;
owned process/stdin/stdout/stderr resources; deterministic timeout,
cancellation, exit and termination; redacted errors; injected process/time
test seams; pure unit fixtures; real `oneagent-mcp` process evidence; and the
minimum bundle/test build metadata required to compile and execute this slice.

## Excluded scope

Eclipse command, workbench selection, EDT nature checks, UI, jobs, preferences,
feature/category/repository, installation, CI/docs completion, new Rust/MCP
behavior, new production dependency, source parsing, or semantic inference.

## Acceptance criteria

- Only one ADR-0056-compatible response can produce success.
- Every process, stream, waiter, timeout and cancellation path terminates and
  has deterministic non-zero evidence.
- Raw stderr, exceptions, executable paths, environment values, and protocol
  payloads do not leak through public failures.
- The real Runtime is launched with an explicit fixture cwd and the seven-tool
  MCP catalog remains unchanged.

## Validation

Run non-zero focused Java unit tests, non-zero real `oneagent-mcp` process
tests, Maven compile/test/package checks on JDK 25, focused Rust MCP process
compatibility, dependency/manifest/generated-artifact audits, and
`git diff --check`.

## Suggested commit message

`Implement Sprint 34 EDT Runtime client`

## Final report additions

Report public types, exact parser/projection and bounds, process lifecycle,
failure redaction, Maven modules/dependencies, process test counts, Rust
compatibility, and preserved behavior.
