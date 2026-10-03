use super::{Board, Color, Piece};
use crate::moves::{Move, Promotion};

/// Test-only bitboard representation used to compare attack detection costs.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BitboardPosition {
    pieces: [u64; 12],
    occupancy: [u64; 2],
    side_to_move: Color,
    en_passant: Option<(usize, usize)>,
    castling: [bool; 4],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BitboardUndo {
    from: usize,
    to: usize,
    moving: usize,
    placed: usize,
    captured: Option<(usize, usize)>,
    rook: Option<(usize, usize, usize)>,
    previous_side: Color,
    previous_en_passant: Option<(usize, usize)>,
    previous_castling: [bool; 4],
}

impl BitboardPosition {
    pub(crate) fn from_board(board: &Board) -> Self {
        let mut position = Self {
            pieces: [0; 12],
            occupancy: [0; 2],
            side_to_move: board.side_to_move,
            en_passant: board.en_passant,
            castling: [
                board.white_kingside_castle,
                board.white_queenside_castle,
                board.black_kingside_castle,
                board.black_queenside_castle,
            ],
        };
        for rank in 0..8 {
            for file in 0..8 {
                let piece = board.squares[rank][file];
                let Some((color, piece_type)) = piece_parts(piece) else {
                    continue;
                };
                let bit = 1_u64 << (rank * 8 + file);
                let index = color_index(color) * 6 + piece_type;
                position.pieces[index] |= bit;
                position.occupancy[color_index(color)] |= bit;
            }
        }
        position
    }

    pub(crate) fn is_square_attacked(&self, rank: usize, file: usize, by_color: Color) -> bool {
        let target = rank * 8 + file;
        let color_offset = color_index(by_color) * 6;

        if self.pieces[color_offset + 1] & knight_attacks(target) != 0
            || self.pieces[color_offset + 5] & king_attacks(target) != 0
        {
            return true;
        }

        let pawn_sources = pawn_attacks(target, by_color);
        if self.pieces[color_offset] & pawn_sources != 0 {
            return true;
        }

        let occupancy = self.occupancy[0] | self.occupancy[1];
        let directions = [
            (1, 0, false),
            (-1, 0, false),
            (0, 1, false),
            (0, -1, false),
            (1, 1, true),
            (1, -1, true),
            (-1, 1, true),
            (-1, -1, true),
        ];
        for (dr, df, diagonal) in directions {
            let mut ray_rank = rank as isize + dr;
            let mut ray_file = file as isize + df;
            let mut distance = 1;
            while (0..8).contains(&ray_rank) && (0..8).contains(&ray_file) {
                let square = ray_rank as usize * 8 + ray_file as usize;
                let bit = 1_u64 << square;
                if occupancy & bit != 0 {
                    let slider_types = if diagonal { [2, 4] } else { [3, 4] };
                    if distance == 1 && self.pieces[color_offset + 5] & bit != 0 {
                        return true;
                    }
                    if slider_types
                        .iter()
                        .any(|&piece_type| self.pieces[color_offset + piece_type] & bit != 0)
                    {
                        return true;
                    }
                    break;
                }
                ray_rank += dr;
                ray_file += df;
                distance += 1;
            }
        }
        false
    }

    pub(crate) fn is_in_check(&self, color: Color) -> bool {
        let king_index = color_index(color) * 6 + 5;
        let Some(square) = (self.pieces[king_index] != 0)
            .then(|| self.pieces[king_index].trailing_zeros() as usize)
        else {
            return false;
        };
        self.is_square_attacked(square / 8, square % 8, color.opposite())
    }

    fn piece_at(&self, square: usize) -> Option<usize> {
        let bit = 1_u64 << square;
        let color_offset = if self.occupancy[0] & bit != 0 {
            0
        } else if self.occupancy[1] & bit != 0 {
            6
        } else {
            return None;
        };
        self.pieces[color_offset..color_offset + 6]
            .iter()
            .position(|piece_mask| piece_mask & bit != 0)
            .map(|piece_index| color_offset + piece_index)
    }

    pub(crate) fn piece_on(&self, rank: usize, file: usize) -> Piece {
        self.piece_at(rank * 8 + file)
            .map(piece_from_index)
            .unwrap_or(Piece::Empty)
    }

    pub(crate) fn active_color(&self) -> Color {
        self.side_to_move
    }

    pub(crate) fn is_capture(&self, mv: Move) -> bool {
        let target = mv.to_rank * 8 + mv.to_file;
        let moving = self.piece_at(mv.from_rank * 8 + mv.from_file);
        self.piece_at(target).is_some()
            || (moving.is_some_and(|piece| piece % 6 == 0)
                && self.en_passant == Some((mv.to_rank, mv.to_file)))
    }

    pub(crate) fn evaluation_controls_square(
        &self,
        target_rank: usize,
        target_file: usize,
        color: Color,
    ) -> bool {
        let color_offset = color_index(color) * 6;
        let target_rank = target_rank as isize;
        let target_file = target_file as isize;
        for piece_type in 1..6 {
            let mut sources = self.pieces[color_offset + piece_type];
            while sources != 0 {
                let square = sources.trailing_zeros() as usize;
                sources &= sources - 1;
                let rank = (square / 8) as isize;
                let file = (square % 8) as isize;
                let dr = target_rank - rank;
                let df = target_file - file;
                let abs_dr = dr.abs();
                let abs_df = df.abs();
                let matches = match piece_type {
                    1 => (abs_dr == 2 && abs_df == 1) || (abs_dr == 1 && abs_df == 2),
                    2 => abs_dr == abs_df && abs_dr > 0,
                    3 => (dr == 0) != (df == 0),
                    4 => (dr == 0) != (df == 0) || (abs_dr == abs_df && abs_dr > 0),
                    5 => abs_dr <= 1 && abs_df <= 1 && (abs_dr != 0 || abs_df != 0),
                    _ => unreachable!(),
                };
                if !matches {
                    continue;
                }
                if piece_type == 1 || piece_type == 5 {
                    return true;
                }
                let mut ray_rank = rank + dr.signum();
                let mut ray_file = file + df.signum();
                while (ray_rank, ray_file) != (target_rank, target_file) {
                    let bit = 1_u64 << (ray_rank as usize * 8 + ray_file as usize);
                    if self.occupancy[0] & bit != 0 || self.occupancy[1] & bit != 0 {
                        break;
                    }
                    ray_rank += dr.signum();
                    ray_file += df.signum();
                }
                if (ray_rank, ray_file) == (target_rank, target_file) {
                    return true;
                }
            }
        }
        false
    }

    /// Generates legal moves from the complete bitboard position state.
    pub(crate) fn generate_legal_moves(&mut self) -> Vec<Move> {
        let color = self.side_to_move;
        self.generate_pseudo_legal_moves()
            .into_iter()
            .filter(|&mv| {
                let bitboard_undo = self.make_move(mv);
                let legal = !self.is_in_check(color);
                self.unmake_move(bitboard_undo);
                legal
            })
            .collect()
    }

    fn generate_pseudo_legal_moves(&self) -> Vec<Move> {
        let color = self.side_to_move;
        let color_index = color_index(color);
        let own = self.occupancy[color_index];
        let enemy = self.occupancy[1 - color_index];
        let mut moves = Vec::new();

        let mut pieces = own;
        while pieces != 0 {
            let from = pieces.trailing_zeros() as usize;
            pieces &= pieces - 1;
            let piece_type = self.piece_at(from).unwrap() % 6;
            match piece_type {
                0 => self.generate_pawn_moves(from, color, enemy, &mut moves),
                1 => self.generate_leaper_moves(
                    from,
                    &[
                        (2, 1),
                        (2, -1),
                        (-2, 1),
                        (-2, -1),
                        (1, 2),
                        (1, -2),
                        (-1, 2),
                        (-1, -2),
                    ],
                    own,
                    &mut moves,
                ),
                5 => {
                    self.generate_leaper_moves(
                        from,
                        &[
                            (1, 0),
                            (-1, 0),
                            (0, 1),
                            (0, -1),
                            (1, 1),
                            (-1, -1),
                            (1, -1),
                            (-1, 1),
                        ],
                        own,
                        &mut moves,
                    );
                    self.generate_castling_moves(from, color, own | enemy, &mut moves);
                }
                2 => self.generate_sliding_moves(
                    from,
                    &[(1, 1), (-1, -1), (1, -1), (-1, 1)],
                    own,
                    enemy,
                    &mut moves,
                ),
                3 => self.generate_sliding_moves(
                    from,
                    &[(1, 0), (-1, 0), (0, 1), (0, -1)],
                    own,
                    enemy,
                    &mut moves,
                ),
                4 => self.generate_sliding_moves(
                    from,
                    &[
                        (1, 0),
                        (-1, 0),
                        (0, 1),
                        (0, -1),
                        (1, 1),
                        (-1, -1),
                        (1, -1),
                        (-1, 1),
                    ],
                    own,
                    enemy,
                    &mut moves,
                ),
                _ => unreachable!(),
            }
        }
        moves
    }

    fn generate_leaper_moves(
        &self,
        from: usize,
        offsets: &[(isize, isize)],
        own: u64,
        moves: &mut Vec<Move>,
    ) {
        let (rank, file) = ((from / 8) as isize, (from % 8) as isize);
        for &(dr, df) in offsets {
            let (target_rank, target_file) = (rank + dr, file + df);
            if !(0..8).contains(&target_rank) || !(0..8).contains(&target_file) {
                continue;
            }
            let to = target_rank as usize * 8 + target_file as usize;
            if own & (1_u64 << to) == 0 {
                moves.push(Move::new(from / 8, from % 8, to / 8, to % 8));
            }
        }
    }

    fn generate_sliding_moves(
        &self,
        from: usize,
        directions: &[(isize, isize)],
        own: u64,
        enemy: u64,
        moves: &mut Vec<Move>,
    ) {
        let rank = (from / 8) as isize;
        let file = (from % 8) as isize;
        for &(dr, df) in directions {
            let (mut target_rank, mut target_file) = (rank + dr, file + df);
            while (0..8).contains(&target_rank) && (0..8).contains(&target_file) {
                let to = target_rank as usize * 8 + target_file as usize;
                let bit = 1_u64 << to;
                if own & bit != 0 {
                    break;
                }
                moves.push(Move::new(from / 8, from % 8, to / 8, to % 8));
                if enemy & bit != 0 {
                    break;
                }
                target_rank += dr;
                target_file += df;
            }
        }
    }

    fn generate_pawn_moves(&self, from: usize, color: Color, enemy: u64, moves: &mut Vec<Move>) {
        let rank = (from / 8) as isize;
        let file = (from % 8) as isize;
        let (forward, start_rank, promotion_rank) = if color == Color::White {
            (1, 1, 7)
        } else {
            (-1, 6, 0)
        };
        let one_rank = rank + forward;
        if (0..8).contains(&one_rank) {
            let one = one_rank as usize * 8 + file as usize;
            if (self.occupancy[0] | self.occupancy[1]) & (1_u64 << one) == 0 {
                push_pawn_move(from, one, promotion_rank, moves);
                let two_rank = rank + 2 * forward;
                if rank as usize == start_rank {
                    let two = two_rank as usize * 8 + file as usize;
                    if (self.occupancy[0] | self.occupancy[1]) & (1_u64 << two) == 0 {
                        moves.push(Move::new(from / 8, from % 8, two / 8, two % 8));
                    }
                }
            }
            for df in [-1, 1] {
                let target_file = file + df;
                if !(0..8).contains(&target_file) {
                    continue;
                }
                let to = one_rank as usize * 8 + target_file as usize;
                if enemy & (1_u64 << to) != 0 {
                    push_pawn_move(from, to, promotion_rank, moves);
                } else if self.en_passant == Some((one_rank as usize, target_file as usize)) {
                    moves.push(Move::new(from / 8, from % 8, to / 8, to % 8));
                }
            }
        }
    }

    fn generate_castling_moves(
        &self,
        from: usize,
        color: Color,
        occupancy: u64,
        moves: &mut Vec<Move>,
    ) {
        let (home, enemy, king_side, queen_side) = if color == Color::White {
            (4, Color::Black, self.castling[0], self.castling[1])
        } else {
            (60, Color::White, self.castling[2], self.castling[3])
        };
        if from != home || self.is_square_attacked(home / 8, home % 8, enemy) {
            return;
        }
        let rank = home / 8;
        if king_side
            && self.piece_at(home + 3) == Some(color_index(color) * 6 + 3)
            && occupancy & ((1_u64 << (home + 1)) | (1_u64 << (home + 2))) == 0
            && !(self.is_square_attacked(rank, 5, enemy) || self.is_square_attacked(rank, 6, enemy))
        {
            moves.push(Move::new(rank, 4, rank, 6));
        }
        if queen_side
            && self.piece_at(home - 4) == Some(color_index(color) * 6 + 3)
            && occupancy & ((1_u64 << (home - 1)) | (1_u64 << (home - 2)) | (1_u64 << (home - 3)))
                == 0
            && !(self.is_square_attacked(rank, 3, enemy) || self.is_square_attacked(rank, 2, enemy))
        {
            moves.push(Move::new(rank, 4, rank, 2));
        }
    }

    /// Applies a move and records all piece and rule metadata needed for undo.
    pub(crate) fn make_move(&mut self, mv: Move) -> BitboardUndo {
        let from = mv.from_rank * 8 + mv.from_file;
        let to = mv.to_rank * 8 + mv.to_file;
        let moving = self.piece_at(from).unwrap();
        let placed = promoted_piece_index(moving, mv.promotion).unwrap_or(moving);
        let previous_side = self.side_to_move;
        let previous_en_passant = self.en_passant;
        let previous_castling = self.castling;
        let capture_square = if moving % 6 == 0
            && self.en_passant == Some((mv.to_rank, mv.to_file))
            && self.piece_at(to).is_none()
        {
            Some(if moving < 6 { to - 8 } else { to + 8 })
        } else if self.piece_at(to).is_some() {
            Some(to)
        } else {
            None
        };
        let captured = capture_square.map(|square| {
            let piece = self.piece_at(square).unwrap();
            (square, piece)
        });

        if moving % 6 == 5 {
            let offset = moving / 6 * 2;
            self.castling[offset] = false;
            self.castling[offset + 1] = false;
        } else if moving % 6 == 3 {
            self.clear_rook_castling(from);
        }
        if let Some((square, piece)) = captured
            && piece % 6 == 3
        {
            self.clear_rook_castling(square);
        }
        let rook = if moving % 6 == 5 && mv.from_file.abs_diff(mv.to_file) == 2 {
            let rank = mv.from_rank * 8;
            if mv.to_file == 6 {
                Some((rank + 7, rank + 5, moving - 2))
            } else {
                Some((rank, rank + 3, moving - 2))
            }
        } else {
            None
        };

        self.remove_piece(moving, from);
        if let Some((square, piece)) = captured {
            self.remove_piece(piece, square);
        }
        self.add_piece(placed, to);
        if let Some((rook_from, rook_to, rook_piece)) = rook {
            self.remove_piece(rook_piece, rook_from);
            self.add_piece(rook_piece, rook_to);
        }
        self.en_passant = if moving % 6 == 0
            && mv.from_file == mv.to_file
            && mv.from_rank.abs_diff(mv.to_rank) == 2
        {
            Some(((mv.from_rank + mv.to_rank) / 2, mv.from_file))
        } else {
            None
        };
        self.side_to_move = self.side_to_move.opposite();

        BitboardUndo {
            from,
            to,
            moving,
            placed,
            captured,
            rook,
            previous_side,
            previous_en_passant,
            previous_castling,
        }
    }

    /// Reverses the exact mask updates recorded while applying a move.
    pub(crate) fn unmake_move(&mut self, undo: BitboardUndo) {
        self.remove_piece(undo.placed, undo.to);
        self.add_piece(undo.moving, undo.from);
        if let Some((square, piece)) = undo.captured {
            self.add_piece(piece, square);
        }
        if let Some((rook_from, rook_to, rook_piece)) = undo.rook {
            self.remove_piece(rook_piece, rook_to);
            self.add_piece(rook_piece, rook_from);
        }
        self.side_to_move = undo.previous_side;
        self.en_passant = undo.previous_en_passant;
        self.castling = undo.previous_castling;
    }

    fn clear_rook_castling(&mut self, square: usize) {
        match square {
            0 => self.castling[1] = false,
            7 => self.castling[0] = false,
            56 => self.castling[3] = false,
            63 => self.castling[2] = false,
            _ => {}
        }
    }

    fn add_piece(&mut self, piece: usize, square: usize) {
        let bit = 1_u64 << square;
        self.pieces[piece] |= bit;
        self.occupancy[piece / 6] |= bit;
    }

    fn remove_piece(&mut self, piece: usize, square: usize) {
        let bit = !(1_u64 << square);
        self.pieces[piece] &= bit;
        self.occupancy[piece / 6] &= bit;
    }

    pub(crate) fn matches_board(&self, board: &Board) -> bool {
        self == &Self::from_board(board)
    }
}

impl Board {
    pub(crate) fn enable_bitboard_shadow(&mut self) {
        self.bitboard_shadow = Some(BitboardPosition::from_board(self));
    }
}

impl PartialEq for BitboardPosition {
    fn eq(&self, other: &Self) -> bool {
        self.pieces == other.pieces
            && self.occupancy == other.occupancy
            && self.side_to_move == other.side_to_move
            && self.en_passant == other.en_passant
            && self.castling == other.castling
    }
}

impl Eq for BitboardPosition {}

fn promoted_piece_index(piece: usize, promotion: Promotion) -> Option<usize> {
    let kind = match promotion {
        Promotion::Queen => 4,
        Promotion::Rook => 3,
        Promotion::Bishop => 2,
        Promotion::Knight => 1,
        Promotion::None => return None,
    };
    Some((piece / 6) * 6 + kind)
}

fn piece_from_index(index: usize) -> Piece {
    use Piece::*;
    match index {
        0 => PawnWhite,
        1 => KnightWhite,
        2 => BishopWhite,
        3 => RookWhite,
        4 => QueenWhite,
        5 => KingWhite,
        6 => PawnBlack,
        7 => KnightBlack,
        8 => BishopBlack,
        9 => RookBlack,
        10 => QueenBlack,
        11 => KingBlack,
        _ => unreachable!(),
    }
}

fn push_pawn_move(from: usize, to: usize, promotion_rank: isize, moves: &mut Vec<Move>) {
    use Promotion::*;
    let (from_rank, from_file, to_rank, to_file) = (from / 8, from % 8, to / 8, to % 8);
    if to_rank as isize == promotion_rank {
        for promotion in [Queen, Rook, Bishop, Knight] {
            moves.push(Move::with_promotion(
                from_rank, from_file, to_rank, to_file, promotion,
            ));
        }
    } else {
        moves.push(Move::new(from_rank, from_file, to_rank, to_file));
    }
}

fn color_index(color: Color) -> usize {
    if color == Color::White { 0 } else { 1 }
}

fn piece_parts(piece: Piece) -> Option<(Color, usize)> {
    use Piece::*;
    match piece {
        PawnWhite => Some((Color::White, 0)),
        KnightWhite => Some((Color::White, 1)),
        BishopWhite => Some((Color::White, 2)),
        RookWhite => Some((Color::White, 3)),
        QueenWhite => Some((Color::White, 4)),
        KingWhite => Some((Color::White, 5)),
        PawnBlack => Some((Color::Black, 0)),
        KnightBlack => Some((Color::Black, 1)),
        BishopBlack => Some((Color::Black, 2)),
        RookBlack => Some((Color::Black, 3)),
        QueenBlack => Some((Color::Black, 4)),
        KingBlack => Some((Color::Black, 5)),
        Empty => None,
    }
}

fn knight_attacks(square: usize) -> u64 {
    let rank = (square / 8) as isize;
    let file = (square % 8) as isize;
    [
        (2, 1),
        (2, -1),
        (-2, 1),
        (-2, -1),
        (1, 2),
        (1, -2),
        (-1, 2),
        (-1, -2),
    ]
    .iter()
    .fold(0, |mask, &(dr, df)| {
        let target_rank = rank + dr;
        let target_file = file + df;
        if (0..8).contains(&target_rank) && (0..8).contains(&target_file) {
            mask | (1_u64 << (target_rank as usize * 8 + target_file as usize))
        } else {
            mask
        }
    })
}

fn king_attacks(square: usize) -> u64 {
    let rank = (square / 8) as isize;
    let file = (square % 8) as isize;
    (-1..=1).fold(0, |mask, dr| {
        (-1..=1).fold(mask, |mask, df| {
            let target_rank = rank + dr;
            let target_file = file + df;
            if (dr != 0 || df != 0)
                && (0..8).contains(&target_rank)
                && (0..8).contains(&target_file)
            {
                mask | (1_u64 << (target_rank as usize * 8 + target_file as usize))
            } else {
                mask
            }
        })
    })
}

fn pawn_attacks(square: usize, pawn_color: Color) -> u64 {
    let rank = (square / 8) as isize;
    let file = (square % 8) as isize;
    let source_rank = rank + if pawn_color == Color::White { -1 } else { 1 };
    [-1, 1].iter().fold(0, |mask, &df| {
        let source_file = file + df;
        if (0..8).contains(&source_rank) && (0..8).contains(&source_file) {
            mask | (1_u64 << (source_rank as usize * 8 + source_file as usize))
        } else {
            mask
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::Undo;

    fn move_sort_key(mv: &Move) -> (usize, usize, usize, usize, u8) {
        let promotion = match mv.promotion {
            Promotion::None => 0,
            Promotion::Queen => 1,
            Promotion::Rook => 2,
            Promotion::Bishop => 3,
            Promotion::Knight => 4,
        };
        (
            mv.from_rank,
            mv.from_file,
            mv.to_rank,
            mv.to_file,
            promotion,
        )
    }

    fn paired_perft(
        board: &mut Board,
        bitboards: &mut BitboardPosition,
        depth: usize,
        verify_each_move: bool,
    ) -> u64 {
        if depth == 0 {
            return 1;
        }
        let moves = bitboards.generate_legal_moves();
        let mut nodes = 0;
        for mv in moves {
            let bitboard_undo = bitboards.make_move(mv);
            let undo: Undo = board.make_move_for_search(mv);
            if verify_each_move {
                assert!(
                    bitboards.matches_board(board),
                    "bitboard mismatch after {mv:?}"
                );
            }
            nodes += paired_perft(board, bitboards, depth - 1, verify_each_move);
            board.unmake_move(undo);
            bitboards.unmake_move(bitboard_undo);
            if verify_each_move {
                assert!(
                    bitboards.matches_board(board),
                    "bitboard mismatch after undo {mv:?}"
                );
            }
        }
        nodes
    }

    #[test]
    fn test_bitboard_attack_detection_matches_mailbox() {
        let positions = [
            Board::new(),
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap(),
            Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap(),
            Board::from_fen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1").unwrap(),
        ];
        for board in positions {
            let bitboards = BitboardPosition::from_board(&board);
            for color in [Color::White, Color::Black] {
                for rank in 0..8 {
                    for file in 0..8 {
                        assert_eq!(
                            bitboards.is_square_attacked(rank, file, color),
                            board.is_square_under_attack(rank, file, color),
                            "attack mismatch at {rank},{file} for {color:?} in {}",
                            board.to_fen()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_bitboard_move_generation_matches_mailbox_perft_and_restores_state() {
        let positions = [
            (Board::new(), 4),
            (
                Board::from_fen(
                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                )
                .unwrap(),
                3,
            ),
            (
                Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap(),
                3,
            ),
            (
                Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap(),
                2,
            ),
            (
                Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap(),
                2,
            ),
            (
                Board::from_fen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1").unwrap(),
                2,
            ),
        ];
        for (mut board, depth) in positions {
            let fen_before = board.to_fen();
            let expected = board.perft(depth);
            let mut bitboards = BitboardPosition::from_board(&board);
            let reference_board = board.clone();
            let mut mailbox_moves = reference_board.generate_all_legal_moves();
            let mut bitboard_moves = bitboards.generate_legal_moves();
            mailbox_moves.sort_by_key(move_sort_key);
            bitboard_moves.sort_by_key(move_sort_key);
            assert_eq!(
                bitboard_moves, mailbox_moves,
                "root move mismatch for {fen_before}"
            );
            let actual = paired_perft(&mut board, &mut bitboards, depth, true);
            assert_eq!(actual, expected, "perft mismatch for {fen_before}");
            assert_eq!(board.to_fen(), fen_before);
            assert!(bitboards.matches_board(&board));
        }
    }

    #[test]
    #[ignore = "timing depends on the machine"]
    fn benchmark_incremental_bitboard_perft() {
        use std::time::Instant;

        let mut mailbox = Board::new();
        let start = Instant::now();
        let mailbox_nodes = mailbox.perft(5);
        let mailbox_elapsed = start.elapsed();

        let mut paired_board = Board::new();
        let mut bitboards = BitboardPosition::from_board(&paired_board);
        let start = Instant::now();
        let paired_nodes = paired_perft(&mut paired_board, &mut bitboards, 5, false);
        let paired_elapsed = start.elapsed();

        assert_eq!(paired_nodes, mailbox_nodes);
        assert!(bitboards.matches_board(&paired_board));
        println!(
            "Start position perft(5): mailbox reference={mailbox_elapsed:?}, bitboard-generated with mirrored state={paired_elapsed:?}, nodes={mailbox_nodes}"
        );
    }
}
