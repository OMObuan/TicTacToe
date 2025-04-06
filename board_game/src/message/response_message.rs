#[derive(Debug, Clone)]
pub enum ManagerToExecutorResMsg {
    ReadyToQuitGameResponse,
}
#[derive(Debug, Clone)]
pub enum ExecutorToManagerResMsg {
    InitGameResponse,
    QuitGameResponse,
    ExecuteGameResponse,
}
