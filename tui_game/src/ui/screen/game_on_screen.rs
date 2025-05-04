#[derive(Clone, Copy)]
pub struct GameOnScreen {}

impl GameOnScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for GameOnScreen {
    fn default() -> Self {
        Self::new()
    }
}
