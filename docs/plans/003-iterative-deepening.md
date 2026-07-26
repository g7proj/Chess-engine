# 003 - Iterative Deepening

## Goal

Search progressively deeper while retaining the best completed result.

## Scope

- Search depths from 1 through the requested limit.
- Reuse the previous principal variation for move ordering.
- Return the last fully completed result if interrupted.
- Report depth, score, nodes, and principal variation through UCI `info` lines.

## Dependencies

Requires search statistics and a clear search result type.

## Tests

Verify deterministic best moves at fixed depths and that an interrupted search returns a legal move from the last completed iteration.
