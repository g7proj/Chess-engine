# Board Rules, Make/Unmake, and Perft Validation

Before a chess engine can search millions of nodes per second, its rules must be **100% exact**. Missing a single edge case—such as losing a castling right when a rook is captured, or an *en passant* capture exposing the king to a horizontal check—will silently corrupt every search tree built on top of it. This guide explains how the engine models board state, applies and reverses moves with **make/unmake**, tracks draws, and proves rule correctness using **perft** and **divide**.

## Board State and FEN

A chess position is more than just where the 32 pieces sit. Two boards with identical piece placement can still have different legal moves. In [`board/mod.rs`](../engine/src/board/mod.rs), `Board` stores:

- `squares: [[Piece; 8]; 8]` — the 8×8 mailbox grid.
- `side_to_move` — `White` or `Black`.
- `white_kingside_castle`, `white_queenside_castle`, `black_kingside_castle`, `black_queenside_castle` — remaining castling rights.
- `en_passant: Option<(usize, usize)>` — the target square behind a pawn that just made a two-square advance (valid for one turn only).
- `halfmove_clock` — plies since the last pawn move or capture (for the 50-move draw rule).
- `fullmove_number` — turn counter, incremented after Black moves.
- `history: HashMap<String, usize>` — counts how many times each position occurred in the game (for threefold repetition).
- `zobrist_key: u64` — incremental hash fingerprint used by the transposition table.

[`board/fen.rs`](../engine/src/board/fen.rs) serializes and parses all six standard **FEN (Forsyth–Edwards Notation)** fields, for example the starting position:

```text
rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
|<------------- piece placement ----------->| | |  | | |
                                         turn-+ |  | | +- fullmove number
                                 castling rights+  | +--- halfmove clock
                                      en-passant---+
```

## Pseudo-Legal Generation vs. Legal Filtering

Generating moves usually happens in two stages:

1. **Pseudo-legal move generation** ([`moves/mod.rs`](../engine/src/moves/mod.rs)): each piece generates moves matching its movement pattern without checking whether the friendly king ends up in check.
   - **Pawns** ([`moves/pawn.rs`](../engine/src/moves/pawn.rs)): single push, double push from the starting rank, diagonal captures, *en passant*, and expanding reaching the last rank into four promotion moves (`Queen`, `Rook`, `Bishop`, `Knight`).
   - **Kings & Castling** ([`moves/king.rs`](../engine/src/moves/king.rs)): normal one-square steps plus kingside/queenside castling, which additionally verify that the path squares are empty and that neither the origin, transit, nor destination square is under attack.
2. **Legal filtering**: for each pseudo-legal move, the engine applies the move, tests `!is_in_check(moving_color)`, and immediately undoes the move.

Why filter by trial make/unmake? It handles subtle rules automatically without special-case pin solvers—most famously the **horizontal en-passant discovered check**:

```text
Rank 5:  .  .  .  K  p  P  .  r    (White pawn on f5 captures e5 e.p. -> e6)
```

Even though the Black rook on `h5` was blocked by *both* pawns before the capture, removing both pawns opens the 5th rank and puts the White king on `d5` in check. Testing `!is_in_check` after applying the move catches this automatically.

## Make/Unmake: Traversing the Tree Without Cloning

During search, the engine explores tens of thousands of positions per second. Cloning the entire `Board` (including its `HashMap` repetition history!) at every node would allocate memory constantly and hurt cache locality.

Instead, [`moves/apply.rs`](../engine/src/moves/apply.rs) mutates one `Board` in place using **make/unmake**:

1. `make_move_for_search(mv)` saves only the information that cannot be inferred from the move alone into a compact `Undo` struct, updates the board and `zobrist_key`, and returns `Undo`.
2. After the recursive search of that child returns, `unmake_move(undo)` restores the board to the exact state before the move.

### What `Undo` Must Remember

Reversing a normal move is easy, but chess rules have irreversible state changes that must be saved explicitly:

- **Captured piece** (`captured_piece`), which may also be an **en-passant pawn** sitting on a different rank from the destination square (`en_passant_capture`).
- **Castling rook movement** (`rook_move`), so undoing `e1g1` moves the rook back from `f1` to `h1`.
- **Previous castling rights** (`previous_castling`): castling rights are lost not only when a king or rook moves, **but also when an opponent captures a rook on its starting corner square** (`a1`, `h1`, `a8`, `h8`).
- **Previous en-passant square** (`previous_en_passant`), because any en-passant target expires after one turn even if unused.
- **Clocks and Zobrist key** (`previous_halfmove_clock`, `previous_fullmove_number`, `previous_zobrist_key`).

### Game Path vs. Search Path

`Board` exposes two entry points:

- `make_move(mv)` — used by the UCI `position ... moves ...` game path; updates the `history: HashMap<String, usize>` repetition table (`record_position`) and stores `position_key` in `Undo`.
- `make_move_for_search(mv)` — skips `HashMap` string formatting and allocation at every search node (see [Plan 001](plans/001-search-state-and-perft.md)).

The bitboard search representation ([`BitboardPosition`](../engine/src/board/bitboard_prototype.rs)) goes one step further: it omits `history`, `halfmove_clock`, and `fullmove_number` altogether, storing a `Copy` `BitboardUndo` on the stack with zero heap allocations.

## Draw Rules

[`board/position.rs`](../engine/src/board/position.rs) implements three standard draw detections:

1. **Threefold repetition (`is_repetition_draw`):** `repetition_key()` encodes piece placement, active color, castling rights, and en-passant square (excluding halfmove/fullmove counters, which do not define a position). A count `>= 3` signals a repetition draw.
2. **Fifty-move rule (`is_50_move_draw`):** triggers when `halfmove_clock >= 100` (100 half-moves = 50 full moves by each side without a pawn push or capture).
3. **Insufficient material (`is_insufficient_material`):** returns `true` when neither side can force checkmate (`K vs K`, `K + minor vs K`, `K + minor vs K + minor`, or `K + 2 knights vs K` against no pawns/rooks/queens/bishops).

## Perft and Divide: Proving Move Generation Correctness

How do you know a move generator and make/unmake implementation has **zero** bugs across millions of edge cases? By running **perft** (*performance test*).

### How `perft` Works

`Board::perft(depth)` counts the exact number of legal leaf positions reachable in `depth` plies from a given board:

```text
perft(0) = 1
perft(1) = number of legal moves (bulk count)
perft(d) = sum over legal moves m of perft(d - 1 after make(m), then unmake(m))
```

Notice the **bulk-counting** shortcut at `depth == 1`: instead of calling `make_move` and `unmake_move` for every leaf move just to return `1`, `perft(1)` returns `moves.len() as u64` directly.

For standard reference positions (like the starting position or **Kiwipete**), exact node counts at every depth are mathematically known and published across the chess-programming community:

- Starting position:
  - `perft(1) = 20`
  - `perft(2) = 400`
  - `perft(3) = 8,902`
  - `perft(4) = 197,281`
  - `perft(5) = 4,865,609`

If even **one** castling check, en-passant edge case, promotion, or `unmake_move` restoration is wrong, the total node count at depth 3–5 diverges immediately.

### Isolating Bugs with `divide`

When a `perft(4)` count is off by 12 nodes, staring at `197,269` vs `197,281` doesn't tell you which rule failed. **`divide`** solves this by printing the subtree count separately for each legal root move:

```text
e2e4: 13160
d2d4: 12800
...
```

Debugging becomes a fast drill-down process:

1. Run `--divide 4` (or `go divide 4` in UCI) and compare each root move's count against a known-good engine.
2. Pick the **first** root move whose count differs.
3. Apply that move (`position startpos moves <move>`) and run `divide 3`.
4. Repeat until `depth == 1`, where the exact missing or illegal move is immediately visible.

You can run both directly from the CLI:

```bash
cd engine
cargo run -- --perft 4
cargo run -- --divide 3
cargo run -- --perft 3 --fen "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1"
```

## Further Reading in This Repository

- [Search State and Perft plan (001)](plans/001-search-state-and-perft.md)
- [Bitboards vs. Mailbox guide](bitboards.md)
- [Move application (`make_move` / `unmake_move`)](../engine/src/moves/apply.rs)
- [Legal move generation and `perft`](../engine/src/moves/mod.rs)
- [Draw detection](../engine/src/board/position.rs)
