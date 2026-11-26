# Bitboards vs. Mailbox Representation

A chess engine spends most of its time answering simple questions: *Which squares have white pawns? Is the king in check? Which squares are empty along this diagonal?* How the board is stored in memory decides how fast those questions can be answered. This guide explains the two board representations in this repository—**mailbox** and **bitboards**—and how bitboard operations work.

## Two Ways to Store a Chessboard

### Mailbox (`Board`)

In [`board/mod.rs`](../engine/src/board/mod.rs), `Board` stores the position as an 8×8 grid:

```rust
pub squares: [[Piece; 8]; 8]
```

This is called a **mailbox** representation: each square index `(rank, file)` acts like a numbered mailbox holding either `Piece::Empty` or a specific piece.

- **Strengths:** Simple, intuitive to read and debug, natural for FEN parsing and rule book-keeping (`history`, `halfmove_clock`, `fullmove_number`).
- **Weaknesses:** Finding all knights, pawns, or occupied squares often means scanning all 64 cells in loops. Checking whether a square is attacked in [`moves/attacks.rs`](../engine/src/moves/attacks.rs) inspects rays and offsets against the 2D array repeatedly.

### Bitboards (`BitboardPosition`)

Because a chessboard has **64 squares** and a modern CPU has **64-bit integer registers (`u64`)**, an entire set of squares fits inside a single number. Each bit (`0`..`63`) represents one square: `1` means present, `0` means absent.

In [`BitboardPosition`](../engine/src/board/bitboard_prototype.rs), the board is represented by:

```rust
pieces: [u64; 12],    // One 64-bit mask per (Color, PieceType)
occupancy: [u64; 2],  // Combined mask of all White pieces [0] and Black pieces [1]
side_to_move: Color,
en_passant: Option<(usize, usize)>,
castling: [bool; 4],
zobrist_key: u64,
```

## Square-to-Bit Mapping

Every square `(rank, file)` with `rank` in `0..8` (`1`..`8`) and `file` in `0..8` (`a`..`h`) maps to bit index:

```text
square_index = rank * 8 + file
bit_mask     = 1_u64 << square_index
```

Visually (little-endian rank-file mapping):

```text
rank 7 | 56 57 58 59 60 61 62 63   (a8 .. h8)
rank 6 | 48 49 50 51 52 53 54 55
  ...
rank 1 |  8  9 10 11 12 13 14 15   (a2 .. h2)
rank 0 |  0  1  2  3  4  5  6  7   (a1 .. h1)
          a  b  c  d  e  f  g  h
```

For example, White's starting pawns sit on rank 1 (bits `8..15`), so `pieces[0]` is `0x0000_0000_0000_FF00`. White's king on `e1` (`rank = 0, file = 4`) is bit `4` (`1 << 4 = 16`).

## Core Bitwise Operations

Bitboards turn loops over 64 squares into single CPU instructions:

- **Combine two sets (union):** `all_pieces = occupancy[0] | occupancy[1]`
- **Test membership / overlap (intersection):** `(enemy & target_bit) != 0`
- **Add a piece to a square:** `mask |= 1_u64 << sq`
- **Remove a piece from a square:** `mask &= !(1_u64 << sq)`
- **Iterate only over active pieces (bit-scanning):**

```rust
let mut pieces = self.occupancy[color_index];
while pieces != 0 {
    let sq = pieces.trailing_zeros() as usize; // Index of lowest set bit
    pieces &= pieces - 1;                      // Clear that lowest set bit
    // ... process piece at `sq` ...
}
```

### How `trailing_zeros()` and `x & (x - 1)` Work

1. `pieces.trailing_zeros()` counts the `0` bits below the least significant `1` bit (a single hardware instruction, `TZCNT`/`BSF`, on modern CPUs), immediately giving the square index of the next piece without visiting empty squares.
2. `pieces & (pieces - 1)` clears only the lowest `1` bit. Subtracting `1` flips the lowest `1` to `0` and all lower `0`s to `1`; ANDing with the original value zeroes them all out.

```text
x         = 0b0101_1000
x - 1     = 0b0101_0111
x & (x-1) = 0b0101_0000   (lowest set bit cleared)
```

If White has 10 pieces left, the loop runs exactly 10 iterations instead of 64.

## Attack Detection and Move Generation

### Leapers (Knights, Kings, Pawns)

Non-sliding pieces move to fixed offsets. In [`bitboard_prototype.rs`](../engine/src/board/bitboard_prototype.rs):

- `is_square_attacked` checks knight, king, and pawn attackers in parallel using bit masks:
  - `self.pieces[color_offset + 1] & knight_attacks(target) != 0` tests **all** enemy knights at once against a single mask of squares from which a knight attacks `target`.
  - The same idea applies to `king_attacks(target)` and `pawn_attacks(target, by_color)`.
- Checking if a king is in check (`is_in_check`) locates the king in O(1) via `self.pieces[king_index].trailing_zeros()` instead of scanning the board for the king first.

### Sliders (Bishops, Rooks, Queens)

Sliding pieces move along rays until blocked by `occupancy = occupancy[0] | occupancy[1]`:

- During move generation (`generate_sliding_moves`), rays step outward until hitting a friendly piece (`own & bit != 0`, stop before square) or an enemy piece (`enemy & bit != 0`, include capture then stop).
- During square-control queries (`evaluation_controls_square`), only the piece masks of the relevant color are iterated via `trailing_zeros()`, and rays stop as soon as they hit any blocker in `occupancy`.

> [!NOTE]
> More advanced engines replace ray loops with precomputed **magic bitboards** or **PEXT** lookup tables for O(1) slider attacks. Per [Plan 014](plans/014-performance-and-parallelism.md), this engine deliberately keeps simple ray stepping until profiling shows slider generation is a major bottleneck.

## Incremental Make/Unmake on Bitboards

When `BitboardPosition::make_move` applies a move, it updates `pieces`, `occupancy`, `castling`, `en_passant`, `side_to_move`, and `zobrist_key` in place, returning a `BitboardUndo` value on the stack. `unmake_move` reverses those mask updates directly without copying the board.

Unlike `Board`, `BitboardPosition` omits `history`, `halfmove_clock`, and `fullmove_number`: the UCI/game layer tracks draw rules on `Board`, and `BitboardPosition::from_board` converts the root position once at the start of a search.

## The Oracle Pattern: Proving Correctness Before Switching

Introducing bitboards directly as a rewrite risks subtle bugs in castling, en passant, or promotions. Instead, the project followed a staged validation process recorded in [Plan 007](plans/007-bitboards.md):

1. **Attack prototype:** Verified bitboard `is_square_attacked` against mailbox across every square of canonical positions.
2. **Shadow state (`#[cfg(test)]`):** Synchronized a `bitboard_shadow` inside `Board::make_move` and asserted `matches_board` after every make/unmake.
3. **Standalone search comparison:** Ran full iterative-deepening alpha-beta + TT on both mailbox and `BitboardPosition` across 10 diverse positions, asserting identical best moves, PVs, scores, node counts, cutoffs, and TT hit counts.
4. **Cargo feature switch:** Shipped behind `bitboard-search` and promoted it to the default Cargo feature only after equivalence and release benchmarks passed (cutting 10-position release search time from ~1.37 s to ~0.62 s, while keeping `--no-default-features` available as the mailbox reference oracle).

## Further Reading in This Repository

- [Bitboards implementation plan (007)](plans/007-bitboards.md)
- [Performance and Parallelism plan (014)](plans/014-performance-and-parallelism.md)
- [Bitboard position source](../engine/src/board/bitboard_prototype.rs)
- [Mailbox board source](../engine/src/board/mod.rs)
