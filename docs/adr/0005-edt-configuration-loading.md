# ADR-0005: EDT Configuration Loading

## Status

Accepted

## Decision

A dedicated EDT adapter reads `src/Configuration/Configuration.mdo` and converts
its UUID and metadata name into `oneagent_workspace::Configuration`.

The domain libraries remain independent from EDT XML and namespace details.
