use {
    std::collections::HashMap,
    serde::{Serialize, Deserialize},
    crate::{
        Command,
        keys::Key,
    }
};

#[derive(Serialize, Deserialize)]
pub struct Bindings {
    #[serde(flatten)]
    map: HashMap<Key, Command>
}

impl Bindings {
    pub fn get(&self, key: Key) -> Option<Command> {
        self.map.get(&key).copied()
    }
}

const DEFAULT_BINDS: &[(Key, Command)] = {
    use {
        Command::*,
        crossterm::event::{
            KeyCode::*,
            KeyModifiers as M
        }
    };

    &[
        (
            Key { code: Char('r'), modifiers: M::NONE },
            FocusRelease
        ),
        (
            Key { code: Char('n'), modifiers: M::NONE },
            FocusNav
        ),
        (
            Key { code: Char('s'), modifiers: M::NONE },
            FocusSearchBar
        ),
        (
            Key { code: Char('l'), modifiers: M::NONE },
            TabLibrary
        ),
        (
            Key { code: Char('e'), modifiers: M::NONE },
            TabExplore
        ),
        (
            Key { code: Tab, modifiers: M::NONE },
            CycleTabs
        ),
        (
            Key { code: Up, modifiers: M::NONE },
            SelectionUp
        ),
        (
            Key { code: Down, modifiers: M::NONE },
            SelectionDown
        ),
        (
            Key { code: Up, modifiers: M::SHIFT },
            ScrollUp
        ),
        (
            Key { code: Down, modifiers: M::SHIFT },
            ScrollDown
        ),
        (
            Key { code: Left, modifiers: M::ALT },
            PageLeft
        ),
        (
            Key { code: Right, modifiers: M::ALT },
            PageRight
        ),
        (
            Key { code: Enter, modifiers: M::NONE },
            Confirm
        ),
        (
            Key { code: Char('+'), modifiers: M::NONE },
            VolumeUp
        ),
        (
            Key { code: Char('-'), modifiers: M::NONE },
            VolumeDown
        ),
        (
            Key { code: Char(' '), modifiers: M::NONE },
            TogglePlay
        ),
        (
            Key { code: Right, modifiers: M::CONTROL },
            NextTrack
        ),
        (
            Key { code: Left, modifiers: M::CONTROL },
            PrevTrack
        )
    ]
};

impl Default for Bindings {
    fn default() -> Self {
        let map = DEFAULT_BINDS
            .iter()
            .copied()
            .collect();

        Bindings { map }
    }
}
