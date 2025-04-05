#[derive(Copy, Clone)]
pub struct Player {}

impl Player {
    pub fn new() -> Self {
        Player {}
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
