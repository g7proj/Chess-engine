use std::io::{self, BufRead};
use engine::uci::handle_uci_command;

/**
This is the main function for the chess engine.
It reads commands from stdin and handles them.
*/
fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(command) = line {
            if command == "quit" {
                break;
            }
            handle_uci_command(command);
        }
    }
}
