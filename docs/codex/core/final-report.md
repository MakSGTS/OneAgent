# Final Report

The final user-visible report should be concise, factual, and written in
Russian unless the user explicitly requests another language.

## Standard sections

Use these sections when relevant:

- Итог
- Исходное состояние
- Что исследовано
- Что изменено
- Файлы созданы
- Файлы изменены
- Поведенческое влияние
- Валидация
- Контекст и бюджет
- Риски и открытые вопросы
- Suggested Commit Message
- Repository Safety
- Final Git Status

Sections may be omitted only when clearly irrelevant for a small task.

## Required content

- Do not make unsupported claims.
- Report exact validation commands and results.
- Report zero matched test filters separately from meaningful test passes.
- State whether a commit was created.
- State whether anything was staged.
- State whether `.codex/` was untouched.
- Distinguish pre-existing changes from task-created changes.
- Include final `git status --short` output.
- Report the Context Manifest preflight decision and measured token telemetry
  when available; use `недоступно` otherwise and never present an estimate as
  measured usage.
- List retained large-log artifact paths without copying complete logs.
- State each status, metric, and validation result once; reference the task
  ledger instead of repeating it in several sections.
