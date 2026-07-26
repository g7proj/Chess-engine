use std::time::Instant;

use engine::board::Board;
use engine::search::find_best_move_with_stats;

#[test]
#[ignore]
fn benchmark_search_make_unmake() {
    let depths: [usize; 2] = [3, 4];

    for depth in depths {
        let iterations: u32 = if depth == 3 { 20 } else { 5 };
        let start: Instant = Instant::now();
        let mut nodes: u64 = 0;
        let mut cutoffs: u64 = 0;

        for _ in 0..iterations {
            let mut board: Board = Board::new();
            let result = find_best_move_with_stats(&mut board, depth);
            assert!(result.best_move.is_some());
            assert!(result.stats.nodes > 0);
            assert!(result.stats.cutoffs > 0);
            nodes += result.stats.nodes;
            cutoffs += result.stats.cutoffs;
            assert_eq!(board.to_fen(), Board::new().to_fen());
        }

        let elapsed_ms: u128 = start.elapsed().as_millis();
        let nps: u128 = if elapsed_ms == 0 {
            nodes as u128
        } else {
            (nodes as u128 * 1000) / elapsed_ms
        };
        println!(
            "search make/unmake depth {}: {} iterations, {} nodes, {} cutoffs in {} ms ({} nps)",
            depth, iterations, nodes, cutoffs, elapsed_ms, nps
        );
    }
}
