use crate::board::{Board, Piece};
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
        let one_step = (r + forward, f);
        if self.in_bounds(one_step) {
            let (nr, nf) = one_step;
            if self.squares[nr as usize][nf as usize] == Empty {
                // promotion?
                if nr as usize == promotion_rank {
                    moves.extend(self.generate_pawn_promotions(rank, file, nr as usize, nf as usize));
                } else {
                    moves.push(Move::new(rank, file, nr as usize, nf as usize));
                }
    
                // 2. Double move (only if the single move is free)
                if rank == start_rank {
                    let two_step = (r + forward * 2, f);
                    if self.in_bounds(two_step) {
                        let (tr, tf) = two_step;
                        if self.squares[tr as usize][tf as usize] == Empty {
                            moves.push(Move::new(rank, file, tr as usize, tf as usize));
                        }
                    }
                }
            }
        }
    
        // 3. Diagonal captures
        for df in [-1, 1] {
            let diag = (r + forward, f + df);
            if self.in_bounds(diag) {
                let (nr, nf) = diag;
                let target = self.squares[nr as usize][nf as usize];
    
                if target != Empty && !self.same_color(piece, target) {
                    if nr as usize == promotion_rank {
                        moves.extend(self.generate_pawn_promotions(rank, file, nr as usize, nf as usize));
                    } else {
                        moves.push(Move::new(rank, file, nr as usize, nf as usize));
                    }
                }
            }
        }
    
        // 4. En passant (implement later)
        if let Some((ep_r, ep_f)) = self.en_passant {
            for df in [-1, 1] {
                let target_file = f + df;
                if target_file >= 0 && target_file < 8 {
                    let attack_r = r + forward;
                    if attack_r == ep_r as isize && target_file == ep_f as isize {
                        moves.push(Move::new(rank, file, ep_r, ep_f));
                    }
                }
            }
        }
    
        moves
    }

    fn generate_pawn_promotions(
        &self,
        fr: usize,
        ff: usize,
        tr: usize,
        tf: usize
    ) -> Vec<Move> {
        use crate::moves::Promotion::*;

        vec![
            Move::with_promotion(fr, ff, tr, tf, Queen),
            Move::with_promotion(fr, ff, tr, tf, Rook),
            Move::with_promotion(fr, ff, tr, tf, Bishop),
            Move::with_promotion(fr, ff, tr, tf, Knight),
        ]
    }
}
