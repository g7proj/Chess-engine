use engine::uci;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let has_cli_mode: bool = args.iter().any(|arg| arg == "--perft" || arg == "--divide" || arg == "--help" || arg == "-h");

    if has_cli_mode {
        if let Err(e) = uci::run_cli(&args) {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    } else {
        uci::run_uci();
    }
}
