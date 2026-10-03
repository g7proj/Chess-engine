# Chess Engine

Rust chess engine with a standard UCI interface. It can be connected to any UCI-compatible GUI such as Cute Chess, Arena, or SCID.

## What is in the repo

- `engine/` - the Rust chess engine
  - board representation
  - move generation
  - FEN load/save
  - legal move filtering
  - check, checkmate, and stalemate detection
  - castling, en passant, and promotion support
  - iterative deepening alpha-beta search with PV ordering, quiescence search, make/unmake traversal, and search statistics
- `ml/` - learning experiments

## Main features

- standard starting position
- load positions from FEN
- generate legal moves
- apply UCI moves
- support castling, en passant, and promotion
- detect check, checkmate, and stalemate
- track repetition draws and the 50-move rule
- write logs to `engine_debug.log`

## Requirements

- Rust toolchain with `cargo`

## Build

```bash
cd engine
cargo build
```

For a release build:

```bash
cd engine
cargo build --release
```

The compiled binary is placed in:

- `engine/target/debug/engine`
- `engine/target/release/engine`

## Command Line Mode

You can run `perft` and `divide` directly from the binary without entering UCI mode.

```bash
cd engine
cargo run -- --perft 4
cargo run -- --divide 3
cargo run -- --perft 3 --fen "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1"
```

If `--fen` is omitted, the start position is used.

## How to use it

The engine speaks UCI. You can run it from the terminal or connect it to a UCI GUI.

### Run from terminal

```bash
cd engine
cargo run
```

Then send UCI commands through stdin:

```text
uci
isready
ucinewgame
position startpos moves e2e4 e7e5
go
quit
```

### Use with a GUI

1. Build the engine.
2. Open your UCI-compatible GUI.
3. Add a new engine and point it to the compiled binary.
4. The GUI will send commands such as `uci`, `isready`, `position`, and `go`.

## Supported UCI commands

- `uci`
- `isready`
- `ucinewgame`
- `position startpos ...`
- `position fen ...`
- `go`
- `go perft <depth>`
- `go divide <depth>`
- `stop`
- `ponderhit`
- `setoption name <name> value <value>`
- `perft <depth>`
- `divide <depth>`
- `quit`

Note: the engine supports standard UCI only. Older custom commands from the removed Python UI are no longer part of the workflow.

## Examples

Start position:

```text
position startpos
go
```

FEN position:

```text
position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
go
```

Position with moves:

```text
position startpos moves e2e4 e7e5 g1f3
go
```

Perft debug:

```text
perft 3
divide 3
```

`divide` output is indented for readability, with one line per root move and a total summary.
UCI mode now prints standard `info depth ... nodes ... nps ... pv ...` lines for `perft` and `divide`.

## Tests

```bash
cd engine
cargo test
```

The test suite covers:

- castling
- en passant
- check and checkmate
- FEN parsing and generation
- legal move generation
- basic UCI parsing
- canonical perft regression positions up to depth 4

There is also an ignored benchmark test:

```bash
cd engine
cargo test --test perft_bench -- --ignored --nocapture
```

It measures `perft` speed on known positions and prints nodes-per-second.

Search performance can be measured with the ignored make/unmake benchmark:

```bash
cargo test --test search_bench -- --ignored --nocapture
```

The benchmark checks that the search returns a move and restores the board after every iteration. It reports elapsed time for depths 3 and 4; it is ignored during normal test runs because timing-based tests are machine-dependent.

UCI `go` searches progressively to the current fixed maximum depth and reports depth, score, nodes, quiescence nodes, NPS, elapsed time, alpha-beta and transposition-table cutoffs/hits, and the principal variation.
`go movetime <milliseconds>` limits iterative deepening between completed iterations. Search runs on a worker, so `stop` can be received while searching and returns the last completed iteration.

## Notes

- Static evaluation combines material, pawn/knight piece-square bonuses, bishop pair, pseudo-legal mobility, center control, pawn structure, minor-piece development, and king shield/attack pressure. Scores are centipawns from the side-to-move perspective; evaluation weights are initial heuristics and need tuning against a position suite.
- Current search is iterative-deepening alpha-beta/negamax with quiescence search, a 65,536-slot Zobrist transposition table (enabled by default), and capture, promotion, check, PV, and transposition-move ordering. Rust callers can disable the table through `SearchLimits`.
- FEN `fullmove number` handling is present, but should be revisited if game-state logic gets expanded.
