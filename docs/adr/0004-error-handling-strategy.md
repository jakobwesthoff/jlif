# 4. Error handling strategy

Date: 2025-07-29

## Status

Accepted

## Context

We need structured errors for library components and rich context for user-facing errors in streaming JSON processing.

## Decision

We will use **hybrid error handling**:
- **Library Layer**: `thiserror` for typed errors (`JsonParserError`, `BufferError`)
- **Application Layer**: `anyhow::Result<T>` for rich error context

## Consequences

- Typed library errors enable programmatic error handling
- Rich error context chains aid debugging of streaming issues
- Requires error type conversion at library boundaries