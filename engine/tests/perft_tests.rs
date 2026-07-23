use engine::board::Board;

fn assert_perft_counts(fen: &str, expected: &[(usize, u64)]) {
    let board: Board = Board::from_fen(fen).expect("valid perft FEN");
    for (depth, nodes) in expected {
        assert_eq!(
            board.perft(*depth),
            *nodes,
            "unexpected perft at depth {} for {}",
            depth,
            fen
        );
    }
}

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

#[test]
fn test_perft_kiwipete_depth_2_to_4() {
    let fen: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    assert_perft_counts(fen, &[(2, 2039), (3, 97862), (4, 4085603)]);
}

#[test]
fn test_perft_position_3_depth_2_to_4() {
    let fen: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    assert_perft_counts(fen, &[(2, 191), (3, 2812), (4, 43238)]);
}

#[test]
fn test_perft_position_4_depth_2_to_4() {
    let fen: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    assert_perft_counts(fen, &[(2, 264), (3, 9467), (4, 422333)]);
}
