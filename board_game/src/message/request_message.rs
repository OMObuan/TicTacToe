#[derive(Debug, Clone)]
pub enum ManagerToExecutorReqMsg {
    InitGameRequest,
    QuitGameRequest,
}
#[derive(Debug, Clone)]
pub enum ExecutorToManagerReqMsg {
    ReadyToQuitGameRequest,
}
