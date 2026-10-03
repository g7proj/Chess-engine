# 012 - Tapered Evaluation

## Goal

Replace one static positional model with middlegame/endgame interpolation.

## Scope

- [ ] Define phase weights for remaining material.
- [ ] Add separate middlegame and endgame piece-square terms.
- [ ] Interpolate material and positional terms by phase.
- [ ] Improve pawn structure, passed pawns, mobility, and king safety incrementally.
- [ ] Tune one feature family at a time against the validation corpus.

## Tests

Preserve side-to-move symmetry, exact score equivalence between board representations, and stable signs for documented FEN examples. Record position-suite changes separately from search-speed measurements.

## Completion

The evaluator remains deterministic, concise, representation-independent, and supported by focused score-direction tests.
