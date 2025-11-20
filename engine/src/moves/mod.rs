use crate::{board::{Board, Color, Piece}, constants::{FILES, RANKS}};
pub mod knight;
pub mod king;
pub mod pawn;
pub mod sliding;
pub mod attacks;
pub mod apply;

#[derive(Debug, Clone, Copy)]
pub enum Promotion {
    Queen,
    Rook,
    Bishop,
    Knight,
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from_rank: usize,
    pub from_file: usize,
    pub to_rank: usize,
    pub to_file: usize,
    pub promotion: Promotion,
}

impl Move {
    pub fn new(fr: usize, ff: usize, tr: usize, tf: usize) -> Self {
        Move {
            from_rank: fr,
            from_file: ff,
            to_rank: tr,
            to_file: tf,
            promotion: Promotion::None,
        }
    }

    pub fn with_promotion(fr: usize, ff: usize, tr: usize, tf: usize, p: Promotion) -> Self {
        Move {
            from_rank: fr,
            from_file: ff,
            to_rank: tr,
            to_file: tf,
            promotion: p,
        }
    }
}

impl Board {
    /**
     * Generate all legal moves for the side to move
     */
    pub fn generate_all_legal_moves(&self) -> Vec<Move> {
        use Piece::*;
        let mut all_moves: Vec<Move> = Vec::new();

        // Iterate through all squares
        for r in 0..RANKS {
            for f in 0..FILES { 
                let piece: Piece = self.squares[r][f];
                if piece != Empty {
                    let piece_color: Color = self.piece_color(piece).unwrap();
                    // Only generate moves for the side_to_move color
                    if piece_color == self.side_to_move {
                        let moves: Vec<Move> = match piece {
                            PawnWhite | PawnBlack => self.generate_pawn_moves(r, f),
                            KnightWhite | KnightBlack => self.generate_knight_moves(r, f),
                            BishopWhite | BishopBlack => self.generate_bishop_moves(r, f),
                            RookWhite | RookBlack => self.generate_rook_moves(r, f),
                            QueenWhite | QueenBlack => self.generate_queen_moves(r, f),
                            KingWhite | KingBlack => self.generate_king_moves(r, f),
                            _ => Vec::new(),
                        };
                        all_moves.extend(moves);
                    }
                }
            }
        }

        // Now filter out illegal moves
        all_moves.into_iter().filter(|mv: &Move| self.is_legal_move(*mv)).collect()
    }

    fn is_legal_move(&self, mv: Move) -> bool {
        // Execute the move in a sandbox, then check if the king is under check
        let mut board_copy: Board = self.clone();
        let color_moved: Color = self.side_to_move;
        board_copy.make_move(mv);
        !board_copy.is_in_check(color_moved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{board::{Board, Color, Piece}, constants::{FILES, RANKS}};

    #[test]
    fn test_generate_all_legal_moves_starting_position() {
        let board: Board = Board::new();
        let moves: Vec<Move> = board.generate_all_legal_moves();

        // starting position should have 20 legal moves (2 move per pawn + 2 move per knight)
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_generate_all_legal_moves_filters_illegal() {
        let mut board: Board = Board::new();
        // Clear the board
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        // Set up: white king on e1, black rook on e8
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::RookBlack;
        board.side_to_move = Color::White;

        let moves: Vec<Move> = board.generate_all_legal_moves();
        // King cannot move to e2 because it would be in check from the rook
        assert!(!moves.iter().any(|m| m.to_rank == 1 && m.to_file == 4));
    }

    #[test]
    fn test_generate_all_legal_moves_empty_board() {
        let mut board: Board = Board::new();
        // Clear the board
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        // Place only a white king in the center of the board
        board.squares[4][4] = Piece::KingWhite;
        board.side_to_move = Color::White;

        let moves: Vec<Move> = board.generate_all_legal_moves();
        // King can move in all the directions
        assert_eq!(moves.len(), 8);
    }

    #[test]
    fn test_generate_all_legal_moves_only_current_side() {
        let board: Board = Board::new();
        let moves: Vec<Move> = board.generate_all_legal_moves();

        // All the move should be for the white pieces
        for mv in moves {
            let piece: Piece = board.squares[mv.from_rank][mv.from_file];
            assert!(board.is_white(piece));
        }
    }

    #[test]
    fn test_is_legal_move_prevents_self_check() {
        let mut board: Board = Board::new();
        // Clear board
        for r in 0..RANKS {
            for f in 0..FILES {
                board.squares[r][f] = Piece::Empty;
            }
        }
        // Set up: white king on e1, white rook on e2, black rook on e8
        board.squares[0][4] = Piece::KingWhite;
        board.squares[1][4] = Piece::RookWhite;
        board.squares[7][4] = Piece::RookBlack;
        board.side_to_move = Color::White;
        
        // Moving the rook on the side would expose the king to check
        let illegal_move: Move = Move::new(1, 4, 1, 5);
        assert!(!board.is_legal_move(illegal_move));
    }
}
