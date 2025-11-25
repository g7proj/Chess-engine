use std::collections::HashMap;

use crate::constants::{FILES, RANKS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Empty,
    PawnWhite,
    KnightWhite,
    BishopWhite,
    RookWhite,
    QueenWhite,
    KingWhite,
    PawnBlack,
    KnightBlack,
    BishopBlack,
    RookBlack,
    QueenBlack,
    KingBlack,
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece_char: &str = match self {
            Piece::Empty => ".",
            Piece::PawnWhite => "P",
            Piece::KnightWhite => "N",
            Piece::BishopWhite => "B",
            Piece::RookWhite => "R",
            Piece::QueenWhite => "Q",
            Piece::KingWhite => "K",
            Piece::PawnBlack => "p",
            Piece::KnightBlack => "n",
            Piece::BishopBlack => "b",
            Piece::RookBlack => "r",
            Piece::QueenBlack => "q",
            Piece::KingBlack => "k",
        };
        write!(f, "{}", piece_char)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c: char = match self {
            Color::White => 'w',
            Color::Black => 'b',
        };
        write!(f, "{}", c)
    }
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pub squares: [[Piece; FILES]; RANKS],
    pub side_to_move: Color,
    pub en_passant: Option<(usize, usize)>,
    pub white_kingside_castle: bool,
    pub white_queenside_castle: bool,
    pub black_kingside_castle: bool,
    pub black_queenside_castle: bool,
    pub position_history: HashMap<String, usize>,
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r_index: i32 = 8;
        for rank in self.squares.iter().rev() {
            r_index -= 1;
            write!(f, "R{} - ", r_index)?;
            for piece in rank.iter() {
                write!(f, "{}", piece)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "Side to move: {}", self.side_to_move)
    }
}

impl Board {
    pub fn new() -> Self {
        use Piece::*;
        // Initialize the board with the starting positions
        let squares: [[Piece; FILES]; RANKS] = [
            [RookWhite, KnightWhite, BishopWhite, QueenWhite, KingWhite, BishopWhite, KnightWhite, RookWhite],
            [PawnWhite; FILES],
            [Empty; FILES],
            [Empty; FILES],
            [Empty; FILES],
            [Empty; FILES],
            [PawnBlack; FILES],
            [RookBlack, KnightBlack, BishopBlack, QueenBlack, KingBlack, BishopBlack, KnightBlack, RookBlack],
        ];
        Board { 
            squares,
            side_to_move: Color::White,
            en_passant: None,
            white_kingside_castle: true,
            white_queenside_castle: true,
            black_kingside_castle: true,
            black_queenside_castle: true,
            position_history: HashMap::new(),
        }
    }

    /**
     * Convert the board to a position string for hashing/comparison
     * Can be improved but works for now
     */
    pub fn to_position_string(&self) -> String {
        let mut pos_str: String = String::new();
        for rank in 0..RANKS {
            for file in 0..FILES {
                let piece: Piece = self.squares[rank][file];
                pos_str.push_str(&format!("{}", piece));
            }
        }
        pos_str.push_str(&format!("{:?}", self.side_to_move));
        pos_str.push_str(&format!("{:?}", self.en_passant));
        pos_str.push_str(&format!("{}", self.white_kingside_castle));
        pos_str.push_str(&format!("{}", self.white_queenside_castle));
        pos_str.push_str(&format!("{}", self.black_kingside_castle));
        pos_str.push_str(&format!("{}", self.black_queenside_castle));

        pos_str
    }

    /**
     * Check for threefold repetition draw
     */
    pub fn is_repetition_draw(&self) -> bool {
        let position_string: String = self.to_position_string();

        // Check how many times this position has occurred
        let position_counter: usize = match self.position_history.get(&position_string) {
            Some(count) => *count,
            None => 0,
        };

        position_counter >= 3
    }

    pub fn record_position(&mut self) {
        let position_string: String = self.to_position_string();

        // Retrieve the counter (or insert one with 0 count) and increment it
        let position_counter: &mut usize = self.position_history.entry(position_string).or_insert(0);
        *position_counter += 1;
    }

    pub fn clear_position_history(&mut self) {
        self.position_history.clear();
    }

    // check if a piece is a knight
    pub fn is_knight(&self, piece: Piece) -> bool {
        matches!(piece, 
            Piece::KnightWhite | Piece::KnightBlack
        )
    }

    // check if a piece is a king
    pub fn is_king(&self, piece: Piece) -> bool {
        matches!(piece, 
            Piece::KingWhite | Piece::KingBlack
        )
    }

    pub fn is_pawn(&self, piece: Piece) -> bool {
        matches!(piece, 
            Piece::PawnWhite | Piece::PawnBlack
        )
    }

    pub fn is_bishop(&self, piece: Piece) -> bool {
        matches!(piece, 
            Piece::BishopWhite | Piece::BishopBlack
        )
    }

    pub fn is_rook(&self, piece: Piece) -> bool {
        matches!(piece, 
            Piece::RookWhite | Piece::RookBlack
        )
    }

    pub fn is_queen(&self, piece: Piece) -> bool {
        matches!(piece, 
            Piece::QueenWhite | Piece::QueenBlack
        )
    }

    pub fn is_empty(&self, piece: Piece) -> bool {
        matches!(piece, 
            Piece::Empty
        )
    }

    pub fn is_white(&self, piece: Piece) -> bool {
        matches!(piece, Piece::PawnWhite | Piece::KnightWhite | Piece::BishopWhite | Piece::RookWhite | Piece::QueenWhite | Piece::KingWhite)
    }

    pub fn is_black(&self, piece: Piece) -> bool {
        matches!(piece, Piece::PawnBlack | Piece::KnightBlack | Piece::BishopBlack | Piece::RookBlack | Piece::QueenBlack | Piece::KingBlack)
    }

    pub fn same_color(&self, piece1: Piece, piece2: Piece) -> bool {
        (self.is_white(piece1) && self.is_white(piece2)) ||
        (self.is_black(piece1) && self.is_black(piece2))
    }

    pub fn piece_color(&self, piece: Piece) -> Option<Color> {
        if self.is_white(piece) {
            Some(Color::White)
        }
        else if self.is_black(piece) {
            Some(Color::Black)
        }
        else {
            None
        }
    }

    // print the board in a nice format
    pub fn print(&self) {
        print!("{}", self);
    }
}

#[cfg(test)]
mod test {
    use crate::constants::in_bounds;

    use super::*;

    #[test]
    fn test_board_initialization() {
        let board: Board = Board::new();
        assert_eq!(board.side_to_move, Color::White);
        assert_eq!(board.en_passant, None);
        assert_eq!(board.white_kingside_castle, true);
        assert_eq!(board.white_queenside_castle, true);
        assert_eq!(board.black_kingside_castle, true);
        assert_eq!(board.black_queenside_castle, true);

        // Check kings position
        assert_eq!(board.squares[0][4], Piece::KingWhite);
        assert_eq!(board.squares[7][4], Piece::KingBlack);
    }

    #[test]
    fn test_piece_helpers() {
        let board = Board::new();
        assert_eq!(board.is_white(Piece::PawnWhite), true);
        assert_eq!(board.is_white(Piece::PawnBlack), false);
        assert_eq!(board.is_black(Piece::PawnBlack), true);
        assert_eq!(board.is_black(Piece::PawnWhite), false);
        assert_eq!(board.same_color(Piece::KnightWhite, Piece::PawnWhite), true);
        assert_eq!(board.same_color(Piece::PawnWhite, Piece::PawnBlack), false);
        assert_eq!(in_bounds(0, 0), true);
        assert_eq!(in_bounds(7, 7), true);
        assert_eq!(in_bounds(2, 8), false);
        assert_eq!(in_bounds(8, 3), false);
    }
}
