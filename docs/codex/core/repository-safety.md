# Repository Safety

Use this module for all Codex tasks unless the prompt explicitly defines a
stricter safety policy.

## Required safety rules

- Read every applicable `AGENTS.md` before changing files.
- Check initial repository state with `git status --short`.
- Preserve pre-existing user changes, including modified and untracked files.
- Do not overwrite, discard, or reformat user work unrelated to the task.
- Do not modify unrelated files.
- Do not modify `.codex/` or global Codex configuration.
- Do not stage files without a separate explicit user command.
- Do not commit without a separate explicit user command.
- Do not run destructive Git commands such as `git reset`, `git clean`,
  `git checkout --`, `git restore`, history rewrite, or force push unless the
  user explicitly requests the exact action.
- Do not add dependencies unless the task scope requires them and the Change
  Contract identifies the impact.
- Do not perform broad formatting unrelated to touched files.
- Report unexpected repository state before proceeding when it may affect scope
  or user-owned changes.
- Print final `git status --short` in the final report.

## Staging and commit policy

Staging and commit require a separate explicit user command. A task request to
implement, review, or document work is not permission to stage or commit.

When the user later requests a commit, stage only the files belonging to that
logical change and keep unrelated or pre-existing untracked files out of the
commit.
