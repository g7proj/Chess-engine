# Documentation

Teaching guides explain a technique for readers who are new to it: purpose, core idea, main steps, and trade-offs. Implementation plans record the scope, status, tests, and benchmarks for a change.

## Teaching guides

- [How the Engine Searches for a Move](search-algorithms.md) explains negamax, alpha-beta pruning, iterative deepening, quiescence search, move ordering (MVV-LVA, killers, history), and PV construction.
- [How the Engine Evaluates a Position](position-evaluation.md) explains material, positional terms, score perspective, and the current evaluation limits.
- [Zobrist Hashing and the Transposition Table](transposition-table.md) explains incremental XOR hashing, hash-table indexing, bound types (`Exact`, `Lower`, `Upper`), replacement, and ply-relative mate score normalization.
- [Bitboards vs. Mailbox Representation](bitboards.md) explains 64-bit board masks, bit-scanning, attack detection, incremental updates, and the oracle validation pattern.
- [Board Rules, Make/Unmake, and Perft Validation](board-and-perft.md) explains FEN state, pseudo-legal vs. legal filtering, reversible `Undo` state, draw rules, and `perft`/`divide` debugging.
- [UCI Protocol and Asynchronous Search Loop](uci-architecture.md) explains the non-blocking `stdin` reader, search worker thread, `AtomicBool` cancellation, and race-free `stop`/`quit` handling.

## Implementation plans

Plans live in [`plans/`](plans/README.md). [`ROADMAP.md`](../ROADMAP.md) tracks which milestones are done and which come next.

- [001 - Search State and Perft](plans/001-search-state-and-perft.md)
- [002 - Search Observability](plans/002-search-observability.md)
- [003 - Iterative Deepening](plans/003-iterative-deepening.md)
- [004 - Quiescence Search](plans/004-quiescence-search.md)
- [005 - Evaluation](plans/005-evaluation.md)
- [006 - Transposition Table](plans/006-transposition-table.md)
- [007 - Bitboards](plans/007-bitboards.md)
- [008 - Move Ordering Heuristics](plans/008-move-ordering.md)
- [009 - UCI Time Management](plans/009-uci-time-management.md)
- [010 - Engine Validation](plans/010-engine-validation.md)
- [011 - Search Pruning](plans/011-search-pruning.md)
- [012 - Tapered Evaluation](plans/012-tapered-evaluation.md)
- [013 - Transposition Table Tuning](plans/013-transposition-table-tuning.md)
- [014 - Performance and Parallelism](plans/014-performance-and-parallelism.md)
