# 005 - Evaluation

## Goal

Replace material-only scoring with a small, explainable static evaluation.

## Scope

Add features incrementally: piece-square tables, mobility, center control, pawn structure, bishop pair, development, and king safety. Keep all values in centipawns and document the sign convention.

## Progress

- [x] Add pawn and knight piece-square bonuses to material evaluation.
- [x] Add a bishop-pair bonus (30 centipawns per side with at least two bishops).
- [x] Record an initial search benchmark for this evaluation baseline.
- [x] Add pseudo-legal mobility and center control.
- [x] Add doubled, isolated, and passed-pawn scoring.
- [x] Add minor-piece development and king shield/attack-pressure scoring.
- [x] Compare the completed evaluation against the earlier search benchmark.

Scores remain from the side-to-move perspective. Piece-square bonuses use mirrored squares for Black. Mobility counts reachable non-friendly squares for non-pawns without filtering king safety. Weights are initial heuristics, not tuned playing-strength claims.

## Tests

Use isolated FEN positions where one feature clearly changes the score. Tests cover central knight placement, mobility, center control, pawn structure, bishop pair, development, king safety, and color/side-to-move symmetry.

## Initial Benchmark

Command: `cargo test --test search_bench -- --ignored --nocapture`

Recorded locally after adding pawn and knight tables: depth 3, 20 iterations, 20,180 nodes, 21,816 NPS; depth 4, 5 iterations, 36,385 nodes, 7,282 NPS. Treat these as a machine-specific baseline, not a cross-machine performance claim.

After adding the bishop-pair bonus, the same command reported depth 3 at 24,167 NPS and depth 4 at 8,811 NPS. Node counts were effectively unchanged (20,180 and 36,365), as expected for a static-evaluation-only change. Timing differences are not evidence of a strength or speed improvement.

With the full evaluation, the same benchmark reported depth 3 at 10,690 NPS and depth 4 at 5,357 NPS (23,080 and 45,325 nodes). This shows a material evaluation cost compared with the bishop-pair-only run; the search explores a different tree, so node counts and elapsed time are not a direct strength comparison. Optimization and tuning should use a fixed FEN suite in a follow-up.

## Rule

Do not tune multiple features at once without benchmark positions and a recorded baseline.
