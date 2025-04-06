#[derive(Debug, Clone)]
pub enum ManagerToExecutorReqMsg {
    InitGameRequest,
    QuitGameRequest,
    ExecuteGameRequest,
}
#[derive(Debug, Clone)]
pub enum ExecutorToManagerReqMsg {
    ReadyToQuitGameRequest,
}
