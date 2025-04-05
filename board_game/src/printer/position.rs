pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum PositionError {
    #[error("Invalid position: ({0}, {1})")]
    InvalidPosition(usize, usize),
}

impl Position {
    pub fn new(x: usize, y: usize) -> Result<Self, PositionError> {
        if x == 0 || y == 0 {
            return Err(PositionError::InvalidPosition(x, y));
        }
        Ok(Self { x, y })
    }
}
