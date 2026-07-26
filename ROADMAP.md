# Engine Roadmap

This roadmap tracks the planned evolution of the chess engine. Detailed implementation notes live in [`docs/plans/`](docs/plans/).

## Now

- [x] Separate game-state updates from search-state updates.
- [x] Convert `perft` traversal to make/unmake.
- [ ] Add search statistics and reproducible performance output.

See [Plan 001](docs/plans/001-search-state-and-perft.md) and [Plan 002](docs/plans/002-search-observability.md).

## Next

- [ ] Add iterative deepening with a principal variation.
- [ ] Add UCI time management and functional `stop` handling.
- [ ] Add quiescence search for tactical stability.

See [Plan 003](docs/plans/003-iterative-deepening.md) and [Plan 004](docs/plans/004-quiescence-search.md).

## Later

- [ ] Improve evaluation beyond material counting.
- [ ] Add Zobrist hashing and a transposition table.
- [ ] Revisit move ordering using transposition, killer, and history heuristics.
- [ ] Evaluate a bitboard representation after profiling.

See [Plan 005](docs/plans/005-evaluation.md), [Plan 006](docs/plans/006-transposition-table.md), and [Plan 007](docs/plans/007-bitboards.md).

## Working Rules

- Preserve perft correctness before optimizing move generation.
- Add focused tests for every rule or search change.
- Record benchmark commands and results with performance-related changes.
- Prefer small, reversible steps over a second parallel engine.
