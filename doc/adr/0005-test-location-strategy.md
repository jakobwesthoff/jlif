# 5. Test location strategy

Date: 2025-07-29

## Status

Accepted

## Context

We need a consistent approach for organizing tests that keeps them close to the code they test while allowing flexibility for integration scenarios.

## Decision

We will **co-locate tests with implementation code** using `#[cfg(test)]` modules in the same files.

Exceptions allowed for true integration tests that span multiple modules, but co-location should be attempted first.

## Consequences

- Tests stay close to implementation for easier maintenance
- Faster development cycle with immediate test visibility
- May need separate integration test files for complex cross-module scenarios
