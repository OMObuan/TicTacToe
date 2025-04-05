use std::time::Duration;

use async_trait::async_trait;
use tokio::{
    sync::mpsc::{self, error::SendError},
    time::sleep,
};
use tracing::trace;

use crate::message::{
    ExecutorToManagerMsg, ManagerToExecutorMsg,
    request_message::{ExecutorToManagerReqMsg, ManagerToExecutorReqMsg},
    response_message::{ExecutorToManagerResMsg, ManagerToExecutorResMsg},
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
    async fn run(&mut self) -> Result<(), GameExecutorError> {
        let (tx, rx) = self.get_channel();
        while let Some(message) = rx.recv().await {
            match message {
                ManagerToExecutorMsg::Request(request_message) => match request_message {
                    ManagerToExecutorReqMsg::InitGameRequest => {
                        trace!("start to init game");
                        tx.send(ExecutorToManagerMsg::Response(
                            ExecutorToManagerResMsg::StartGameResponse,
                        ))
                        .await?;
                        sleep(Duration::from_secs(2)).await;
                        tx.send(ExecutorToManagerMsg::Request(
                            ExecutorToManagerReqMsg::ReadyToQuitGameRequest,
                        ))
                        .await?;
                    }
                    ManagerToExecutorReqMsg::QuitGameRequest => {
                        trace!("execute quiting game");
                        tx.send(ExecutorToManagerMsg::Response(
                            ExecutorToManagerResMsg::QuitGameResponse,
                        ))
                        .await?;
                    }
                },
                ManagerToExecutorMsg::Response(response_message) => match response_message {
                    ManagerToExecutorResMsg::ReadyToQuitGameResponse => {}
                },
            }
        }

        Ok(())
    }

    // fn get_rx(&mut self) -> Option<&mut mpsc::Receiver<ManagerToExecutorMsg>>;

    // fn get_tx(&self) -> Option<&mpsc::Sender<ExecutorToManagerMsg>>;

    fn get_channel(
        &mut self,
    ) -> (
        &mpsc::Sender<ExecutorToManagerMsg>,
        &mut mpsc::Receiver<ManagerToExecutorMsg>,
    );

    fn draw_opening_screen(&mut self);
}
