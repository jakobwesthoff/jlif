# 7. Use mimalloc as global allocator

Date: 2025-07-30

## Status

Accepted

## Context

After optimizing string building to eliminate allocation churn (achieving 3x speedup), we wanted to further optimize the remaining allocations from JSON parsing internals, string operations, and general program overhead.

## Decision

We will use **mimalloc** as the global allocator to replace Rust's default system allocator.

## Consequences

- Additional 1.27x performance improvement on large buffers (combined 3.8x total speedup)
- Better memory allocation patterns for remaining allocations in JSON parsing and program execution
- Minimal complexity increase - single dependency and 3 lines of setup code
- Microsoft Research-backed allocator with proven performance characteristics