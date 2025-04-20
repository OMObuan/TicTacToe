use std::{
    fmt::Debug,
    str::{FromStr, SplitWhitespace},
    sync::Arc,
};

use async_trait::async_trait;
use board_game::{
    game_executor::{GameExecutor, GameExecutorError},
    message::{
        ExecutorToManagerMsg, ManagerToExecutorMsg, request_message::ExecutorToManagerReqMsg,
    },
    player::Player,
};

use tokio::sync::{Mutex, Notify, mpsc};
use tracing::{info, trace};

pub struct TuiGameExecutor {
    tx: Option<Arc<Mutex<mpsc::Sender<ExecutorToManagerMsg>>>>,
    rx: Option<Arc<Mutex<mpsc::Receiver<ManagerToExecutorMsg>>>>,
    is_win: Arc<Mutex<bool>>,
    is_win_notify: Arc<Notify>,
    is_tile_on_notify: Arc<Notify>,
}

impl TuiGameExecutor {
    pub fn new() -> Self {
        Self {
            tx: None,
            rx: None,
            is_win: Arc::new(Mutex::new(false)),
            is_win_notify: Arc::new(Notify::new()),
            is_tile_on_notify: Arc::new(Notify::new()),
        }
    }

    pub fn set_rx(mut self, rx: mpsc::Receiver<ManagerToExecutorMsg>) -> Self {
        self.rx = Some(Arc::new(Mutex::new(rx)));
        self
    }

    pub fn set_tx(mut self, tx: mpsc::Sender<ExecutorToManagerMsg>) -> Self {
        self.tx = Some(Arc::new(Mutex::new(tx)));
        self
    }
}

impl Default for TuiGameExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GameExecutor for TuiGameExecutor {
    fn get_tx(&self) -> Result<Arc<Mutex<mpsc::Sender<ExecutorToManagerMsg>>>, GameExecutorError> {
        Ok(self.tx.clone().unwrap())
    }

    fn get_rx(
        &self,
    ) -> Result<Arc<Mutex<mpsc::Receiver<ManagerToExecutorMsg>>>, GameExecutorError> {
        Ok(self.rx.clone().unwrap())
    }

    async fn init_game_impl(&self) -> Result<(), GameExecutorError> {
        trace!("init game");
        Ok(())
    }

    async fn quit_game_impl(&self) -> Result<(), GameExecutorError> {
        trace!("execute quiting game");
        Ok(())
    }

    async fn execute_game_impl(&self) -> Result<(), GameExecutorError> {
        let tx = self.get_tx()?;
        trace!("start executing game");
        let mut reader = Reader::new();
        let mut player = [Player::new(), Player::new()];
        player[0].id = 0;
        player[1].id = 1;
        let mut turns = 0;
        loop {
            trace!("turn {} begins", turns + 1);
            let posx = reader.read_int::<usize>();
            let posy = reader.read_int::<usize>();
            trace!("start send tile's position: {posx}, {posy}");
            tx.lock()
                .await
                .send(ExecutorToManagerMsg::Request(
                    ExecutorToManagerReqMsg::TileOnByPlayerRequesst(
                        player[turns % 2].clone(),
                        posx,
                        posy,
                    ),
                ))
                .await?;
            self.is_tile_on_notify.notified().await;
            if *self.is_win.lock().await {
                break;
            }
            trace!("finish send");
            turns += 1;
            // dbg!(turns, posx, posy);
        }
        trace!("finalize executing game");
        tx.lock()
            .await
            .send(ExecutorToManagerMsg::Request(
                ExecutorToManagerReqMsg::ReadyToQuitGameRequest,
            ))
            .await?;
        Ok(())
    }

    async fn player_win_impl(&self, player: &Player) -> Result<(), GameExecutorError> {
        self.is_win_notify.notify_one();
        info!("player {} Win", player.id);
        Ok(())
    }

    async fn process_tile_on_by_player_response(
        &self,
        is_win: bool,
    ) -> Result<(), GameExecutorError> {
        trace!("process tile on by player response");
        *self.is_win.lock().await = is_win;
        self.is_tile_on_notify.notify_one();
        Ok(())
    }
}
struct Reader<'a> {
    buf: *mut String,
    nums: Option<SplitWhitespace<'a>>,
}

impl Reader<'_> {
    fn new() -> Self {
        Self {
            buf: std::ptr::null_mut(),
            nums: None,
        }
    }

    fn get_line(&mut self) {
        let mut new_line = Box::new(String::new());
        std::io::stdin().read_line(&mut new_line).unwrap();
        if !self.buf.is_null() {
            let _ = unsafe { Box::from_raw(self.buf) };
        }
        self.buf = Box::into_raw(new_line);
        self.nums = unsafe { Some((*self.buf).split_whitespace()) };
    }

    fn read_token(&mut self) -> &str {
        if self.nums.is_none() {
            self.get_line();
        }

        loop {
            match self.nums.as_mut().unwrap().next() {
                Some(val) => {
                    break val;
                }
                None => {
                    self.get_line();
                }
            }
        }
    }

    fn read_int<T: FromStr>(&mut self) -> T
    where
        <T as FromStr>::Err: Debug,
    {
        self.read_token().parse().unwrap()
    }
}

impl Drop for Reader<'_> {
    fn drop(&mut self) {
        if !self.buf.is_null() {
            let _ = unsafe { Box::from_raw(self.buf) };
        }
    }
}

unsafe impl Send for Reader<'_> {}
