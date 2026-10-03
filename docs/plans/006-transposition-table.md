# 006 - Transposition Table

## Goal

Reuse scores for positions reached through different move orders.

## Scope

- Add incremental Zobrist keys for pieces, side, castling, and en passant.
- Store key, depth, score, bound type, and best move.
- Probe before searching and store results after search.
- Use the stored best move for move ordering.

## Progress

- [x] Add deterministic 64-bit Zobrist keys for pieces, side to move, castling rights, and en-passant square.
- [x] Update keys incrementally on make and restore the prior key on unmake.
- [x] Add a fixed 65,536-slot transposition table with depth-preferred replacement.
- [x] Probe exact/lower/upper bounds and order moves by the stored best move.
- [x] Normalize mate scores by ply and expose TT hit/cutoff counters.
- [x] Verify search results with the table enabled and disabled.

The table is enabled by default and can be disabled with `SearchLimits::use_transposition_table`. The `Board` exposes mutable position fields, so each search refreshes its root key before relying on incremental child keys.

## Dependencies

Requires stable make/unmake and clear score/bound semantics.

## Tests

Verify key changes for every relevant state component, incremental-key agreement and restoration after normal/special moves, mate-score normalization, and identical best move/score with the table disabled or enabled.

## Benchmark

Command: `cargo test --test search_bench -- --ignored --nocapture`

With the table enabled, the current local baseline reported depth 3 at 9,974 NPS (23,080 nodes, no table hits) and depth 4 at 5,094 NPS (42,575 nodes, 505 hits and 305 TT cutoffs). Timing is machine-specific; depth-3 start-position search has no transpositions at this horizon. Use the benchmark after move-ordering or table-size changes.
