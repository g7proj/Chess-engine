use crate::{board::{Board, Color, Piece}, moves::Move};

impl Board {
    pub fn is_in_check(&self, color: Color) -> bool {
        use Piece::*;

        // find king position
        let king_piece: Piece = if color == Color::White { KingWhite } else { KingBlack };
        let mut king_rank = 0;
        let mut king_file = 0;
        let mut king_found = false;

        for r in 0..8 {
            for f in 0..8 {
                if self.squares[r][f] == king_piece {
                    king_rank = r;
                    king_file = f;
                    king_found = true;
                    break;
                }
            }
            if king_found { break; }
        }

        if !king_found { return false; }

        // Check if some opponent piece can attak the king
        let opponent_color: Color = color.opposite();
        self.is_square_under_attack(king_rank, king_file, opponent_color)
    }

    /**
     * Return true if the side to move is in check and have no legal move
     */
    pub fn is_checkmate(&self) -> bool {
        // Check if the side_to_move is under check and cannot move
        self.is_in_check(self.side_to_move) && self.generate_all_legal_moves().is_empty()
    }

    pub fn is_stalemate(&self) -> bool {
        // if the side to move is in check there is no stalemate
        if self.is_in_check(self.side_to_move) {
            return false;
        }
        // check if there are no legal move for the side to move
        let moves: Vec<Move> = self.generate_all_legal_moves();
        let is_stale: bool = moves.is_empty();

        is_stale
    }

    pub fn is_square_under_attack(&self, rank: usize, file: usize, attack_by_color: Color) -> bool {
        use Piece::*;

        for r in 0..8 {
            for f in 0..8 {
                let piece = self.squares[r][f];
                if piece != Empty {
                    let piece_color: Option<Color> = self.piece_color(piece);
                    if piece_color == Some(attack_by_color) {
                        if self.piece_attacks_square(piece, r, f, rank, file) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn piece_attacks_square(
        &self,
        piece: Piece,
        from_rank: usize,
        from_file: usize,
        target_rank: usize,
        target_file: usize,
    ) -> bool {
        use Piece::*;

        match piece {
            PawnWhite => from_rank + 1 == target_rank && from_file.abs_diff(target_file) == 1,
            PawnBlack => from_rank >= 1 && from_rank - 1 == target_rank && from_file.abs_diff(target_file) == 1,
            KnightWhite | KnightBlack => {
                let dr = from_rank.abs_diff(target_rank);
                let df = from_file.abs_diff(target_file);
                (dr == 2 && df == 1) || (dr == 1 && df == 2)
            }
            BishopWhite | BishopBlack => self.attacks_along_ray(from_rank, from_file, target_rank, target_file, &[(1, 1), (1, -1), (-1, 1), (-1, -1)]),
            RookWhite | RookBlack => self.attacks_along_ray(from_rank, from_file, target_rank, target_file, &[(1, 0), (-1, 0), (0, 1), (0, -1)]),
            QueenWhite | QueenBlack => self.attacks_along_ray(from_rank, from_file, target_rank, target_file, &[(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)]),
            KingWhite | KingBlack => {
                let dr = from_rank.abs_diff(target_rank);
                let df = from_file.abs_diff(target_file);
                dr <= 1 && df <= 1 && (dr != 0 || df != 0)
            }
            _ => false,
        }
    }

    fn attacks_along_ray(
        &self,
        from_rank: usize,
        from_file: usize,
        target_rank: usize,
        target_file: usize,
        directions: &[(isize, isize)],
    ) -> bool {
        for (dr, df) in directions {
            let mut r = from_rank as isize + dr;
            let mut f = from_file as isize + df;

            while crate::constants::in_bounds(r, f) {
                if r as usize == target_rank && f as usize == target_file {
                    return true;
                }
                if self.squares[r as usize][f as usize] != Piece::Empty {
                    break;
                }
                r += dr;
                f += df;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{Board, Piece};

    #[test]
    fn test_square_attacked_by_knight() {
        let mut b = Board::new();
        // Clear
        for r in 0..8 { for f in 0..8 { b.squares[r][f] = Piece::Empty; } }
        b.squares[4][4] = Piece::KingWhite;
        b.squares[2][3] = Piece::KnightBlack;
        assert!(b.is_in_check(crate::board::Color::White));
    }

    #[test]
    fn test_square_attacked_by_pawn() {
        let mut b = Board::new();
        for r in 0..8 { for f in 0..8 { b.squares[r][f] = Piece::Empty; } }
        b.squares[4][4] = Piece::KingWhite;
        // black pawn attacking from (5,3)
        b.squares[5][3] = Piece::PawnBlack;
        assert!(b.is_in_check(crate::board::Color::White));
    }

    #[test]
    fn test_square_attacked_by_rook() {
        let mut b = Board::new();
        for r in 0..8 { for f in 0..8 { b.squares[r][f] = Piece::Empty; } }
        b.squares[4][4] = Piece::KingWhite;
        b.squares[4][7] = Piece::RookBlack;
        assert!(b.is_in_check(crate::board::Color::White));
    }

    #[test]
    fn test_square_attacked_by_bishop_and_queen() {
        let mut b = Board::new();
        for r in 0..8 { for f in 0..8 { b.squares[r][f] = Piece::Empty; } }
        b.squares[4][4] = Piece::KingWhite;
        b.squares[1][1] = Piece::BishopBlack;
        b.squares[2][6] = Piece::QueenBlack;
        assert!(b.is_in_check(crate::board::Color::White));
    }

    #[test]
    fn test_square_not_attacked() {
        let mut b = Board::new();
        for r in 0..8 { for f in 0..8 { b.squares[r][f] = Piece::Empty; } }
        b.squares[4][4] = Piece::KingWhite;
        assert!(!b.is_in_check(crate::board::Color::White));
    }
}
