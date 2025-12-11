#![cfg(windows)]

use {
    std::{
        ptr,
        sync::atomic::{AtomicPtr, Ordering::SeqCst},
    },
    winapi::{
        ctypes::{c_int, c_short},
        shared::{
            minwindef::{
                BOOL,
                FALSE,
                HINSTANCE__ as HINSTANCE,
                LPARAM,
                LPVOID,
                LRESULT,
                TRUE,
                ULONG,
                WPARAM,
            },
            ntdef::HANDLE,
            windef::{HHOOK__ as HHOOK, HWND as HWNDPTR, HWND__ as HWND},
        },
        um::{
            winnt::DLL_PROCESS_ATTACH,
            winuser::{
                CallNextHookEx,
                PostMessageA,
                SetWindowsHookExA,
                UnhookWindowsHookEx,
                APPCOMMAND_MEDIA_NEXTTRACK,
                APPCOMMAND_MEDIA_PLAY_PAUSE,
                APPCOMMAND_MEDIA_PREVIOUSTRACK,
                APPCOMMAND_MEDIA_STOP,
                GET_APPCOMMAND_LPARAM,
                HSHELL_APPCOMMAND,
                WH_SHELL,
                WM_APPCOMMAND,
            },
        },
    },
};

const fn atomic_null<T>() -> AtomicPtr<T> {
    AtomicPtr::new(ptr::null_mut())
}

#[no_mangle]
#[link_section = ".shared"]
pub static NOTIFY_WINDOW: AtomicPtr<HWND> = atomic_null();

#[no_mangle]
#[link_section = ".shared"]
pub static SHELL_HOOK: AtomicPtr<HHOOK> = atomic_null();

static INSTANCE: AtomicPtr<HINSTANCE> = atomic_null();

type MediaCommand = c_short;

const MEDIA_COMMANDS: &[MediaCommand] = &[
    APPCOMMAND_MEDIA_PLAY_PAUSE,
    APPCOMMAND_MEDIA_NEXTTRACK,
    APPCOMMAND_MEDIA_PREVIOUSTRACK,
    APPCOMMAND_MEDIA_STOP,
];

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(instance: HANDLE, reason: ULONG, _: LPVOID) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        INSTANCE.store(instance.cast(), SeqCst)
    }

    TRUE
}

#[no_mangle]
pub extern "system" fn set_shell_hook(window: HWNDPTR) -> BOOL {
    let notify_window = NOTIFY_WINDOW.load(SeqCst);

    if window.is_null() || !notify_window.is_null() {
        return FALSE
    }

    let hook = unsafe {
        SetWindowsHookExA(
            // hook in before messages reach any app
            WH_SHELL,
            Some(handle_message),
            INSTANCE.load(SeqCst),
            // hook into all apps
            0,
        )
    };

    if !hook.is_null() {
        SHELL_HOOK.store(hook, SeqCst);
        NOTIFY_WINDOW.store(window, SeqCst);
        TRUE
    } else {
        FALSE
    }
}

#[no_mangle]
pub extern "system" fn unset_shell_hook(window: HWNDPTR) -> BOOL {
    let notify_window = NOTIFY_WINDOW.load(SeqCst);

    // are we being called by the correct application?
    if window != notify_window || notify_window.is_null() {
        return FALSE
    }

    let hook = SHELL_HOOK.swap(ptr::null_mut(), SeqCst);

    if unsafe { UnhookWindowsHookEx(hook) } == TRUE {
        NOTIFY_WINDOW.store(ptr::null_mut(), SeqCst);
        TRUE
    } else {
        FALSE
    }
}

extern "system" fn handle_message(code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let notify_window = NOTIFY_WINDOW.load(SeqCst);

    if code == HSHELL_APPCOMMAND
        && !notify_window.is_null()
        && MEDIA_COMMANDS.contains(&GET_APPCOMMAND_LPARAM(l_param))
    {
        return unsafe { PostMessageA(notify_window, WM_APPCOMMAND, w_param, l_param) as _ }
    }

    unsafe { CallNextHookEx(ptr::null_mut(), code, w_param, l_param) }
}
