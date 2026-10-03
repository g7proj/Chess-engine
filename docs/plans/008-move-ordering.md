# 008 - Move Ordering Heuristics

## Goal

Improve alpha-beta cutoffs by ordering likely-good quiet moves after PV/TT moves, captures, and promotions.

## Implementation

- [x] Keep the PV/TT move as the highest-priority move.
- [x] Add two killer quiet moves per ply, learned from beta cutoffs.
- [x] Add a color/from/to history score, rewarding quiet cutoff moves and penalizing previously searched quiet moves.
- [x] Preserve capture, promotion, castling, and check scoring.
- [x] Test killer/history updates and verify benchmark node/time changes.

Heuristics live for one search and persist across iterative-deepening passes. History values are bounded; captures and promotions do not update killer/history tables.

## Tests

Run `cargo test`. Focused tests verify TT-move priority, two killer slots, color-specific history, quiet-move maluses, and existing enabled/disabled TT result equivalence.

## Benchmark

Command: `cargo test --test search_bench -- --ignored --nocapture`

On the same local debug benchmark harness, before killer/history ordering the engine reported depth 3 at 23,080 nodes / 2,314 ms and depth 4 at 42,575 nodes / 8,357 ms. With the new heuristics it reported depth 3 at 13,660 nodes / 1,423 ms and depth 4 at 9,600 nodes / 2,225 ms. These are machine-specific search-tree measurements, not a playing-strength result; validate future tuning with a fixed position suite.
