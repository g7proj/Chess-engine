# 001 - Search State and Perft

## Status

The game/search move-application split is complete. Converting `perft` to make/unmake remains in scope.

## Goal

Remove game-history overhead from search and make `perft` use the same make/unmake discipline as alpha-beta.

## Scope

- Add a search-specific move application path that does not update repetition history unless required.
- Keep full history updates in the game/UCI path.
- Convert recursive perft from board cloning to make/unmake.
- Preserve restoration of pieces, castling, en passant, clocks, side to move, and history.

## Tests

- Round-trip normal moves, captures, castling, en passant, and promotion.
- Compare clone-based and make/unmake perft counts on all canonical positions.
- Assert the root board is unchanged after perft.

## Completion

Perft counts remain unchanged, no search node clones remain, and benchmark output improves or clearly identifies the next bottleneck.
