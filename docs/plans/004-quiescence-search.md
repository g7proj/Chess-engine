# 004 - Quiescence Search

## Status

Complete for captures, en passant, and promotions. Quiet checks are deferred.

## Goal

Avoid evaluating tactically unstable positions at the normal search horizon.

## Scope

At the depth boundary, run a stand-pat evaluation and continue with legal captures, en passant, and promotions. Reuse alpha-beta bounds and make/unmake. Track quiescence nodes separately while including them in total search nodes.

## Tests

- Use positions with hanging pieces, forced recaptures, and promotions.
- Verify quiet positions terminate at stand pat and checked positions search legal evasions.
- Verify checkmate detection and board restoration after tactical search.
- Verify captures, en passant, promotions, and self-check legality.
