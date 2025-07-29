# 2. CLI argument parsing library choice

Date: 2025-07-29

## Status

Accepted

## Context

We need CLI argument parsing with automatic help generation and validation for streaming parameters like `--max-lines` and regex filters.

## Decision

We will use **clap** with derive macros for CLI argument parsing.

## Consequences

- Declarative CLI definitions reduce boilerplate
- Type-safe parsing with built-in validation
- Increased binary size compared to manual parsing