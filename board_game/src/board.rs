use crate::{
    consts::{HEIGHT, WIDTH},
    tile::Tile,
};

pub struct Board {
    pub board: [[Option<Tile>; WIDTH]; HEIGHT],
}
