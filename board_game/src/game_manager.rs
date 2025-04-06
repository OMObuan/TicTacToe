use std::sync::Arc;

use tokio::sync::{
    Mutex,
    mpsc::{self, error::SendError},
};
use tracing::trace;

use crate::{
    board::Board,
    consts::PLAYER_NUM,
    message::{
        ExecutorToManagerMsg, ManagerToExecutorMsg,
        request_message::{ExecutorToManagerReqMsg, ManagerToExecutorReqMsg},
        response_message::ExecutorToManagerResMsg,
    },
    player::Player,
};

#[derive(Debug, thiserror::Error)]
pub enum GameManagerError {
    #[error("Channel error")]
    ChannelError,
    #[error("Send error: {0:?}")]
    SendError(#[from] SendError<ManagerToExecutorMsg>),
    #[error("Message error")]
    MessageError,
}

pub struct GameManager {
    pub board: Board,
    pub players: [Player; PLAYER_NUM],
    pub rx: Option<Arc<Mutex<mpsc::Receiver<ExecutorToManagerMsg>>>>,
    pub tx: Option<Arc<Mutex<mpsc::Sender<ManagerToExecutorMsg>>>>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            players: [Player::new(), Player::new()],
            rx: None,
            tx: None,
        }
    }

    pub fn set_rx(mut self, rx: mpsc::Receiver<ExecutorToManagerMsg>) -> Self {
        self.rx = Some(Arc::new(Mutex::new(rx)));
        self
    }

    pub fn set_tx(mut self, tx: mpsc::Sender<ManagerToExecutorMsg>) -> Self {
        self.tx = Some(Arc::new(Mutex::new(tx)));
        self
    }

    pub fn get_rx(
        &self,
    ) -> Result<Arc<Mutex<mpsc::Receiver<ExecutorToManagerMsg>>>, GameManagerError> {
        self.rx.clone().ok_or(GameManagerError::ChannelError)
    }

    pub fn get_tx(
        &self,
    ) -> Result<Arc<Mutex<mpsc::Sender<ManagerToExecutorMsg>>>, GameManagerError> {
        self.tx.clone().ok_or(GameManagerError::ChannelError)
    }

    pub async fn start(&self) -> Result<(), GameManagerError> {
        let (tx, rx) = (self.get_tx()?, self.get_rx()?);
        tx.lock()
            .await
            .send(ManagerToExecutorMsg::Request(
                ManagerToExecutorReqMsg::InitGameRequest,
            ))
            .await?;
        while let Some(message) = rx.lock().await.recv().await {
            match message {
                ExecutorToManagerMsg::Request(request_message) => match request_message {
                    ExecutorToManagerReqMsg::ReadyToQuitGameRequest => {
                        self.ready_to_quit_game().await?;
                    }
                },
                ExecutorToManagerMsg::Response(response_message) => match response_message {
                    ExecutorToManagerResMsg::InitGameResponse => {
                        self.process_init_game_response().await?;
                    }
                    ExecutorToManagerResMsg::QuitGameResponse => {
                        trace!("quit game");
                        break;
                    }
                    ExecutorToManagerResMsg::ExecuteGameResponse => {
                        self.process_execute_game_response().await?;
                    }
                },
            }
        }
        Ok(())
    }

    pub async fn ready_to_quit_game(&self) -> Result<(), GameManagerError> {
        let tx = self.get_tx()?;
        trace!("ready to quit game");
        tx.lock()
            .await
            .send(ManagerToExecutorMsg::Request(
                ManagerToExecutorReqMsg::QuitGameRequest,
            ))
            .await?;
        Ok(())
    }

    pub async fn process_init_game_response(&self) -> Result<(), GameManagerError> {
        let tx = self.get_tx()?;
        trace!("execute the game");
        // sleep(Duration::from_secs(1)).await;
        tx.lock()
            .await
            .send(ManagerToExecutorMsg::Request(
                ManagerToExecutorReqMsg::ExecuteGameRequest,
            ))
            .await?;
        Ok(())
    }

    pub async fn process_execute_game_response(&self) -> Result<(), GameManagerError> {
        Ok(())
    }
}

impl Default for GameManager {
    fn default() -> Self {
        Self::new()
    }
}
