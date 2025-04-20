#[derive(Debug, Clone)]
pub enum ManagerToExecutorResMsg {
    ReadyToQuitGameResponse,
    // bool value to indicate whether a player had won
    TileOnByPlayerResponse(bool),
}
#[derive(Debug, Clone)]
pub enum ExecutorToManagerResMsg {
    InitGameResponse,
    QuitGameResponse,
    ExecuteGameResponse,
    PlayerWinResponse,
}
