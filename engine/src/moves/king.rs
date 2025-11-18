use crate::board::{Board, Piece};
use crate::moves::Move;

impl Board {
    pub fn generate_king_moves(&self, rank: usize, file: usize) -> Vec<Move> {
        let mut moves = Vec::new();
        let piece = self.squares[rank][file];

        // check if the piece is a king
        if !self.is_king(piece) {
            return moves;
        }

        // generate the moves (offsets from the current position)
        let king_offsets: &[(isize, isize)] = &[
            (1, 0), (-1, 0),
            (0, 1), (0, -1),
            (1, 1), (-1, -1),
            (1, -1), (-1, 1),
        ];
        
        for (dr, df) in king_offsets {
            let new_rank = rank as isize + dr;
            let new_file = file as isize + df;

            // check if the new position is on the board
            if new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
                let target_piece = self.squares[new_rank as usize][new_file as usize];

                // check if the target position is empty or has an opponent piece
                if !self.same_color(piece, target_piece) {
                    moves.push(Move::new(rank, file, new_rank as usize, new_file as usize));
                }
            }
        }

        // generate castling moves
        use Piece::*;
        let is_white = self.is_white(piece);
        let color = if piece == KingWhite { 'w' } else { 'b' };
        let opponent_color = if color == 'b' { 'w' } else { 'b' };

        // short castling (kingside)
        if is_white && rank == 0 && file == 4 && self.white_kingside_castle {
            // check that f1 and g1 are empty
            if self.squares[0][5]  == Empty && self.squares[0][6]  == Empty && 
               self.squares[0][7] == RookWhite {
                // The king cannot be under check
                // The king cannot pass through f1 (which is under attack)
                // The king cannot end up in g1 (which is under attack)
                if !self.is_square_under_attack(0, 4, opponent_color) &&   // e1 not under attack
                   !self.is_square_under_attack(0, 5, opponent_color) &&   // f1 not under attack
                   !self.is_square_under_attack(0, 6, opponent_color) {    // g1 not under attack
                    moves.push(Move::new(rank, file, 0, 6)); // e1 -> g1
                }
            }
        }
        if !is_white && rank == 7 && file == 4 && self.black_kingside_castle {
            // check that f8 and g8 are empty
            if self.squares[7][5]  == Empty && self.squares[7][6]  == Empty &&
               self.squares[7][7] == RookBlack {
                // The king cannot be under check
                // The king cannot pass through f8 (which is under attack)
                // The king cannot end up in g8 (which is under attack)
                if !self.is_square_under_attack(7, 4, opponent_color) &&   // e8 not under attack
                   !self.is_square_under_attack(7, 5, opponent_color) &&   // f8 not under attack
                   !self.is_square_under_attack(7, 6, opponent_color) {    // g8 not under attack
                    moves.push(Move::new(rank, file, 7, 6)); // e8 -> g8
                }
            }
        }

        // long castling (queenside)
        if is_white && rank == 0 && file == 4 && self.white_queenside_castle {
            // check that b1, c1 and d1 are empty
            if self.squares[0][1]  == Empty && self.squares[0][2]  == Empty && self.squares[0][3]  == Empty && 
               self.squares[0][0] == RookWhite {
                // The king cannot be under check
                // The king cannot pass through d1 (which is under attack)
                // The king cannot end up in c1 (which is under attack)
                if !self.is_square_under_attack(0, 4, opponent_color) &&   // e1 not under attack
                   !self.is_square_under_attack(0, 3, opponent_color) &&   // d1 not under attack
                   !self.is_square_under_attack(0, 2, opponent_color) {    // c1 not under attack
                    moves.push(Move::new(rank, file, 0, 2)); // e1 -> c1
                }
            }
        }
        if !is_white && rank == 7 && file == 4 && self.black_queenside_castle {
            // check that b8, c8 and d8 are empty
            if self.squares[7][1]  == Empty && self.squares[7][2]  == Empty && self.squares[7][3]  == Empty &&
               self.squares[7][0] == RookBlack {
                // The king cannot be under check
                // The king cannot pass through d8 (which is under attack)
                // The king cannot end up in c8 (which is under attack)
                if !self.is_square_under_attack(7, 4, opponent_color) &&   // e8 not under attack
                   !self.is_square_under_attack(7, 3, opponent_color) &&   // d8 not under attack
                   !self.is_square_under_attack(7, 2, opponent_color) {    // c8 not under attack
                    moves.push(Move::new(rank, file, 7, 2)); // e8 -> c8
                }
            }
        }

        moves
    }
}
