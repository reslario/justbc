use {
    super::{Result, ErrorKind, Error},
    windows_dll::dll,
    kernel32::GetModuleHandleA,
    std::{
        io,
        ptr,
    },
    winapi::{
        shared::{
            windef::HWND,
            minwindef::{BOOL, FALSE, HINSTANCE}
        },
        um::winuser::{
            WNDCLASSA,
            CW_USEDEFAULT,
            DefWindowProcA,
            DestroyWindow,
            RegisterClassA,
            CreateWindowExA,
            UnregisterClassA,
            WS_EX_NOACTIVATE
        }
    }
};

#[allow(non_snake_case)]
#[dll("./media_keys_hook.dll")]
extern "system" {
    fn set_shell_hook(window: HWND) -> BOOL;
    fn unset_shell_hook(window: HWND) -> BOOL;
}

pub struct Instance {
    instance: HINSTANCE,
    pub window: HWND,
}

impl Instance {
    pub fn new() -> Result<Instance> {
        unsafe {
            let instance = ensure_ne(
                GetModuleHandleA(ptr::null()).cast(),
                ptr::null_mut(),
                ErrorKind::GetModuleHandle
            )?;
            
            ensure_ne(
                RegisterClassA(&window_class(instance)),
                0,
                ErrorKind::RegisterClass
            )?;
    
            let window = ensure_ne(
                window(instance),
                ptr::null_mut(),
                ErrorKind::CreateWindow
            )?;
    
            ensure_ne(
                set_shell_hook(window),
                FALSE,
                ErrorKind::SetHook
            )?;
    
            Ok(Instance {
                instance,
                window
            })
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            unset_shell_hook(self.window);
            DestroyWindow(self.window);
            UnregisterClassA(CLASS_NAME, self.instance);
        }
    }
}

unsafe impl Sync for Instance {}
unsafe impl Send for Instance {}

const CLASS_NAME: *const i8 = b"justbc-media-keys-hook-window".as_ptr().cast();

fn window_class(instance: HINSTANCE) -> WNDCLASSA {
    WNDCLASSA {
        style: 0,
        lpfnWndProc: Some(DefWindowProcA),
        hInstance: instance,
        lpszClassName: CLASS_NAME,
        cbClsExtra: 0,
        cbWndExtra: 0,
        hIcon: ptr::null_mut(),
        hCursor: ptr::null_mut(),
        hbrBackground: ptr::null_mut(),
        lpszMenuName: ptr::null_mut(),
    }
}

fn window(instance: HINSTANCE) -> HWND {
    unsafe {
        CreateWindowExA(
            0,
            CLASS_NAME,
            ptr::null(),
            WS_EX_NOACTIVATE,
            CW_USEDEFAULT,
            CW_USEDEFAULT, 
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            ptr::null_mut(),
            ptr::null_mut(),
            instance,
            ptr::null_mut()
        )
    }
}

fn ensure_ne<T: PartialEq>(val: T, not: T, error_kind: ErrorKind) -> Result<T> {
    if val != not {
        Ok(val)
    } else {
        Err(Error {
            kind: error_kind,
            io_err: io::Error::last_os_error()
        })
    }
}
