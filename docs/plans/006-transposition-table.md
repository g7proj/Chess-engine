# 006 - Transposition Table

## Goal

Reuse scores for positions reached through different move orders.

## Scope

- Add incremental Zobrist keys for pieces, side, castling, and en passant.
- Store key, depth, score, bound type, and best move.
- Probe before searching and store results after search.
- Use the stored best move for move ordering.

## Dependencies

Requires stable make/unmake and clear score/bound semantics.

## Tests

Verify key changes for every relevant state component, restoration after undo, and identical results with the table disabled or enabled.
