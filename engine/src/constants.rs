pub const RANKS: usize = 8;
pub const FILES: usize = 8;

pub const BOARD_SIZE: usize = RANKS * FILES;

#[inline]
pub fn in_bounds(r: isize, f: isize) -> bool {
    r >= 0 && r < RANKS as isize && f >= 0 && f < FILES as isize
}
