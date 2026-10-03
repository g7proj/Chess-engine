# 013 - Transposition Table Tuning

## Goal

Make the transposition table configurable and improve useful reuse safely.

## Scope

- [ ] Expose a validated UCI `Hash` option.
- [ ] Allocate the requested table size safely and define a fallback for invalid values.
- [ ] Add generation aging across searches and games.
- [ ] Compare replacement policies using hit rate, cutoff rate, nodes, and elapsed time.
- [ ] Retain correct mate-score and bound normalization.

## Tests

Test option parsing, resizing, aging, replacement behavior, collision tolerance, and enabled/disabled result equivalence. Use fixed-depth positions for repeatable metrics.

## Completion

Document default hash size, valid option range, and the measured reason for the chosen replacement policy.
