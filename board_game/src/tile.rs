use crate::player::Player;

#[derive(Clone, PartialEq, Debug)]
pub struct Tile {
    pub owner: Option<Player>,
}
