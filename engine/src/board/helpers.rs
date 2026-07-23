use super::{Board, Color, Piece};

impl Board {
    pub fn piece_color(&self, piece: Piece) -> Option<Color> {
        piece.color()
    }

    pub fn same_color(&self, piece1: Piece, piece2: Piece) -> bool {
        if piece1 == Piece::Empty || piece2 == Piece::Empty {
            return false;
        }
        piece1.color() == piece2.color()
    }

    pub fn is_pawn(&self, piece: Piece) -> bool {
        piece == Piece::PawnWhite || piece == Piece::PawnBlack
    }

    pub fn is_knight(&self, piece: Piece) -> bool {
        piece == Piece::KnightWhite || piece == Piece::KnightBlack
    }

    pub fn is_bishop(&self, piece: Piece) -> bool {
        piece == Piece::BishopWhite || piece == Piece::BishopBlack
    }

    pub fn is_rook(&self, piece: Piece) -> bool {
        piece == Piece::RookWhite || piece == Piece::RookBlack
    }

    pub fn is_queen(&self, piece: Piece) -> bool {
        piece == Piece::QueenWhite || piece == Piece::QueenBlack
    }

    pub fn is_king(&self, piece: Piece) -> bool {
        piece == Piece::KingWhite || piece == Piece::KingBlack
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::in_bounds;
    #[test]
    fn test_piece_helpers() {
        let board: Board = Board::new();
        assert_eq!(board.same_color(Piece::KnightWhite, Piece::PawnWhite), true);
        assert_eq!(board.same_color(Piece::PawnWhite, Piece::PawnBlack), false);
        assert_eq!(in_bounds(0, 0), true);
        assert_eq!(in_bounds(7, 7), true);
        assert_eq!(in_bounds(2, 8), false);
        assert_eq!(in_bounds(8, 3), false);
    }
}
