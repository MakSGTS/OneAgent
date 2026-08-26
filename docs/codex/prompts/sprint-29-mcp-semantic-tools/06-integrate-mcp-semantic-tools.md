# Integrate MCP Semantic Tools

Use the Runtime Service profile/template. Read the Sprint plan, investigation,
ADR-0051, ADR-0037, ADR-0050, Tasks 3-5 code, MCP transport/process and
Workspace builder/process tests.

Compose the semantic server from one immutable startup snapshot rooted at the
process working directory, then run the existing bounded stdio lifecycle.
Expose exactly the accepted catalog and preserve EOF, cancellation, terminal
failure, stdout purity, and existing Runtime HTTP/Workspace/CLI behavior. Add
real child-process evidence for discovery, list, each tool family, invalid and
unknown calls, ordering, repetition, diagnostics-only stderr, and shutdown over
the tracked mixed workspace fixture. No watching, reload, remote client, or
real side effect.

Run non-zero transport/process and affected Runtime/CLI tests plus the canonical
full workspace gate. Commit: `Integrate Sprint 29 MCP semantic tools`.
