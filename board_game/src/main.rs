use board_game::{Color, ColorType, Printer};

fn main() {
    println!(
        "{}Hello, world!{}",
        Printer::get_rgb_ansi(ColorType::ForeGround, Color::new_from_hex(0x66FF33)),
        Printer::reset_rgb_ansi()
    );
    println!("hahaha");
}
