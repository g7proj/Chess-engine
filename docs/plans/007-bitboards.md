# 007 - Bitboards

## Goal

Evaluate whether bitboards provide a worthwhile speedup after profiling.

## Scope

Measure time spent in move generation, attack detection, make/unmake, and search. Only then prototype bitboards behind focused tests or a separate representation module.

## Constraints

Do not replace the current representation without perft equivalence, make/unmake equivalence, and benchmark evidence. Keep the rule model understandable while comparing implementations.

## Completion

Adopt bitboards only if the measured gain justifies the added complexity and all canonical perft counts remain unchanged.
