pub struct Move {
    pub from: u8,
    pub to: u8,
}

impl Move {
    pub fn new(from: u8, to: u8) -> Self {
        Move { from, to }
    }
}
