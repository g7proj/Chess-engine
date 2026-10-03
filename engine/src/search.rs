use std::cmp::Reverse;
use std::time::Instant;

use crate::board::{Board, Piece};
use crate::moves::{Move, Promotion};

const MATE_SCORE: i32 = 30_000;

/// Stores counters and timing data collected during one search.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SearchStats {
    pub depth: usize,
    pub nodes: u64,
    pub cutoffs: u64,
    pub elapsed_ms: u128,
}

impl SearchStats {
    /// Returns the measured search speed in nodes per second.
    pub fn nps(&self) -> u64 {
        if self.elapsed_ms == 0 {
            self.nodes
        } else {
            ((self.nodes as u128 * 1000) / self.elapsed_ms) as u64
        }
    }
}

/// Contains the selected move, principal variation, score, and search metrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub pv: Vec<Move>,
    pub score: i32,
    pub stats: SearchStats,
}

/// Defines an optional time budget for one search.
#[derive(Debug, Clone, Copy, Default)]
pub struct SearchLimits {
    pub time_limit_ms: Option<u128>,
}

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

    let undo: crate::moves::Undo = board.make_move_for_search(*mv);
    let gives_check = board.is_in_check(board.side_to_move);
    board.unmake_move(undo);
    if gives_check {
        score += 500;
    }

    score
}

/// Returns legal moves sorted by search priority.
fn ordered_legal_moves(board: &mut Board, preferred_move: Option<Move>) -> Vec<Move> {
    let mut moves: Vec<Move> = board.generate_all_legal_moves_mut();
    moves.sort_by_key(|mv| {
        let pv_bonus: i32 = if Some(*mv) == preferred_move {
            1_000_000
        } else {
            0
        };
        Reverse(move_order_score(board, mv) + pv_bonus)
    });
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
fn negamax_alpha_beta(
    board: &mut Board,
    depth: usize,
    mut alpha: i32,
    beta: i32,
    ply: usize,
    preferred_pv: &[Move],
    pv_table: &mut [Vec<Move>],
    stats: &mut SearchStats,
) -> i32 {
    stats.nodes += 1;
    pv_table[ply].clear();
    if depth == 0 {
        return evaluate_position(board);
    }

    let moves: Vec<Move> = ordered_legal_moves(board, preferred_pv.get(ply).copied());
    if moves.is_empty() {
        if board.is_in_check(board.side_to_move) {
            return -MATE_SCORE + depth as i32;
        }
        return 0;
    }

    for mv in moves {
        let undo: crate::moves::Undo = board.make_move_for_search(mv);
        let score: i32 = -negamax_alpha_beta(
            board,
            depth - 1,
            -beta,
            -alpha,
            ply + 1,
            preferred_pv,
            pv_table,
            stats,
        );
        board.unmake_move(undo);
        if score >= beta {
            stats.cutoffs += 1;
            let child_pv: Vec<Move> = pv_table[ply + 1].clone();
            pv_table[ply] = vec![mv];
            pv_table[ply].extend(child_pv);
            return beta;
        }
        if score > alpha {
            alpha = score;
            let child_pv: Vec<Move> = pv_table[ply + 1].clone();
            pv_table[ply] = vec![mv];
            pv_table[ply].extend(child_pv);
        }
    }

    alpha
}

/// Finds the move with the highest evaluated score for the side to move.
///
/// Returns `None` when the position has no legal moves.
/// The board is temporarily mutated during search and restored before return.
pub fn find_best_move(board: &mut Board, depth: usize) -> Option<Move> {
    find_best_move_with_stats(board, depth).best_move
}

/// Searches for the best move and returns metrics for the completed search.
pub fn find_best_move_with_stats(board: &mut Board, depth: usize) -> SearchResult {
    search_at_depth(board, depth, &[])
}

/// Searches progressively deeper and reuses the previous principal variation.
pub fn find_best_move_iterative_with_stats(board: &mut Board, max_depth: usize) -> SearchResult {
    find_best_move_iterative_with_limits(board, max_depth, SearchLimits::default())
}

/// Searches progressively deeper until the depth or time budget is exhausted.
pub fn find_best_move_iterative_with_limits(
    board: &mut Board,
    max_depth: usize,
    limits: SearchLimits,
) -> SearchResult {
    let start: Instant = Instant::now();
    if max_depth == 0 {
        return search_at_depth(board, 0, &[]);
    }

    let mut result: SearchResult = search_at_depth(board, 1, &[]);
    for depth in 2..=max_depth {
        if limits
            .time_limit_ms
            .is_some_and(|ms| start.elapsed().as_millis() >= ms)
        {
            break;
        }
        let next: SearchResult = search_at_depth(board, depth, &result.pv);
        if next.best_move.is_some() {
            result = next;
        }
    }
    result
}

fn search_at_depth(board: &mut Board, depth: usize, preferred_pv: &[Move]) -> SearchResult {
    let start: Instant = Instant::now();
    let mut stats: SearchStats = SearchStats {
        depth,
        nodes: 1,
        ..SearchStats::default()
    };
    let moves: Vec<Move> = ordered_legal_moves(board, preferred_pv.first().copied());
    if moves.is_empty() {
        stats.elapsed_ms = start.elapsed().as_millis();
        return SearchResult {
            best_move: None,
            pv: Vec::new(),
            score: 0,
            stats,
        };
    }

    let mut pv_table: Vec<Vec<Move>> = vec![Vec::new(); depth + 2];
    let mut best_move: Move = moves[0];
    let mut best_score: i32 = i32::MIN;
    let mut alpha: i32 = i32::MIN + 1;
    let beta: i32 = i32::MAX - 1;

    for mv in moves {
        let undo: crate::moves::Undo = board.make_move_for_search(mv);
        let score: i32 = -negamax_alpha_beta(
            board,
            depth.saturating_sub(1),
            -beta,
            -alpha,
            1,
            preferred_pv,
            &mut pv_table,
            &mut stats,
        );
        board.unmake_move(undo);
        if score > best_score {
            best_score = score;
            best_move = mv;
            let child_pv: Vec<Move> = pv_table[1].clone();
            pv_table[0] = vec![mv];
            pv_table[0].extend(child_pv);
        }
        if score > alpha {
            alpha = score;
        }
    }

    stats.elapsed_ms = start.elapsed().as_millis();
    SearchResult {
        best_move: Some(best_move),
        pv: pv_table[0].clone(),
        score: best_score,
        stats,
    }
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

        let moves: Vec<Move> = ordered_legal_moves(&mut board, None);
        assert_eq!(moves[0], Move::new(0, 0, 7, 0));
    }

    #[test]
    fn test_move_ordering_prefers_promotion() {
        let mut board: Board = empty_board();
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[6][0] = Piece::PawnWhite;
        board.side_to_move = Color::White;

        let moves: Vec<Move> = ordered_legal_moves(&mut board, None);
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

    #[test]
    fn test_search_reports_statistics() {
        let mut board: Board = Board::new();
        let result: SearchResult = find_best_move_with_stats(&mut board, 3);

        assert!(result.best_move.is_some());
        assert_eq!(result.stats.depth, 3);
        assert!(result.stats.nodes > 1);
        assert!(result.stats.cutoffs > 0);
        assert!(result.stats.nps() > 0);
    }

    #[test]
    fn test_iterative_search_returns_complete_pv() {
        let mut board: Board = Board::new();
        let result: SearchResult = find_best_move_iterative_with_stats(&mut board, 3);

        assert!(result.best_move.is_some());
        assert_eq!(result.stats.depth, 3);
        assert!(!result.pv.is_empty());
        assert_eq!(result.pv[0], result.best_move.unwrap());
        assert!(result.pv.len() <= 3);
    }

    #[test]
    fn test_iterative_search_respects_zero_time_budget_between_iterations() {
        let mut board: Board = Board::new();
        let result = find_best_move_iterative_with_limits(
            &mut board,
            3,
            SearchLimits {
                time_limit_ms: Some(0),
            },
        );

        assert_eq!(result.stats.depth, 1);
        assert!(result.best_move.is_some());
    }
}
