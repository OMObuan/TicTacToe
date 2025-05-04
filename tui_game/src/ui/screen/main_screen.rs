use std::sync::Arc;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use tokio::sync::Mutex;

#[derive(Clone, Copy)]
pub struct MainScreen {
    pub menu_select: Option<CurrentSelectMenu>,
}

impl MainScreen {
    pub fn new() -> Self {
        Self { menu_select: None }
    }
}

impl Default for MainScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, TryFromPrimitive)]
#[repr(usize)]
pub enum CurrentSelectMenu {
    StartGame,
    QuitGame,
}

pub const SELECT_MENU_NUMS: usize = 2;
