# How the Engine Evaluates a Position

The search needs a numeric estimate when it reaches a position it will not explore further. The evaluator assigns that estimate by adding material and positional bonuses for both sides. This guide describes the current implementation in `engine/src/search.rs`; its weights are simple, explainable heuristics and are not tuned to represent objective winning chances.

## Reading the Score

Scores use centipawns (cp), where 100 cp is the conventional value assigned to one pawn. A score of `+100` means the side to move is estimated to have an advantage worth one pawn; `-100` favors the opponent. This is a comparison scale, not a probability: `+100` does not mean a 100% chance to win.

The evaluator first builds a score from White's perspective. It returns that score if White is to move, and negates it if Black is to move. Therefore, the same board with the opposite side to move has the opposite score. This convention lets negamax use one scoring rule at every ply.

Checkmate scores are separate from ordinary positional evaluation. The search uses a value near `30,000` cp for mate and adjusts it by search ply so it prefers a faster win and a slower loss. Do not interpret a mate score as a material advantage of that many pawns.

## What Contributes to the Score

Conceptually, the evaluator computes:

```text
white_score = material_and_piece_squares
            + bishop_pair
            + mobility
            + center_control
            + pawn_structure
            + development
            + king_safety

score_for_side_to_move = white_to_move ? white_score : -white_score
```

Each positional feature is computed for both colors and added for White or subtracted for Black. Unless stated otherwise, the values below are in centipawns.

### Material

The current values are:

| Piece | Value |
| --- | ---: |
| Pawn | 100 |
| Knight | 320 |
| Bishop | 330 |
| Rook | 500 |
| Queen | 900 |
| King | 0 |

The king's value is zero because it is not traded as material; checkmate is handled by the search's terminal score. For a simple material-only comparison, a side with one extra pawn contributes `+100` to its score. If White has that pawn, the evaluator's material contribution is `+100` when White is to move and `-100` when Black is to move. Other positional terms can move the final score away from those numbers.

### Piece-Square Bonuses

The evaluator adds table bonuses to pawns and knights according to their square. Pawn bonuses encourage useful advances and central placement; knight bonuses favor central squares and discourage edge squares. Black uses a rotated version of the same tables so equivalent positions are treated symmetrically.

Other pieces currently receive their material value but no piece-square-table bonus. This is a deliberate small baseline, not a claim that rooks, bishops, and queens have no positional preferences.

### Bishop Pair

A side with at least two bishops receives `+30`. The bonus is applied once per side, regardless of whether that side has two or more bishops.

### Mobility

For each non-pawn piece, the evaluator counts reachable squares that are not occupied by a friendly piece. Each counted square is worth `+2` to that piece's side. Sliding pieces stop at the first occupied square; an enemy-occupied square is counted as reachable, then the ray stops.

This is **pseudo-legal mobility**: it does not remove destinations that expose the king to check. It is a cheap activity estimate, not a count of legal moves.

### Center Control

The four central squares (`d4`, `e4`, `d5`, `e5`) are checked for control. Each square controlled by a side adds `+8` for that side. A square is counted once even if several pieces attack it.

### Pawn Structure

- **Doubled pawns:** each pawn beyond the first on a file costs `12`. Three pawns on one file therefore incur two such penalties.
- **Isolated pawns:** each pawn with no friendly pawn on either neighboring file costs `10`.
- **Passed pawns:** a pawn with no opposing pawn ahead of it on its own or either neighboring file receives a rank-based bonus.

The passed-pawn bonuses by advancement rank are `[0, 5, 10, 20, 35, 55, 80, 0]`. White uses its rank directly; Black uses the mirrored rank. The final table entry is zero because a pawn on the last rank should already have promoted in a legal position.

These rules are intentionally local. The evaluator does not yet model pawn chains, backward pawns, blocked passers, or whether a passer can be stopped by the king.

### Development

Each knight or bishop not on its own home rank adds `+10` for its side. This is a simple development proxy: it does not check whether the piece has a useful square, whether it can move, or whether it has returned to its home rank after developing.

### King Safety

For each king, each friendly pawn on one of the three squares immediately in front of it adds `+10` as pawn-shield cover. Each of the king's eight neighboring squares controlled by the opponent costs `8`.

This is a compact king-safety signal. It does not evaluate open files, attacking-piece count, escape squares, or whether an attacked neighboring square is actually a legal king destination.

## Where Evaluation Is Used

The evaluator supplies a score at the search horizon. At ordinary depth-zero nodes, quiescence search first resolves selected captures and promotions, then uses static evaluation in quiet positions. It also uses static evaluation as the stand-pat option when the side to move is not in check. At regular negamax nodes, an empty legal-move list is scored as checkmate or stalemate; in quiescence, being in check disables stand-pat and an empty evasion list is scored as checkmate.

Since the score determines which moves look best, changing a weight can alter the search tree and chosen move. A score increase for one feature does not automatically mean the engine became stronger; evaluate changes against fixed positions and match results.

## Limits and Tuning

The current evaluator uses one set of weights for every game phase. It has no middlegame/endgame interpolation, learned parameters, or probability calibration. Some terms are intentionally rough proxies, and several features can reward overlapping ideas such as activity or central control.

When changing evaluation:

1. Change one feature or weight family at a time.
2. Add focused FEN tests that assert the expected direction of the score change.
3. Preserve color symmetry and mailbox/bitboard score equivalence.
4. Record results on a fixed position suite; use paired games to assess playing-strength changes.

## Implementation References

- `evaluate_position` combines all terms and converts the result to side-to-move perspective.
- `piece_value` defines the material values.
- `mobility_score`, `center_control_score`, `pawn_structure_score`, `development_score`, and `king_safety_score` implement the positional terms.
- [Evaluation plan](plans/005-evaluation.md) records the feature history and tests.
- [Search algorithms guide](search-algorithms.md) explains how the search consumes these scores.
- [Evaluator source](../engine/src/search.rs) contains the weights and implementation.
