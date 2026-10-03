# 005 - Evaluation

## Goal

Replace material-only scoring with a small, explainable static evaluation.

## Scope

Add features incrementally: piece-square tables, mobility, center control, pawn structure, bishop pair, development, and king safety. Keep all values in centipawns and document the sign convention.

## Progress

- [x] Add pawn and knight piece-square bonuses to material evaluation.
- [x] Add a bishop-pair bonus (30 centipawns per side with at least two bishops).
- [x] Record an initial search benchmark for this evaluation baseline.
- [ ] Compare future evaluation changes against this baseline under the same conditions.
- [ ] Evaluate mobility, pawn structure, bishop pair, development, and king safety as separate changes.

Scores remain from the side-to-move perspective. Piece-square bonuses use mirrored squares for Black. The implemented features are pawn/knight piece-square bonuses and a bishop-pair bonus; other evaluation features remain separate work.

## Tests

Use isolated FEN positions where one feature clearly changes the score. Tests cover central knight placement and color/side-to-move symmetry. Add tactical regression positions only after the evaluation remains deterministic.

## Initial Benchmark

Command: `cargo test --test search_bench -- --ignored --nocapture`

Recorded locally after adding pawn and knight tables: depth 3, 20 iterations, 20,180 nodes, 21,816 NPS; depth 4, 5 iterations, 36,385 nodes, 7,282 NPS. Treat these as a machine-specific baseline, not a cross-machine performance claim.

After adding the bishop-pair bonus, the same command reported depth 3 at 24,167 NPS and depth 4 at 8,811 NPS. Node counts were effectively unchanged (20,180 and 36,365), as expected for a static-evaluation-only change. Timing differences are not evidence of a strength or speed improvement.

## Rule

Do not tune multiple features at once without benchmark positions and a recorded baseline.
