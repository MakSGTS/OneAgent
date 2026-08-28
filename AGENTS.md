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

# macOS GUI-Dependent Validation

- On macOS, treat VS Code or Electron Extension Host tests, Eclipse, SWT/Cocoa,
  `1cedt`, `1cedtstart`, and `1cedtcli` as GUI-dependent processes. A CLI-like
  executable name or flag such as `--version` does not prove that the process
  is headless.
- Do not make a trial GUI launch inside the ordinary workspace sandbox. Before
  the first required launch, use the product's explicit approval mechanism to
  request execution outside the GUI-restricted sandbox. External-path access
  still requires the separate authorization defined above.
- The repository owner pre-authorizes required host execution for these
  GUI-dependent validations. Do not ask for a separate conversational
  confirmation before issuing the product's normal approval request with the
  exact command and justification. This pre-authorization does not bypass a
  product approval prompt and does not authorize unrelated host commands,
  elevated operating-system privileges, external-path access, application
  bundle changes, or code-signing changes.
- If host execution is not approved or unavailable, stop that validation step
  and report the missing evidence. Do not repeatedly retry a process that exits
  with `SIGABRT` in AppKit, `_RegisterApplication`, or `NSApplication`.
- Prefer metadata inspection or a documented genuinely headless entry point
  when it can answer the question without starting a GUI application.
- Preserve downloaded or installed application bundles and their signatures.
  Do not change a bundle identifier, modify a signed bundle, or run `codesign`
  as a workaround for sandbox launch failures unless the current task is
  explicitly about application signing and the user authorizes that change.
- Run required GUI-dependent validations sequentially and allow each host to
  terminate cleanly before starting another one.
- When a validation command uses a shell pipeline, enable `set -o pipefail`
  before the pipeline or capture and verify the status of every stage. Never
  report the final filter's zero exit code as the GUI process result.
