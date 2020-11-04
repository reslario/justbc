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

pub trait StyleExt: Sized + Copy {
    fn bold(self) -> Self;
    fn dim(self) -> Self;
    fn italic(self) -> Self;
    fn underlined(self) -> Self;
    fn slow_blink(self) -> Self;
    fn rapid_blink(self) -> Self;
    fn reversed(self) -> Self;
    fn hidden(self) -> Self;
    fn crossed_out(self) -> Self;
}

macro_rules! add_mod {
    ($to:ident, $name:ident) => {
        $to.add_modifier(tui::style::Modifier::$name)
    };
}

impl StyleExt for tui::style::Style {
    fn bold(self) -> Self {
        add_mod!(self, BOLD)
    }

    fn dim(self) -> Self {
        add_mod!(self, DIM)
    }

    fn italic(self) -> Self {
        add_mod!(self, ITALIC)
    }

    fn underlined(self) -> Self {
        add_mod!(self, UNDERLINED)
    }

    fn slow_blink(self) -> Self {
        add_mod!(self, SLOW_BLINK)
    }

    fn rapid_blink(self) -> Self {
        add_mod!(self, RAPID_BLINK)
    }

    fn reversed(self) -> Self {
        add_mod!(self, REVERSED)
    }

    fn hidden(self) -> Self {
        add_mod!(self, HIDDEN)
    }

    fn crossed_out(self) -> Self {
        add_mod!(self, CROSSED_OUT)
    }
}
