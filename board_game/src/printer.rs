use clear::ClearType;
use color::{Color, ColorType};
use position::Position;

pub mod clear;
pub mod color;
pub mod position;

pub struct Printer {}

impl Printer {
    pub fn get_rgb_ansi(color_type: ColorType, color: Color) -> String {
        format!(
            "\x1b[{};2;{};{};{}m",
            Into::<u8>::into(color_type),
            color.red,
            color.green,
            color.blue
        )
    }

    pub fn reset_rgb_ansi() -> &'static str {
        "\x1b[39m\x1b[49m"
    }

    pub fn clear(clear_type: ClearType) -> &'static str {
        match clear_type {
            ClearType::BeforeCursor => "\x1b[1J]",
            ClearType::AfterCursor => "\x1b[0J]",
            ClearType::EntireScreen => "\x1b[2J]",
        }
    }

    pub fn move_cursor(position: Position) -> String {
        format!("\x1b[{};{}H", position.x, position.y)
    }
}
