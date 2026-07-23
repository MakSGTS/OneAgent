# Parser Workflow

Use this workflow for serialized source parsing.

## Required evidence

- Use real source evidence.
- Do not invent XML elements, attributes, fields, or source formats.
- Add realistic fixtures when production parsing is implemented.
- Keep domain parsing separate from graph emission unless the task explicitly
  includes both.
- Define malformed-input behavior.
- Define optional and missing field behavior.
- Keep parsing deterministic.
- Add positive and negative tests.

## Investigation-only outcome

If the real source layout is not known, stop before parser implementation and
report the missing production-source evidence. Do not create speculative parser
fields.

## Boundary

Do not emit semantic graph edges from a parser task unless the prompt explicitly
includes graph emission.
