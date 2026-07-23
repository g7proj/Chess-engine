use engine::board::{Board, Color, Piece};

#[test]
fn test_check() {
    let mut board: Board = Board::new();
    // Clear the board
    for r in 0..8 {
        for f in 0..8 {
            board.squares[r][f] = Piece::Empty;
        }
    }
    // Set up: white king on e1, black rook on e8
    board.squares[0][4] = Piece::KingWhite;
    board.squares[7][4] = Piece::RookBlack;
    board.side_to_move = Color::White;
    assert!(board.is_in_check(Color::White));

    board.squares[7][4] = Piece::Empty;
    board.squares[7][5] = Piece::RookBlack;
    assert!(!board.is_in_check(Color::White));
}

#[test]
fn test_checkmate() {
    let mut board: Board = Board::new();
    // Clear the board
    for r in 0..8 {
        for f in 0..8 {
            board.squares[r][f] = Piece::Empty;
        }
    }
    // Set up: white king on e1, black rook on e8
    board.squares[0][4] = Piece::KingWhite;
    board.squares[7][4] = Piece::RookBlack;
    // This is check, not checkmate
    assert!(!board.is_checkmate());

    board.squares[7][3] = Piece::RookBlack;
    board.squares[7][5] = Piece::QueenBlack;
    // now this is checkmate
    board.side_to_move = Color::White;
    assert!(board.is_checkmate());
}

#[test]
fn test_stalemate() {
    let mut board: Board = Board::new();
    // Clear the board
    for r in 0..8 {
        for f in 0..8 {
            board.squares[r][f] = Piece::Empty;
        }
    }

    // Setup the board
    board.side_to_move = Color::White;
    board.squares[0][0] = Piece::KingWhite;
    board.squares[1][1] = Piece::RookBlack;

    // The white king has one move (capture the rook)
    assert!(!board.is_stalemate());

    // Place a pawn to guard the rook
    board.squares[2][2] = Piece::PawnBlack;
    // The white king has no move and is not under check
    assert!(board.is_stalemate());

    // If there is a queen instead of the rook, the king is under check and has no moves.
    // This is not a stalemate, this is a checkmate
    board.squares[1][1] = Piece::QueenBlack;
    assert!(!board.is_stalemate());
}
