# Engine Roadmap

This roadmap tracks the planned evolution of the chess engine. Detailed implementation notes live in [`docs/plans/`](docs/plans/).

## Completed Foundations

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
- [x] Compare standalone bitboard and mailbox search at matching depths across ten positions, requiring equal move, PV, score, and search counters.
- [x] Decide that measured gains justify staged production integration while retaining mailbox as a correctness oracle.
- [x] Integrate bitboard search behind the reversible `bitboard-search` Cargo feature, initially retaining mailbox as default and test oracle.
- [x] Smoke-test both release UCI builds at the same position/depth and verify equal move, PV, score, and search counters.
- [x] Extend the alternating release comparison and exact search-equivalence assertions to ten positions.
- [x] Make bitboard search the Cargo default after equivalent-search, perft, full-suite, performance, and release UCI validation passed.

See [Plans 001-008](docs/plans/) for completed implementation details.

## Next: UCI and Validation

- [ ] Complete UCI search limits: `wtime`, `btime`, `winc`, `binc`, `depth`, `nodes`, and `infinite`.
- [ ] Add UCI options for hash size and diagnostics, with validated parsing and defaults.
- [ ] Build a reproducible paired-opening UCI self-play suite using bitboard and mailbox builds.
- [ ] Add a tactical and regression position corpus; turn each fixed defect into a FEN-based test.

See [Plan 009](docs/plans/009-uci-time-management.md) and [Plan 010](docs/plans/010-engine-validation.md).

## Then: Search Strength

- [ ] Add aspiration windows around the previous iterative-deepening score.
- [ ] Add late-move reductions with tactical and PV safeguards.
- [ ] Add null-move pruning and validate zugzwang-sensitive endgames.
- [ ] Add futility pruning only after node and tactical regression measurements are available.
- [ ] Add narrowly-scoped extensions for check, recapture, and passed-pawn promotion threats.

See [Plan 011](docs/plans/011-search-pruning.md).

## Later: Evaluation and Tables

- [ ] Introduce a middlegame/endgame phase model and tapered evaluation.
- [ ] Tune material and positional terms against the position suite.
- [ ] Improve pawn structure, mobility, passed pawns, and king safety incrementally.
- [ ] Make transposition-table size configurable through UCI `Hash`.
- [ ] Evaluate replacement and aging policies using hit-rate and search benchmarks.

See [Plan 012](docs/plans/012-tapered-evaluation.md) and [Plan 013](docs/plans/013-transposition-table-tuning.md).

## Future: Performance and Parallelism

- [ ] Profile the default bitboard search before changing move-generation internals.
- [ ] Consider magic bitboards or PEXT only when profiling identifies slider attacks as a bottleneck.
- [ ] Remove measured allocations and unnecessary ordering work.
- [ ] Prototype root parallelism after stable time management and validation infrastructure exist.
- [ ] Evaluate Lazy SMP only after root parallelism is correct and benchmarked.

See [Plan 014](docs/plans/014-performance-and-parallelism.md).

## Working Rules

- Preserve perft correctness before optimizing move generation.
- Add focused tests for every rule or search change.
- Record benchmark commands and results with performance-related changes.
- Prefer small, reversible steps over a second parallel engine.
- Do not claim playing-strength gains without paired-engine or external-engine match evidence.
