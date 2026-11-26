# UCI Protocol and Asynchronous Search Loop

Chess GUIs (such as Cute Chess, Arena, or SCID) communicate with chess engines over standard input/output using the **Universal Chess Interface (UCI)** text protocol. One tricky design requirement of UCI is responsiveness: while the engine is busy searching a position, the GUI must still be able to send `stop` or `quit` at any moment without freezing. This guide explains how [`uci/mod.rs`](../engine/src/uci/mod.rs) handles UCI commands and runs searches concurrently.

## How a UCI Session Works

UCI keeps the GUI in charge of the game state. A typical interaction over `stdin`/`stdout` looks like this:

```text
GUI    -> Engine: uci
Engine -> GUI:    id name Rust Chess Engine
                  id author Denis Altomare
                  uciok
GUI    -> Engine: isready
Engine -> GUI:    readyok
GUI    -> Engine: position startpos moves e2e4 e7e5
GUI    -> Engine: go movetime 1000
Engine -> GUI:    info depth 3 score cp 25 nodes 1420 nps 142000 time 10 pv g1f3 b8c6 f1b5
                  info string qnodes 410 cutoffs 280 tthits 12 ttcutoffs 5
                  bestmove g1f3
```

Because the engine rebuilds the position from `position ...` before each `go`, search bugs cannot leak corrupt board state across moves, and GUI tools can inspect any mid-game position directly.

## Why Search Cannot Block `stdin`

If the main loop read a line, ran `find_best_move(...)` synchronously on the main thread, and only read the next line afterward:

1. A `stop` command sent by the GUI while the engine is thinking would sit unread in the OS pipe buffer until the search finished on its own.
2. `quit` during a search would hang the GUI.

To avoid blocking, [`run_uci`](../engine/src/uci/mod.rs) separates **input reading**, **command dispatch**, and **tree search** across threads:

```text
+---------------+       mpsc::channel       +--------------------+
| Stdin Reader  | ------------------------> |   UCI Main Loop    |
|   (thread)    |      command strings      |  (polls 10 ms if   |
+---------------+                           |   search active)   |
                                            +---------+----------+
                                                      |
                   spawn on `go` (board.clone())      | Arc<AtomicBool>
                   +----------------------------------+ (stop flag)
                   v
         +-------------------+    mpsc::channel     (result)
         |   Search Worker   | -------------------> back to Main Loop
         |     (thread)      |                      -> prints `info` + `bestmove`
         +-------------------+
```

### Step-by-Step Lifecycle

1. **Dedicated `stdin` reader thread:** Continuously reads lines from `std::io::stdin()` and forwards them through an `mpsc::channel<String>`.
2. **Idle vs. active polling:**
   - When **no search is running**, the main loop blocks cleanly on `input_rx.recv()`, using zero CPU.
   - When **a search is active** (`active_search = Some((receiver, stop_flag))`), the main loop first checks `receiver.try_recv()` for a completed result, then waits up to `10 ms` on `input_rx.recv_timeout(...)`. This keeps CPU usage low while waking up immediately on `stop`, `quit`, or search completion.
3. **Isolated worker board:** On `go`, the engine clones `board` into the worker thread. Searching on a clone guarantees that even an interrupted search cannot leave half-applied moves on the UCI game board.
4. **Cooperative cancellation (`Arc<AtomicBool>`):** Both `negamax` and `quiescence` periodically call `search_should_stop(limits, deadline)`, which checks the `AtomicBool` flag and the optional `Instant` deadline.
5. **Safe fallback on early stop:** Iterative deepening updates `SearchResult` only when an iteration completes (`complete == true`). Before starting depth 1, it initializes a legal fallback move from the root move list—so even an immediate `stop` at `0 ms` emits a valid `bestmove` instead of `bestmove (none)` in non-terminal positions.

## Handling `stop` and `quit` Without Race Conditions

When `stop` arrives while `active_search` is `Some((receiver, stop))`:

1. `active_search.take()` takes ownership of the active handle so a second `stop` or a late `try_recv()` cannot print `bestmove` twice.
2. `stop.store(true, Ordering::Relaxed)` signals the worker to unwind.
3. `receiver.recv()` waits for the worker to finish unwinding and emit its last completed `SearchResult`, then prints `info ...` and `bestmove ...` exactly once.

The `quit` handler performs the same orderly shutdown before breaking out of the loop.

## Current Limits and Planned Extensions

Right now, `parse_search_limits` reads `movetime <ms>` and calls `find_best_move_iterative_with_limits` with a fixed maximum depth of `3`, while `setoption` stores options in a `HashMap` and `ponderhit` is a placeholder. Extending `go` to support `wtime`, `btime`, `winc`, `binc`, `depth`, `nodes`, `infinite`, validated UCI options (`Hash`, diagnostics), and clock-aware time allocation is tracked in [Plan 009](plans/009-uci-time-management.md).

## Further Reading in This Repository

- [UCI Time Management plan (009)](plans/009-uci-time-management.md)
- [Engine Validation plan (010)](plans/010-engine-validation.md)
- [UCI and CLI implementation](../engine/src/uci/mod.rs)
- [Search entry points and `SearchLimits`](../engine/src/search.rs)
