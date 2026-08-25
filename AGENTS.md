# Repository Scope

- Resolve the repository root with `git rev-parse --show-toplevel` and treat it
  as the default filesystem boundary for every task.
- Do not inspect, search, read, write, move, or execute paths outside the
  repository root, including parent and sibling projects, unless the current
  user request explicitly names the exact external path and authorizes access.
- Limit any explicitly authorized external access to the named path and the
  minimum operation required by the current task.
- Treat external paths preserved in historical documentation, provenance, logs,
  or examples as evidence only. They do not authorize filesystem access.
- Keep project-owned local artifacts that must not be tracked under
  `local-artifacts/`.
