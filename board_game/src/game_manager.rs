use std::time::Duration;

use tokio::{
    sync::mpsc::{self, error::SendError},
    time::sleep,
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
    pub rx: Option<mpsc::Receiver<ExecutorToManagerMsg>>,
    pub tx: Option<mpsc::Sender<ManagerToExecutorMsg>>,
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
        self.rx = Some(rx);
        self
    }

    pub fn set_tx(mut self, tx: mpsc::Sender<ManagerToExecutorMsg>) -> Self {
        self.tx = Some(tx);
        self
    }

    pub async fn start(&mut self) -> Result<(), GameManagerError> {
        let tx = self.tx.as_ref().ok_or(GameManagerError::ChannelError)?;
        let rx = self.rx.as_mut().ok_or(GameManagerError::ChannelError)?;
        tx.send(ManagerToExecutorMsg::Request(
            ManagerToExecutorReqMsg::InitGameRequest,
        ))
        .await?;
        while let Some(message) = rx.recv().await {
            match message {
                ExecutorToManagerMsg::Request(request_message) => match request_message {
                    ExecutorToManagerReqMsg::ReadyToQuitGameRequest => {
                        trace!("ready to quit game");
                        tx.send(ManagerToExecutorMsg::Request(
                            ManagerToExecutorReqMsg::QuitGameRequest,
                        ))
                        .await?;
                    }
                },
                ExecutorToManagerMsg::Response(response_message) => match response_message {
                    ExecutorToManagerResMsg::StartGameResponse => {
                        trace!("execute the game");
                        sleep(Duration::from_secs(1)).await;
                    }
                    ExecutorToManagerResMsg::QuitGameResponse => {
                        trace!("quit game");
                        break;
                    }
                },
            }
        }
        Ok(())
    }
}

impl Default for GameManager {
    fn default() -> Self {
        Self::new()
    }
}
