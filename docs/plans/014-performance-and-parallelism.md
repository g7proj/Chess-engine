# 014 - Performance and Parallelism

## Goal

Improve measured throughput without weakening correctness or reproducibility.

## Scope

- [ ] Profile default bitboard search on the fixed position suite.
- [ ] Optimize slider attacks only if profiling identifies them as material cost.
- [ ] Remove demonstrated allocation or sorting overhead.
- [ ] Prototype root-parallel search with deterministic single-thread fallback.
- [ ] Consider Lazy SMP only after root parallelism has reliable time and stop handling.

## Rules

Do not introduce magic bitboards, PEXT, or multithreading from assumption alone. Record command, hardware context, position suite, nodes, NPS, elapsed time, and correctness checks for every claimed improvement.

## Completion

Each optimization has a benchmark result, focused tests, and a documented fallback path.
