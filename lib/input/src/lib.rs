pub mod binds;
pub mod keys;

pub use keys::Key;

use {
    crossterm::event,
    serde::{Deserialize, Serialize},
    std::time::Duration,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Command {
    FocusRelease,
    FocusNav,
    FocusSearchBar,
    TabLibrary,
    TabExplore,
    CycleTabs,
    SelectionUp,
    SelectionDown,
    ScrollUp,
    ScrollDown,
    Confirm,
    VolumeUp,
    VolumeDown,
    TogglePlay,
    NextTrack,
    PrevTrack,
    SkipAhead,
    SkipBack,
}

pub fn keys() -> impl Iterator<Item = Key> {
    // we need to pass a non-zero timeout duration for multiple keys
    // to be read
    std::iter::from_fn(|| event::poll(Duration::from_millis(1)).ok())
        .take_while(<_>::clone)
        .filter_map(|_| event::read().ok())
        .filter_map(|event| match event {
            event::Event::Key(event) => event.into(),
            _ => None,
        })
        .map(<_>::into)
}
