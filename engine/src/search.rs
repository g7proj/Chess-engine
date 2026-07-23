use std::cmp::Reverse;

use crate::board::{Board, Piece};
use crate::moves::{Move, Promotion};

const MATE_SCORE: i32 = 30_000;

/// Returns the signed material value of a piece.
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

/// Returns a material score in centipawns from the side-to-move perspective.
/// Positive values favor the side to move; negative values favor its opponent.
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

/// Returns the material bonus assigned to a promotion move.
fn move_promotion_bonus(promotion: Promotion) -> i32 {
    match promotion {
        Promotion::Queen => 900,
        Promotion::Rook => 500,
        Promotion::Bishop => 330,
        Promotion::Knight => 320,
        Promotion::None => 0,
    }
}

/// Scores a legal move for alpha-beta move ordering.
fn move_order_score(board: &mut Board, mv: &Move) -> i32 {
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

    let undo = board.make_move(*mv);
    let gives_check = board.is_in_check(board.side_to_move);
    board.unmake_move(undo);
    if gives_check {
        score += 500;
    }

    score
}

/// Returns legal moves sorted by search priority.
fn ordered_legal_moves(board: &mut Board) -> Vec<Move> {
    let mut moves: Vec<Move> = board.generate_all_legal_moves_mut();
    moves.sort_by_key(|mv| Reverse(move_order_score(board, mv)));
    moves
}

/// Searches a position with negamax alpha-beta pruning.
///
/// Returns the best score in centipawns from the side-to-move perspective:
/// positive values favor that side, negative values favor its opponent.
/// The search evaluates child positions with a sign flip, updates `alpha`,
/// and skips remaining moves when `alpha >= beta`.
///
/// `depth` is the number of plies remaining. `alpha` is the best score already
/// guaranteed, while `beta` is the opponent's cutoff bound.
fn negamax_alpha_beta(board: &mut Board, depth: usize, mut alpha: i32, beta: i32) -> i32 {
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
        let undo = board.make_move(mv);
        let score: i32 = -negamax_alpha_beta(board, depth - 1, -beta, -alpha);
        board.unmake_move(undo);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

/// Finds the move with the highest evaluated score for the side to move.
///
/// Returns `None` when the position has no legal moves.
/// The board is temporarily mutated during search and restored before return.
pub fn find_best_move(board: &mut Board, depth: usize) -> Option<Move> {
    let moves: Vec<Move> = ordered_legal_moves(board);
    if moves.is_empty() {
        return None;
    }

    let mut best_move: Move = moves[0];
    let mut best_score: i32 = i32::MIN;
    let mut alpha: i32 = i32::MIN + 1;
    let beta: i32 = i32::MAX - 1;

    for mv in moves {
        let undo = board.make_move(mv);
        let score: i32 = -negamax_alpha_beta(board, depth.saturating_sub(1), -beta, -alpha);
        board.unmake_move(undo);
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

        let moves: Vec<Move> = ordered_legal_moves(&mut board);
        assert_eq!(moves[0], Move::new(0, 0, 7, 0));
    }

    #[test]
    fn test_move_ordering_prefers_promotion() {
        let mut board: Board = empty_board();
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[6][0] = Piece::PawnWhite;
        board.side_to_move = Color::White;

        let moves: Vec<Move> = ordered_legal_moves(&mut board);
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

        let best: Move = find_best_move(&mut board, 1).expect("best move");
        assert_eq!(best, Move::new(0, 0, 7, 0));
    }

    #[test]
    fn test_search_restores_board_state() {
        let mut board: Board = Board::new();
        let fen_before: String = board.to_fen();
        let history_before = board.history.clone();

        let _ = find_best_move(&mut board, 3);

        assert_eq!(board.to_fen(), fen_before);
        assert_eq!(board.history, history_before);
    }
}
