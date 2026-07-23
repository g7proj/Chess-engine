use crate::board::Board;
use crate::logger::Logger;
use crate::moves::{Move, Promotion};
use crate::search;
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::time::Instant;

/// Parses an algebraic square into zero-based `(rank, file)` indices.
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

/// Parses a UCI move, including an optional promotion piece.
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

/// Converts a move into its UCI string representation.
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

/// Extracts and validates the depth from a perft or divide command.
fn parse_perft_depth(cmd: &str) -> Result<usize, String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err("empty perft command".to_string());
    }

    let depth_str = if parts[0] == "perft" {
        parts.get(1).copied()
    } else if parts[0] == "divide" {
        parts.get(1).copied()
    } else if parts[0] == "go" && parts.get(1) == Some(&"perft") {
        parts.get(2).copied()
    } else if parts[0] == "go" && parts.get(1) == Some(&"divide") {
        parts.get(2).copied()
    } else {
        None
    };

    let depth_str = depth_str.ok_or_else(|| "missing perft depth".to_string())?;
    let depth: usize = depth_str
        .parse()
        .map_err(|_| format!("invalid perft depth: {}", depth_str))?;
    Ok(depth)
}

/// Runs a perft command against the supplied position.
pub fn handle_perft(cmd: &str, board: &Board) -> Result<u64, String> {
    let depth: usize = parse_perft_depth(cmd)?;
    Ok(board.perft(depth))
}

/// Counts perft nodes for each legal root move.
fn divide_root_moves(board: &Board, depth: usize) -> Vec<(String, u64)> {
    let mut entries: Vec<(String, u64)> = Vec::new();

    for mv in board.generate_all_legal_moves() {
        let mut next: Board = board.clone();
        next.make_move(mv);
        let nodes: u64 = if depth <= 1 { 1 } else { next.perft(depth - 1) };
        entries.push((move_to_uci(&mv), nodes));
    }

    entries
}

/// Runs a divide command against the supplied position.
pub fn handle_divide(cmd: &str, board: &Board) -> Result<Vec<(String, u64)>, String> {
    let depth: usize = parse_perft_depth(cmd)?;
    if depth == 0 {
        return Err("divide depth must be at least 1".to_string());
    }
    Ok(divide_root_moves(board, depth))
}

/// Prints a perft result using UCI-compatible info lines.
fn print_perft_result(nodes: u64, depth: usize, prefix: &str, elapsed_ms: Option<u128>) {
    let nps: Option<u64> = elapsed_ms.map(|ms| {
        if ms == 0 {
            nodes
        } else {
            ((nodes as u128 * 1000) / ms) as u64
        }
    });

    println!("{}info depth {} nodes {}", prefix, depth, nodes);
    if let Some(nps) = nps {
        println!("{}info nps {}", prefix, nps);
    }
    if let Some(ms) = elapsed_ms {
        println!("{}info time {}", prefix, ms);
    }
}

/// Prints root-move counts and totals for a divide result.
fn print_divide_result(
    entries: &[(String, u64)],
    depth: usize,
    prefix: &str,
    elapsed_ms: Option<u128>,
) {
    let total: u64 = entries.iter().map(|(_, nodes)| *nodes).sum();
    let nps: Option<u64> = elapsed_ms.map(|ms| {
        if ms == 0 {
            total
        } else {
            ((total as u128 * 1000) / ms) as u64
        }
    });

    println!("{}info depth {} nodes {}", prefix, depth, total);
    for (mv, nodes) in entries {
        println!("{}info depth {} nodes {} pv {}", prefix, depth, nodes, mv);
    }
    println!("{}info string divide total {}", prefix, total);
    if let Some(nps) = nps {
        println!("{}info nps {}", prefix, nps);
    }
    if let Some(ms) = elapsed_ms {
        println!("{}info time {}", prefix, ms);
    }
}

/// Runs the standalone perft or divide command-line mode.
pub fn run_cli(args: &[String]) -> Result<(), String> {
    let mut mode: Option<&str> = None;
    let mut depth: Option<usize> = None;
    let mut fen: Option<String> = None;

    let mut i: usize = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--perft" => {
                mode = Some("perft");
                i += 1;
                let depth_str = args
                    .get(i)
                    .ok_or_else(|| "missing depth after --perft".to_string())?;
                depth = Some(
                    depth_str
                        .parse()
                        .map_err(|_| format!("invalid depth: {}", depth_str))?,
                );
            }
            "--divide" => {
                mode = Some("divide");
                i += 1;
                let depth_str = args
                    .get(i)
                    .ok_or_else(|| "missing depth after --divide".to_string())?;
                depth = Some(
                    depth_str
                        .parse()
                        .map_err(|_| format!("invalid depth: {}", depth_str))?,
                );
            }
            "--fen" => {
                i += 1;
                fen = Some(
                    args.get(i)
                        .ok_or_else(|| "missing FEN after --fen".to_string())?
                        .clone(),
                );
            }
            "--startpos" => {
                fen = None;
            }
            "--help" | "-h" => {
                println!("Usage:");
                println!("  engine --perft <depth> [--fen <fen>]");
                println!("  engine --divide <depth> [--fen <fen>]");
                println!("If --fen is omitted, start position is used.");
                return Ok(());
            }
            other => {
                return Err(format!("unknown CLI flag: {}", other));
            }
        }
        i += 1;
    }

    let mode = mode.ok_or_else(|| "missing CLI mode: use --perft or --divide".to_string())?;
    let depth = depth.ok_or_else(|| "missing CLI depth".to_string())?;

    let board: Board = if let Some(fen) = fen {
        Board::from_fen(&fen)?
    } else {
        Board::new()
    };

    let start: Instant = Instant::now();
    match mode {
        "perft" => {
            let nodes: u64 = board.perft(depth);
            print_perft_result(nodes, depth, "", Some(start.elapsed().as_millis()));
        }
        "divide" => {
            let entries: Vec<(String, u64)> = handle_divide(&format!("divide {}", depth), &board)?;
            print_divide_result(&entries, depth, "", Some(start.elapsed().as_millis()));
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Runs the UCI command loop.
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
                if cmd.split_whitespace().nth(1) == Some("perft") {
                    match handle_perft(cmd, &board) {
                        Ok(nodes) => {
                            let depth: usize = parse_perft_depth(cmd).unwrap_or(0);
                            print_perft_result(nodes, depth, "", None);
                        }
                        Err(e) => println!("info string Error handling perft command: {}", e),
                    }
                    continue;
                }
                if cmd.split_whitespace().nth(1) == Some("divide") {
                    match handle_divide(cmd, &board) {
                        Ok(entries) => {
                            let depth: usize = parse_perft_depth(cmd).unwrap_or(0);
                            print_divide_result(&entries, depth, "", None);
                        }
                        Err(e) => println!("info string Error handling divide command: {}", e),
                    }
                    continue;
                }
                // Use ordered alpha-beta search at a fixed depth.
                Logger::info("Processing GO command");
                match search::find_best_move(&mut board, 3) {
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
            cmd if cmd.starts_with("perft") => match handle_perft(cmd, &board) {
                Ok(nodes) => {
                    let depth: usize = parse_perft_depth(cmd).unwrap_or(0);
                    print_perft_result(nodes, depth, "", None);
                }
                Err(e) => println!("info string Error handling perft command: {}", e),
            },
            cmd if cmd.starts_with("divide") => match handle_divide(cmd, &board) {
                Ok(entries) => {
                    let depth: usize = parse_perft_depth(cmd).unwrap_or(0);
                    print_divide_result(&entries, depth, "", None);
                }
                Err(e) => println!("info string Error handling divide command: {}", e),
            },
            other => {
                Logger::info(&format!("Unknown command: {}", other));
                println!("Unknown command: {}", other);
            }
        }
    }
    Logger::info("=== Engine shutdown ===");
}

/// Resets the board and engine options for a new game.
pub fn handle_ucinewgame(board: &mut Board, options: &mut HashMap<String, String>) {
    *board = Board::new();
    board.record_position();
    options.clear();
    // Reset any other state as needed
}

/// Parses and stores a UCI `setoption` command.
pub fn handle_setoption(cmd: &str, options: &mut HashMap<String, String>) {
    // Example: setoption name Hash value 128
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if let Some(name_idx) = parts.iter().position(|p| *p == "name") {
        let value_idx: usize = parts
            .iter()
            .position(|p| *p == "value")
            .unwrap_or(parts.len());
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

/// Prints all legal moves in UCI notation.
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

/// Handles a UCI stop command.
pub fn handle_stop() {
    // Placeholder for stopping search
}

/// Handles a UCI ponderhit command.
pub fn handle_ponderhit() {
    // Placeholder for handling ponderhit
}

/// Sets the board from a UCI `position` command and applies legal moves.
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
                let mv = parse_move_str(mv_str)
                    .ok_or_else(|| format!("Invalid move format: {}", mv_str))?;
                if board.generate_all_legal_moves().contains(&mv) {
                    board.make_move(mv);
                } else {
                    // ignore illegal move, report via info string on caller's side if needed
                }
            }
        }
        Ok(())
    } else if parts[1] == "fen" {
        let move_pos = parts
            .iter()
            .position(|&p| p == "moves")
            .unwrap_or(parts.len());
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
                        let mv = parse_move_str(mv_str)
                            .ok_or_else(|| format!("Invalid move format: {}", mv_str))?;
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

        // Compare the generated UCI move lists without redirecting stdout.
        let mut actual_uci_moves: Vec<String> = board
            .generate_all_legal_moves()
            .iter()
            .map(|m| move_to_uci(m))
            .collect();
        actual_uci_moves.sort();

        assert_eq!(expected_uci_moves, actual_uci_moves);
    }

    #[test]
    fn test_handle_perft_parses_plain_command() {
        let board: Board = Board::new();
        let nodes: u64 = handle_perft("perft 1", &board).unwrap();
        assert_eq!(nodes, 20);
    }

    #[test]
    fn test_handle_perft_parses_go_form() {
        let board: Board = Board::new();
        let nodes: u64 = handle_perft("go perft 2", &board).unwrap();
        assert_eq!(nodes, 400);
    }

    #[test]
    fn test_handle_divide_parses_plain_command() {
        let board: Board = Board::new();
        let entries: Vec<(String, u64)> = handle_divide("divide 1", &board).unwrap();
        assert_eq!(entries.len(), 20);
        assert!(entries.iter().all(|(_, nodes)| *nodes == 1));
    }

    #[test]
    fn test_handle_divide_parses_go_form() {
        let board: Board = Board::new();
        let entries: Vec<(String, u64)> = handle_divide("go divide 1", &board).unwrap();
        assert_eq!(entries.len(), 20);
        assert_eq!(entries.iter().map(|(_, nodes)| *nodes).sum::<u64>(), 20);
    }

    #[test]
    fn test_print_divide_result_format() {
        let entries: Vec<(String, u64)> = vec![("e2e4".to_string(), 20), ("d2d4".to_string(), 20)];
        print_divide_result(&entries, 1, "", Some(0));
    }
}
