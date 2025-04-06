use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{
    Mutex,
    mpsc::{self, error::SendError},
};

use crate::message::{
    ExecutorToManagerMsg, ManagerToExecutorMsg, request_message::ManagerToExecutorReqMsg,
    response_message::ManagerToExecutorResMsg,
};

#[derive(Debug, thiserror::Error)]
pub enum GameExecutorError {
    #[error("Get rx error")]
    GetRxError,
    #[error("Get tx error")]
    GetTxError,
    #[error("Message error")]
    MessageErr,
    #[error("Send error: {0:?}")]
    SendError(#[from] SendError<ExecutorToManagerMsg>),
}

#[async_trait]
pub trait GameExecutor {
    async fn run(&self) -> Result<(), GameExecutorError> {
        let rx = self.get_rx()?;
        while let Some(message) = rx.lock().await.recv().await {
            match message {
                ManagerToExecutorMsg::Request(request_message) => match request_message {
                    ManagerToExecutorReqMsg::InitGameRequest => {
                        self.init_game().await?;
                    }
                    ManagerToExecutorReqMsg::QuitGameRequest => {
                        self.quit_game().await?;
                    }
                    ManagerToExecutorReqMsg::ExecuteGameRequest => {
                        self.execute_game().await?;
                    }
                },
                ManagerToExecutorMsg::Response(response_message) => match response_message {
                    ManagerToExecutorResMsg::ReadyToQuitGameResponse => {}
                },
            }
        }

        Ok(())
    }

    fn get_tx(&self) -> Result<Arc<Mutex<mpsc::Sender<ExecutorToManagerMsg>>>, GameExecutorError>;

    fn get_rx(&self)
    -> Result<Arc<Mutex<mpsc::Receiver<ManagerToExecutorMsg>>>, GameExecutorError>;

    async fn draw_opening_screen(&self) -> Result<(), GameExecutorError>;

    async fn init_game(&self) -> Result<(), GameExecutorError>;

    async fn quit_game(&self) -> Result<(), GameExecutorError>;

    async fn execute_game(&self) -> Result<(), GameExecutorError>;
}
