use crate::board::{Board, Piece};
use crate::constants::{FILES, in_bounds};
use crate::moves::Move;

impl Board {
    pub fn generate_pawn_moves(&self, rank: usize, file: usize) -> Vec<Move> {
        use Piece::*;
        let mut moves = Vec::new();
        let piece = self.squares[rank][file];

        let (forward, start_rank, promotion_rank) = match piece {
            PawnWhite => (1, 1, 7),
            PawnBlack => (-1, 6, 0),
            _ => return moves,
        };

        let r = rank as isize;
        let f = file as isize;

        // 1. Single move (one step forward)
        let (one_step_r, one_step_f) = (r + forward, f);
        if in_bounds(one_step_r, one_step_f) {
            let (nr, nf) = (one_step_r, one_step_f);
            if self.squares[nr as usize][nf as usize] == Empty {
                // promotion?
                if nr as usize == promotion_rank {
                    moves.extend(self.generate_pawn_promotions(
                        rank,
                        file,
                        nr as usize,
                        nf as usize,
                    ));
                } else {
                    moves.push(Move::new(rank, file, nr as usize, nf as usize));
                }

                // 2. Double move (only if the single move is free)
                if rank == start_rank {
                    let (two_step_r, two_step_f) = (r + forward * 2, f);
                    if in_bounds(two_step_r, two_step_f) {
                        let (tr, tf) = (two_step_r, two_step_f);
                        if self.squares[tr as usize][tf as usize] == Empty {
                            moves.push(Move::new(rank, file, tr as usize, tf as usize));
                        }
                    }
                }
            }
        }

        // 3. Diagonal captures
        for df in [-1, 1] {
            let (diag_r, diag_f) = (r + forward, f + df);
            if in_bounds(diag_r, diag_f) {
                let (nr, nf) = (diag_r, diag_f);
                let target = self.squares[nr as usize][nf as usize];

                if target != Empty && !self.same_color(piece, target) {
                    if nr as usize == promotion_rank {
                        moves.extend(self.generate_pawn_promotions(
                            rank,
                            file,
                            nr as usize,
                            nf as usize,
                        ));
                    } else {
                        moves.push(Move::new(rank, file, nr as usize, nf as usize));
                    }
                }
            }
        }

        // 4. En passant
        if let Some((ep_r, ep_f)) = self.en_passant {
            for df in [-1, 1] {
                let target_file = f + df;
                if target_file >= 0 && target_file < FILES as isize {
                    let attack_r = r + forward;
                    if attack_r == ep_r as isize && target_file == ep_f as isize {
                        moves.push(Move::new(rank, file, ep_r, ep_f));
                    }
                }
            }
        }

        moves
    }

    fn generate_pawn_promotions(&self, fr: usize, ff: usize, tr: usize, tf: usize) -> Vec<Move> {
        use crate::moves::Promotion::*;

        vec![
            Move::with_promotion(fr, ff, tr, tf, Queen),
            Move::with_promotion(fr, ff, tr, tf, Rook),
            Move::with_promotion(fr, ff, tr, tf, Bishop),
            Move::with_promotion(fr, ff, tr, tf, Knight),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::{FILES, RANKS};

    use super::*;

    #[test]
    fn test_pawn_initial_double_move() {
        let board: Board = Board::new();
        // retrieve the pawn moves
        let moves: Vec<Move> = board.generate_pawn_moves(1, 3);
        // should have 2 available moves
        assert_eq!(moves.len(), 2);
    }

    #[test]
    fn test_pawn_en_passant() {
        let mut board: Board = Board::new();
        // set up board for en passant
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        board.squares[4][3] = Piece::PawnBlack;
        board.squares[4][4] = Piece::PawnWhite;
        board.en_passant = Some((5, 3));

        // generate moves for white pawn
        let moves: Vec<Move> = board.generate_pawn_moves(4, 4);
        // Check the en passant move
        assert!(moves.iter().any(|m| m.to_rank == 5 && m.to_file == 3));
    }

    #[test]
    fn test_pawn_promotion_with_capture() {
        let mut board: Board = Board::new();
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        board.squares[6][0] = Piece::PawnWhite;
        board.squares[7][1] = Piece::RookBlack;
        let moves: Vec<Move> = board.generate_pawn_moves(6, 0);
        // There must be a promotion in (7,1)
        assert!(moves.iter().any(|m| m.to_rank == 7
            && m.to_file == 1
            && matches!(
                m.promotion,
                crate::moves::Promotion::Queen
                    | crate::moves::Promotion::Rook
                    | crate::moves::Promotion::Bishop
                    | crate::moves::Promotion::Knight
            )));
    }
}
