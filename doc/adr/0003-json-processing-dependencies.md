# 3. JSON processing dependencies

Date: 2025-07-29

## Status

Accepted

## Context

We need JSON parsing with pretty-printing and colorized terminal output for streaming data processing.

## Decision

We will use:
- **serde_json** for JSON parsing and pretty-printing
- **colored** for syntax highlighting and colorized terminal output

## Consequences

- High-performance JSON parsing with detailed error information
- Colorized output improves readability of formatted JSON
- Additional dependencies increase binary size