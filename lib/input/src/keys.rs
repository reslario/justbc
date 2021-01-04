use {
    std::fmt,
    serde::Serialize,
    crossterm::event::KeyEvent
};

pub use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers
}

impl Key {
    pub fn ctrl(&self) -> bool {
        self.modifiers.intersects(KeyModifiers::CONTROL)
    }

    pub fn alt(&self) -> bool {
        self.modifiers.intersects(KeyModifiers::ALT)
    }

    pub fn shift(&self) -> bool {
        self.modifiers.intersects(KeyModifiers::SHIFT)
    }
}

impl From<KeyEvent> for Key {
    fn from(event: KeyEvent) -> Self {
        Key {
            code: event.code,
            modifiers: event.modifiers
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use KeyCode::*;

        FmtModifiers(self.modifiers).fmt(f)?;

        match self.code {
            Char(c) => return c.fmt(f),
            F(n) => return write!(f, "f{}", n),
            Backspace => string::BACKSPACE,
            Enter => string::ENTER,
            Left => string::LEFT,
            Right => string::RIGHT,
            Up => string::UP,
            Down => string::DOWN,
            Home => string::HOME,
            End => string::END,
            PageUp => string::PAGE_UP,
            PageDown => string::PAGE_DOWN,
            Tab => string::TAB,
            BackTab => string::BACK_TAB,
            Delete => string::DOWN,
            Insert => string::INSERT,
            Null => string::NULL,
            Esc => string::ESC,
        }.fmt(f)
    }
}

struct FmtModifiers(KeyModifiers);

impl fmt::Display for FmtModifiers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let FmtModifiers(mdf) = self;

        let mut write = |s| write!(f, "{}{}", s, string::SEP);

        if mdf.intersects(KeyModifiers::CONTROL) {
            write(string::CTRL)?
        } 
        
        if mdf.intersects(KeyModifiers::ALT) {
            write(string::ALT)?
        }
    
        if mdf.intersects(KeyModifiers::SHIFT) {
            write(string::SHIFT)?
        }

        Ok(())
    }
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        self.to_string().serialize(serializer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    InvalidModifier,
    MissingKey(KeyModifiers),
    InvalidF(std::num::ParseIntError)
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InvalidModifier => f.write_str("invalid modifier"),
            ParseError::MissingKey(mdf) => write!(f, "missing key after '{}'", FmtModifiers(*mdf)),
            ParseError::InvalidF(e) => write!(f, "invalid f key: {}", e)
        }
    }
}

impl std::str::FromStr for Key {
    type Err = ParseError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let mut modifiers = KeyModifiers::NONE;

        while let Some(idx) = s.find(string::SEP) {
            let (current, rem) = s.split_at(idx);

            if let Some(mdf) = add_modifier(current)? {
                modifiers |= mdf
            } else {
                return Ok(Key {
                    code: KeyCode::Char(sep_char()),
                    modifiers
                })
            };

            s = skip_char(rem)
        }

        parse_key(s, modifiers)
    }
}

fn add_modifier(s: &str) -> Result<Option<KeyModifiers>, ParseError> {
    Ok(match s {
        string::CTRL => KeyModifiers::CONTROL,
        string::ALT => KeyModifiers::ALT,
        string::SHIFT => KeyModifiers::SHIFT,
        "" => return Ok(None),
        _ => return Err(ParseError::InvalidModifier)
    }.into())
}

fn skip_char(s: &str) -> &str {
    let mut chars = s.chars();
    chars.next();
    chars.as_str()
}

fn parse_key(s: &str, modifiers: KeyModifiers) -> Result<Key, ParseError> {
    use KeyCode::*;

    let code = match s {
        string::BACKSPACE => Backspace,
        string::ENTER => Enter,
        string::LEFT => Left,
        string::RIGHT => Right,
        string::UP => Up,
        string::DOWN => Down,
        string::HOME => Home,
        string::END => End,
        string::PAGE_UP => PageUp,
        string::PAGE_DOWN => PageDown,
        string::TAB => Tab,
        string::BACK_TAB => BackTab,
        string::DELETE => Delete,
        string::INSERT => Insert,
        string::NULL => Null,
        string::ESC => Esc,
        s => {
            let mut chars = s.chars();

            match chars
                .next()
                .ok_or(ParseError::MissingKey(modifiers))?
            {
                'f' if !chars.as_str().is_empty() => chars
                    .as_str()
                    .parse()
                    .map(F)
                    .map_err(ParseError::InvalidF)?,
                c => Char(c),
            }
        }
    };

    Ok(Key {
        code,
        modifiers
    })
}

fn sep_char() -> char {
    string::SEP.parse().unwrap()
}

impl <'de> serde::Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        <&str>::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

mod string {
    pub const SEP: &str = "+";

    pub const CTRL: &str = "ctrl";
    pub const ALT: &str = "alt";
    pub const SHIFT: &str = "shift";

    pub const BACKSPACE: &str = "backspace"; 
    pub const ENTER: &str = "enter"; 
    pub const LEFT: &str = "left"; 
    pub const RIGHT: &str = "right"; 
    pub const UP: &str = "up"; 
    pub const DOWN: &str = "down"; 
    pub const HOME: &str = "home"; 
    pub const END: &str = "end"; 
    pub const PAGE_UP: &str = "pageup"; 
    pub const PAGE_DOWN: &str = "pagedown"; 
    pub const TAB: &str = "tab"; 
    pub const BACK_TAB: &str = "backtab"; 
    pub const DELETE: &str = "delete"; 
    pub const INSERT: &str = "insert"; 
    pub const NULL: &str = "null"; 
    pub const ESC: &str = "esc"; 
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn roundtrip() {
        let key = Key { code: KeyCode::F(1), modifiers: KeyModifiers::all() };

        assert_eq!(key, key.to_string().parse().unwrap())
    }

    #[test]
    fn all_modifiers_char() {
        let key = "ctrl+alt+shift+a".parse().unwrap();

        assert_eq!(Key { code: KeyCode::Char('a'), modifiers: KeyModifiers::all() }, key)
    }

    #[test]
    fn no_modifiers() {
        let key = "backspace".parse().unwrap();

        assert_eq!(Key { code: KeyCode::Backspace, modifiers: KeyModifiers::NONE }, key)
    }

    #[test]
    fn weird_order() {
        let key = "alt+shift+ctrl+up".parse().unwrap();

        assert_eq!(Key { code: KeyCode::Up, modifiers: KeyModifiers::all() }, key)
    }

    #[test]
    fn plus() {
        let key = "ctrl++".parse().unwrap();

        assert_eq!(Key { code: KeyCode::Char('+'), modifiers: KeyModifiers::CONTROL }, key)
    }

    #[test]
    fn invalid_modifier() {
        let key = "no+down".parse::<Key>();

        assert_eq!(Err(ParseError::InvalidModifier), key)
    }

    #[test]
    fn f_key() {
        let key = "alt+f12".parse().unwrap();

        assert_eq!(Key { code: KeyCode::F(12), modifiers: KeyModifiers::ALT }, key)
    }

    #[test]
    fn invalid_f() {
        let key = "alt+ftp".parse::<Key>();

        assert!(matches!(key, Err(ParseError::InvalidF(_))))
    }

    #[test]
    fn missing_key() {
        let key = "shift+".parse::<Key>();

        assert_eq!(Err(ParseError::MissingKey(KeyModifiers::SHIFT)), key)
    }
}
