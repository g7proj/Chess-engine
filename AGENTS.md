# Repository Guidelines

## Project Structure

- `engine/` contains the Rust chess engine and UCI executable.
- `engine/src/board/` implements board state, FEN, pieces, colors, and draw rules.
- `engine/src/moves/` implements move generation, legality, attacks, and make/unmake.
- `engine/src/search.rs` contains negamax alpha-beta search and move ordering.
- `engine/src/uci/` handles UCI commands, CLI perft, and divide output.
- `engine/tests/` contains integration, regression, perft, and benchmark tests.
- `ml/` contains learning experiments and is separate from the current engine path.

## Build, Test, and Development Commands

Run commands from `engine/`:

```bash
cargo build                         # Debug build
cargo build --release               # Optimized build
cargo fmt --all -- --check          # Verify formatting
cargo test                          # Run the complete test suite
cargo run                            # Start the UCI loop
cargo run -- --perft 4               # Run CLI perft
cargo run -- --divide 3              # Run CLI divide
cargo test --test search_bench -- --ignored --nocapture
```

The canonical Kiwipete perft test is intentionally slow. Benchmark tests are ignored during normal runs because timing depends on the machine.

## Coding Style and Naming

Use Rust 2024 conventions, four-space indentation, `snake_case` for functions and variables, `PascalCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Run `cargo fmt` before submitting changes. Keep implementation concise, add Rustdoc `///` comments to public or non-obvious functions, and avoid comments that merely restate code. All the comments and variable names should be in English. The code should be clear, readable, correct and efficient.

## Testing Guidelines

Use Rust's built-in test framework. Name tests `test_<behavior>` and keep rule changes covered by focused unit tests plus perft regression cases. For move-state changes, verify both the resulting position and make/unmake restoration, including castling, en passant, and promotion.

## Commits and Pull Requests

Use short, imperative commit subjects such as `Add alpha-beta search` or `Update docs`. Keep commits focused. Pull requests should explain the behavioral change, list validation commands and benchmark results, and call out any intentional performance or UCI changes. Include relevant FEN positions or perft counts when changing chess rules.

## Architecture Notes

Preserve the separation between board rules, move generation, search, and UCI handling. Prefer correctness-tested incremental improvements over introducing a second engine or changing board representation without profiling evidence.

## Roadmap and Plans

Use `ROADMAP.md` for milestone status and `docs/plans/` for implementation details. Keep both updated when a planned feature changes scope, dependencies, or completion status.
