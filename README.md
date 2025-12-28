# Chess Project

- engine/ – Rust engine
- ui/ – (legacy) Python UI (archived)
- ml/ – learning experiments
- protocol/ – communication specs

How to use the engine with an external GUI (Cute Chess / Arena)
--------------------------------------------------------------

1) Build the engine (requires Rust toolchain / cargo)

   cd engine
   cargo build

   The compiled binary will be at `engine/target/debug/engine` (or `.../release/engine` for release).

2) Load engine into Cute Chess (or another UCI-compatible GUI)

- Open Cute Chess (or Arena/SCID).
- Add a new engine and point it to the compiled executable above.
- Use standard UCI workflow: the GUI will send `uci`, `isready`, `ucinewgame`, `position` and `go`.

3) Notes and testing

- This engine implements the standard UCI commands only. Custom commands previously used by the Python GUI (e.g. `listmoves`, `showfen`) were removed.
- To verify special moves (castling, en-passant, promotions) test sequences in the GUI and confirm the engine returns `bestmove` correctly.

If you want a lightweight client to exercise the engine from Python, consider using `python-chess` to build a small test harness that translates user moves into `position ... moves ...` and parses `bestmove` responses.
