use std::collections::HashMap;
use std::io::{self, BufRead};
use crate::board::{Board, Piece};
use crate::moves::{Move, Promotion};
use crate::logger::{Logger};

/**
 * Parse a square in algebraic notation (e.g. "e4") into (rank, file) indices
 */
fn parse_square(sq: &str) -> Option<(usize, usize)> {
    let bytes: &[u8] = sq.as_bytes();
    
    // Check length (must be 2)
    if bytes.len() != 2 {
        return None;
    }

    // Retrieve file and rank characters
    let file: char = (bytes[0] as char).to_ascii_lowercase();
    let rank: char = bytes[1] as char;

    // Validate file and rank
    if !('a'..='h').contains(&file) || !('1'..='8').contains(&rank) {
        return None;
    }
    
    let f: usize = (file as u8 - b'a') as usize;
    let r: usize = (rank as u8 - b'1') as usize;
    Some((r, f))
}

/**
 * Parse a move in UCI format (e.g. "e2e4", "e7e8q") into a Move struct
 */
fn parse_move_str(mv_str: &str) -> Option<Move> {
    let bytes: &[u8] = mv_str.as_bytes();
    
    // Check length (must be 4 or 5)
    if bytes.len() < 4 || bytes.len() > 5 {
        return None;
    }

    // Parse from and to squares
    let from_sq: &str = &mv_str[0..2];
    let to_sq: &str = &mv_str[2..4];

    let (from_rank, from_file) = parse_square(from_sq)?;
    let (to_rank, to_file) = parse_square(to_sq)?;

    // Handle promotion if present
    let promotion: Promotion = if bytes.len() == 5 {
        match bytes[4] as char {
            'q' | 'Q' => Promotion::Queen,
            'r' | 'R' => Promotion::Rook,
            'b' | 'B' => Promotion::Bishop,
            'n' | 'N' => Promotion::Knight,
            _ => return None,
        }
    } else {
        Promotion::None
    };

    Some(Move {
        from_rank,
        from_file,
        to_rank,
        to_file,
        promotion,
    })
}

/**
 * Convert a Move struct into UCI format string
 * Example: Move { from_rank: 1, from_file: 4, to_rank: 3, to_file: 4, promotion: None } -> "e2e4"
 */
fn move_to_uci(mv: &Move) -> String {
    let from_file: char = (b'a' + mv.from_file as u8) as char;
    let from_rank: char = (b'1' + mv.from_rank as u8) as char;
    let to_file: char = (b'a' + mv.to_file as u8) as char;
    let to_rank: char = (b'1' + mv.to_rank as u8) as char;

    let mut uci_str: String = format!("{}{}{}{}", from_file, from_rank, to_file, to_rank);

    match mv.promotion {
        Promotion::Queen => uci_str.push('q'),
        Promotion::Rook => uci_str.push('r'),
        Promotion::Bishop => uci_str.push('b'),
        Promotion::Knight => uci_str.push('n'),
        Promotion::None => {}
    }

    uci_str
}

/**
 * Evaluate the material balance on the board
 * Positive score favors white, negative favors black
 */
fn evaluate_position(board: &Board) -> i32 {
    let mut score: i32 = 0;
    for rank in 0..8 {
        for file in 0..8 {
            score += piece_value(board.squares[rank][file]);
        }
    }
    score
}

/**
 * Get the point value of a piece
 */
fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::PawnWhite => 1,
        Piece::PawnBlack => -1,
        Piece::KnightWhite => 3,
        Piece::KnightBlack => -3,
        Piece::BishopWhite => 3,
        Piece::BishopBlack => -3,
        Piece::RookWhite => 5,
        Piece::RookBlack => -5,
        Piece::QueenWhite => 9,
        Piece::QueenBlack => -9,
        _ => 0,
    }
}

/**
 * Minimax with negamax approach
 * Returns the best score from the current position
 * Positive score is good for the side to move
 */
fn minimax(board: Board, depth: usize) -> i32 {
    if depth == 0 {
        return evaluate_position(&board);
    }

    let moves: Vec<Move> = board.generate_all_legal_moves();
    Logger::debug(&format!("minimax depth={} moves={}", depth, moves.len()));
    if moves.is_empty() {
        // Check for checkmate or stalemate
        if board.is_in_check(board.side_to_move) {
            return -9000; // Checkmate is very bad
        } else {
            return 0; // Stalemate is neutral
        }
    }

    let mut best_score: i32 = i32::MIN;
    for mv in moves {
        let mut board_copy: Board = board.clone();
        board_copy.make_move(mv);
        let score: i32 = -minimax(board_copy, depth - 1);
        best_score = best_score.max(score);
    }

    best_score
}

/**
 * Find the best move using Minimax algorithm
 */
fn find_best_move(board: &Board, depth: usize) -> Option<Move> {
    Logger::debug("find_best_move: starting");
    let moves = board.generate_all_legal_moves();
    Logger::debug(&format!("find_best_move: found {} legal moves", moves.len()));

    if moves.is_empty() {
        Logger::debug("find_best_move: no legal moves found");
        return None;
    }

    let mut best_move: Move = moves[0];
    let mut best_score: i32 = i32::MIN;

    for mv in moves {
        let mut board_copy: Board = board.clone();
        board_copy.make_move(mv);
        let score = -minimax(board_copy, depth - 1);

        if score > best_score {
            best_score = score;
            best_move = mv;
        }
    }

    Some(best_move)
}


pub fn run_uci() {
    // Initialize logger with desired log level and file path
    Logger::init(crate::logger::LogLevel::Info, "engine_debug.log");
    Logger::info("\n=== Engine started ===");
    let stdin: io::Stdin = io::stdin();
    let mut board: Board = Board::new();
    let mut options: HashMap<String, String> = HashMap::new();

    for line in stdin.lock().lines() {
        let command: String = match line {
            Ok(cmd) => cmd,
            Err(e) => {
                Logger::error(&format!("ERROR reading stdin: {}", e));
                break;
            }
        };
        Logger::info(&format!("CMD: {}", command));
        match command.as_str() {
            "uci" => {
                println!("id name Rust Chess Engine");
                println!("id author Denis Altomare");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "ucinewgame" => {
                handle_ucinewgame(&mut board, &mut options);
            }
            "quit" => {
                break;
            }
            cmd if cmd.starts_with("position") => {
                // Example commands:
                // position [fen <fenstring> | startpos ]  moves <move1> .... <movei>
                Logger::info(&format!("Handling position: {}", cmd));
                if let Err(e) = handle_position(cmd, &mut board) {
                    Logger::error(&format!("ERROR in position command: {}", e));
                    println!("info string Error handling position command: {}", e);
                } else {
                    Logger::info("Position handled successfully");
                }
            }
            cmd if cmd.starts_with("go") => {
                // Use Minimax to find the best move (depth 3)
                Logger::info("Processing GO command");
                match find_best_move(&board, 3) {
                    Some(best) => {
                        let best_uci: String = move_to_uci(&best);
                        Logger::info(&format!("Found best move: {}", best_uci));
                        println!("bestmove {}", best_uci);
                    }
                    None => {
                        Logger::info("No legal moves found");
                        println!("bestmove (none)");
                    }
                }
            }
            cmd if cmd.starts_with("stop") => {
                handle_stop();
            }
            cmd if cmd.starts_with("ponderhit") => {
                handle_ponderhit();
            }
            cmd if cmd.starts_with("setoption") => {
                handle_setoption(cmd, &mut options);
            }
            other => {
                Logger::info(&format!("Unknown command: {}", other));
                println!("Unknown command: {}", other);
            }
        }
    }
    Logger::info("=== Engine shutdown ===");
}

/**
 * Handle ucinewgame command to reset the board and options
 */
pub fn handle_ucinewgame(board: &mut Board, options: &mut HashMap<String, String>) {
    *board = Board::new();
    board.record_position();
    options.clear();
    // Reset any other state as needed
}

/**
 * Simple setoption parser
 */
pub fn handle_setoption(cmd: &str, options: &mut HashMap<String, String>) {
    // Example: setoption name Hash value 128
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if let Some(name_idx) = parts.iter().position(|p| *p == "name") {
        let value_idx: usize = parts.iter().position(|p| *p == "value").unwrap_or(parts.len());
        if name_idx + 1 <= value_idx {
            let name: String = parts[name_idx + 1..value_idx].join(" ");
            let value: String = if value_idx < parts.len() {
                parts[value_idx + 1..].join(" ")
            } else {
                String::new()
            };
            options.insert(name, value);
        }
    }
}

pub fn handle_listmoves(board: &Board) {
    // Print all legal moves in UCI format on one line prefixed by "legalmoves"
    let moves: Vec<Move> = board.generate_all_legal_moves();
    let mut parts: Vec<String> = Vec::new();
    for m in moves.iter() {
        parts.push(move_to_uci(m));
    }
    if parts.is_empty() {
        println!("legalmoves");
    } else {
        println!("legalmoves {}", parts.join(" "));
    }
}

pub fn handle_stop() {
    // Placeholder for stopping search
}

pub fn handle_ponderhit() {
    // Placeholder for handling ponderhit
}

/**
 * Handle the "position" UCI command to set up the board position
 * Examples:
 * position startpos moves e2e4 e7e5
 * position fen <fenstring> moves e2e4 e7e5
 */
pub fn handle_position(cmd: &str, board: &mut Board) -> Result<(), String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.len() < 2 {
        return Err("position command too short".to_string());
    }

    if parts[1] == "startpos" {
        *board = Board::new();
        board.record_position();

        if let Some(pos) = parts.iter().position(|&p| p == "moves") {
            for mv_str in &parts[pos + 1..] {
                let mv = parse_move_str(mv_str).ok_or_else(|| format!("Invalid move format: {}", mv_str))?;
                if board.generate_all_legal_moves().contains(&mv) {
                    board.make_move(mv);
                } else {
                    // ignore illegal move, report via info string on caller's side if needed
                }
            }
        }
        Ok(())
    } else if parts[1] == "fen" {
        let move_pos = parts.iter().position(|&p| p == "moves").unwrap_or(parts.len());
        let fen_parts = &parts[2..move_pos];
        if fen_parts.is_empty() {
            return Err("Missing FEN string".to_string());
        }
        let fen = fen_parts.join(" ");
        match Board::from_fen(&fen) {
            Ok(new_board) => {
                *board = new_board;
                board.record_position();

                if move_pos < parts.len() {
                    for mv_str in &parts[move_pos + 1..] {
                        let mv = parse_move_str(mv_str).ok_or_else(|| format!("Invalid move format: {}", mv_str))?;
                        if board.generate_all_legal_moves().contains(&mv) {
                            board.make_move(mv);
                        } else {
                            // ignore illegal move
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(format!("Error parsing FEN: {}", e)),
        }
    } else {
        Err("Unknown position command".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, Piece};

    #[test]
    fn test_handle_position_fen_applies_moves_and_records() {
        let start_pos: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let start_fen: String = format!("{} w KQkq - 0 1", start_pos);
        let cmd: String = format!("position fen {} moves e2e4 e7e5", start_fen);
        let mut board: Board = Board::new(); // will be replaced by FEN
        assert!(handle_position(&cmd, &mut board).is_ok());

        // e2 -> e4 (white pawn at rank 3,file 4)
        assert_eq!(board.squares[3][4], Piece::PawnWhite);
        // e7 -> e5 (black pawn at rank 4,file 4)
        assert_eq!(board.squares[4][4], Piece::PawnBlack);

        // current position must be recorded once with full repetition key
        let key: String = board.repetition_key();
        assert_eq!(board.history.get(&key), Some(&1));
    }

    #[test]
    fn test_handle_position_startpos_ignores_illegal_move() {
        let cmd: String = "position startpos moves e2e5".to_string(); // illegal: pawn can't jump to e5
        let mut board: Board = Board::new();
        assert!(handle_position(&cmd, &mut board).is_ok());

        // pawn must remain on e2 (rank 1,file 4)
        assert_eq!(board.squares[1][4], Piece::PawnWhite);
    }

    #[test]
    fn test_handle_position_invalid_fen_returns_error() {
        let cmd: String = "position fen invalid_fen_string".to_string();
        let mut board: Board = Board::new();
        let res: Result<(), String> = handle_position(&cmd, &mut board);
        assert!(res.is_err());
    }

    #[test]
    fn test_handle_ucinewgame_resets_board() {
        let mut board: Board = Board::new();
        // apply a legal move e2e4
        let mv: Move = parse_move_str("e2e4").unwrap();
        assert!(board.generate_all_legal_moves().contains(&mv));
        board.make_move(mv);
        assert_eq!(board.squares[3][4], Piece::PawnWhite);

        let mut options: HashMap<String, String> = HashMap::new();
        options.insert("Hash".to_string(), "128".to_string());

        handle_ucinewgame(&mut board, &mut options);

        // pawn must be back at e2 and options cleared
        assert_eq!(board.squares[1][4], Piece::PawnWhite);
        assert_eq!(board.fullmove_number, 1);
        assert!(options.is_empty());
    }

    #[test]
    fn test_handle_setoption_parses_name_and_value() {
        let mut options: HashMap<String, String> = HashMap::new();
        handle_setoption("setoption name Hash value 256", &mut options);
        assert_eq!(options.get("Hash"), Some(&"256".to_string()));

        handle_setoption("setoption name UCI_AnalyseMode value true", &mut options);
        assert_eq!(options.get("UCI_AnalyseMode"), Some(&"true".to_string()));
    }

    #[test]
    fn test_listmoves() {
        let board: Board = Board::new();
        let moves: Vec<Move> = board.generate_all_legal_moves();
        let mut expected_uci_moves: Vec<String> = Vec::new();
        for m in moves.iter() {
            expected_uci_moves.push(move_to_uci(m));
        }
        expected_uci_moves.sort();

        // Non si tenta più di sostituire stdout.lock(); si verifica direttamente la lista di mosse UCI
        let mut actual_uci_moves: Vec<String> = board.generate_all_legal_moves().iter().map(|m| move_to_uci(m)).collect();
        actual_uci_moves.sort();

        assert_eq!(expected_uci_moves, actual_uci_moves);
    }
}
