use crate::{
    consts::{HEIGHT, WIDTH},
    tile::Tile,
};

pub struct Board {
    pub board: [[Option<Tile>; WIDTH]; HEIGHT],
}

impl Board {
    pub fn new() -> Self {
        Board {
            board: [[None; WIDTH]; HEIGHT],
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
