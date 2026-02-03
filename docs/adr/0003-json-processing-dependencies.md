# 3. JSON processing dependencies

Date: 2025-07-29

## Status

Accepted

## Context

We need JSON parsing with pretty-printing and colorized output for line-by-line input processing with multi-line JSON buffering.

## Decision

We will use:
- **serde_json** for JSON parsing and pretty-printing
- **colored_json** for JSON syntax highlighting and colorized terminal output

## Consequences

- High-performance JSON parsing with detailed error information
- Colorized output improves readability of formatted JSON
- Additional dependencies increase binary size