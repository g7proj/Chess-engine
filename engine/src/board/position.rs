use crate::{board::Piece::{self, *}, constants::{FILES, RANKS}};

use super::Board;
use std::collections::HashMap;

impl Board {
    pub fn record_position(&mut self) {
        let fen: String = self.to_fen();
        let position_counter: &mut usize = self.history.entry(fen).or_insert(0);
        *position_counter += 1;
    }

    pub fn is_repetition_draw(&self) -> bool {
        let fen: String = self.to_fen();
        if let Some(&count) = self.history.get(&fen) {
            count >= 3
        } else {
            false
        }
    }

    pub fn is_50_move_draw(&self) -> bool {
        self.halfmove_clock >= 100
    }

    pub fn clear_position_history(&mut self) {
        self.history.clear();
    }

    pub fn is_insufficient_material(&self) -> bool {
        // If both sides have king or king and bishop or king and knight
        // then it's insufficient material
        let mut pieces: HashMap<Piece, usize> = HashMap::new();

        for r in 0..RANKS {
            for f in 0..FILES {
                let piece: Piece = self.squares[r][f];
                let piece_count: &mut usize = pieces.entry(piece).or_insert(0);
                *piece_count += 1;
            }
        }

        if pieces.contains_key(&PawnWhite) || pieces.contains_key(&PawnBlack) ||
           pieces.contains_key(&RookWhite) || pieces.contains_key(&RookBlack) ||
           pieces.contains_key(&QueenWhite) || pieces.contains_key(&QueenBlack) {
            return false; // sufficient material
        }
        let white_bishops: usize = *pieces.get(&BishopWhite).unwrap_or(&0);
        let black_bishops: usize = *pieces.get(&BishopBlack).unwrap_or(&0);
        let white_knights: usize = *pieces.get(&KnightWhite).unwrap_or(&0);
        let black_knights: usize = *pieces.get(&KnightBlack).unwrap_or(&0);

        // king vs king
        if white_bishops == 0 && black_bishops == 0 && white_knights == 0 && black_knights == 0 {
            return true;
        }
        // king and bishop vs king
        if (white_bishops == 1 && black_bishops == 0 && white_knights == 0 && black_knights == 0) ||
           (black_bishops == 1 && white_bishops == 0 && white_knights == 0 && black_knights == 0) {
            return true;
        }
        // king and knight vs king
        if (white_knights == 1 && black_knights == 0 && white_bishops == 0 && black_bishops == 0) ||
           (black_knights == 1 && white_knights == 0 && white_bishops == 0 && black_bishops == 0) {
            return true;
        }
        // king and two knights vs king is (chess.com says) insufficient material
        if (white_knights == 2 && black_knights == 0 && white_bishops == 0 && black_bishops == 0) ||
           (black_knights == 2 && white_knights == 0 && white_bishops == 0 && black_bishops == 0) {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repetition_draw_with_fen() {
        let mut board = Board::new();
        let fen = board.to_fen();
        board.history.insert(fen.clone(), 3);
        assert!(board.is_repetition_draw());
    }

    #[test]
    fn test_50_move_draw() {
        let mut board = Board::new();
        board.halfmove_clock = 100;
        assert!(board.is_50_move_draw());
    }

    #[test]
    fn test_insufficient_material_kings_only() {
        let mut board: Board = Board::new();
        // Clear the board
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        // Place only kings
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;

        assert_eq!(board.is_insufficient_material(), true);
    }

    #[test]
    fn test_insufficient_material_king_and_bishop() {
        let mut board: Board = Board::new();
        // Clear the board
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        // Place kings and a bishop
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[1][2] = Piece::BishopWhite;

        assert_eq!(board.is_insufficient_material(), true);
    }

    #[test]
    fn test_insufficient_material_king_and_knight() {
        let mut board: Board = Board::new();
        // Clear the board
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        // Place kings and a knight
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[1][2] = Piece::KnightWhite;

        assert_eq!(board.is_insufficient_material(), true);
    }

    #[test]
    fn test_insufficient_material_king_and_two_knights() {
        let mut board: Board = Board::new();
        // Clear the board
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        // Place kings and two knights
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[1][2] = Piece::KnightWhite;
        board.squares[1][5] = Piece::KnightWhite;

        assert_eq!(board.is_insufficient_material(), true);
    }

    #[test]
    fn test_sufficient_material_with_pawn() {
        let mut board: Board = Board::new();
        // Clear the board
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        // Place kings and a pawn
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[1][2] = Piece::PawnWhite;

        assert_eq!(board.is_insufficient_material(), false);
    }
}