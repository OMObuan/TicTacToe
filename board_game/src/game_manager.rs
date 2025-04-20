use std::sync::Arc;

use tokio::sync::{
    Mutex,
    mpsc::{self, error::SendError},
};
use tracing::{info, trace};

use crate::{
    board::Board,
    consts::{HEIGHT, WIDTH},
    message::{
        ExecutorToManagerMsg, ManagerToExecutorMsg,
        request_message::{ExecutorToManagerReqMsg, ManagerToExecutorReqMsg},
        response_message::{ExecutorToManagerResMsg, ManagerToExecutorResMsg},
    },
    player::Player,
    tile::Tile,
};

#[derive(Debug, thiserror::Error)]
pub enum GameManagerError {
    #[error("Channel error")]
    ChannelError,
    #[error("Send error: {0:?}")]
    SendError(#[from] SendError<ManagerToExecutorMsg>),
    #[error("Message error")]
    MessageError,
    #[error("Join error: {0:?}")]
    JoinError(#[from] tokio::task::JoinError),
}

pub struct GameManager {
    pub board: Arc<Mutex<Board>>,
    pub rx: Option<Arc<Mutex<mpsc::Receiver<ExecutorToManagerMsg>>>>,
    pub tx: Option<Arc<Mutex<mpsc::Sender<ManagerToExecutorMsg>>>>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            board: Arc::new(Mutex::new(Board::new())),
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

    pub async fn start(self: Arc<Self>) -> Result<(), GameManagerError> {
        let (tx, rx) = (self.get_tx()?, self.get_rx()?);
        tx.lock()
            .await
            .send(ManagerToExecutorMsg::Request(
                ManagerToExecutorReqMsg::InitGameRequest,
            ))
            .await?;
        let mut tasks = vec![];
        while let Some(message) = rx.lock().await.recv().await {
            match message {
                ExecutorToManagerMsg::Request(request_message) => match request_message {
                    ExecutorToManagerReqMsg::ReadyToQuitGameRequest => {
                        let manager = self.clone();
                        let task = tokio::spawn(async move {
                            manager.ready_to_quit_game().await?;
                            Ok::<(), GameManagerError>(())
                        });
                        tasks.push(task);
                    }
                    ExecutorToManagerReqMsg::TileOnByPlayerRequesst(player, posx, posy) => {
                        let manager = self.clone();
                        let task = tokio::spawn(async move {
                            manager.tile_on_by_player(player, posx, posy).await?;
                            Ok::<(), GameManagerError>(())
                        });
                        tasks.push(task);
                    }
                },
                ExecutorToManagerMsg::Response(response_message) => match response_message {
                    ExecutorToManagerResMsg::InitGameResponse => {
                        let manager = self.clone();
                        let task = tokio::spawn(async move {
                            manager.process_init_game_response().await?;
                            Ok::<(), GameManagerError>(())
                        });
                        tasks.push(task);
                    }
                    ExecutorToManagerResMsg::QuitGameResponse => {
                        trace!("quit game");
                        for task in tasks {
                            task.abort();
                        }
                        break;
                    }
                    ExecutorToManagerResMsg::ExecuteGameResponse => {
                        let manager = self.clone();
                        let task = tokio::spawn(async move {
                            manager.process_execute_game_response().await?;
                            Ok::<(), GameManagerError>(())
                        });
                        tasks.push(task);
                    }
                    ExecutorToManagerResMsg::PlayerWinResponse => {
                        trace!("player win");
                    }
                },
            }
        }
        Ok(())
    }

    pub async fn ready_to_quit_game(&self) -> Result<(), GameManagerError> {
        let tx = self.get_tx()?;
        trace!("ready to quit game");
        self.ready_to_quit_game_impl().await?;
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

    async fn check_win(&self, player: &Player) -> bool {
        let board = self.board.lock().await;
        let player_tile = Tile {
            owner: Some(player.clone()),
        };

        // Check rows and columns
        for i in 0..HEIGHT {
            let mut row_win = true;
            let mut col_win = true;
            for j in 0..WIDTH {
                if board.board[i][j] != Some(player_tile.clone()) {
                    row_win = false;
                }
                if board.board[j][i] != Some(player_tile.clone()) {
                    col_win = false;
                }
            }
            if row_win || col_win {
                return true;
            }
        }

        // Check diagonals
        let mut diag1_win = true;
        let mut diag2_win = true;
        for i in 0..HEIGHT {
            if board.board[i][i] != Some(player_tile.clone()) {
                diag1_win = false;
            }
            if board.board[i][HEIGHT - 1 - i] != Some(player_tile.clone()) {
                diag2_win = false;
            }
        }

        diag1_win || diag2_win
    }

    async fn tile_on_by_player(
        &self,
        player: Player,
        posx: usize,
        posy: usize,
    ) -> Result<(), GameManagerError> {
        let tx = self.get_tx()?;
        trace!("get message tile on by player");
        let is_win = self.tile_on_by_player_impl(player, posx, posy).await?;
        tx.lock()
            .await
            .send(ManagerToExecutorMsg::Response(
                ManagerToExecutorResMsg::TileOnByPlayerResponse(is_win),
            ))
            .await?;
        Ok(())
    }

    async fn tile_on_by_player_impl(
        &self,
        player: Player,
        posx: usize,
        posy: usize,
    ) -> Result<bool, GameManagerError> {
        let tx = self.get_tx()?;
        self.board.lock().await.board[posx][posy] = Some(Tile {
            owner: Some(player.clone()),
        });
        info!("tile on by player: {:?}", self.board.lock().await.board);
        Ok(if self.check_win(&player).await {
            tx.lock()
                .await
                .send(ManagerToExecutorMsg::Request(
                    ManagerToExecutorReqMsg::PlayerWinRequest(player),
                ))
                .await?;
            true
        } else {
            false
        })
    }

    async fn ready_to_quit_game_impl(&self) -> Result<(), GameManagerError> {
        Ok(())
    }
}

impl Default for GameManager {
    fn default() -> Self {
        Self::new()
    }
}
