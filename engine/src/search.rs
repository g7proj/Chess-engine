use std::cmp::Reverse;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;

use crate::board::{Board, Piece};
use crate::moves::{Move, Promotion};

const MATE_SCORE: i32 = 30_000;
const MAX_QUIESCENCE_PLY: usize = 32;
const PAWN_POSITION_BONUS: [[i32; 8]; 8] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    [5, 10, 10, -20, -20, 10, 10, 5],
    [5, -5, -10, 0, 0, -10, -5, 5],
    [0, 0, 0, 20, 20, 0, 0, 0],
    [5, 5, 10, 25, 25, 10, 5, 5],
    [10, 10, 20, 30, 30, 20, 10, 10],
    [50, 50, 50, 50, 50, 50, 50, 50],
    [0, 0, 0, 0, 0, 0, 0, 0],
];
const KNIGHT_POSITION_BONUS: [[i32; 8]; 8] = [
    [-50, -40, -30, -30, -30, -30, -40, -50],
    [-40, -20, 0, 5, 5, 0, -20, -40],
    [-30, 5, 10, 15, 15, 10, 5, -30],
    [-30, 0, 15, 20, 20, 15, 0, -30],
    [-30, 5, 15, 20, 20, 15, 5, -30],
    [-30, 0, 10, 15, 15, 10, 0, -30],
    [-40, -20, 0, 0, 0, 0, -20, -40],
    [-50, -40, -30, -30, -30, -30, -40, -50],
];

/// Stores counters and timing data collected during one search.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SearchStats {
    pub depth: usize,
    pub nodes: u64,
    pub quiescence_nodes: u64,
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
#[derive(Debug, Clone, Default)]
pub struct SearchLimits {
    pub time_limit_ms: Option<u128>,
    pub stop: Option<Arc<AtomicBool>>,
}

/// Returns the material value of a piece in centipawns.
fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::PawnWhite | Piece::PawnBlack => 100,
        Piece::KnightWhite | Piece::KnightBlack => 320,
        Piece::BishopWhite | Piece::BishopBlack => 330,
        Piece::RookWhite | Piece::RookBlack => 500,
        Piece::QueenWhite | Piece::QueenBlack => 900,
        _ => 0,
    }
}

/// Returns material and piece-square scores from the side-to-move perspective.
/// Positive values favor the side to move; negative values favor its opponent.
fn evaluate_position(board: &Board) -> i32 {
    let mut white_score: i32 = 0;
    for rank in 0..8 {
        for file in 0..8 {
            let piece: Piece = board.squares[rank][file];
            let (piece_score, positional_bonus): (i32, i32) = match piece {
                Piece::PawnWhite => (100, PAWN_POSITION_BONUS[rank][file]),
                Piece::PawnBlack => (100, PAWN_POSITION_BONUS[7 - rank][7 - file]),
                Piece::KnightWhite => (320, KNIGHT_POSITION_BONUS[rank][file]),
                Piece::KnightBlack => (320, KNIGHT_POSITION_BONUS[7 - rank][7 - file]),
                _ => (piece_value(piece), 0),
            };
            let sign: i32 = match piece.color() {
                Some(crate::board::Color::White) => 1,
                Some(crate::board::Color::Black) => -1,
                None => 0,
            };
            white_score += sign * (piece_score + positional_bonus);
        }
    }
    if board.side_to_move == crate::board::Color::White {
        white_score
    } else {
        -white_score
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
    limits: &SearchLimits,
    deadline: Option<Instant>,
) -> Option<i32> {
    pv_table[ply].clear();
    if search_should_stop(limits, deadline) {
        return None;
    }
    if depth == 0 {
        return quiescence_search(board, alpha, beta, ply, stats, limits, deadline);
    }
    stats.nodes += 1;

    let moves: Vec<Move> = ordered_legal_moves(board, preferred_pv.get(ply).copied());
    if moves.is_empty() {
        if board.is_in_check(board.side_to_move) {
            return Some(-MATE_SCORE + depth as i32);
        }
        return Some(0);
    }

    for mv in moves {
        let undo: crate::moves::Undo = board.make_move_for_search(mv);
        let child_score: Option<i32> = negamax_alpha_beta(
            board,
            depth - 1,
            -beta,
            -alpha,
            ply + 1,
            preferred_pv,
            pv_table,
            stats,
            limits,
            deadline,
        );
        board.unmake_move(undo);
        let score: i32 = -child_score?;
        if score >= beta {
            stats.cutoffs += 1;
            let child_pv: Vec<Move> = pv_table[ply + 1].clone();
            pv_table[ply] = vec![mv];
            pv_table[ply].extend(child_pv);
            return Some(beta);
        }
        if score > alpha {
            alpha = score;
            let child_pv: Vec<Move> = pv_table[ply + 1].clone();
            pv_table[ply] = vec![mv];
            pv_table[ply].extend(child_pv);
        }
    }

    Some(alpha)
}

/// Extends leaf evaluation through legal captures and promotions.
fn quiescence_search(
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    ply: usize,
    stats: &mut SearchStats,
    limits: &SearchLimits,
    deadline: Option<Instant>,
) -> Option<i32> {
    stats.nodes += 1;
    stats.quiescence_nodes += 1;
    if search_should_stop(limits, deadline) {
        return None;
    }

    let in_check: bool = board.is_in_check(board.side_to_move);
    if ply >= MAX_QUIESCENCE_PLY {
        return Some(evaluate_position(board));
    }
    if !in_check {
        let stand_pat: i32 = evaluate_position(board);
        if stand_pat >= beta {
            stats.cutoffs += 1;
            return Some(beta);
        }
        alpha = alpha.max(stand_pat);
    }

    let moves: Vec<Move> = ordered_legal_moves(board, None)
        .into_iter()
        .filter(|mv| in_check || is_capture(board, mv) || mv.promotion != Promotion::None)
        .collect();
    if in_check && moves.is_empty() {
        return Some(-MATE_SCORE + ply as i32);
    }

    for mv in moves {
        let undo: crate::moves::Undo = board.make_move_for_search(mv);
        let child_score: Option<i32> =
            quiescence_search(board, -beta, -alpha, ply + 1, stats, limits, deadline);
        board.unmake_move(undo);
        let score: i32 = -child_score?;
        if score >= beta {
            stats.cutoffs += 1;
            return Some(beta);
        }
        alpha = alpha.max(score);
    }

    Some(alpha)
}

fn is_capture(board: &Board, mv: &Move) -> bool {
    board.squares[mv.to_rank][mv.to_file] != Piece::Empty
        || (board.is_pawn(board.squares[mv.from_rank][mv.from_file])
            && board.en_passant == Some((mv.to_rank, mv.to_file)))
}

fn search_should_stop(limits: &SearchLimits, deadline: Option<Instant>) -> bool {
    limits
        .stop
        .as_ref()
        .is_some_and(|flag| flag.load(Ordering::Relaxed))
        || deadline.is_some_and(|limit| Instant::now() >= limit)
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
    search_at_depth(board, depth, &[], &SearchLimits::default(), None)
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
    let search_start: Instant = Instant::now();
    let deadline: Option<Instant> = limits
        .time_limit_ms
        .map(|ms| search_start + std::time::Duration::from_millis(ms.min(u64::MAX as u128) as u64));
    if max_depth == 0 {
        return search_at_depth(board, 0, &[], &limits, deadline);
    }

    let fallback: Option<Move> = board.generate_all_legal_moves().first().copied();
    let mut result: SearchResult = SearchResult {
        best_move: fallback,
        pv: fallback.into_iter().collect(),
        score: evaluate_position(board),
        stats: SearchStats::default(),
    };
    for depth in 1..=max_depth {
        if limits
            .stop
            .as_ref()
            .is_some_and(|flag| flag.load(Ordering::Relaxed))
        {
            // Stop signal received, exit the search loop
            break;
        }
        if search_should_stop(&limits, deadline) {
            break;
        }
        let next: SearchResult = search_at_depth(board, depth, &result.pv, &limits, deadline);
        if next.stats.depth == depth && next.best_move.is_some() {
            result = next;
        } else {
            break;
        }
    }
    result
}

fn search_at_depth(
    board: &mut Board,
    depth: usize,
    preferred_pv: &[Move],
    limits: &SearchLimits,
    deadline: Option<Instant>,
) -> SearchResult {
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
        let child_score: Option<i32> = negamax_alpha_beta(
            board,
            depth.saturating_sub(1),
            -beta,
            -alpha,
            1,
            preferred_pv,
            &mut pv_table,
            &mut stats,
            limits,
            deadline,
        );
        board.unmake_move(undo);
        let Some(child_score) = child_score else {
            stats.elapsed_ms = start.elapsed().as_millis();
            return SearchResult {
                best_move: None,
                pv: Vec::new(),
                score: 0,
                stats,
            };
        };
        let score: i32 = -child_score;
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
    fn test_evaluation_prefers_central_knight() {
        let central =
            Board::from_fen("4k3/8/8/8/3N4/8/8/4K3 w - - 0 1").expect("valid central-knight FEN");
        let edge =
            Board::from_fen("4k3/8/8/8/8/8/8/N3K3 w - - 0 1").expect("valid edge-knight FEN");

        assert!(evaluate_position(&central) > evaluate_position(&edge));
    }

    #[test]
    fn test_evaluation_is_symmetric_for_colors_and_side_to_move() {
        let white_to_move =
            Board::from_fen("4k3/8/8/8/3N4/4P3/8/4K3 w - - 0 1").expect("valid white position FEN");
        let black_to_move = Board::from_fen("4k3/8/4p3/3n4/8/8/8/4K3 b - - 0 1")
            .expect("valid mirrored position FEN");

        assert_eq!(
            evaluate_position(&white_to_move),
            evaluate_position(&black_to_move)
        );
        assert_eq!(
            evaluate_position(&white_to_move),
            -evaluate_position(&Board {
                side_to_move: Color::Black,
                ..white_to_move.clone()
            })
        );
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
                stop: None,
            },
        );

        assert_eq!(result.stats.depth, 0);
        assert!(result.best_move.is_some());
    }

    #[test]
    fn test_quiescence_search_sees_forced_recapture() {
        let fen: &str = "k7/8/8/2p5/3r4/8/8/3QK3 w - - 0 1";
        let mut board: Board = Board::from_fen(fen).expect("valid tactical FEN");
        let capture: Move = Move::new(0, 3, 3, 3);
        assert!(board.generate_all_legal_moves().contains(&capture));
        let undo = board.make_move_for_search(capture);
        assert_eq!(evaluate_position(&board), -800);
        let mut stats: SearchStats = SearchStats::default();
        let score: i32 = quiescence_search(
            &mut board,
            i32::MIN + 1,
            i32::MAX - 1,
            0,
            &mut stats,
            &SearchLimits::default(),
            None,
        )
        .expect("search should complete");
        board.unmake_move(undo);

        assert_eq!(score, 125);
        assert!(stats.quiescence_nodes > 1);
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn test_quiescence_search_counts_en_passant_capture() {
        let fen: &str = "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1";
        let mut board: Board = Board::from_fen(fen).expect("valid en passant FEN");
        let mut stats: SearchStats = SearchStats::default();
        let score: i32 = quiescence_search(
            &mut board,
            i32::MIN + 1,
            i32::MAX - 1,
            0,
            &mut stats,
            &SearchLimits::default(),
            None,
        )
        .expect("search should complete");

        assert_eq!(score, 130);
        assert!(stats.quiescence_nodes > 1);
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn test_quiescence_search_searches_check_evasions() {
        let fen: &str = "4k3/8/8/8/8/8/8/K3R3 b - - 0 1";
        let mut board: Board = Board::from_fen(fen).expect("valid check FEN");
        let mut stats: SearchStats = SearchStats::default();
        let score = quiescence_search(
            &mut board,
            i32::MIN + 1,
            i32::MAX - 1,
            0,
            &mut stats,
            &SearchLimits::default(),
            None,
        )
        .expect("search should complete");

        assert!(board.is_in_check(board.side_to_move));
        assert!(stats.quiescence_nodes > 1);
        assert!(score > -MATE_SCORE);
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn test_quiescence_search_detects_checkmate() {
        let fen: &str = "7k/5KQ1/8/8/8/8/8/8 b - - 0 1";
        let mut board: Board = Board::from_fen(fen).expect("valid checkmate FEN");
        let mut stats: SearchStats = SearchStats::default();
        let score = quiescence_search(
            &mut board,
            i32::MIN + 1,
            i32::MAX - 1,
            0,
            &mut stats,
            &SearchLimits::default(),
            None,
        )
        .expect("search should complete");

        assert_eq!(score, -MATE_SCORE);
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn test_stopped_search_returns_legal_fallback() {
        let mut board: Board = Board::new();
        let legal_moves: Vec<Move> = board.generate_all_legal_moves();
        let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
        let result = find_best_move_iterative_with_limits(
            &mut board,
            5,
            SearchLimits {
                time_limit_ms: None,
                stop: Some(stop),
            },
        );

        assert!(result.best_move.is_some_and(|mv| legal_moves.contains(&mv)));
        assert_eq!(result.stats.depth, 0);
        assert_eq!(board.to_fen(), Board::new().to_fen());
    }
}
