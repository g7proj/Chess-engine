use super::{Board, Piece, Color};

impl Board {
    /// Generate a FEN position string (piece placement only)
    pub fn position_string(&self) -> String {
        let mut position_string: String = String::new();

        // Piece placement (from white's perspective)
        for rank in (0..8).rev() {
            let mut empty_count = 0;
            for file in 0..8 {
                let piece = self.squares[rank][file];
                if piece == Piece::Empty {
                    empty_count += 1;
                } else {
                    if empty_count > 0 {
                        position_string.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    position_string.push(piece.to_fen_char());
                }
            }
            if empty_count > 0 {
                position_string.push_str(&empty_count.to_string());
            }
            if rank > 0 {
                position_string.push('/');
            }
        }
        position_string
    }

    /// Generate a FEN string representing the current board position
    pub fn to_fen(&self) -> String {
        // 1. Piece placement (from white's perspective)
        let mut fen: String = self.position_string();
        fen.push(' ');

        // 2. Active color
        fen.push(self.side_to_move.to_fen_char());

        fen.push(' ');

        // 3. Castling rights
        let mut castling = String::new();
        if self.white_kingside_castle {
            castling.push('K');
        }
        if self.white_queenside_castle {
            castling.push('Q');
        }
        if self.black_kingside_castle {
            castling.push('k');
        }
        if self.black_queenside_castle {
            castling.push('q');
        }
        if castling.is_empty() {
            castling.push('-');
        }
        fen.push_str(&castling);

        fen.push(' ');

        // 4. En passant target square
        if let Some((ep_r, ep_f)) = self.en_passant {
            let file_char = (b'a' + ep_f as u8) as char;
            let rank_char = (b'1' + ep_r as u8) as char;
            fen.push(file_char);
            fen.push(rank_char);
        } else {
            fen.push('-');
        }

        fen.push(' ');

        // 5. Halfmove clock (50-move rule)
        fen.push_str(&self.halfmove_clock.to_string());

        fen.push(' ');

        // 6. Fullmove number
        let fullmove = 1 + (self.halfmove_clock / 2);
        fen.push_str(&fullmove.to_string());

        fen
    }

    /// Load a board position from a FEN string
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let mut board = Board::new();
        let parts: Vec<&str> = fen.split_whitespace().collect();

        if parts.len() < 6 {
            return Err("FEN must have 6 parts".to_string());
        }

        // 1. Parse piece placement
        for r in 0..8 {
            board.squares[r] = [Piece::Empty; 8];
        }

        let placement = parts[0];
        let mut rank = 7;
        let mut file = 0;

        for ch in placement.chars() {
            if ch == '/' {
                rank -= 1;
                file = 0;
            } else if ch.is_ascii_digit() {
                file += ch.to_digit(10).unwrap() as usize;
            } else {
                if file >= 8 {
                    return Err("Invalid FEN: file out of bounds".to_string());
                }
                board.squares[rank][file] = Piece::from_fen_char(ch)?;
                file += 1;
            }
        }

        // 2. Active color
        board.side_to_move = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("Invalid active color".to_string()),
        };

        // 3. Castling rights
        let castling = parts[2];
        board.white_kingside_castle = castling.contains('K');
        board.white_queenside_castle = castling.contains('Q');
        board.black_kingside_castle = castling.contains('k');
        board.black_queenside_castle = castling.contains('q');

        // 4. En passant
        board.en_passant = if parts[3] == "-" {
            None
        } else {
            let ep_file = (parts[3].chars().nth(0).unwrap() as u8 - b'a') as usize;
            let ep_rank = (parts[3].chars().nth(1).unwrap() as u8 - b'1') as usize;
            Some((ep_rank, ep_file))
        };

        // 5. Halfmove clock
        board.halfmove_clock = parts[4].parse().unwrap_or(0);

        Ok(board)
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..8).rev() {
            write!(f, "{} ", rank + 1)?;
            for file in 0..8 {
                let piece = self.squares[rank][file];
                write!(f, "{} ", piece.to_fen_char())?;
            }
            writeln!(f)?;
        }
        writeln!(f, "  a b c d e f g h")?;
        writeln!(f, "Side to move: {}", self.side_to_move.to_fen_char())?;
        writeln!(f, "Fullmove number: {}", self.fullmove_number)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fen_startpos() {
        let board = Board::new();
        let fen = board.to_fen();
        assert_eq!(fen, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    }

    #[test]
    fn test_from_fen_startpos() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = Board::from_fen(fen).unwrap();
        assert_eq!(board.to_fen(), fen);
    }
}
