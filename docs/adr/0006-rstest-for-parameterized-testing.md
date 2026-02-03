# 6. rstest for parameterized testing

Date: 2025-07-29

## Status

Accepted

## Context

We need to test multiple similar scenarios (different JSON types, various overflow conditions) without duplicating test logic.

## Decision

We will use **rstest** for parameterized testing, primarily as a data provider for test cases with multiple input variations.

## Consequences

- Reduces test code duplication for similar scenarios
- Clear separation between test data and test logic
- Additional dependency but focused on development/testing only
