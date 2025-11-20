use crate::{board::{Board, Color, Piece}, moves::Move};

impl Board {
    pub fn is_in_check(&self, color: Color) -> bool {
        use Piece::*;

        // find king position
        let king_piece = if color == Color::White { KingWhite } else { KingBlack };
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

        for r in 0..8 {
            for f in 0..8 {
                let piece = self.squares[r][f];
                if piece != Empty {
                    let piece_color: Option<Color> = self.piece_color(piece);
                    if piece_color == Some(opponent_color) {
                        // Generate all moves for this piece
                        let moves = match piece {
                            PawnWhite | PawnBlack => self.generate_pawn_moves(r, f),
                            KnightWhite | KnightBlack => self.generate_knight_moves(r, f),
                            BishopWhite | BishopBlack => self.generate_bishop_moves(r, f),
                            RookWhite | RookBlack => self.generate_rook_moves(r, f),
                            QueenWhite | QueenBlack => self.generate_queen_moves(r, f),
                            KingWhite | KingBlack => self.generate_king_moves(r, f),
                            _ => Vec::new(),
                        };

                        // Check if one move is attacking the king
                        for mv in moves {
                            if mv.to_rank == king_rank && mv.to_file == king_file {
                                // Check!
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
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
                        // Generate all moves for this piece
                        let moves = match piece {
                            PawnWhite | PawnBlack => self.generate_pawn_moves(r, f),
                            KnightWhite | KnightBlack => self.generate_knight_moves(r, f),
                            BishopWhite | BishopBlack => self.generate_bishop_moves(r, f),
                            RookWhite | RookBlack => self.generate_rook_moves(r, f),
                            QueenWhite | QueenBlack => self.generate_queen_moves(r, f),
                            KingWhite | KingBlack => self.generate_king_moves(r, f),
                            _ => Vec::new(),
                        };

                        // check if one move attacks the target square
                        for mv in moves {
                            if mv.to_rank == rank && mv.to_file == file {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
}
