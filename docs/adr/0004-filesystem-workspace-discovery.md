# ADR-0004: Filesystem Workspace Discovery

## Status

Accepted

## Context

`OneAgent` must locate `1C:Enterprise` projects without coupling the domain
model to filesystem APIs or a specific IDE.

## Decision

Workspace discovery is split into a domain port and adapters.

- `oneagent-workspace` defines `WorkspaceDetector`.
- `oneagent-workspace-fs` implements recursive filesystem discovery.
- The first supported format is an EDT project containing both `.project` and
  `src/Configuration/Configuration.mdo`.
- Discovery has a configurable recursion limit.
- Build output, version-control and IDE service directories are ignored.
- An EDT project is treated as a discovery boundary and is not traversed further.

## Consequences

- Future adapters may discover projects through EDT APIs, VS Code or remote
  services without changing the domain model.
- Detection rules remain independently testable.
- Designer XML support can be added as a separate detector rule.
