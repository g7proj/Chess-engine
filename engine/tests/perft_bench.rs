use std::time::Instant;

use engine::board::Board;

fn run_benchmark_case(name: &str, board: &Board, depth: usize, expected: u64) {
    let start: Instant = Instant::now();
    let nodes: u64 = board.perft(depth);
    let elapsed_ms: u128 = start.elapsed().as_millis();
    let nps: u128 = if elapsed_ms == 0 {
        nodes as u128
    } else {
        (nodes as u128 * 1000) / elapsed_ms
    };

    assert_eq!(nodes, expected, "unexpected node count for {}", name);
    println!("{}: {} nodes in {} ms ({} nps)", name, nodes, elapsed_ms, nps);
}

#[test]
#[ignore]
fn benchmark_perft_known_positions() {
    let startpos: Board = Board::new();
    run_benchmark_case("startpos depth 4", &startpos, 4, 197_281);

    let castling_fen: &str = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    let castling: Board = Board::from_fen(castling_fen).expect("valid castling FEN");
    run_benchmark_case("castling depth 1", &castling, 1, 26);
}
