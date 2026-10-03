use crate::constants::{FILES, RANKS};
use std::collections::HashMap;

#[cfg(test)]
pub(crate) mod bitboard_prototype;
pub mod color;
pub mod fen;
pub mod helpers;
pub mod piece;
pub mod position;
pub(crate) mod zobrist;

pub use color::Color;
pub use piece::Piece;

#[derive(Debug, Clone)]
pub struct Board {
    pub squares: [[Piece; FILES]; RANKS],
    pub side_to_move: Color,
    pub en_passant: Option<(usize, usize)>,
    pub white_kingside_castle: bool,
    pub white_queenside_castle: bool,
    pub black_kingside_castle: bool,
    pub black_queenside_castle: bool,
    pub history: HashMap<String, usize>,
    pub halfmove_clock: usize,
    pub fullmove_number: usize,
    pub(crate) zobrist_key: u64,
    #[cfg(test)]
    pub(crate) bitboard_shadow: Option<bitboard_prototype::BitboardPosition>,
}

impl Board {
    pub fn new() -> Self {
        use Piece::*;
        let squares: [[Piece; FILES]; RANKS] = [
            [
                RookWhite,
                KnightWhite,
                BishopWhite,
                QueenWhite,
                KingWhite,
                BishopWhite,
                KnightWhite,
                RookWhite,
            ],
            [PawnWhite; FILES],
            [Empty; FILES],
            [Empty; FILES],
            [Empty; FILES],
            [Empty; FILES],
            [PawnBlack; FILES],
            [
                RookBlack,
                KnightBlack,
                BishopBlack,
                QueenBlack,
                KingBlack,
                BishopBlack,
                KnightBlack,
                RookBlack,
            ],
        ];
        let mut board = Board {
            squares,
            side_to_move: Color::White,
            en_passant: None,
            white_kingside_castle: true,
            white_queenside_castle: true,
            black_kingside_castle: true,
            black_queenside_castle: true,
            history: HashMap::new(),
            halfmove_clock: 0,
            fullmove_number: 1,
            zobrist_key: 0,
            #[cfg(test)]
            bitboard_shadow: None,
        };
        board.refresh_zobrist_key();
        board
    }

    /// Returns the deterministic Zobrist key for the current position.
    pub fn zobrist_key(&self) -> u64 {
        self.calculate_zobrist_key()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_board_initialization() {
        let board: Board = Board::new();
        print!("{}", board);
        assert_eq!(board.side_to_move, Color::White);
        assert_eq!(board.en_passant, None);
        assert_eq!(board.white_kingside_castle, true);
        assert_eq!(board.white_queenside_castle, true);
        assert_eq!(board.black_kingside_castle, true);
        assert_eq!(board.black_queenside_castle, true);
        assert_eq!(board.fullmove_number, 1);

        // Check kings position
        assert_eq!(board.squares[0][4], Piece::KingWhite);
        assert_eq!(board.squares[7][4], Piece::KingBlack);
    }
}
