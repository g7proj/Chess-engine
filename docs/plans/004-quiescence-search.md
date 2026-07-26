# 004 - Quiescence Search

## Goal

Avoid evaluating tactically unstable positions at the normal search horizon.

## Scope

At the depth boundary, run a stand-pat evaluation and continue with legal captures, promotions, and checks. Reuse alpha-beta bounds and make/unmake. Add a separate quiescence node counter.

## Tests

- Use positions with hanging pieces, forced recaptures, and promotions.
- Verify that quiet positions terminate at stand pat.
- Verify no illegal capture or self-check is introduced.
