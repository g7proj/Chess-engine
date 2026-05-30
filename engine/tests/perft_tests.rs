use engine::board::Board;

#[test]
fn test_perft_start_position_depth_1_to_3() {
    let board: Board = Board::new();

    assert_eq!(board.perft(1), 20);
    assert_eq!(board.perft(2), 400);
    assert_eq!(board.perft(3), 8902);
}

#[test]
fn test_perft_castling_position_depth_1() {
    let fen: &str = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    let board: Board = Board::from_fen(fen).expect("valid castling FEN");

    assert_eq!(board.perft(1), 26);
}

#[test]
fn test_perft_en_passant_position_depth_1() {
    let fen: &str = "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1";
    let board: Board = Board::from_fen(fen).expect("valid en passant FEN");

    assert_eq!(board.perft(1), 7);
}

#[test]
fn test_perft_promotion_position_depth_1() {
    let fen: &str = "4k3/P7/8/8/8/8/8/4K3 w - - 0 1";
    let board: Board = Board::from_fen(fen).expect("valid promotion FEN");

    assert_eq!(board.perft(1), 9);
}
