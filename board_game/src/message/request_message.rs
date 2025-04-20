use crate::player::Player;

#[derive(Debug, Clone)]
pub enum ManagerToExecutorReqMsg {
    InitGameRequest,
    QuitGameRequest,
    ExecuteGameRequest,
    PlayerWinRequest(Player),
}
#[derive(Debug, Clone)]
pub enum ExecutorToManagerReqMsg {
    ReadyToQuitGameRequest,
    TileOnByPlayerRequesst(Player, usize, usize),
}
