# 007 - Bitboards

## Goal

Evaluate whether bitboards provide a worthwhile speedup after profiling.

## Scope

Measure time spent in move generation, attack detection, make/unmake, and search. Only then prototype bitboards behind focused tests or a separate representation module.

## Progress

- [x] Add a release-mode profile benchmark for legal generation, attack queries, search make/unmake, and full search.
- [x] Add a test-only bitboard attack-detection prototype and compare its answers against the mailbox implementation over every square in canonical positions.
- [x] Prototype incremental bitboard updates through make/unmake and verify against mailbox perft, including special moves.
- [x] Benchmark the mirrored perft traversal against mailbox-only perft.
- [x] Integrate bitboard legality checks into a test-only alpha-beta prototype and compare equivalent results and speed.
- [x] Benchmark the prototype over a representative position suite with equivalent search results and node counts.
- [x] Compare bitboard legality against the complete production search path before deciding whether integration is justified.
- [x] Prototype bitboard-authoritative piece placement, legal move generation, and attacks; retain mailbox metadata as the test oracle and compare perft/full search.
- [x] Move side-to-move, castling, and en-passant metadata into the standalone bitboard position and verify make/unmake restoration.
- [x] Port static evaluation through a shared board/bitboard read interface and verify exact score equivalence.
- [x] Optimize bitboard evaluation access and controls-square queries; verify exact scores and benchmark full search.
- [x] Add a standalone test-only bitboard alpha-beta/quiescence search and verify scores/state restoration against mailbox search without TT.
- [ ] Port Zobrist/TT, search heuristics, time control, and complete statistics; compare equivalent full searches before considering integration.

## Profile Findings

Command: `cargo test --release --lib benchmark_profile_engine_primitives -- --ignored --nocapture`

On the local release build, 3,000 start-position legal-generation calls produced 60,000 legal moves in 12.66 ms. 256,000 mailbox attack queries took 28.14 ms; the bitboard prototype took 8.79 ms after a one-time 14 µs conversion. 60,000 make/unmake pairs took 1.52 ms, and five depth-4 searches processed 9,600 nodes in 234 ms. These are focused microbenchmarks, not whole-engine attribution. The attack prototype suggests bitboards may help attack detection, but it has no incremental updates and is not used by production search.

Decision after primitive profiling: keep the mailbox engine unchanged until the incremental prototype is validated and compared end-to-end.

## Incremental Prototype Findings

The test-only generator now enumerates piece placement from bitboard masks and filters legality using bitboard attacks. It handles normal moves, captures, en passant, castling, and all promotion choices. The standalone state tracks side-to-move, en-passant square, and castling rights, and restores them with make/unmake. A mailbox copy is still maintained as an oracle. Root legal-move sets and paired traversal match mailbox perft for start position depth 4, Kiwipete depth 3, the endgame regression position depth 3, and targeted special-move positions; masks and rule metadata are checked after every make and undo.

Command: `cargo test --release --lib benchmark_incremental_bitboard_perft -- --ignored --nocapture`

On the local release build, start-position perft(5) took 1.43 s for mailbox reference and 0.79 s for bitboard-generated moves with mirrored mailbox state (4,865,609 nodes), about 45% faster in this run. This remains a prototype benchmark: the mailbox copy is still updated for equivalence checks and search state.

Decision after paired perft: retain the mailbox production path until bitboard legality is measured inside equivalent search workloads.

## Search Prototype Findings

A test-only bitboard shadow is synchronized through make/unmake; under unit tests, legal move generation and `is_in_check` use bitboard masks. This routes the existing production search implementation through bitboard-native move generation while retaining mailbox metadata and evaluation. The normal binary and production board remain unaffected. Equivalence tests cover start position, Kiwipete, promotion, en passant, castling, and endgame positions.

Command: `cargo test --release --lib benchmark_bitboard_search_prototype -- --ignored --nocapture`

The benchmark covers start position (depth 4), Kiwipete (3), an endgame (4), a middlegame (3), promotion (3), and castling (3), using full iterative deepening, quiescence, transposition table, move ordering, and the generic evaluator on bitboard state. Alternating run order produced identical best moves, scores, PVs, node counts, quiescence counts, cutoffs, and TT statistics (30,973 nodes total). With the bitboard evaluator enabled, two local release runs took 1.12-1.13 s mailbox and 0.85 s bitboard shadow, about 25% faster.

Command: `cargo test --release --lib benchmark_bitboard_evaluation -- --ignored --nocapture`

The bitboard lookup first checks color occupancy and then searches only that color's six piece masks. Square-control evaluation enumerates matching piece masks directly and checks slider blockers against combined occupancy, preserving the existing evaluator semantics. The 40,000-evaluation benchmark produced the same checksum (2,660,000); the latest run took 79 ms for bitboards and 96 ms for mailbox. Timings vary by run and machine.

The full-search benchmark on six positions produced identical best moves, scores, PVs, node counts, quiescence counts, cutoffs, and TT statistics (30,973 nodes total). The latest release run took 641 ms for bitboard shadow and 1.11 s for mailbox, about 42% faster in aggregate. This remains a test-only shadow benchmark: search still mirrors mailbox state for make/unmake and non-evaluation game state, and the normal binary is unchanged. The next step is to remove those remaining dependencies and benchmark an actually standalone bitboard search before considering production migration.

## Standalone Search Prototype

The test-only standalone search now performs legal move generation, make/unmake, alpha-beta negamax, quiescence, and evaluation directly against `BitboardPosition`; it does not construct or access a `Board` while searching. Tests compare its fixed-depth score with the mailbox search with TT disabled and verify that bitboard state is fully restored. The latest release benchmark matched scores at start depth 3 (5.5 ms vs 8.0 ms), Kiwipete depth 2 (24 ms vs 177 ms), and endgame depth 3 (1.9 ms vs 4.8 ms). This is not yet an apples-to-apples full engine comparison: the standalone prototype lacks transposition hashing/table, killer/history ordering, time control, and full search statistics. Port those components and rerun equivalent searches before drawing an integration conclusion.

## Constraints

Do not replace the current representation without perft equivalence, make/unmake equivalence, and benchmark evidence. Keep the rule model understandable while comparing implementations.

## Completion

Adopt bitboards only if the measured gain justifies the added complexity and all canonical perft counts remain unchanged.
