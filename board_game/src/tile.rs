use crate::player::Player;

#[derive(Copy, Clone)]
pub struct Tile {
    pub owner: Option<Player>,
}
