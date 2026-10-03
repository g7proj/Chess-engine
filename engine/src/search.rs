use std::cmp::Reverse;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;

use crate::board::{Board, Color, Piece};
use crate::moves::{Move, Promotion};

const MATE_SCORE: i32 = 30_000;
const MAX_QUIESCENCE_PLY: usize = 32;
const BISHOP_PAIR_BONUS: i32 = 30;
const CENTER_CONTROL_BONUS: i32 = 8;
const MOBILITY_BONUS: i32 = 2;
const DOUBLED_PAWN_PENALTY: i32 = 12;
const ISOLATED_PAWN_PENALTY: i32 = 10;
const DEVELOPED_MINOR_BONUS: i32 = 10;
const KING_SHIELD_BONUS: i32 = 10;
const KING_PRESSURE_PENALTY: i32 = 8;
const TRANSPOSITION_TABLE_SIZE: usize = 1 << 16;
const MATE_SCORE_THRESHOLD: i32 = MATE_SCORE - 1_000;
const KILLER_MOVE_BONUS: i32 = 8_000;
const HISTORY_SCORE_LIMIT: i32 = 2_000;
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
    pub tt_hits: u64,
    pub tt_cutoffs: u64,
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
#[derive(Debug, Clone)]
pub struct SearchLimits {
    pub time_limit_ms: Option<u128>,
    pub stop: Option<Arc<AtomicBool>>,
    pub use_transposition_table: bool,
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self {
            time_limit_ms: None,
            stop: None,
            use_transposition_table: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundType {
    Exact,
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy)]
struct TranspositionEntry {
    key: u64,
    depth: usize,
    score: i32,
    bound: BoundType,
    best_move: Option<Move>,
}

struct TranspositionTable {
    entries: Vec<Option<TranspositionEntry>>,
}

struct SearchHeuristics {
    killers: Vec<[Option<Move>; 2]>,
    history: [[[i32; 64]; 64]; 2],
}

impl SearchHeuristics {
    fn new(max_ply: usize) -> Self {
        Self {
            killers: vec![[None; 2]; max_ply.max(1)],
            history: [[[0; 64]; 64]; 2],
        }
    }

    fn record_quiet_cutoff(&mut self, mv: Move, color: Color, ply: usize, depth: usize) {
        if let Some(killers) = self.killers.get_mut(ply)
            && killers[0] != Some(mv)
        {
            killers[1] = killers[0];
            killers[0] = Some(mv);
        }
        self.update_history(mv, color, depth as i32 * depth as i32);
    }

    fn penalize_quiet(&mut self, mv: Move, color: Color, depth: usize) {
        self.update_history(mv, color, -(depth as i32 * depth as i32 / 2).max(1));
    }

    fn update_history(&mut self, mv: Move, color: Color, bonus: i32) {
        let color_index = if color == Color::White { 0 } else { 1 };
        let from = mv.from_rank * 8 + mv.from_file;
        let to = mv.to_rank * 8 + mv.to_file;
        let score = &mut self.history[color_index][from][to];
        *score = (*score + bonus).clamp(-HISTORY_SCORE_LIMIT, HISTORY_SCORE_LIMIT);
    }

    fn quiet_move_bonus(&self, mv: &Move, color: Color, ply: usize) -> i32 {
        if mv.promotion != Promotion::None {
            return 0;
        }
        if let Some(killers) = self.killers.get(ply) {
            if killers[0] == Some(*mv) {
                return KILLER_MOVE_BONUS;
            }
            if killers[1] == Some(*mv) {
                return KILLER_MOVE_BONUS - 500;
            }
        }
        let color_index = if color == Color::White { 0 } else { 1 };
        let from = mv.from_rank * 8 + mv.from_file;
        let to = mv.to_rank * 8 + mv.to_file;
        self.history[color_index][from][to]
    }
}

impl TranspositionTable {
    fn new(enabled: bool) -> Self {
        Self {
            entries: if enabled {
                vec![None; TRANSPOSITION_TABLE_SIZE]
            } else {
                Vec::new()
            },
        }
    }

    fn probe(&self, key: u64) -> Option<TranspositionEntry> {
        self.entries[key as usize & (self.entries.len() - 1)].filter(|entry| entry.key == key)
    }

    fn store(&mut self, entry: TranspositionEntry) {
        let index = entry.key as usize & (self.entries.len() - 1);
        let slot = &mut self.entries[index];
        if slot.is_none_or(|old| entry.depth >= old.depth) {
            *slot = Some(entry);
        }
    }
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
    let mut white_bishops: usize = 0;
    let mut black_bishops: usize = 0;
    for rank in 0..8 {
        for file in 0..8 {
            let piece: Piece = board.squares[rank][file];
            match piece {
                Piece::BishopWhite => white_bishops += 1,
                Piece::BishopBlack => black_bishops += 1,
                _ => {}
            }
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
    if white_bishops >= 2 {
        white_score += BISHOP_PAIR_BONUS;
    }
    if black_bishops >= 2 {
        white_score -= BISHOP_PAIR_BONUS;
    }
    for color in [Color::White, Color::Black] {
        let sign: i32 = if color == Color::White { 1 } else { -1 };
        white_score += sign
            * (mobility_score(board, color)
                + center_control_score(board, color)
                + pawn_structure_score(board, color)
                + development_score(board, color)
                + king_safety_score(board, color));
    }
    if board.side_to_move == Color::White {
        white_score
    } else {
        -white_score
    }
}

/// Scores pseudo-legal mobility for non-pawn pieces in centipawns.
fn mobility_score(board: &Board, color: Color) -> i32 {
    let mut mobility: i32 = 0;
    for rank in 0..8 {
        for file in 0..8 {
            let piece: Piece = board.squares[rank][file];
            if piece.color() != Some(color) || matches!(piece, Piece::PawnWhite | Piece::PawnBlack)
            {
                continue;
            }
            mobility += piece_mobility(board, piece, rank, file, color);
        }
    }
    mobility * MOBILITY_BONUS
}

/// Counts reachable non-friendly squares for one piece.
fn piece_mobility(board: &Board, piece: Piece, rank: usize, file: usize, color: Color) -> i32 {
    match piece {
        Piece::KnightWhite | Piece::KnightBlack => count_jump_targets(
            board,
            rank,
            file,
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
            color,
        ),
        Piece::KingWhite | Piece::KingBlack => count_jump_targets(
            board,
            rank,
            file,
            &[
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ],
            color,
        ),
        Piece::BishopWhite | Piece::BishopBlack => count_ray_targets(
            board,
            rank,
            file,
            &[(1, 1), (1, -1), (-1, 1), (-1, -1)],
            color,
        ),
        Piece::RookWhite | Piece::RookBlack => count_ray_targets(
            board,
            rank,
            file,
            &[(1, 0), (-1, 0), (0, 1), (0, -1)],
            color,
        ),
        Piece::QueenWhite | Piece::QueenBlack => count_ray_targets(
            board,
            rank,
            file,
            &[
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ],
            color,
        ),
        _ => 0,
    }
}

fn count_jump_targets(
    board: &Board,
    rank: usize,
    file: usize,
    offsets: &[(isize, isize)],
    color: Color,
) -> i32 {
    offsets
        .iter()
        .filter_map(|&(dr, df)| {
            let target_rank = rank as isize + dr;
            let target_file = file as isize + df;
            ((0..8).contains(&target_rank) && (0..8).contains(&target_file))
                .then_some((target_rank as usize, target_file as usize))
        })
        .filter(|&(target_rank, target_file)| {
            board.squares[target_rank][target_file].color() != Some(color)
        })
        .count() as i32
}

fn count_ray_targets(
    board: &Board,
    rank: usize,
    file: usize,
    directions: &[(isize, isize)],
    color: Color,
) -> i32 {
    let mut count = 0;
    for &(dr, df) in directions {
        let mut target_rank = rank as isize + dr;
        let mut target_file = file as isize + df;
        while (0..8).contains(&target_rank) && (0..8).contains(&target_file) {
            let target = board.squares[target_rank as usize][target_file as usize];
            if target.color() == Some(color) {
                break;
            }
            count += 1;
            if target != Piece::Empty {
                break;
            }
            target_rank += dr;
            target_file += df;
        }
    }
    count
}

/// Scores attacks on the four central squares in centipawns.
fn center_control_score(board: &Board, color: Color) -> i32 {
    [(3, 3), (3, 4), (4, 3), (4, 4)]
        .into_iter()
        .filter(|&(rank, file)| square_controlled_by(board, rank, file, color))
        .count() as i32
        * CENTER_CONTROL_BONUS
}

/// Scores doubled, isolated, and passed pawns for one side.
fn pawn_structure_score(board: &Board, color: Color) -> i32 {
    let mut files = [0_usize; 8];
    let mut pawns = Vec::new();
    for (rank, row) in board.squares.iter().enumerate() {
        for (file, &piece) in row.iter().enumerate() {
            if piece.color() == Some(color) && matches!(piece, Piece::PawnWhite | Piece::PawnBlack)
            {
                files[file] += 1;
                pawns.push((rank, file));
            }
        }
    }

    let mut score: i32 = files
        .iter()
        .map(|&count| count.saturating_sub(1) as i32 * -DOUBLED_PAWN_PENALTY)
        .sum();
    for (rank, file) in pawns {
        let has_neighbor = (file > 0 && files[file - 1] > 0) || (file < 7 && files[file + 1] > 0);
        if !has_neighbor {
            score -= ISOLATED_PAWN_PENALTY;
        }

        let forward = if color == Color::White {
            (rank + 1)..8
        } else {
            0..rank
        };
        let passed = forward.into_iter().all(|enemy_rank| {
            let start_file = file.saturating_sub(1);
            let end_file = (file + 1).min(7);
            (start_file..=end_file).all(|enemy_file| {
                !matches!(
                    board.squares[enemy_rank][enemy_file],
                    Piece::PawnWhite | Piece::PawnBlack
                ) || board.squares[enemy_rank][enemy_file].color() == Some(color)
            })
        });
        if passed {
            let advancement = if color == Color::White {
                rank
            } else {
                7 - rank
            };
            score += [0, 5, 10, 20, 35, 55, 80, 0][advancement];
        }
    }
    score
}

/// Rewards knights and bishops developed off their home rank.
fn development_score(board: &Board, color: Color) -> i32 {
    let home_rank = if color == Color::White { 0 } else { 7 };
    board
        .squares
        .iter()
        .enumerate()
        .flat_map(|(rank, squares)| squares.iter().map(move |&piece| (rank, piece)))
        .filter(|&(rank, piece)| {
            piece.color() == Some(color)
                && matches!(
                    piece,
                    Piece::KnightWhite
                        | Piece::KnightBlack
                        | Piece::BishopWhite
                        | Piece::BishopBlack
                )
                && rank != home_rank
        })
        .count() as i32
        * DEVELOPED_MINOR_BONUS
}

/// Scores pawn cover and enemy attacks around a side's king.
fn king_safety_score(board: &Board, color: Color) -> i32 {
    let king = if color == Color::White {
        Piece::KingWhite
    } else {
        Piece::KingBlack
    };
    let Some((rank, file)) = board.squares.iter().enumerate().find_map(|(rank, row)| {
        row.iter()
            .position(|&piece| piece == king)
            .map(|file| (rank, file))
    }) else {
        return 0;
    };

    let forward_rank = if color == Color::White {
        rank + 1
    } else {
        rank.checked_sub(1).unwrap_or(8)
    };
    let shield = if forward_rank < 8 {
        (file.saturating_sub(1)..=(file + 1).min(7))
            .filter(|&shield_file| {
                board.squares[forward_rank][shield_file].color() == Some(color)
                    && matches!(
                        board.squares[forward_rank][shield_file],
                        Piece::PawnWhite | Piece::PawnBlack
                    )
            })
            .count() as i32
            * KING_SHIELD_BONUS
    } else {
        0
    };

    let pressure = (rank.saturating_sub(1)..=(rank + 1).min(7))
        .flat_map(|near_rank| {
            (file.saturating_sub(1)..=(file + 1).min(7))
                .map(move |near_file| (near_rank, near_file))
        })
        .filter(|&(near_rank, near_file)| {
            (near_rank != rank || near_file != file)
                && square_controlled_by(board, near_rank, near_file, color.opposite())
        })
        .count() as i32
        * KING_PRESSURE_PENALTY;
    shield - pressure
}

/// Returns whether a piece attacks a target square along its movement pattern.
fn piece_controls_square(
    board: &Board,
    piece: Piece,
    rank: usize,
    file: usize,
    target_rank: usize,
    target_file: usize,
) -> bool {
    let dr = target_rank as isize - rank as isize;
    let df = target_file as isize - file as isize;
    let abs_dr = dr.abs();
    let abs_df = df.abs();
    let pattern_matches = match piece {
        Piece::KnightWhite | Piece::KnightBlack => {
            (abs_dr == 2 && abs_df == 1) || (abs_dr == 1 && abs_df == 2)
        }
        Piece::BishopWhite | Piece::BishopBlack => abs_dr == abs_df && abs_dr > 0,
        Piece::RookWhite | Piece::RookBlack => (dr == 0) != (df == 0),
        Piece::QueenWhite | Piece::QueenBlack => {
            (dr == 0) != (df == 0) || (abs_dr == abs_df && abs_dr > 0)
        }
        Piece::KingWhite | Piece::KingBlack => {
            abs_dr <= 1 && abs_df <= 1 && (abs_dr != 0 || abs_df != 0)
        }
        _ => false,
    };
    if !pattern_matches {
        return false;
    }
    if matches!(
        piece,
        Piece::KnightWhite | Piece::KnightBlack | Piece::KingWhite | Piece::KingBlack
    ) {
        return true;
    }

    let step_rank = dr.signum();
    let step_file = df.signum();
    let mut current_rank = rank as isize + step_rank;
    let mut current_file = file as isize + step_file;
    while (current_rank, current_file) != (target_rank as isize, target_file as isize) {
        if board.squares[current_rank as usize][current_file as usize] != Piece::Empty {
            return false;
        }
        current_rank += step_rank;
        current_file += step_file;
    }
    true
}

fn square_controlled_by(
    board: &Board,
    target_rank: usize,
    target_file: usize,
    color: Color,
) -> bool {
    (0..8).any(|rank| {
        (0..8).any(|file| {
            let piece = board.squares[rank][file];
            piece.color() == Some(color)
                && piece_controls_square(board, piece, rank, file, target_rank, target_file)
        })
    })
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
fn ordered_legal_moves(
    board: &mut Board,
    preferred_move: Option<Move>,
    heuristics: Option<&SearchHeuristics>,
    ply: usize,
) -> Vec<Move> {
    let mut moves: Vec<Move> = board.generate_all_legal_moves_mut();
    moves.sort_by_key(|mv| {
        let pv_bonus: i32 = if Some(*mv) == preferred_move {
            1_000_000
        } else {
            0
        };
        let moving_color = board.squares[mv.from_rank][mv.from_file]
            .color()
            .unwrap_or(board.side_to_move);
        let quiet_bonus = if !is_capture(board, mv) {
            heuristics.map_or(0, |ordering| {
                ordering.quiet_move_bonus(mv, moving_color, ply)
            })
        } else {
            0
        };
        Reverse(move_order_score(board, mv) + pv_bonus + quiet_bonus)
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
    mut beta: i32,
    ply: usize,
    preferred_pv: &[Move],
    pv_table: &mut [Vec<Move>],
    stats: &mut SearchStats,
    limits: &SearchLimits,
    deadline: Option<Instant>,
    transposition_table: &mut TranspositionTable,
    heuristics: &mut SearchHeuristics,
) -> Option<i32> {
    pv_table[ply].clear();
    if search_should_stop(limits, deadline) {
        return None;
    }
    if depth == 0 {
        return quiescence_search(board, alpha, beta, ply, stats, limits, deadline);
    }
    stats.nodes += 1;

    let original_alpha = alpha;
    let original_beta = beta;
    let key = board.zobrist_key;
    let tt_entry = if limits.use_transposition_table {
        transposition_table.probe(key)
    } else {
        None
    };
    let tt_move = tt_entry.and_then(|entry| entry.best_move);
    let tt_lower_bound_applied =
        tt_entry.is_some_and(|entry| entry.depth >= depth && entry.bound == BoundType::Lower);
    if let Some(entry) = tt_entry {
        stats.tt_hits += 1;
        if entry.depth >= depth {
            let score = score_from_table(entry.score, ply);
            match entry.bound {
                BoundType::Exact => {
                    pv_table[ply] = entry.best_move.into_iter().collect();
                    stats.tt_cutoffs += 1;
                    return Some(score);
                }
                BoundType::Lower => alpha = alpha.max(score),
                BoundType::Upper => beta = beta.min(score),
            }
            if alpha >= beta {
                pv_table[ply] = entry.best_move.into_iter().collect();
                stats.tt_cutoffs += 1;
                return Some(score);
            }
        }
    }

    let moves: Vec<Move> = ordered_legal_moves(
        board,
        tt_move.or_else(|| preferred_pv.get(ply).copied()),
        Some(heuristics),
        ply,
    );
    if moves.is_empty() {
        if board.is_in_check(board.side_to_move) {
            return Some(-MATE_SCORE + ply as i32);
        }
        return Some(0);
    }

    let moving_color = board.side_to_move;
    let mut searched_quiet_moves = Vec::new();
    for mv in moves {
        let is_quiet = !is_capture(board, &mv) && mv.promotion == Promotion::None;
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
            transposition_table,
            heuristics,
        );
        board.unmake_move(undo);
        let score: i32 = -child_score?;
        if score >= beta {
            stats.cutoffs += 1;
            if is_quiet {
                heuristics.record_quiet_cutoff(mv, moving_color, ply, depth);
                for previous in searched_quiet_moves {
                    heuristics.penalize_quiet(previous, moving_color, depth);
                }
            }
            let child_pv: Vec<Move> = pv_table[ply + 1].clone();
            pv_table[ply] = vec![mv];
            pv_table[ply].extend(child_pv);
            if limits.use_transposition_table {
                transposition_table.store(TranspositionEntry {
                    key,
                    depth,
                    score: score_to_table(beta, ply),
                    bound: BoundType::Lower,
                    best_move: Some(mv),
                });
            }
            return Some(beta);
        }
        if score > alpha {
            alpha = score;
            let child_pv: Vec<Move> = pv_table[ply + 1].clone();
            pv_table[ply] = vec![mv];
            pv_table[ply].extend(child_pv);
        }
        if is_quiet {
            searched_quiet_moves.push(mv);
        }
    }

    if limits.use_transposition_table {
        let bound = if alpha <= original_alpha {
            BoundType::Upper
        } else if alpha >= original_beta {
            BoundType::Lower
        } else if tt_lower_bound_applied && pv_table[ply].is_empty() {
            BoundType::Lower
        } else {
            BoundType::Exact
        };
        transposition_table.store(TranspositionEntry {
            key,
            depth,
            score: score_to_table(alpha, ply),
            bound,
            best_move: pv_table[ply].first().copied(),
        });
    }
    Some(alpha)
}

fn score_to_table(score: i32, ply: usize) -> i32 {
    if score >= MATE_SCORE_THRESHOLD {
        score + ply as i32
    } else if score <= -MATE_SCORE_THRESHOLD {
        score - ply as i32
    } else {
        score
    }
}

fn score_from_table(score: i32, ply: usize) -> i32 {
    if score >= MATE_SCORE_THRESHOLD {
        score - ply as i32
    } else if score <= -MATE_SCORE_THRESHOLD {
        score + ply as i32
    } else {
        score
    }
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

    let moves: Vec<Move> = ordered_legal_moves(board, None, None, ply)
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
    let mut transposition_table = TranspositionTable::new(true);
    let mut heuristics = SearchHeuristics::new(depth + 2);
    search_at_depth(
        board,
        depth,
        &[],
        &SearchLimits::default(),
        None,
        &mut transposition_table,
        &mut heuristics,
    )
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
    let mut transposition_table = TranspositionTable::new(limits.use_transposition_table);
    let mut heuristics = SearchHeuristics::new(max_depth + 2);
    let search_start: Instant = Instant::now();
    let deadline: Option<Instant> = limits
        .time_limit_ms
        .map(|ms| search_start + std::time::Duration::from_millis(ms.min(u64::MAX as u128) as u64));
    if max_depth == 0 {
        return search_at_depth(
            board,
            0,
            &[],
            &limits,
            deadline,
            &mut transposition_table,
            &mut heuristics,
        );
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
        let next: SearchResult = search_at_depth(
            board,
            depth,
            &result.pv,
            &limits,
            deadline,
            &mut transposition_table,
            &mut heuristics,
        );
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
    transposition_table: &mut TranspositionTable,
    heuristics: &mut SearchHeuristics,
) -> SearchResult {
    board.refresh_zobrist_key();
    let start: Instant = Instant::now();
    let mut stats: SearchStats = SearchStats {
        depth,
        nodes: 1,
        ..SearchStats::default()
    };
    let root_key = board.zobrist_key;
    let root_entry = if limits.use_transposition_table {
        transposition_table.probe(root_key)
    } else {
        None
    };
    if root_entry.is_some() {
        stats.tt_hits += 1;
    }
    let root_move = root_entry
        .and_then(|entry| entry.best_move)
        .or_else(|| preferred_pv.first().copied());
    let moves: Vec<Move> = ordered_legal_moves(board, root_move, Some(heuristics), 0);
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
            transposition_table,
            heuristics,
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

    if limits.use_transposition_table {
        transposition_table.store(TranspositionEntry {
            key: root_key,
            depth,
            score: score_to_table(best_score, 0),
            bound: BoundType::Exact,
            best_move: Some(best_move),
        });
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

        let moves: Vec<Move> = ordered_legal_moves(&mut board, None, None, 0);
        assert_eq!(moves[0], Move::new(0, 0, 7, 0));
    }

    #[test]
    fn test_move_ordering_prioritizes_transposition_move() {
        let mut board = Board::new();
        let preferred = board.generate_all_legal_moves().last().copied().unwrap();

        assert_eq!(
            ordered_legal_moves(&mut board, Some(preferred), None, 0)[0],
            preferred
        );
    }

    #[test]
    fn test_killer_and_history_heuristics_prioritize_quiet_moves() {
        let mut board = Board::new();
        let first_killer = Move::new(0, 6, 2, 5);
        let second_killer = Move::new(1, 4, 3, 4);
        let mut heuristics = SearchHeuristics::new(8);

        heuristics.record_quiet_cutoff(first_killer, Color::White, 2, 4);
        heuristics.record_quiet_cutoff(second_killer, Color::White, 2, 2);

        assert_eq!(
            heuristics.quiet_move_bonus(&second_killer, Color::White, 2),
            KILLER_MOVE_BONUS
        );
        assert_eq!(
            heuristics.quiet_move_bonus(&first_killer, Color::White, 2),
            KILLER_MOVE_BONUS - 500
        );
        let first_from = first_killer.from_rank * 8 + first_killer.from_file;
        let first_to = first_killer.to_rank * 8 + first_killer.to_file;
        assert_eq!(heuristics.history[0][first_from][first_to], 16);
        assert_eq!(heuristics.history[1][first_from][first_to], 0);

        let ordered = ordered_legal_moves(&mut board, None, Some(&heuristics), 2);
        assert_eq!(ordered[0], second_killer);
    }

    #[test]
    fn test_history_heuristic_penalizes_quiet_moves_previously_searched() {
        let mv = Move::new(0, 6, 2, 5);
        let mut heuristics = SearchHeuristics::new(4);
        heuristics.update_history(mv, Color::White, 20);
        heuristics.penalize_quiet(mv, Color::White, 4);

        let from = mv.from_rank * 8 + mv.from_file;
        let to = mv.to_rank * 8 + mv.to_file;
        assert_eq!(heuristics.history[0][from][to], 12);
    }

    #[test]
    fn test_move_ordering_prefers_promotion() {
        let mut board: Board = empty_board();
        board.squares[0][4] = Piece::KingWhite;
        board.squares[7][4] = Piece::KingBlack;
        board.squares[6][0] = Piece::PawnWhite;
        board.side_to_move = Color::White;

        let moves: Vec<Move> = ordered_legal_moves(&mut board, None, None, 0);
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
    fn test_evaluation_rewards_bishop_pair() {
        let pair =
            Board::from_fen("4k3/8/8/8/8/8/8/2B1KB2 w - - 0 1").expect("valid bishop-pair FEN");
        let single =
            Board::from_fen("4k3/8/8/8/8/8/8/2B1K3 w - - 0 1").expect("valid single-bishop FEN");

        assert!(evaluate_position(&pair) > evaluate_position(&single) + BISHOP_PAIR_BONUS);
    }

    #[test]
    fn test_evaluation_rewards_bishop_pair_for_black() {
        let pair = Board::from_fen("2b1kb2/8/8/8/8/8/8/4K3 b - - 0 1")
            .expect("valid black bishop-pair FEN");
        let single = Board::from_fen("2b1k3/8/8/8/8/8/8/4K3 b - - 0 1")
            .expect("valid black single-bishop FEN");

        assert!(evaluate_position(&pair) > evaluate_position(&single) + BISHOP_PAIR_BONUS);
    }

    #[test]
    fn test_evaluation_rewards_mobility_and_center_control() {
        let central_knight =
            Board::from_fen("4k3/8/8/8/3N4/8/8/4K3 w - - 0 1").expect("valid central-knight FEN");
        let edge_knight =
            Board::from_fen("4k3/8/8/8/8/8/8/N3K3 w - - 0 1").expect("valid edge-knight FEN");
        assert!(
            mobility_score(&central_knight, Color::White)
                > mobility_score(&edge_knight, Color::White)
        );

        let central_rook =
            Board::from_fen("4k3/8/8/8/8/8/8/3RK3 w - - 0 1").expect("valid central-file rook FEN");
        assert!(center_control_score(&central_rook, Color::White) > 0);
    }

    #[test]
    fn test_evaluation_scores_pawn_structure_and_passed_pawns() {
        let doubled =
            Board::from_fen("7k/pp6/8/8/8/P7/P7/7K w - - 0 1").expect("valid doubled-pawn FEN");
        let advanced =
            Board::from_fen("7k/8/4P3/8/8/8/8/7K w - - 0 1").expect("valid passed-pawn FEN");

        assert!(pawn_structure_score(&doubled, Color::White) < 0);
        assert!(pawn_structure_score(&advanced, Color::White) > 0);
    }

    #[test]
    fn test_evaluation_rewards_development_and_king_shield() {
        let developed =
            Board::from_fen("4k3/8/8/8/8/2N5/8/4K3 w - - 0 1").expect("valid developed-knight FEN");
        let undeveloped = Board::from_fen("4k3/8/8/8/8/8/8/1N2K3 w - - 0 1")
            .expect("valid undeveloped-knight FEN");
        assert!(
            development_score(&developed, Color::White)
                > development_score(&undeveloped, Color::White)
        );

        let shielded =
            Board::from_fen("4k3/8/8/8/8/8/3PPP2/4K3 w - - 0 1").expect("valid king-shield FEN");
        let exposed =
            Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").expect("valid exposed-king FEN");
        assert!(
            king_safety_score(&shielded, Color::White) > king_safety_score(&exposed, Color::White)
        );
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
    fn test_transposition_table_matches_disabled_search() {
        let mut with_table_board = Board::new();
        let with_table =
            find_best_move_iterative_with_limits(&mut with_table_board, 4, SearchLimits::default());
        let mut without_table_board = Board::new();
        let without_table = find_best_move_iterative_with_limits(
            &mut without_table_board,
            4,
            SearchLimits {
                use_transposition_table: false,
                ..SearchLimits::default()
            },
        );

        assert_eq!(with_table.best_move, without_table.best_move);
        assert_eq!(with_table.score, without_table.score);
        assert!(with_table.stats.tt_hits > 0);
        assert!(with_table.stats.tt_cutoffs > 0);
        assert_eq!(with_table_board.to_fen(), without_table_board.to_fen());
    }

    #[test]
    fn test_transposition_table_preserves_mate_distance_scores() {
        for ply in [0, 1, 7, 42] {
            for score in [MATE_SCORE - 9, -MATE_SCORE + 9, 123, -456] {
                assert_eq!(score_from_table(score_to_table(score, ply), ply), score);
            }
        }
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
                use_transposition_table: true,
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
        assert_eq!(evaluate_position(&board), -868);
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

        assert_eq!(score, 146);
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

        assert_eq!(score, 175);
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
                use_transposition_table: true,
            },
        );

        assert!(result.best_move.is_some_and(|mv| legal_moves.contains(&mv)));
        assert_eq!(result.stats.depth, 0);
        assert_eq!(board.to_fen(), Board::new().to_fen());
    }
}
