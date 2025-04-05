#[derive(Debug, Clone)]
pub enum ManagerToExecutorResMsg {
    ReadyToQuitGameResponse,
}
#[derive(Debug, Clone)]
pub enum ExecutorToManagerResMsg {
    StartGameResponse,
    QuitGameResponse,
}
