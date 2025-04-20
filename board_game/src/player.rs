#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub id: usize,
}

impl Player {
    pub fn new() -> Self {
        Player { id: 0 }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
