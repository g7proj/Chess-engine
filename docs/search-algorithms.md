# How the Engine Searches for a Move

This guide follows the engine's search from one position to a selected move. It explains five connected ideas: negamax, alpha-beta pruning, iterative deepening, quiescence search, and move ordering. The production search uses the standalone bitboard position by default; the mailbox implementation follows the same search ideas and remains available with `--no-default-features`.

## Scores and Search Depth

The evaluator returns a score in centipawns from the point of view of the side to move. One pawn is roughly 100 centipawns. A score of `+120` means the side to move is estimated to be ahead by about 1.2 pawns; `-120` favors the opponent. These are estimates, not guaranteed game outcomes. Checkmate uses a much larger score so the search prefers mate over material.

A ply is one player's move. A depth of 3 means the search examines three plies beyond the current position, then evaluates the resulting positions (with quiescence search extending some lines). At the root, the engine compares its legal moves and returns the move with the best searched score.

## Negamax: One Rule for Both Players

Chess is a zero-sum game: a position that is good for White is equally bad for Black. Minimax usually describes separate maximizing and minimizing turns. Negamax uses the same operation at every turn by always scoring for the player whose turn it is:

```text
score(position) = max over legal moves of -score(position after move)
```

After a move, the opponent becomes the side to move. Their positive score is therefore negative for us, which explains the minus sign. The same sign change applies to the alpha-beta window: a parent search over `[alpha, beta]` calls its child with `[-beta, -alpha]`.

### Small example

Suppose White is choosing between `Nf3` and `e4`. At the leaves, scores are always from the side-to-move perspective:

| White's move | Black's reply | Leaf score (for White to move) | Black's score after sign flip | White's result |
| --- | --- | ---: | ---: | ---: |
| `Nf3` | `d5` | `+30` | `-30` | `+30` |
| `Nf3` | `e5` | `-10` | `+10` | `-10` |
| `e4` | `c5` | `+20` | `-20` | `+20` |

After `Nf3`, Black chooses the reply with the highest score for Black: `max(-30, +10) = +10`. White therefore evaluates `Nf3` as `-10`. After `e4`, Black's best reply scores `-20` for Black, so White sees `+20`. White selects `e4`, the line with the highest score from White's perspective. The engine does not need separate maximizing and minimizing functions.

The numbers above illustrate score perspective, not a claim about those opening moves. Actual evaluation also depends on deeper replies, captures, and the configured search depth.

## Alpha-Beta: Stop When a Branch Cannot Matter

Negamax can examine every legal continuation to the requested depth. Alpha-beta returns the same best move and score while skipping branches that cannot change the decision.

- `alpha` is the best score the current side can already guarantee from alternatives examined so far. It is a lower bound.
- `beta` is the score at which the opponent has a reason to avoid this branch. It is an upper bound imposed by an earlier choice.
- When a node finds `score >= beta`, it has a beta cutoff. Further moves cannot make this branch more relevant to the parent, so the remaining moves are not searched.

### Cutoff example with chess moves

At the root, White first searches `Nf3` and finds a line worth `+30` centipawns. White now has a known option worth at least `+30`, so root `alpha = +30`.

White next considers `e4`. It is Black's turn, and this child is searched with the negated window `[-beta, -alpha]`. For this example the relevant upper bound for Black is `beta = -30`. Suppose Black's first reply, `...c5`, leads to a position scored `+20` for White to move. Negamax flips that to `-20` for Black. Since `-20 >= -30`, Black has found a reply that holds White to at most `+20`, below White's already available `+30` from `Nf3`.

The search can stop examining Black's other replies to `e4`: White will choose `Nf3` instead. Alpha-beta has pruned work, not changed the result. In general, a cutoff is possible only after the current move proves the branch cannot improve the decision already available above it.

If there are `b` legal moves per position and depth `d`, a full tree can grow on the order of `b^d`. Alpha-beta's savings depend strongly on move ordering: finding strong moves early causes cutoffs sooner. In the best ordering, the number of examined nodes can approach the scale of `b^(d/2)`; poor ordering can leave the search close to the full tree. These are growth estimates, not fixed node counts.

## Iterative Deepening: Search Depths in Sequence

Instead of searching only to depth 5, the engine searches depth 1, then 2, then 3, and so on up to its limit or time budget. Each iteration is a complete search at that depth. Iterative deepening does not mean that depth 5 is replaced by a shallower answer: the engine keeps the result from the deepest iteration that finished completely.

The previous iteration provides two useful things:

1. A legal move is available quickly, even if a later iteration runs out of time.
2. Its principal variation (PV), the best line found so far, gives the next iteration a strong move-ordering hint.

For example, a depth-3 iteration may return `e4, ...c5, Nf3`. At depth 4, the search tries `e4` first, then follows the previous line where it remains legal. If a time limit or `stop` interrupts depth 4, the engine returns the completed depth-3 result rather than a partially searched line.

The deeper iteration is still a new search over the tree at its target depth. The engine does not merely continue the previous leaf evaluations. It reuses the PV, transposition table, and search heuristics to make that new search more efficient.

## Quiescence Search: Avoid Evaluating a Tactical Explosion

Ordinary depth-limited search evaluates a position as soon as its remaining depth reaches zero. This can produce an unstable result if the last move captures a piece and the opponent can immediately recapture. The leaf score would count the apparent gain without resolving the obvious reply.

Quiescence search continues past the nominal depth while the position is tactically unsettled. When the side to move is not in check, it first evaluates the option to make no capture now; this is called the **stand-pat** score. It then searches legal captures and promotions, recursively, until the available tactical sequence ends or a cutoff applies.

```text
if not in check:
    stand_pat = evaluate(position)
    if stand_pat >= beta: return beta
    alpha = max(alpha, stand_pat)

search legal captures and promotions
```

Stand-pat is not allowed while in check: the player must answer the check. In that case quiescence searches all legal evasions, including quiet king moves or blocking moves. If there are no legal evasions, the position is checkmate. A maximum quiescence ply limit prevents an endless tactical sequence from recursing without bound.

Quiescence improves leaf stability, but it is still selective: it does not search every quiet move. It can also grow large in positions with many captures, which is why it has its own node counter and cutoff behavior.

## Move Ordering: Search Promising Moves First

Move ordering does not change which moves are legal or the minimax result. It changes the order in which they are searched. Because alpha-beta can stop after a cutoff, a good order can make the same search much cheaper.

The current search prioritizes:

1. The transposition-table move or previous PV move.
2. Captures, ranked using victim and attacker values.
3. Promotions.
4. Moves that give check and castling.
5. Quiet moves favored by killer and history heuristics.

The **killer heuristic** remembers quiet moves that caused a beta cutoff at the same search ply. The **history heuristic** records quiet moves that have often caused cutoffs, indexed by side, origin square, and destination square. Quiet moves searched before a successful cutoff receive a history penalty, helping later searches distinguish them from the move that actually caused the cutoff.

These scores are priorities, not chess evaluation scores. Ordering a capture first does not mean the engine has decided the capture is best; it means the capture is worth checking early because it may quickly establish a useful bound. The search still evaluates every move that is needed to determine the result.

## Putting the Pieces Together

```text
for depth = 1, 2, ... until limit or stop:
    order moves using prior PV, transposition table, and heuristics
    search each required move with negamax
        pass the negated alpha-beta window to the child
        at depth zero, continue with quiescence search
        stop a branch when score reaches beta
    if this depth completed:
        save score, best move, and principal variation
return the last completed result
```

In the code, `negamax_alpha_beta` and `quiescence_search` are the mailbox versions; `bitboard_negamax` and `bitboard_quiescence` implement the default bitboard path. `find_best_move_iterative_with_limits` selects the configured implementation and preserves the iterative search result. Move ordering is implemented by `ordered_legal_moves` and `bitboard_ordered_moves`.

## Further Reading in This Repository

- [Position evaluation guide](position-evaluation.md)
- [Iterative deepening plan](plans/003-iterative-deepening.md)
- [Quiescence search plan](plans/004-quiescence-search.md)
- [Move ordering plan](plans/008-move-ordering.md)
- [Search implementation](../engine/src/search.rs)
