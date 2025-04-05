use crate::{board::Board, consts::PLAYER_NUM, player::Player};

pub struct GameManager {
    pub board: Board,
    pub players: [Player; PLAYER_NUM],
}
