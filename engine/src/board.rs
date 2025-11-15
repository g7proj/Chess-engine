#[derive(Debug, Clone)]
pub struct Board {
    pub side_to_move: char,
}

impl Board {
    pub fn new() -> Self {
        Board { side_to_move: 'w' }
    }
}
