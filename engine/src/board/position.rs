use crate::{board::Piece::*, constants::{FILES, RANKS}};

use super::Board;

impl Board {
    pub fn repetition_key(&self) -> String {
        let mut key: String = self.position_string();
        key.push(' ');
        key.push(self.side_to_move.to_fen_char());
        key.push(' ');

        let mut castling = String::new();
        if self.white_kingside_castle {
            castling.push('K');
        }
        if self.white_queenside_castle {
            castling.push('Q');
        }
        if self.black_kingside_castle {
            castling.push('k');
        }
        if self.black_queenside_castle {
            castling.push('q');
        }
        if castling.is_empty() {
            castling.push('-');
        }
        key.push_str(&castling);
        key.push(' ');

        if let Some((ep_rank, ep_file)) = self.en_passant {
            key.push((b'a' + ep_file as u8) as char);
            key.push((b'1' + ep_rank as u8) as char);
        } else {
            key.push('-');
        }

        key
    }

    pub fn record_position(&mut self) {
        let key: String = self.repetition_key();
        let position_counter: &mut usize = self.history.entry(key).or_insert(0);
        *position_counter += 1;
    }

    pub fn is_repetition_draw(&self) -> bool {
        let key: String = self.repetition_key();
        self.history.get(&key).copied().unwrap_or(0) >= 3
    }

    pub fn is_50_move_draw(&self) -> bool {
        self.halfmove_clock >= 100
    }

    pub fn clear_position_history(&mut self) {
        self.history.clear();
    }

    pub fn is_insufficient_material(&self) -> bool {
        let mut white_bishops: usize = 0;
        let mut black_bishops: usize = 0;
        let mut white_knights: usize = 0;
        let mut black_knights: usize = 0;

        for r in 0..RANKS {
            for f in 0..FILES {
                match self.squares[r][f] {
                    PawnWhite | PawnBlack | RookWhite | RookBlack | QueenWhite | QueenBlack => {
                        return false;
                    }
                    BishopWhite => white_bishops += 1,
                    BishopBlack => black_bishops += 1,
                    KnightWhite => white_knights += 1,
                    KnightBlack => black_knights += 1,
                    _ => {}
                }
            }
        }

        let white_minors: usize = white_bishops + white_knights;
        let black_minors: usize = black_bishops + black_knights;

        if white_minors == 0 && black_minors == 0 {
            return true;
        }

        if (white_minors == 1 && black_minors == 0) || (white_minors == 0 && black_minors == 1) {
            return true;
        }

        if white_minors == 1 && black_minors == 1 {
            return true;
        }

        if white_knights == 2 && white_bishops == 0 && black_minors == 0 {
            return true;
        }

        if black_knights == 2 && black_bishops == 0 && white_minors == 0 {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Piece;

    #[test]
    fn test_repetition_draw() {
        let mut board: Board = Board::new();
        let key: String = board.repetition_key();
        board.history.insert(key, 3);
        assert!(board.is_repetition_draw());
    }

    #[test]
    fn test_repetition_key_changes_with_side_to_move() {
        let board: Board = Board::new();
        let mut other: Board = board.clone();
        other.side_to_move = crate::board::Color::Black;

        assert_ne!(board.repetition_key(), other.repetition_key());
    }

    #[test]
    fn test_50_move_draw() {
        let mut board: Board = Board::new();
        board.halfmove_clock = 100;
        assert!(board.is_50_move_draw());
    }

    #[test]
    fn test_insufficient_material_kings_only() {
        let mut board: Board = Board::new();
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;

        assert!(board.is_insufficient_material());
    }

    #[test]
    fn test_insufficient_material_king_and_bishop() {
        let mut board: Board = Board::new();
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[1][2] = Piece::BishopWhite;

        assert!(board.is_insufficient_material());
    }

    #[test]
    fn test_insufficient_material_king_and_knight() {
        let mut board: Board = Board::new();
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[1][2] = Piece::KnightWhite;

        assert!(board.is_insufficient_material());
    }

    #[test]
    fn test_insufficient_material_king_and_two_knights() {
        let mut board: Board = Board::new();
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[1][2] = Piece::KnightWhite;
        board.squares[1][5] = Piece::KnightWhite;

        assert!(board.is_insufficient_material());
    }

    #[test]
    fn test_insufficient_material_single_minor_each_side() {
        let mut board: Board = Board::new();
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[1][2] = Piece::BishopWhite;
        board.squares[6][5] = Piece::KnightBlack;

        assert!(board.is_insufficient_material());
    }

    #[test]
    fn test_sufficient_material_with_pawn() {
        let mut board: Board = Board::new();
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[1][2] = Piece::PawnWhite;

        assert!(!board.is_insufficient_material());
    }

    #[test]
    fn test_sufficient_material_king_and_bishop_vs_king_and_bishop() {
        let mut board: Board = Board::new();
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[1][2] = Piece::BishopWhite;
        board.squares[6][5] = Piece::BishopBlack;

        assert!(board.is_insufficient_material());
    }

    #[test]
    fn test_threefold_repetition_by_moves() {
        use crate::moves::Move;
        let mut b: Board = Board::new();
        b.record_position();

        let w1: Move = Move::new(0, 6, 2, 5);
        let b1: Move = Move::new(7, 6, 5, 5);
        let w2: Move = Move::new(2, 5, 0, 6);
        let b2: Move = Move::new(5, 5, 7, 6);

        for _ in 0..2 {
            b.make_move(w1);
            b.make_move(b1);
            b.make_move(w2);
            b.make_move(b2);
        }

        assert!(b.is_repetition_draw(), "expected threefold repetition");
    }

    #[test]
    fn test_50_move_draw_by_repeated_knight_moves() {
        use crate::moves::Move;
        let mut b: Board = Board::new();

        let to_f3: Move = Move::new(0, 6, 2, 5);
        let to_g1: Move = Move::new(2, 5, 0, 6);

        for i in 0..100 {
            if i % 2 == 0 {
                b.make_move(to_f3);
            } else {
                b.make_move(to_g1);
            }
        }

        assert!(b.is_50_move_draw(), "expected 50-move draw after 100 halfmoves");
        assert_eq!(b.halfmove_clock, 100);
    }
}
