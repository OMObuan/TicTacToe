use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{
    Mutex,
    mpsc::{self, error::SendError},
};

use crate::{
    message::{
        ExecutorToManagerMsg, ManagerToExecutorMsg,
        request_message::ManagerToExecutorReqMsg,
        response_message::{ExecutorToManagerResMsg, ManagerToExecutorResMsg},
    },
    player::Player,
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
    #[error("Join error: {0:?}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Not at game")]
    NotAtGame,
    #[error("Io error: {0:?}")]
    IoError(#[from] std::io::Error),
    #[error("Unknown error: {0:?}")]
    Unknown(#[from] anyhow::Error),
}

#[async_trait]
pub trait GameExecutor {
    async fn run(self: Arc<Self>) -> Result<(), GameExecutorError>
    where
        Self: 'static,
    {
        let rx = self.get_rx()?;
        let mut tasks = vec![];
        while let Some(message) = rx.lock().await.recv().await {
            match message {
                ManagerToExecutorMsg::Request(request_message) => match request_message {
                    ManagerToExecutorReqMsg::InitGameRequest => {
                        let executor = self.clone();
                        let task = tokio::spawn(async move {
                            executor.init_game().await?;
                            Ok::<(), GameExecutorError>(())
                        });
                        tasks.push(task);
                    }
                    ManagerToExecutorReqMsg::QuitGameRequest => {
                        self.quit_game().await?;
                        for task in tasks {
                            task.abort();
                        }
                        break;
                    }
                    ManagerToExecutorReqMsg::ExecuteGameRequest => {
                        let executor = self.clone();
                        let task = tokio::spawn(async move {
                            executor.execute_game().await?;
                            Ok::<(), GameExecutorError>(())
                        });
                        tasks.push(task);
                    }
                    ManagerToExecutorReqMsg::PlayerWinRequest(player) => {
                        let executor = self.clone();
                        let task = tokio::spawn(async move {
                            executor.player_win(&player).await?;
                            Ok::<(), GameExecutorError>(())
                        });
                        tasks.push(task);
                    }
                },
                ManagerToExecutorMsg::Response(response_message) => match response_message {
                    ManagerToExecutorResMsg::ReadyToQuitGameResponse => {}
                    ManagerToExecutorResMsg::TileOnByPlayerResponse(is_win) => {
                        let executor = self.clone();
                        let task = tokio::spawn(async move {
                            executor.process_tile_on_by_player_response(is_win).await?;
                            Ok::<(), GameExecutorError>(())
                        });
                        tasks.push(task);
                    }
                },
            }
        }

        Ok(())
    }

    fn get_tx(&self) -> Result<Arc<Mutex<mpsc::Sender<ExecutorToManagerMsg>>>, GameExecutorError>;

    fn get_rx(&self)
    -> Result<Arc<Mutex<mpsc::Receiver<ManagerToExecutorMsg>>>, GameExecutorError>;

    async fn init_game(&self) -> Result<(), GameExecutorError> {
        let tx = self.get_tx()?;
        self.init_game_impl().await?;
        tx.lock()
            .await
            .send(ExecutorToManagerMsg::Response(
                ExecutorToManagerResMsg::InitGameResponse,
            ))
            .await
            .unwrap();
        Ok(())
    }

    async fn init_game_impl(&self) -> Result<(), GameExecutorError>;

    async fn quit_game(&self) -> Result<(), GameExecutorError> {
        let tx = self.get_tx()?;
        self.quit_game_impl().await?;
        tx.lock()
            .await
            .send(ExecutorToManagerMsg::Response(
                ExecutorToManagerResMsg::QuitGameResponse,
            ))
            .await?;
        Ok(())
    }

    async fn quit_game_impl(&self) -> Result<(), GameExecutorError>;

    async fn execute_game(&self) -> Result<(), GameExecutorError> {
        let tx = self.get_tx()?;
        self.execute_game_impl().await?;
        tx.lock()
            .await
            .send(ExecutorToManagerMsg::Response(
                ExecutorToManagerResMsg::ExecuteGameResponse,
            ))
            .await?;
        Ok(())
    }

    async fn execute_game_impl(&self) -> Result<(), GameExecutorError>;

    async fn player_win(&self, player: &Player) -> Result<(), GameExecutorError> {
        let tx = self.get_tx()?;
        self.player_win_impl(player).await?;
        tx.lock()
            .await
            .send(ExecutorToManagerMsg::Response(
                ExecutorToManagerResMsg::PlayerWinResponse,
            ))
            .await?;
        Ok(())
    }

    async fn player_win_impl(&self, player: &Player) -> Result<(), GameExecutorError>;

    async fn process_tile_on_by_player_response(
        &self,
        is_win: bool,
    ) -> Result<(), GameExecutorError>;
}
