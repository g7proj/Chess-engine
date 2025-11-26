use std::io::{self, BufRead};
use crate::board::Board;
use crate::moves::{Move, Promotion};

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

pub fn run_uci() {
    let stdin: io::Stdin = io::stdin();
    let mut board: Board = Board::new();

    for line in stdin.lock().lines() {
        let command: String = line.unwrap();
        match command.as_str() {
            "uci" => {
                println!("id name Rust Chess Engine");
                println!("id author Denis Altomare");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "quit" => {
                break;
            }
            cmd if cmd.starts_with("position") => {
                // Example commands:
                // position [fen <fenstring> | startpos ]  moves <move1> .... <movei>
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] == "startpos" {
                    board = Board::new();
                    // find moves keyword
                    if let Some(pos) = parts.iter().position(|&p| p == "moves") {
                        for mv_str in &parts[pos+1..] {
                            if let Some(mv) = parse_move_str(mv_str) {
                                board.make_move(mv);
                            }
                        }
                    }
                } else if parts.len() >= 3 && parts[1] == "fen" {
                    // minimal FEN: not fully supported -> ignore for now
                    // TODO: implement FEN parser
                    // apply moves if present
                    if let Some(pos) = parts.iter().position(|&p| p == "moves") {
                        for mv_str in &parts[pos+1..] {
                            if let Some(mv) = parse_move_str(mv_str) {
                                board.make_move(mv);
                            }
                        }
                    }
                }
            }
            cmd if cmd.starts_with("go") => {
                // Very simple: pick first legal move
                let moves: Vec<Move> = board.generate_all_legal_moves();
                if moves.is_empty() {
                    println!("bestmove (none)");
                } else {
                    let best: Move = moves[0];
                    let best_uci: String = move_to_uci(&best);
                    println!("bestmove {}", best_uci);
                }
            }
            cmd if cmd.starts_with("setoption") => {
                // For now, ignore options
            }
            other => {
                println!("Unknown command: {}", other);
            }
        }
    }
}
