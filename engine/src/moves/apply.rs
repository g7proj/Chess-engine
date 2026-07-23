use crate::board::{Board, Color, Piece};
use crate::logger::Logger;
use crate::moves::Move;

impl Board {
    pub fn make_move(&mut self, mv: Move) {
        use crate::moves::Promotion;
        use Piece::*;

        let fr: usize = mv.from_rank;
        let ff: usize = mv.from_file;
        let tr: usize = mv.to_rank;
        let tf: usize = mv.to_file;

        let moving_piece: Piece = self.squares[fr][ff];

        // Save current en passant before the reset
        let old_ep: Option<(usize, usize)> = self.en_passant;
        // Reset en passant unless this is a double pawn move
        self.en_passant = None;

        // ---------------------
        // 0. Castling
        // ---------------------
        let is_castling: bool = self.is_king(moving_piece)
            && (
                (fr == 0 && ff == 4 && tr == 0 && tf == 6) || // White kingside
                                (fr == 0 && ff == 4 && tr == 0 && tf == 2) || // White queenside
                                (fr == 7 && ff == 4 && tr == 7 && tf == 6) || // Black kingside
                                (fr == 7 && ff == 4 && tr == 7 && tf == 2)
                // Black queenside
            );
        if is_castling {
            // kingside
            if tf == 6 {
                // move the rook
                if fr == 0 {
                    // White rook
                    self.squares[0][7] = Empty;
                    self.squares[0][5] = RookWhite;
                } else {
                    // Black rook
                    self.squares[7][7] = Empty;
                    self.squares[7][5] = RookBlack;
                }
            }
            // queenside
            if tf == 2 {
                // move the rook
                if fr == 0 {
                    // White rook
                    self.squares[0][0] = Empty;
                    self.squares[0][3] = RookWhite;
                } else {
                    // Black rook
                    self.squares[7][0] = Empty;
                    self.squares[7][3] = RookBlack;
                }
            }
            // the king is moved at the end of the function
        }

        // update castling rights when a king is moving
        if self.is_king(moving_piece) {
            if moving_piece == KingWhite {
                self.white_kingside_castle = false;
                self.white_queenside_castle = false;
            } else {
                self.black_kingside_castle = false;
                self.black_queenside_castle = false;
            }
        }
        // update castling rights when a rook is moving
        if self.is_rook(moving_piece) {
            if fr == 0 && ff == 0 {
                self.white_queenside_castle = false;
            } else if fr == 0 && ff == 7 {
                self.white_kingside_castle = false;
            } else if fr == 7 && ff == 0 {
                self.black_queenside_castle = false;
            } else if fr == 7 && ff == 7 {
                self.black_kingside_castle = false;
            }
        }
        // update castling rights when a rook gets captured
        let captured_piece = self.squares[tr][tf];
        if self.is_rook(captured_piece) {
            if tr == 0 && tf == 0 {
                self.white_queenside_castle = false;
            } else if tr == 0 && tf == 7 {
                self.white_kingside_castle = false;
            } else if tr == 7 && tf == 0 {
                self.black_queenside_castle = false;
            } else if tr == 7 && tf == 7 {
                self.black_kingside_castle = false;
            }
        }

        // ---------------------
        // 1. En passant capture
        // ---------------------
        // Check if the destination square is en passant square
        if let Some((ep_r, ep_f)) = old_ep {
            if self.is_pawn(moving_piece) && tr == ep_r && tf == ep_f {
                // Capture the pawn behind the EP
                let captured_rank: usize;
                let expected_captured_piece: Piece;
                if moving_piece == PawnWhite {
                    captured_rank = tr - 1;
                    expected_captured_piece = PawnBlack;
                } else {
                    // Black pawn
                    captured_rank = tr + 1;
                    expected_captured_piece = PawnWhite;
                }
                // Capture piece
                let captured_piece: Piece = self.squares[captured_rank][ep_f];
                if captured_piece == expected_captured_piece {
                    Logger::debug(&format!("Captured piece: {}", captured_piece));
                    self.squares[captured_rank][ep_f] = Empty;
                } else {
                    // invalid en passant capture!
                    Logger::warning("Invalid en passant capture attempted!");
                }
            }
        }

        // ---------------------
        // 2. Normal capture
        // ---------------------
        // Check if there is a enemy piece in the destination square
        let destination_square: Piece = self.squares[tr][tf];
        if self.same_color(moving_piece, destination_square) {
            // invalid move!
        } else {
            if destination_square != Piece::Empty {
                // Capture piece
                Logger::debug(&format!("Captured piece: {}", destination_square));
            }
        }

        // ---------------------
        // 3. Promotion
        // ---------------------
        let piece_to_place: Piece = match (moving_piece, mv.promotion) {
            (PawnWhite, Promotion::Queen) => QueenWhite,
            (PawnWhite, Promotion::Rook) => RookWhite,
            (PawnWhite, Promotion::Bishop) => BishopWhite,
            (PawnWhite, Promotion::Knight) => KnightWhite,
            (PawnBlack, Promotion::Queen) => QueenBlack,
            (PawnBlack, Promotion::Rook) => RookBlack,
            (PawnBlack, Promotion::Bishop) => BishopBlack,
            (PawnBlack, Promotion::Knight) => KnightBlack,
            _ => moving_piece,
        };

        // ---------------------
        // 4. Double step
        // ---------------------
        if moving_piece == PawnWhite && fr == 1 && tr == 3 {
            self.en_passant = Some((2, ff));
        }
        if moving_piece == PawnBlack && fr == 6 && tr == 4 {
            self.en_passant = Some((5, ff));
        }

        // ---------------------
        // 5. Execute the move
        // ---------------------
        self.squares[fr][ff] = Empty;
        self.squares[tr][tf] = piece_to_place;

        // ---------------------
        // 6. Switch moving side
        // ---------------------
        if self.side_to_move == Color::White {
            self.side_to_move = Color::Black;
        } else {
            self.side_to_move = Color::White;
        }

        // Record position in history
        self.record_position();
        // Update halfmove clock
        if self.is_pawn(moving_piece) || destination_square != Piece::Empty {
            // pawn moved or capture occurred => reset halfmove clock
            self.halfmove_clock = 0;
        } else {
            // no pawn move and no capture => increment halfmove clock
            self.halfmove_clock += 1;
        }
        // Update fullmove number (after Black's move increment fullmove number)
        if self.side_to_move == Color::White {
            self.fullmove_number += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{Board, Color, Piece};
    use crate::moves::Move;

    #[test]
    fn test_make_move_records_position() {
        let mut b: Board = Board::new();
        assert!(b.history.is_empty());
        let mv: Move = Move::new(1, 4, 3, 4); // e2e4
        b.make_move(mv);
        let key: String = b.repetition_key();
        assert_eq!(b.history.get(&key), Some(&1));
    }

    #[test]
    fn test_halfmove_clock_behavior() {
        let mut b: Board = Board::new();
        assert_eq!(b.halfmove_clock, 0);
        b.make_move(Move::new(1, 4, 3, 4)); // e2e4 pawn
        assert_eq!(b.halfmove_clock, 0);
        b.make_move(Move::new(6, 4, 4, 4)); // e7e5 pawn
        assert_eq!(b.halfmove_clock, 0);
        b.make_move(Move::new(0, 6, 2, 5)); // g1f3 knight
        assert_eq!(b.halfmove_clock, 1);

        // capture resets halfmove clock
        let mut c: Board = Board::new();
        for r in 0..8 {
            for f in 0..8 {
                c.squares[r][f] = Piece::Empty;
            }
        }
        c.squares[0][4] = Piece::KingWhite;
        c.squares[7][4] = Piece::KingBlack;
        c.squares[4][4] = Piece::PawnWhite;
        c.squares[5][5] = Piece::PawnBlack;
        c.side_to_move = Color::White;
        c.halfmove_clock = 10;
        c.make_move(Move::new(4, 4, 5, 5)); // capture pawn
        assert_eq!(c.halfmove_clock, 0);
    }

    #[test]
    fn test_fullmove_number_increment() {
        let mut b: Board = Board::new();
        assert_eq!(b.fullmove_number, 1);
        b.make_move(Move::new(1, 4, 3, 4)); // e2e4 white
        assert_eq!(b.fullmove_number, 1);
        b.make_move(Move::new(6, 4, 4, 4)); // e7e5 black
        assert_eq!(b.fullmove_number, 2);
    }
}
