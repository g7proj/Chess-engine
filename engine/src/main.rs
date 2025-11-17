use engine::board::{Board, Piece};
use engine::moves::Move;

fn main() {
    let mut board = Board::new();

    // Pulisci tutto
    for r in 0..8 {
        for f in 0..8 {
            board.squares[r][f] = Piece::Empty;
        }
    }

    // Posizione:
    // Nero: pedone in d5: (4,3)
    // Bianco: pedone in e5: (4,4)
    board.squares[4][3] = Piece::PawnBlack;
    board.squares[4][4] = Piece::PawnWhite;

    // En passant target (d6): (5,3)
    board.en_passant = Some((5, 3));

    println!("Before move:");
    board.print();

    let mv = Move::new(4,4,5,3);  // e5 → d6 (EN PASSANT)

    println!("EP is: {:?}", board.en_passant);
    println!("Move is: from=({},{}), to=({},{})", mv.from_rank, mv.from_file, mv.to_rank, mv.to_file);

    board.make_move(mv);

    println!("After move:");
    board.print();
}