use crate::board::{Board};
use crate::moves::Move;

impl Board {
    pub fn generate_knight_moves(&self, rank: usize, file: usize) -> Vec<Move> {
        let mut moves = Vec::new();
        let piece = self.squares[rank][file];

        // check if the piece is a knight
        if !self.is_knight(piece) {
            return moves;
        }

        // generate the moves (offsets from the current position)
        let knight_offsets: &[(isize, isize)] = &[
            (2, 1), (2, -1),
            (-2, 1), (-2, -1),
            (1, 2), (1, -2),
            (-1, 2), (-1, -2),
        ];

        for (dr, df) in knight_offsets {
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

        moves
    }
}
