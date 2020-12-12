//! # How this works
//! 
//! To receive media key events on Windows while the app isn't in focus,
//! we create a .dll (the code for which is in /lib/media-keys-hook)
//! that installs a shell hook, intercepting the system's messages and redirecting
//! the relevant ones to a dummy window we created, so we can process them.
//! If that wasn't convoluted enough, this .dll also needs to utilise shared memory.
//!
//! This seems to be more or less the intended way to solve this problem, which is
//! honestly terrifying.

mod rc;
mod hook;

use {
    crate::MediaKey,
    std::{
        io,
        fmt,
        mem::MaybeUninit,
    },
    winapi::{
        shared::minwindef::TRUE,
        um::winuser::{
            MSG,
            PM_REMOVE,
            PeekMessageA,
            WM_APPCOMMAND,
            GET_APPCOMMAND_LPARAM,
            APPCOMMAND_MEDIA_STOP,
            APPCOMMAND_MEDIA_NEXTTRACK,
            APPCOMMAND_MEDIA_PLAY_PAUSE,
            APPCOMMAND_MEDIA_PREVIOUSTRACK,
        }
    }
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ErrorKind {
    GetModuleHandle,
    RegisterClass,
    CreateWindow,
    SetHook,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    io_err: io::Error
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kind = match self.kind {
            ErrorKind::GetModuleHandle => "get module handle",
            ErrorKind::RegisterClass => "register message window class",
            ErrorKind::CreateWindow => "create message window",
            ErrorKind::SetHook => "set media key hook"
        };

        write!(f, "failed to {}: {}", kind, self.io_err)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.io_err)
    }
}

type Result<T = ()> = std::result::Result<T, Error>;

pub struct Listener {
    reference: rc::Ref
}

impl Listener {
    pub fn new() -> Result<Listener> {
        rc::Ref::new()
            .map(|reference| Listener { reference })
    }

    pub fn keys(&self) -> impl Iterator<Item = MediaKey> + '_ {
        std::iter::from_fn(move || self.get_message())
            .filter_map(key_from_msg)
    }

    fn get_message(&self) -> Option<MSG> {
        let mut msg = MaybeUninit::uninit();

        unsafe {
            if PeekMessageA(msg.as_mut_ptr(), self.reference.get_window(), 0, 0, PM_REMOVE) == TRUE {
                Some(msg.assume_init())
            } else {
                None
            }
        }
    }
}

fn key_from_msg(msg: MSG) -> Option<MediaKey> {
    if msg.message != WM_APPCOMMAND {
        return None
    }
 
    match GET_APPCOMMAND_LPARAM(msg.lParam) {
        APPCOMMAND_MEDIA_STOP => MediaKey::Stop,
        APPCOMMAND_MEDIA_NEXTTRACK => MediaKey::NextTrack,
        APPCOMMAND_MEDIA_PLAY_PAUSE => MediaKey::PlayPause,
        APPCOMMAND_MEDIA_PREVIOUSTRACK => MediaKey::PrevTrack,
        _ => return None
    }.into()
}
