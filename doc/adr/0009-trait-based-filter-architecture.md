# 9. Trait-based filter architecture

Date: 2025-08-02

## Status

Accepted

## Context

We need an extensible filtering system that supports current regex filtering while enabling future filter types like JSON path matching and field-specific filters. The system must have type safety and good performance characteristics.

## Decision

We will use a **Filter trait with enum_dispatch** and **borrowed references for FilterInput** to create an extensible filtering architecture.

## Consequences

- enum_dispatch provides zero-cost dispatch as a performance benefit over dynamic dispatch
- FilterInput<'a> uses borrowed references to avoid cloning large JSON values during filtering
- Type safety prevents filters from receiving Incomplete buffer states at compile time
- Easy extension for future filter types (JsonPathFilter, FieldFilter, CompositeFilter)
- Performance characteristics maintain baseline processing speeds with filtering improvements