# 002 - Search Observability

## Status

Complete for fixed-depth alpha-beta search. Iterative-deepening statistics will extend this result later.

## Goal

Make search performance measurable without changing search results.

## Scope

Add a search statistics structure containing depth, nodes, cutoffs, elapsed time, and nodes per second. Keep counters optional or cheap enough for normal UCI use. Expose stable benchmark output for fixed positions and depths.

## Tests

- Verify node and cutoff counters are non-zero on non-trivial searches.
- Verify repeated searches return the same move and score.
- Keep timing checks informational rather than enforcing machine-specific limits.

## Completion

The benchmark can compare revisions using the same FEN, depth, and build mode.
