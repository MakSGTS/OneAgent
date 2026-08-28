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

# Git Branch and Release Workflow

- Create each version branch from `main` and include the version number in its
  name: `codex/v<major>.<minor>`.
- Create each sprint implementation branch from the current version branch and
  include both the version and sprint numbers in its name:
  `codex/v<major>.<minor>-sprint-<number>`.
- After the sprint implementation and its required validation succeed, merge
  the sprint branch into the version branch with `git merge --no-ff`.
- Create sprint review branches from the version branch after the corresponding
  implementation merge. Name them
  `codex/v<major>.<minor>-sprint-<number>-review`.
- Create release review branches from the version branch and name them
  `codex/v<major>.<minor>-release-review`.
- Keep review work separate from implementation fixes. When a review requires
  remediation, create
  `codex/v<major>.<minor>-sprint-<number>-remediation` from the version branch,
  merge the remediation back with `git merge --no-ff`, and repeat the required
  review gate.
- Merge successful review branches back into the version branch with
  `git merge --no-ff`.
- After the release review and all required validation succeed, merge the
  version branch into `main` with `git merge --no-ff` and tag the resulting
  release commit with the exact version, for example `v0.7`.
- Push every commit immediately to `origin`, including merge commits. On the
  first push of a new branch, set its upstream with
  `git push -u origin <branch>`; use `git push origin <branch>` afterwards.
- Push only the current working branch. Push `main` only after the completed
  version has been merged into it.
- If a push fails, stop before making further changes or commits, report the
  exact failure, and resolve or agree on the cause before continuing.
- These push rules do not authorize creating a commit. Create commits only when
  the current user request explicitly authorizes them.

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
