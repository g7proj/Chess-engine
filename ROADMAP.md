# Engine Roadmap

This roadmap tracks the planned evolution of the chess engine. Detailed implementation notes live in [`docs/plans/`](docs/plans/).

## Now

- [x] Separate game-state updates from search-state updates.
- [x] Convert `perft` traversal to make/unmake.
- [x] Add search statistics and reproducible performance output.

See [Plan 001](docs/plans/001-search-state-and-perft.md) and [Plan 002](docs/plans/002-search-observability.md).

## Next

- [x] Add iterative deepening with a principal variation.
- [x] Add UCI time management and functional `stop` handling.
- [x] Add quiescence search for tactical stability.

See [Plan 003](docs/plans/003-iterative-deepening.md) and [Plan 004](docs/plans/004-quiescence-search.md).

## Later

- [x] Complete evaluation improvements (material, positional tables, mobility, center, pawn structure, bishop pair, development, and king safety).
- [x] Add Zobrist hashing and a transposition table.
- [x] Revisit move ordering using transposition, killer, and history heuristics.
- [x] Profile move generation, attack detection, make/unmake, and search; verify a test-only bitboard attack prototype.
- [x] Prototype reversible incremental bitboard updates and verify perft equivalence for normal and special moves.
- [x] Benchmark paired perft to measure bitboard-maintenance overhead.
- [x] Compare mailbox and bitboard-legality alpha-beta prototypes for equivalent best move, score, nodes, and state restoration.
- [x] Benchmark the bitboard search prototype across representative positions and verify equivalent scores, moves, and node counts.
- [x] Compare bitboard legality against the complete production search path.
- [x] Prototype bitboard-native piece placement, legal move generation, and attacks; compare canonical perft and full search.
- [x] Make side-to-move, castling, and en-passant state self-contained in the bitboard prototype.
- [x] Port static evaluation to the bitboard position and verify score equivalence.
- [x] Optimize bitboard evaluation access and controls-square queries; verify exact scores and benchmark full search.
- [x] Prototype standalone bitboard alpha-beta and quiescence search; verify scores and benchmark against mailbox search without TT.
- [x] Add incremental Zobrist hashing and the existing transposition-table bound semantics to standalone bitboard search.
- [x] Reuse killer/history heuristics in standalone bitboard move ordering and verify ordering behavior.
- [x] Add iterative deepening, stop/deadline handling, principal variation, and complete `SearchStats` to standalone bitboard search.
- [x] Compare standalone bitboard and mailbox search at matching depths across six positions, requiring equal move, PV, score, and search counters.
- [x] Decide that measured gains justify staged production integration while retaining mailbox as a correctness oracle.
- [x] Integrate bitboard search behind the reversible `bitboard-search` Cargo feature; keep mailbox as the default and test oracle.
- [ ] Run release UCI match/performance validation and decide whether the bitboard feature should become the default.

See [Plan 005](docs/plans/005-evaluation.md), [Plan 006](docs/plans/006-transposition-table.md), [Plan 007](docs/plans/007-bitboards.md), and [Plan 008](docs/plans/008-move-ordering.md).

## Working Rules

- Preserve perft correctness before optimizing move generation.
- Add focused tests for every rule or search change.
- Record benchmark commands and results with performance-related changes.
- Prefer small, reversible steps over a second parallel engine.
