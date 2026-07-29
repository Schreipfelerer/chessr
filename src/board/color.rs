use crate::board::Color;

impl Color {
    pub const ALL: [Color; 2] = [Color::White, Color::Black];
    pub fn flip(self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

impl Into<Color> for bool {
    fn into(self) -> Color {
        match self {
            false => Color::White,
            true => Color::Black,
        }
    }
}