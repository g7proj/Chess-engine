use std::cmp::Reverse;

use crate::board::{Board, Piece};
use crate::moves::{Move, Promotion};

const MATE_SCORE: i32 = 30_000;

fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::PawnWhite => 100,
        Piece::PawnBlack => -100,
        Piece::KnightWhite => 320,
        Piece::KnightBlack => -320,
        Piece::BishopWhite => 330,
        Piece::BishopBlack => -330,
        Piece::RookWhite => 500,
        Piece::RookBlack => -500,
        Piece::QueenWhite => 900,
        Piece::QueenBlack => -900,
        _ => 0,
    }
}

fn evaluate_position(board: &Board) -> i32 {
    let mut score: i32 = 0;
    for rank in 0..8 {
        for file in 0..8 {
            score += piece_value(board.squares[rank][file]);
        }
    }
    if board.side_to_move == crate::board::Color::White {
        score
    } else {
        -score
    }
}

fn move_promotion_bonus(promotion: Promotion) -> i32 {
    match promotion {
        Promotion::Queen => 900,
        Promotion::Rook => 500,
        Promotion::Bishop => 330,
        Promotion::Knight => 320,
        Promotion::None => 0,
    }
}

fn move_order_score(board: &Board, mv: &Move) -> i32 {
    let moving_piece: Piece = board.squares[mv.from_rank][mv.from_file];
    let target_piece: Piece = board.squares[mv.to_rank][mv.to_file];

    let mut score: i32 = 0;

    if target_piece != Piece::Empty {
        score += 10_000 + piece_value(target_piece).abs() - piece_value(moving_piece).abs() / 10;
    }

    score += move_promotion_bonus(mv.promotion) * 10;

    if moving_piece == Piece::KingWhite || moving_piece == Piece::KingBlack {
        if mv.from_file.abs_diff(mv.to_file) == 2 {
            score += 200;
        }
    }

    let mut next: Board = board.clone();
    next.make_move(*mv);
    if next.is_in_check(next.side_to_move) {
        score += 500;
    }

    score
}

fn ordered_legal_moves(board: &Board) -> Vec<Move> {
    let mut moves: Vec<Move> = board.generate_all_legal_moves();
    moves.sort_by_key(|mv| Reverse(move_order_score(board, mv)));
    moves
}

fn negamax_alpha_beta(board: &Board, depth: usize, mut alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return evaluate_position(board);
    }

    let moves: Vec<Move> = ordered_legal_moves(board);
    if moves.is_empty() {
        if board.is_in_check(board.side_to_move) {
            return -MATE_SCORE + depth as i32;
        }
        return 0;
    }

    for mv in moves {
        let mut next: Board = board.clone();
        next.make_move(mv);
        let score: i32 = -negamax_alpha_beta(&next, depth - 1, -beta, -alpha);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

pub fn find_best_move(board: &Board, depth: usize) -> Option<Move> {
    let moves: Vec<Move> = ordered_legal_moves(board);
    if moves.is_empty() {
        return None;
    }

    let mut best_move: Move = moves[0];
    let mut best_score: i32 = i32::MIN;
    let mut alpha: i32 = i32::MIN + 1;
    let beta: i32 = i32::MAX - 1;

    for mv in moves {
        let mut next: Board = board.clone();
        next.make_move(mv);
        let score: i32 = -negamax_alpha_beta(&next, depth.saturating_sub(1), -beta, -alpha);
        if score > best_score {
            best_score = score;
            best_move = mv;
        }
        if score > alpha {
            alpha = score;
        }
    }

    Some(best_move)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Color;

    fn empty_board() -> Board {
        let mut board: Board = Board::new();
        for r in 0..8 {
            for f in 0..8 {
                board.squares[r][f] = Piece::Empty;
            }
        }
        board
    }

    #[test]
    fn test_move_ordering_prefers_capture() {
        let mut board: Board = empty_board();
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[0][0] = Piece::RookWhite;
        board.squares[7][0] = Piece::QueenBlack;
        board.side_to_move = Color::White;

        let moves: Vec<Move> = ordered_legal_moves(&board);
        assert_eq!(moves[0], Move::new(0, 0, 7, 0));
    }

    #[test]
    fn test_move_ordering_prefers_promotion() {
        let mut board: Board = empty_board();
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[6][0] = Piece::PawnWhite;
        board.side_to_move = Color::White;

        let moves: Vec<Move> = ordered_legal_moves(&board);
        assert!(moves[0].promotion != Promotion::None);
    }

    #[test]
    fn test_find_best_move_prefers_capturing_queen() {
        let mut board: Board = empty_board();
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[0][0] = Piece::RookWhite;
        board.squares[7][0] = Piece::QueenBlack;
        board.side_to_move = Color::White;

        let best: Move = find_best_move(&board, 1).expect("best move");
        assert_eq!(best, Move::new(0, 0, 7, 0));
    }
}
