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
        let piece_char = match self {
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

#[derive(Debug, Clone)]
pub struct Board {
    pub squares: [[Piece; 8]; 8],
    pub side_to_move: char,
    pub en_passant: Option<(usize, usize)>,
    pub white_kingside_castle: bool,
    pub white_queenside_castle: bool,
    pub black_kingside_castle: bool,
    pub black_queenside_castle: bool,
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r_index = 8;
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
        let squares = [
            [RookWhite, KnightWhite, BishopWhite, QueenWhite, KingWhite, BishopWhite, KnightWhite, RookWhite],
            [PawnWhite; 8],
            [Empty; 8],
            [Empty; 8],
            [Empty; 8],
            [Empty; 8],
            [PawnBlack; 8],
            [RookBlack, KnightBlack, BishopBlack, QueenBlack, KingBlack, BishopBlack, KnightBlack, RookBlack],
        ];
        Board { 
            squares,
            side_to_move: 'w',
            en_passant: None,
            white_kingside_castle: true,
            white_queenside_castle: true,
            black_kingside_castle: true,
            black_queenside_castle: true,
        }
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

    // print the board in a nice format
    pub fn print(&self) {
        print!("{}", self);
    }
}
