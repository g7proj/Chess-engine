use crate::board::{Board, Color, Piece};
use crate::logger::Logger;
use crate::moves::{Move, Undo};

impl Board {
    /// Applies a move and returns the state required to undo it.
    pub fn make_move(&mut self, mv: Move) -> Undo {
        self.make_move_internal(mv, true)
    }

    /// Applies a move for search without updating game repetition history.
    pub(crate) fn make_move_for_search(&mut self, mv: Move) -> Undo {
        self.make_move_internal(mv, false)
    }

    fn make_move_internal(&mut self, mv: Move, record_history: bool) -> Undo {
        use crate::moves::Promotion;
        use Piece::*;

        let fr: usize = mv.from_rank;
        let ff: usize = mv.from_file;
        let tr: usize = mv.to_rank;
        let tf: usize = mv.to_file;

        let moving_piece: Piece = self.squares[fr][ff];
        let previous_side: Color = self.side_to_move;
        let previous_en_passant: Option<(usize, usize)> = self.en_passant;
        let previous_castling: (bool, bool, bool, bool) = (
            self.white_kingside_castle,
            self.white_queenside_castle,
            self.black_kingside_castle,
            self.black_queenside_castle,
        );
        let previous_halfmove_clock: usize = self.halfmove_clock;
        let previous_fullmove_number: usize = self.fullmove_number;
        let mut en_passant_capture: Option<((usize, usize), Piece)> = None;
        let mut rook_move: Option<((usize, usize), (usize, usize), Piece)> = None;

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
                    rook_move = Some(((0, 7), (0, 5), self.squares[0][7]));
                    // White rook
                    self.squares[0][7] = Empty;
                    self.squares[0][5] = RookWhite;
                } else {
                    rook_move = Some(((7, 7), (7, 5), self.squares[7][7]));
                    // Black rook
                    self.squares[7][7] = Empty;
                    self.squares[7][5] = RookBlack;
                }
            }
            // queenside
            if tf == 2 {
                // move the rook
                if fr == 0 {
                    rook_move = Some(((0, 0), (0, 3), self.squares[0][0]));
                    // White rook
                    self.squares[0][0] = Empty;
                    self.squares[0][3] = RookWhite;
                } else {
                    rook_move = Some(((7, 0), (7, 3), self.squares[7][0]));
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
                    en_passant_capture = Some(((captured_rank, ep_f), captured_piece));
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

        let position_key: Option<String> = if record_history {
            self.record_position();
            Some(self.repetition_key())
        } else {
            None
        };
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

        Undo {
            from: (fr, ff),
            to: (tr, tf),
            moving_piece,
            captured_piece,
            en_passant_capture,
            rook_move,
            previous_side,
            previous_en_passant,
            previous_castling,
            previous_halfmove_clock,
            previous_fullmove_number,
            position_key,
        }
    }

    /// Restores the board state returned by `make_move`.
    pub fn unmake_move(&mut self, undo: Undo) {
        if let Some(position_key) = undo.position_key {
            if let Some(count) = self.history.get_mut(&position_key) {
                *count -= 1;
                if *count == 0 {
                    self.history.remove(&position_key);
                }
            }
        }

        self.squares[undo.from.0][undo.from.1] = undo.moving_piece;
        self.squares[undo.to.0][undo.to.1] = undo.captured_piece;

        if let Some(((rank, file), piece)) = undo.en_passant_capture {
            self.squares[rank][file] = piece;
        }
        if let Some(((from_rank, from_file), (to_rank, to_file), piece)) = undo.rook_move {
            self.squares[from_rank][from_file] = piece;
            self.squares[to_rank][to_file] = Piece::Empty;
        }

        self.side_to_move = undo.previous_side;
        self.en_passant = undo.previous_en_passant;
        (
            self.white_kingside_castle,
            self.white_queenside_castle,
            self.black_kingside_castle,
            self.black_queenside_castle,
        ) = undo.previous_castling;
        self.halfmove_clock = undo.previous_halfmove_clock;
        self.fullmove_number = undo.previous_fullmove_number;
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{Board, Color, Piece};
    use crate::moves::Move;

    fn assert_round_trip(mut board: Board, mv: Move) {
        let fen_before: String = board.to_fen();
        let history_before = board.history.clone();
        let undo = board.make_move(mv);
        board.unmake_move(undo);
        assert_eq!(board.to_fen(), fen_before);
        assert_eq!(board.history, history_before);
    }

    #[test]
    fn test_make_unmake_restores_normal_move() {
        assert_round_trip(Board::new(), Move::new(1, 4, 3, 4));
    }

    #[test]
    fn test_search_move_does_not_update_history() {
        let mut board: Board = Board::new();
        let undo = board.make_move_for_search(Move::new(1, 4, 3, 4));
        assert!(board.history.is_empty());
        board.unmake_move(undo);
        assert!(board.history.is_empty());
    }

    #[test]
    fn test_make_unmake_restores_castling() {
        let board: Board =
            Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").expect("valid castling FEN");
        assert_round_trip(board, Move::new(0, 4, 0, 6));
    }

    #[test]
    fn test_make_unmake_restores_en_passant() {
        let board: Board =
            Board::from_fen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1").expect("valid en passant FEN");
        assert_round_trip(board, Move::new(4, 4, 5, 3));
    }

    #[test]
    fn test_make_unmake_restores_promotion() {
        let board: Board =
            Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").expect("valid promotion FEN");
        assert_round_trip(
            board,
            Move::with_promotion(6, 0, 7, 0, crate::moves::Promotion::Queen),
        );
    }

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
