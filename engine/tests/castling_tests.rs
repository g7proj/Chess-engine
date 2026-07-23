use engine::board::{Board, Piece};

#[test]
fn test_complete_castling() {
    let mut board = Board::new();
    // Set up for castling
    board.squares[0][5] = Piece::Empty; // f1
    board.squares[0][6] = Piece::Empty; // g1
    board.squares[0][1] = Piece::Empty; // b1
    board.squares[0][2] = Piece::Empty; // c1
    board.squares[0][3] = Piece::Empty; // d1

    let moves = board.generate_king_moves(0, 4);

    // check castling moves
    assert!(moves.iter().any(|m| m.to_file == 6)); // Kingside
    assert!(moves.iter().any(|m| m.to_file == 2)); // Queenside

    // Execute castling
    let castling_move = moves.iter().find(|m| m.to_file == 6).unwrap();
    board.make_move(*castling_move);

    // check king in g1
    assert_eq!(board.squares[0][6], Piece::KingWhite);
    // check rook in f1
    assert_eq!(board.squares[0][5], Piece::RookWhite);
    // check castling rights
    assert!(board.white_kingside_castle == false);
}

#[test]
fn test_complete_castling_under_check() {
    let mut board = Board::new();
    // Set up for castling
    board.squares[0][5] = Piece::Empty; // f1
    board.squares[0][6] = Piece::Empty; // g1
    board.squares[0][1] = Piece::Empty; // b1
    board.squares[0][2] = Piece::Empty; // c1
    board.squares[0][3] = Piece::Empty; // d1

    // place a knight attacking the king
    board.squares[2][5] = Piece::KnightBlack;

    let moves = board.generate_king_moves(0, 4);

    // check castling moves
    assert!(moves.iter().any(|m| m.to_file == 6) == false); // Kingside
    assert!(moves.iter().any(|m| m.to_file == 2) == false); // Queenside
    // check castling rights (should be still available)
    assert!(board.white_kingside_castle == true);
    assert!(board.white_queenside_castle == true);
}
