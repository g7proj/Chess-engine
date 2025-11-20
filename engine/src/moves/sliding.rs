use crate::board::{Board, Piece};
use crate::constants::in_bounds;
use crate::moves::Move;

impl Board {
    pub fn generate_bishop_moves(&self, rank: usize, file: usize) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let piece: Piece = self.squares[rank][file];
        
        // check if the piece is a bishop
        if !self.is_bishop(piece) {
            return moves;
        }

        let directions: [(isize, isize); 4] = [
            (1, 1), (-1, -1),
            (1, -1), (-1, 1),
        ];

        for (dr, df) in directions {
            let mut new_rank = rank as isize + dr;
            let mut new_file = file as isize + df;

            while in_bounds(new_rank, new_file) {
                let target_piece: Piece = self.squares[new_rank as usize][new_file as usize];

                if self.is_empty(target_piece) {
                    // if the target position is empty add to the moves
                    moves.push(Move::new(rank, file, new_rank as usize, new_file as usize));
                }
                else if !self.same_color(piece, target_piece) {
                    // if the target position has an opponent piece add to the moves and break
                    moves.push(Move::new(rank, file, new_rank as usize, new_file as usize));
                    break;
                }
                else {
                    // if the target position has a same color piece break
                    break;
                }
                new_rank += dr;
                new_file += df;
            }
        }

        moves
    }

    pub fn generate_rook_moves(&self, rank: usize, file: usize) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let piece: Piece = self.squares[rank][file];
        
        // check if the piece is a rook
        if !self.is_rook(piece) {
            return moves;
        }

        let directions: [(isize, isize); 4] = [
            (1, 0), (-1, 0),
            (0, 1), (0, -1),
        ];
        
        for (dr, df) in directions {
            let mut new_rank: isize = rank as isize + dr;
            let mut new_file: isize = file as isize + df;

            while in_bounds(new_rank, new_file) {
                let target_piece: Piece = self.squares[new_rank as usize][new_file as usize];

                if self.is_empty(target_piece) {
                    // if the target position is empty add to the moves
                    moves.push(Move::new(rank, file, new_rank as usize, new_file as usize));
                }
                else if !self.same_color(piece, target_piece) {
                    // if the target position has an opponent piece add to the moves and break
                    moves.push(Move::new(rank, file, new_rank as usize, new_file as usize));
                    break;
                }
                else {
                    // if the target position has a same color piece break
                    break;
                }
                new_rank += dr;
                new_file += df;
            }
        }

        moves
    }

    pub fn generate_queen_moves(&self, rank: usize, file: usize) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let piece: Piece = self.squares[rank][file];
        
        // check if the piece is a queen
        if !self.is_queen(piece) {
            return moves;
        }

        let directions: [(isize, isize); 8] = [
            (1, 0), (-1, 0),
            (0, 1), (0, -1),
            (1, 1), (-1, -1),
            (1, -1), (-1, 1),
        ];
        
        for (dr, df) in directions {
            let mut new_rank: isize = rank as isize + dr;
            let mut new_file: isize = file as isize + df;

            while in_bounds(new_rank, new_file) {
                let target_piece = self.squares[new_rank as usize][new_file as usize];

                if self.is_empty(target_piece) {
                    // if the target position is empty add to the moves
                    moves.push(Move::new(rank, file, new_rank as usize, new_file as usize));
                }
                else if !self.same_color(piece, target_piece) {
                    // if the target position has an opponent piece add to the moves and break
                    moves.push(Move::new(rank, file, new_rank as usize, new_file as usize));
                    break;
                }
                else {
                    // if the target position has a same color piece break
                    break;
                }
                new_rank += dr;
                new_file += df;
            }
        }
        moves
    }
}
