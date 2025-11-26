use super::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Piece {
    pub fn to_fen_char(&self) -> char {
        use Piece::*;
        match self {
            PawnWhite => 'P',
            PawnBlack => 'p',
            KnightWhite => 'N',
            KnightBlack => 'n',
            BishopWhite => 'B',
            BishopBlack => 'b',
            RookWhite => 'R',
            RookBlack => 'r',
            QueenWhite => 'Q',
            QueenBlack => 'q',
            KingWhite => 'K',
            KingBlack => 'k',
            Empty => ' ',
        }
    }

    pub fn from_fen_char(ch: char) -> Result<Self, String> {
        use Piece::*;
        match ch {
            'P' => Ok(PawnWhite),
            'p' => Ok(PawnBlack),
            'N' => Ok(KnightWhite),
            'n' => Ok(KnightBlack),
            'B' => Ok(BishopWhite),
            'b' => Ok(BishopBlack),
            'R' => Ok(RookWhite),
            'r' => Ok(RookBlack),
            'Q' => Ok(QueenWhite),
            'q' => Ok(QueenBlack),
            'K' => Ok(KingWhite),
            'k' => Ok(KingBlack),
            _ => Err(format!("Invalid FEN character: {}", ch)),
        }
    }

    pub fn color(&self) -> Option<Color> {
        use Piece::*;
        match self {
            PawnWhite | KnightWhite | BishopWhite | RookWhite | QueenWhite | KingWhite => {
                Some(Color::White)
            }
            PawnBlack | KnightBlack | BishopBlack | RookBlack | QueenBlack | KingBlack => {
                Some(Color::Black)
            }
            Empty => None,
        }
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_fen_char())
    }
}