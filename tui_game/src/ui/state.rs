use super::screen::{game_on_screen::GameOnScreen, main_screen::MainScreen};

#[derive(Clone, Copy)]
pub enum CurrentScreen {
    Main(MainScreen),
    GameOn(GameOnScreen),
}
