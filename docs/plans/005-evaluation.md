# 005 - Evaluation

## Goal

Replace material-only scoring with a small, explainable static evaluation.

## Scope

Add features incrementally: piece-square tables, mobility, center control, pawn structure, bishop pair, development, and king safety. Keep all values in centipawns and document the sign convention.

## Tests

Use isolated FEN positions where one feature clearly changes the score. Add tactical regression positions only after the evaluation remains deterministic.

## Rule

Do not tune multiple features at once without benchmark positions and a recorded baseline.
