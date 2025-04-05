use request_message::{ExecutorToManagerReqMsg, ManagerToExecutorReqMsg};
use response_message::{ExecutorToManagerResMsg, ManagerToExecutorResMsg};

pub mod request_message;
pub mod response_message;

#[derive(Debug, Clone)]
pub enum ManagerToExecutorMsg {
    Request(ManagerToExecutorReqMsg),
    Response(ManagerToExecutorResMsg),
}
#[derive(Debug, Clone)]
pub enum ExecutorToManagerMsg {
    Request(ExecutorToManagerReqMsg),
    Response(ExecutorToManagerResMsg),
}
