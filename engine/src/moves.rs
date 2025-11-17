use crate::board::{Board, Piece};

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

    pub fn generate_bishop_moves(&self, rank: usize, file: usize) -> Vec<Move> {
        let mut moves = Vec::new();
        let piece = self.squares[rank][file];
        
        // check if the piece is a bishop
        if !self.is_bishop(piece) {
            return moves;
        }

        let directions = [
            (1, 1), (-1, -1),
            (1, -1), (-1, 1),
        ];

        for (dr, df) in directions {
            let mut new_rank = rank as isize + dr;
            let mut new_file = file as isize + df;

            while new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
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

    pub fn generate_rook_moves(&self, rank: usize, file: usize) -> Vec<Move> {
        let mut moves = Vec::new();
        let piece = self.squares[rank][file];
        
        // check if the piece is a rook
        if !self.is_rook(piece) {
            return moves;
        }

        let directions = [
            (1, 0), (-1, 0),
            (0, 1), (0, -1),
        ];
        
        for (dr, df) in directions {
            let mut new_rank = rank as isize + dr;
            let mut new_file = file as isize + df;

            while new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
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

    pub fn generate_queen_moves(&self, rank: usize, file: usize) -> Vec<Move> {
        let mut moves = Vec::new();
        let piece = self.squares[rank][file];
        
        // check if the piece is a queen
        if !self.is_queen(piece) {
            return moves;
        }

        let directions = [
            (1, 0), (-1, 0),
            (0, 1), (0, -1),
            (1, 1), (-1, -1),
            (1, -1), (-1, 1),
        ];
        
        for (dr, df) in directions {
            let mut new_rank = rank as isize + dr;
            let mut new_file = file as isize + df;

            while new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
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

    pub fn generate_pawn_moves(&self, rank: usize, file: usize) -> Vec<Move> {
        use Piece::*;
        let mut moves = Vec::new();
        let piece = self.squares[rank][file];
        
        let (forward, start_rank, promotion_rank) = match piece {
            PawnWhite => (1, 1, 7),
            PawnBlack => (-1, 6, 0),
            _ => return moves,
        };

        let r = rank as isize;
        let f = file as isize;
    
        // 1. Single move (one step forward)
        let one_step = (r + forward, f);
        if self.in_bounds(one_step) {
            let (nr, nf) = one_step;
            if self.squares[nr as usize][nf as usize] == Empty {
                // promotion?
                if nr as usize == promotion_rank {
                    moves.extend(self.generate_pawn_promotions(rank, file, nr as usize, nf as usize));
                } else {
                    moves.push(Move::new(rank, file, nr as usize, nf as usize));
                }
    
                // 2. Double move (only if the single move is free)
                if rank == start_rank {
                    let two_step = (r + forward * 2, f);
                    if self.in_bounds(two_step) {
                        let (tr, tf) = two_step;
                        if self.squares[tr as usize][tf as usize] == Empty {
                            moves.push(Move::new(rank, file, tr as usize, tf as usize));
                        }
                    }
                }
            }
        }
    
        // 3. Diagonal captures
        for df in [-1, 1] {
            let diag = (r + forward, f + df);
            if self.in_bounds(diag) {
                let (nr, nf) = diag;
                let target = self.squares[nr as usize][nf as usize];
    
                if target != Empty && !self.same_color(piece, target) {
                    if nr as usize == promotion_rank {
                        moves.extend(self.generate_pawn_promotions(rank, file, nr as usize, nf as usize));
                    } else {
                        moves.push(Move::new(rank, file, nr as usize, nf as usize));
                    }
                }
            }
        }
    
        // 4. En passant (implement later)
        if let Some((ep_r, ep_f)) = self.en_passant {
            for df in [-1, 1] {
                let target_file = f + df;
                if target_file >= 0 && target_file < 8 {
                    let attack_r = r + forward;
                    if attack_r == ep_r as isize && target_file == ep_f as isize {
                        moves.push(Move::new(rank, file, ep_r, ep_f));
                    }
                }
            }
        }
    
        moves
    }

    fn generate_pawn_promotions(
        &self,
        fr: usize,
        ff: usize,
        tr: usize,
        tf: usize
    ) -> Vec<Move> {
        use crate::moves::Promotion::*;

        vec![
            Move::with_promotion(fr, ff, tr, tf, Queen),
            Move::with_promotion(fr, ff, tr, tf, Rook),
            Move::with_promotion(fr, ff, tr, tf, Bishop),
            Move::with_promotion(fr, ff, tr, tf, Knight),
        ]
    }

    fn in_bounds(&self, pos: (isize, isize)) -> bool {
        let (r, f) = pos;
        r >= 0 && r < 8 && f >= 0 && f < 8
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

    pub fn is_in_check(&self, color: char) -> bool {
        use Piece::*;

        // find king position
        let king_piece = if color =='w' { KingWhite } else { KingBlack };
        let mut king_rank = 0;
        let mut king_file = 0;
        let mut king_found = false;

        for r in 0..8 {
            for f in 0..8 {
                if self.squares[r][f] == king_piece {
                    king_rank = r;
                    king_file = f;
                    king_found = true;
                    break;
                }
            }
            if king_found { break; }
        }

        if !king_found { return false; }

        // Check if some opponent piece can attak the king
        let opponent_color = if color == 'w' { 'b' } else  { 'w' };

        for r in 0..8 {
            for f in 0..8 {
                let piece = self.squares[r][f];
                if piece != Empty {
                    let piece_color = if self.is_white(piece) { 'w' } else { 'b' };
                    if piece_color == opponent_color {
                        // Generate all moves for this piece
                        let moves = match piece {
                            PawnWhite | PawnBlack => self.generate_pawn_moves(r, f),
                            KnightWhite | KnightBlack => self.generate_knight_moves(r, f),
                            BishopWhite | BishopBlack => self.generate_bishop_moves(r, f),
                            RookWhite | RookBlack => self.generate_rook_moves(r, f),
                            QueenWhite | QueenBlack => self.generate_queen_moves(r, f),
                            KingWhite | KingBlack => self.generate_king_moves(r, f),
                            _ => Vec::new(),
                        };

                        // Check if one move is attacking the king
                        for mv in moves {
                            if mv.to_rank == king_rank && mv.to_file == king_file {
                                // Check!
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    pub fn is_square_under_attack(&self, rank: usize, file: usize, attack_by_color: char) -> bool {
        use Piece::*;

        for r in 0..8 {
            for f in 0..8 {
                let piece = self.squares[r][f];
                if piece != Empty {
                    let piece_color = if self.is_white(piece) { 'w' } else { 'b' };
                    if piece_color == attack_by_color {
                        // Generate all moves for this piece
                        let moves = match piece {
                            PawnWhite | PawnBlack => self.generate_pawn_moves(r, f),
                            KnightWhite | KnightBlack => self.generate_knight_moves(r, f),
                            BishopWhite | BishopBlack => self.generate_bishop_moves(r, f),
                            RookWhite | RookBlack => self.generate_rook_moves(r, f),
                            QueenWhite | QueenBlack => self.generate_queen_moves(r, f),
                            KingWhite | KingBlack => self.generate_king_moves(r, f),
                            _ => Vec::new(),
                        };

                        // check if one move attacks the target square
                        for mv in moves {
                            if mv.to_rank == rank && mv.to_file == file {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    pub fn make_move(&mut self, mv: Move) {
        use Piece::*;
        use crate::moves::Promotion;

        let fr = mv.from_rank;
        let ff = mv.from_file;
        let tr = mv.to_rank;
        let tf = mv.to_file;

        let moving_piece = self.squares[fr][ff];

        // Save current en passant before the reset
        let old_ep = self.en_passant;
        // Reset en passant unless this is a double pawn move
        self.en_passant = None;

        // ---------------------
        // 0. Castling
        // ---------------------
        let is_castling = self.is_king(moving_piece) &&
                            (
                                (fr == 0 && ff == 4 && tr == 0 && tf == 6) || // White kingside
                                (fr == 0 && ff == 4 && tr == 0 && tf == 2) || // White queenside
                                (fr == 7 && ff == 4 && tr == 7 && tf == 6) || // Black kingside
                                (fr == 7 && ff == 4 && tr == 7 && tf == 2)    // Black queenside
                            );
        if is_castling {
            // kingside
            if tf == 6 {
                // move the rook
                if fr == 0 {
                    // White rook
                    self.squares[0][7] = Empty;
                    self.squares[0][5] = RookWhite;
                }
                else {
                    // Black rook
                    self.squares[7][7] = Empty;
                    self.squares[7][5] = RookBlack;
                }
            }
            // queenside
            if tf == 2 {
                // move the rook
                if fr == 0 {
                    // White rook
                    self.squares[0][0] = Empty;
                    self.squares[0][3] = RookWhite;
                }
                else {
                    // Black rook
                    self.squares[7][0] = Empty;
                    self.squares[7][3] = RookBlack;
                }
            }
            // the king is moved at the end of the function
        }
        
        // update castling rights when a king is moving
        if self.is_king(moving_piece) {
            if moving_piece == KingWhite {
                self.white_kingside_castle = false;
                self.white_queenside_castle = false;
            } else {
                self.black_kingside_castle = false;
                self.black_queenside_castle = false;
            }
        }
        // update castling rights when a rook is moving
        if self.is_rook(moving_piece) {
            if fr == 0 && ff == 0 {
                self.white_queenside_castle = false;
            } else if fr == 0 && ff == 7 {
                self.white_kingside_castle = false;
            } else if fr == 7 && ff == 0 {
                self.black_queenside_castle = false;
            } else if fr == 7 && ff == 7 {
                self.black_kingside_castle = false;
            }
        }
        // update castling rights when a rook gets captured
        let captured_piece = self.squares[tr][tf];
        if self.is_rook(captured_piece) {
            if tr == 0 && tf == 0 {
                self.white_queenside_castle = false;
            } else if tr == 0 && tf == 7 {
                self.white_kingside_castle = false;
            } else if tr == 7 && tf == 0 {
                self.black_queenside_castle = false;
            } else if tr == 7 && tf == 7 {
                self.black_kingside_castle = false;
            }
        }

        // ---------------------
        // 1. En passant capture
        // ---------------------
        // Check if the destination square is en passant square
        if let Some((ep_r, ep_f)) = old_ep {
            if self.is_pawn(moving_piece) && tr == ep_r && tf == ep_f {
                // Capture the pawn behind the EP
                let captured_rank;
                if moving_piece == PawnWhite {
                    captured_rank = tr - 1;
                }
                else { // Black pawn
                    captured_rank = tr + 1;
                }
                // Capture piece
                println!("Captured piece: {}", self.squares[captured_rank][ep_f]);
                self.squares[captured_rank][ep_f] = Empty;
            }
        }
        
        // ---------------------
        // 2. Normal capture
        // ---------------------
        // Check if there is a enemy piece in the destination square
        let destination_square = self.squares[tr][tf];
        if self.same_color(moving_piece, destination_square) {
            // invalid move!
        }
        else {
            if destination_square != Piece::Empty {
                // Capture piece
                println!("Captured piece: {}", destination_square);
            }
        }

        
        // ---------------------
        // 3. Promotion
        // ---------------------
        let piece_to_place = match (moving_piece, mv.promotion) {
            (PawnWhite, Promotion::Queen) => QueenWhite,
            (PawnWhite, Promotion::Rook) => RookWhite,
            (PawnWhite, Promotion::Bishop) => BishopWhite,
            (PawnWhite, Promotion::Knight) => KnightWhite,
            (PawnBlack, Promotion::Queen) => QueenBlack,
            (PawnBlack, Promotion::Rook) => RookBlack,
            (PawnBlack, Promotion::Bishop) => BishopBlack,
            (PawnBlack, Promotion::Knight) => KnightBlack,
            _ => moving_piece,
        };

        // ---------------------
        // 4. Double step
        // ---------------------
        if moving_piece == PawnWhite && fr == 1 && tr == 3 {
            self.en_passant = Some((2, ff));
        }
        if moving_piece == PawnBlack && fr == 6 && tr == 4 {
            self.en_passant = Some((5, ff));
        }

        // ---------------------
        // 5. Execute the move
        // ---------------------
        self.squares[fr][ff] = Empty;
        self.squares[tr][tf] = piece_to_place;

        // ---------------------
        // 6. Switch moving side
        // ---------------------
        if self.side_to_move == 'w' {
            self.side_to_move = 'b';
        }
        else {
            self.side_to_move = 'w';
        }
    }
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
