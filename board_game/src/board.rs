use crate::{
    consts::{HEIGHT, WIDTH},
    tile::Tile,
};

pub struct Board {
    pub board: Vec<Vec<Option<Tile>>>,
}

impl Board {
    pub fn new() -> Self {
        let board = vec![vec![None; WIDTH]; HEIGHT];

        Board { board }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
