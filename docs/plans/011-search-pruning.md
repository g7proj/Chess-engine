# 011 - Search Pruning

## Goal

Reduce searched nodes while preserving tactical reliability.

## Order

- [ ] Add aspiration windows with full-window re-search on fail-low or fail-high.
- [ ] Add late-move reductions for quiet, non-PV moves; re-search on improvement.
- [ ] Add null-move pruning with depth, check, and endgame safeguards.
- [ ] Add futility pruning only where static evaluation makes the bound safe enough.
- [ ] Evaluate extensions for checks, recaptures, and promotion threats one at a time.

## Tests

For every heuristic, add tactical positions where pruning must not hide a best move. Compare baseline and enabled results at fixed depth, record nodes and elapsed time, and keep a disable switch while tuning.

## Completion

Enable a heuristic only after it reduces nodes on the position suite without tactical regressions or unstable UCI behavior.
