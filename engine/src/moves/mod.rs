pub mod knight;
pub mod king;
pub mod pawn;
pub mod sliding;
pub mod attacks;
pub mod apply;

#[derive(Debug, Clone, Copy)]
pub enum Promotion {
    Queen,
    Rook,
    Bishop,
    Knight,
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from_rank: usize,
    pub from_file: usize,
    pub to_rank: usize,
    pub to_file: usize,
    pub promotion: Promotion,
}

impl Move {
    pub fn new(fr: usize, ff: usize, tr: usize, tf: usize) -> Self {
        Move {
            from_rank: fr,
            from_file: ff,
            to_rank: tr,
            to_file: tf,
            promotion: Promotion::None,
        }
    }

    pub fn with_promotion(fr: usize, ff: usize, tr: usize, tf: usize, p: Promotion) -> Self {
        Move {
            from_rank: fr,
            from_file: ff,
            to_rank: tr,
            to_file: tf,
            promotion: p,
        }
    }
}
