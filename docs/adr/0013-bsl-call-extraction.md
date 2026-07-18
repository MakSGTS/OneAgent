# ADR-0013: BSL Call Extraction

## Status

Accepted

## Context

The semantic graph contains BSL declaration nodes, but call relations cannot be
created until calls are extracted from module source.

## Decision

Add a conservative call-extraction stage to `oneagent-bsl`.

- Direct and qualified identifiers followed by `(` are recognized.
- Declarations, comments, preprocessor directives and control-flow keywords are ignored.
- Calls contain a stable ID, callee name and one-based source line.
- The first implementation is line-oriented and deliberately conservative.
- Name resolution and graph `calls` edges remain separate later stages.
- Calls retain the name of the containing procedure or function when available.
- Calls outside procedures and functions have no caller scope.

## Consequences

- Call extraction stays independent from EDT and graph infrastructure.
- False positives are reduced by excluding known language constructs.
- A future full parser can replace the implementation behind `BslCallExtractor`.