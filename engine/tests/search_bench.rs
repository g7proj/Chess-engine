use std::time::Instant;

use engine::board::Board;
use engine::search::find_best_move;

#[test]
#[ignore]
fn benchmark_search_make_unmake() {
    let depths: [usize; 2] = [3, 4];

    for depth in depths {
        let iterations: u32 = if depth == 3 { 20 } else { 5 };
        let start: Instant = Instant::now();

        for _ in 0..iterations {
            let mut board: Board = Board::new();
            assert!(find_best_move(&mut board, depth).is_some());
            assert_eq!(board.to_fen(), Board::new().to_fen());
        }

        let elapsed_ms: u128 = start.elapsed().as_millis();
        println!(
            "search make/unmake depth {}: {} iterations in {} ms",
            depth, iterations, elapsed_ms
        );
    }
}
