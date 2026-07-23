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
  - alpha-beta search with basic move ordering and make/unmake traversal
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

## Notes

- The engine uses a simple material evaluation.
- Current search is alpha-beta/negamax with simple capture, promotion, and check ordering.
- FEN `fullmove number` handling is present, but should be revisited if game-state logic gets expanded.
