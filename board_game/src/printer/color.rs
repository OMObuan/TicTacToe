pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub enum ColorType {
    ForeGround,
    BackGround,
}

#[derive(Debug, thiserror::Error)]
pub enum ColorError {
    #[error("Invalid hex string: {0:?}")]
    HexErr(#[from] std::num::ParseIntError),
}

impl From<ColorType> for u8 {
    fn from(color_type: ColorType) -> Self {
        match color_type {
            ColorType::ForeGround => 38,
            ColorType::BackGround => 48,
        }
    }
}
impl Color {
    pub fn new_from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Color { red, green, blue }
    }
    pub fn new_from_hex_str(hex: &str) -> Result<Self, ColorError> {
        let hex = hex.trim_start_matches('#');
        let red = u8::from_str_radix(&hex[0..2], 16)?;
        let green = u8::from_str_radix(&hex[2..4], 16)?;
        let blue = u8::from_str_radix(&hex[4..6], 16)?;
        Ok(Color { red, green, blue })
    }

    pub fn new_from_hex(hex: u32) -> Self {
        let red = ((hex >> 16) & 0xFF) as u8;
        let green = ((hex >> 8) & 0xFF) as u8;
        let blue = (hex & 0xFF) as u8;
        // dbg!(red, green, blue);
        Color { red, green, blue }
    }
}
