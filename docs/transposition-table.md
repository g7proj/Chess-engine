# Zobrist Hashing and the Transposition Table

In chess, the exact same board position can often be reached through different sequences of moves. Searching that same position again from scratch wastes work. This guide explains how the engine fingerprints positions with **Zobrist hashing** and caches search results in a **transposition table**.

## What Is a Transposition?

Consider two opening move orders from the starting position:

- Line A: `1. e4 e5 2. Nf3 Nc6`
- Line B: `1. Nf3 Nc6 2. e4 e5`

Both lines reach the exact same position, with the same pieces, the same side to move, and the same castling and en-passant rights. If the engine already analyzed Line A to depth 4, encountering the same position in Line B does not require re-searching the entire subtree.

To reuse work safely, the engine needs two things:

1. A fast, compact key that identifies a chess position (`u64`).
2. A table that stores what the search learned about that key.

## Zobrist Hashing: Fingerprinting a Board in O(1)

Comparing full 64-square boards in a cache is slow and takes too much memory. Instead, **Zobrist hashing** combines pre-assigned 64-bit random-looking numbers using bitwise XOR (`^`).

### Why XOR?

XOR has two properties that make it ideal for board games:

1. **Order independence:** `(A ^ B) ^ C == A ^ (B ^ C)`. Moving pieces in a different order produces the exact same hash.
2. **Self-inverse:** `X ^ X == 0`, so `(H ^ X) ^ X == H`. Removing a key uses the exact same operation as adding it.

```text
Hash before move:          H
Remove White knight on g1: H ^= key(WhiteKnight, g1)
Place White knight on f3:  H ^= key(WhiteKnight, f3)
Flip side to move:         H ^= side_key
```

Instead of rescanning all 64 squares after every move, updating the hash takes only a handful of XOR operations—**constant time O(1)**—and `unmake_move` simply restores the saved `previous_zobrist_key`.

### What Must Be Included in the Key?

Two positions are identical in chess rules only if all rule-relevant state matches. The engine includes four components in [`zobrist.rs`](../engine/src/board/zobrist.rs):

| Component | Key IDs | Why it matters |
| --- | --- | --- |
| Piece placement | `12 piece kinds × 64 squares` (`0..767`) | Piece types, colors, and squares |
| Side to move | `SIDE_KEY_ID = 768` (XORed when Black to move) | Same pieces with Black vs White to move are different positions |
| Castling rights | `CASTLING_KEY_ID = 769..772` (one bit per right `K, Q, k, q`) | Losing a castling right changes future legal moves |
| En-passant square | `EN_PASSANT_KEY_ID = 773..836` (`rank * 8 + file`) | An en-passant capture is legal for only one turn |

Notice that **halfmove clock** and **fullmove number** are intentionally excluded: transpositions reached at different move numbers generally have the same tactical and positional evaluation (though draw rules at 50 moves remain tracked separately on the game board).

### Deterministic Keys Without a Runtime RNG

Rather than storing a large table of random numbers generated at startup, [`zobrist_value`](../engine/src/board/zobrist.rs) hashes each integer ID through a deterministic 64-bit mixing function (SplitMix64-style multiplications and shifts). Every ID always produces the same well-distributed 64-bit value across runs, which makes debugging and mailbox/bitboard equivalence tests completely reproducible.

### Keeping Incremental Updates Exact

Every special rule affects the Zobrist key during `make_move`:

- **Capture:** XOR out the captured piece on its square (or behind the target square for *en passant*).
- **Promotion:** XOR out the pawn on its origin square and XOR in the promoted piece on the destination square.
- **Castling:** XOR out and in both the king and the rook, plus any castling flags that became `false`.
- **Castling rights lost passively:** Moving a rook or capturing an opponent's rook on its corner square clears that corner's castling right, so its castling key must be toggled off too.
- **En passant:** Toggle off any previous `en_passant` square (since it expires after one move), and toggle on the new one if a pawn moved two squares.

In [`BitboardPosition::make_move`](../engine/src/board/bitboard_prototype.rs), the engine toggles out the old rule state (`update_rule_state_hash`) before updating castling/en-passant flags and toggles the new rule state back in afterward. The mailbox `Board::make_move_internal` computes the same XOR changes explicitly. Unit tests verify that incremental updates match a full board recalculation after every legal move.

## How the Transposition Table Works

In [`search.rs`](../engine/src/search.rs), `TranspositionTable` is a flat array of `65,536` (`1 << 16`) optional slots (`TRANSPOSITION_TABLE_SIZE`).

### Indexing and Collisions

To map a 64-bit key to a slot index in `0..65,535`, the table masks the lower 16 bits:

```text
index = key & (table_len - 1)
```

Because `table_len` is a power of two (`65,536`), `key & 0xFFFF` replaces a slower modulo division.

Two risks arise in any fixed-size hash table:

1. **Index collision:** Two different 64-bit keys can share the same lower 16 bits and map to the same slot. Therefore, `probe` always checks `entry.key == key` before trusting a slot.
2. **64-bit hash collision:** Two distinct positions could theoretically produce the exact same 64-bit key (probability ~ $2^{-64}$). In practice this is vanishingly rare, though chess engines treat TT entries cautiously (for example, by not blindly assuming an old best move is legal if a collision ever occurred; here both paths currently share deterministic keys and verified depths).

### Replacement Policy

When two positions map to the same index, the engine uses a **depth-preferred** replacement rule in `TranspositionTable::store`:

```text
store new entry if slot is empty OR new_entry.depth >= old_entry.depth
```

A deeper search represents much more work than a shallow search, so keeping deeper entries avoids overwriting valuable subtrees with cheap depth-1 leaves. (Adding generation aging across moves/games is planned in [Plan 013](plans/013-transposition-table-tuning.md).)

## Score Bounds: Exact, Lower, and Upper

Alpha-beta pruning does not always compute the exact minimax value of a node. Sometimes it stops early because a branch is already too good or too bad relative to `[alpha, beta]`. Storing an approximate score as if it were exact would corrupt later searches.

Each `TranspositionEntry` therefore records a `BoundType`:

| Bound | When stored | Meaning | Safe reuse condition |
| --- | --- | --- | --- |
| `Exact` | `original_alpha < score < beta` | All relevant replies were checked; `score` is the true value at this depth. | Can return `score` immediately. |
| `Lower` | Beta cutoff (`score >= beta`) | Node failed high; true score is **at least** `score` (`true >= score`). | Raise `alpha = max(alpha, score)`; cut off if `alpha >= beta`. |
| `Upper` | No move beat `original_alpha` (`score <= original_alpha`) | Node failed low; true score is **at most** `score` (`true <= score`). | Lower `beta = min(beta, score)`; cut off if `alpha >= beta`. |

### Why Depth Matters on Probe

An entry searched to remaining `depth = 2` cannot safely cut off a request that needs `depth = 5`, because a shallower search might miss a tactical refutation at ply 4 or 5. Thus:

- **Score cutoffs / bound tightening** apply only when `entry.depth >= depth`.
- **Move ordering (`best_move`)** is always useful, even when `entry.depth < depth`! Trying the previously best move first often causes immediate alpha-beta cutoffs in iterative deepening.

## Mate Score Normalization

Checkmate scores create a subtle trap when stored in a transposition table.

Suppose `MATE_SCORE = 30_000`. The search encodes mate distance using the current `ply` from the root so it prefers faster mates:

```text
checkmate at ply 4  ->  mate score = -(30_000 - 4) = -29_996  (for the mated side)
parent at ply 3 sees ->  +29_996  (mate in 1 move)
```

Now suppose the engine stores `+29_996` into the TT while searching a node at `ply = 3`, and later encounters the **same position** at `ply = 5`:

- If it stored `+29_996` raw, at `ply = 5` it would still read "distance = 4 from root" instead of "1 ply below me", which is inconsistent and can even distort mate comparisons.

To make mate scores **position-relative** (distance from the stored node, not from the root), [`score_to_table` and `score_from_table`](../engine/src/search.rs) adjust scores whose magnitude reaches `MATE_SCORE_THRESHOLD` (`29_000`):

```text
On store (at ply):
  if score >=  MATE_SCORE_THRESHOLD: store score + ply
  if score <= -MATE_SCORE_THRESHOLD: store score - ply

On probe (at new_ply):
  if entry.score >=  MATE_SCORE_THRESHOLD: return entry.score - new_ply
  if entry.score <= -MATE_SCORE_THRESHOLD: return entry.score + new_ply
```

With this adjustment, a "mate in 1 from this position" (`+29_999` inside the table) becomes `+29_996` when probed at `ply = 3` and `+29_994` when probed at `ply = 5`—always representing the exact mate distance from the root of the current search.

## Trade-offs and Current Limits

- **Fixed table size:** Currently hardcoded to `65,536` entries (`TRANSPOSITION_TABLE_SIZE`), allocated fresh per `find_best_move_*` call rather than persisted across moves with generation aging.
- **No quiescence TT entries:** Quiescence search currently does not probe or store in the TT, keeping the table focused on full-depth nodes.
- **Correctness verification:** Search tests compare TT-enabled vs TT-disabled runs and verify mate-distance preservation (`test_transposition_table_matches_disabled_search`, `test_transposition_table_preserves_mate_distance_scores`).

## Further Reading in This Repository

- [How the Engine Searches for a Move](search-algorithms.md)
- [Transposition Table plan (006)](plans/006-transposition-table.md)
- [Transposition Table Tuning plan (013)](plans/013-transposition-table-tuning.md)
- [Zobrist implementation](../engine/src/board/zobrist.rs)
- [TranspositionTable and search integration](../engine/src/search.rs)
