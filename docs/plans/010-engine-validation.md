# 010 - Engine Validation

## Goal

Measure correctness, stability, and playing strength independently.

## Scope

- [ ] Add a FEN-based tactical corpus covering mates, forks, pins, promotions, and endgames.
- [ ] Add regression positions for every fixed search or rule defect.
- [ ] Create a paired-opening UCI self-play runner with colors swapped.
- [ ] Record engine build, limits, openings, results, failures, and elapsed time.
- [ ] Compare mailbox and bitboard output at fixed limits before using self-play results.

## Rules

Perft and fixed-depth equivalence establish correctness. Paired matches establish only measured relative playing results. Timing benchmarks establish performance. Keep these results separate in reports.

## Completion

The suite is reproducible locally, fails clearly on protocol or illegal-move errors, and produces a compact match summary suitable for future tuning comparisons.
