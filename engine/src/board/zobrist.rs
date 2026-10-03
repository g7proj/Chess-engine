use super::{Board, Piece};

const SIDE_KEY_ID: u64 = 768;
const CASTLING_KEY_ID: u64 = 769;
const EN_PASSANT_KEY_ID: u64 = 773;

impl Board {
    pub(crate) fn refresh_zobrist_key(&mut self) {
        self.zobrist_key = self.calculate_zobrist_key();
    }

    pub(crate) fn calculate_zobrist_key(&self) -> u64 {
        let mut key = 0;
        for rank in 0..8 {
            for file in 0..8 {
                let piece = self.squares[rank][file];
                if let Some(index) = piece_index(piece) {
                    key ^= zobrist_value(index * 64 + (rank * 8 + file) as u64);
                }
            }
        }
        if self.side_to_move == super::Color::Black {
            key ^= zobrist_value(SIDE_KEY_ID);
        }
        for (index, enabled) in [
            self.white_kingside_castle,
            self.white_queenside_castle,
            self.black_kingside_castle,
            self.black_queenside_castle,
        ]
        .into_iter()
        .enumerate()
        {
            if enabled {
                key ^= zobrist_value(CASTLING_KEY_ID + index as u64);
            }
        }
        if let Some((rank, file)) = self.en_passant {
            key ^= zobrist_value(EN_PASSANT_KEY_ID + (rank * 8 + file) as u64);
        }
        key
    }
}

pub(crate) fn piece_key(piece: Piece, rank: usize, file: usize) -> u64 {
    piece_index(piece)
        .map(|index| zobrist_value(index * 64 + (rank * 8 + file) as u64))
        .unwrap_or(0)
}

pub(crate) fn side_key() -> u64 {
    zobrist_value(SIDE_KEY_ID)
}

pub(crate) fn castling_key(index: usize) -> u64 {
    zobrist_value(CASTLING_KEY_ID + index as u64)
}

pub(crate) fn en_passant_key(rank: usize, file: usize) -> u64 {
    zobrist_value(EN_PASSANT_KEY_ID + (rank * 8 + file) as u64)
}

fn piece_index(piece: Piece) -> Option<u64> {
    use Piece::*;
    match piece {
        PawnWhite => Some(0),
        KnightWhite => Some(1),
        BishopWhite => Some(2),
        RookWhite => Some(3),
        QueenWhite => Some(4),
        KingWhite => Some(5),
        PawnBlack => Some(6),
        KnightBlack => Some(7),
        BishopBlack => Some(8),
        RookBlack => Some(9),
        QueenBlack => Some(10),
        KingBlack => Some(11),
        Empty => None,
    }
}

fn zobrist_value(id: u64) -> u64 {
    let mut value = 0x9e37_79b9_7f4a_7c15_u64.wrapping_mul(id + 1);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::{Move, Promotion};

    #[test]
    fn test_zobrist_key_covers_side_castling_and_en_passant() {
        let initial = Board::new();
        let mut changed = initial.clone();
        changed.side_to_move = super::super::Color::Black;
        assert_ne!(initial.zobrist_key(), changed.zobrist_key());

        changed = initial.clone();
        changed.white_kingside_castle = false;
        assert_ne!(initial.zobrist_key(), changed.zobrist_key());

        changed = initial.clone();
        changed.en_passant = Some((2, 4));
        assert_ne!(initial.zobrist_key(), changed.zobrist_key());

        changed = initial.clone();
        changed.squares[1][0] = Piece::Empty;
        assert_ne!(initial.zobrist_key(), changed.zobrist_key());

        assert_eq!(
            Board::from_fen(&initial.to_fen()).unwrap().zobrist_key,
            initial.zobrist_key
        );
    }

    #[test]
    fn test_incremental_zobrist_key_matches_recalculation_after_special_moves() {
        let cases = [
            (Board::new(), Move::new(1, 4, 3, 4)),
            (
                Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap(),
                Move::new(0, 4, 0, 6),
            ),
            (
                Board::from_fen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1").unwrap(),
                Move::new(4, 4, 5, 3),
            ),
            (
                Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap(),
                Move::with_promotion(6, 0, 7, 0, Promotion::Queen),
            ),
            (
                Board::from_fen("r3k3/8/8/8/8/8/8/R3K3 w q - 0 1").unwrap(),
                Move::new(0, 0, 7, 0),
            ),
        ];

        for (mut board, mv) in cases {
            let original = board.zobrist_key;
            let undo = board.make_move_for_search(mv);
            assert_eq!(board.zobrist_key, board.calculate_zobrist_key());
            board.unmake_move(undo);
            assert_eq!(board.zobrist_key, original);
        }
    }
}
