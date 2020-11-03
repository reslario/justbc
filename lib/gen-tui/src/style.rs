use tui::style::Color;

pub trait ColorExt: Sized + Copy {
    fn is_bright(self) -> bool;
    fn invert(self) -> Self;
}

impl ColorExt for Color {
    fn is_bright(self) -> bool {
        use Color::*;

        match self {
            Reset => true,
            Black => false,
            Red => false,
            Green => false,
            Yellow => false,
            Blue => false,
            Magenta => false,
            Cyan => false,
            Gray => false,
            DarkGray => false,
            LightRed => true,
            LightGreen => true,
            LightYellow => true,
            LightBlue => true,
            LightMagenta => true,
            LightCyan => true,
            White => true,
            Rgb(r, g, b) => is_bright(r, g, b),
            Indexed(_) => unimplemented!()
        }
    }
    
    fn invert(self) -> Self {
        use Color::*;

        match self {
            Reset => Black,
            Black => White,
            Red => LightCyan,
            Green => LightMagenta,
            Yellow => Blue,
            Blue => Yellow,
            Magenta => Green,
            Cyan => LightRed,
            Gray => DarkGray,
            DarkGray => Gray,
            LightRed => Cyan,
            LightGreen => LightMagenta,
            LightYellow => LightBlue,
            LightBlue => LightYellow,
            LightMagenta => LightGreen,
            LightCyan => LightRed,
            White => Black,
            Rgb(r, g, b) => Rgb(!r, !g, !b),
            Indexed(_) => unimplemented!()
        }
    }
}

fn is_bright(r: u8, g: u8, b: u8) -> bool {
    0.2126 * f32::from(r) 
        + 0.7152 * f32::from(g)
        + 0.0722 * f32::from(b)
        >= 128.
}
