# 8. Regex crate for pattern matching

Date: 2025-08-02

## Status

Accepted

## Context

We need efficient pattern matching for filtering JSON and text output. The filtering system must handle both simple string matches and complex regular expressions with minimal performance overhead.

## Decision

We will use the **regex** crate for all pattern matching operations in our filtering system.

## Consequences

- Battle-tested regex engine with extensive optimizations used by ripgrep (rg)
- Built-in case-insensitive matching support via (?i) prefix
- Performance characteristics proven in high-throughput tools like ripgrep
- Single dependency covers both simple string matching and complex pattern requirements