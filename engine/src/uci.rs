pub fn handle_uci_command(command: String) {
    match command.as_str() {
        "uci" => {
            println!("id name Rust Chess Engine");
            println!("id author Denis Altomare");
            println!("uciok");
        }
        "isready" => {
            println!("readyok");
        }
        other => {
            println!("Unknown command: {}", other);
        }
    }
}
