use {
    std::{
        io,
        ptr,
        sync::atomic::{Ordering, AtomicBool},
    },
    winapi::{
        Class,
        Interface,
        ctypes::c_void,
        shared::{
            winerror::S_OK,
            guiddef::REFIID,
            wtypes::PROPERTYKEY,
            minwindef::{DWORD, ULONG}
        },
        um::{
            combaseapi,
            winnt::{LPCWSTR, HRESULT},
            unknwnbase::{IUnknown, IUnknownVtbl},
            mmdeviceapi::{MMDeviceEnumerator, IMMDeviceEnumerator, IMMNotificationClient, IMMNotificationClientVtbl, EDataFlow, ERole}
        }
    }
};

static UPDATED: AtomicBool = AtomicBool::new(false);

const CLIENT_VTABLE: IMMNotificationClientVtbl = IMMNotificationClientVtbl {
    parent: IUnknownVtbl {
        QueryInterface,
        AddRef,
        Release,
    },
    OnDeviceStateChanged,
    OnDeviceAdded,
    OnDeviceRemoved,
    OnDefaultDeviceChanged,
    OnPropertyValueChanged,
};

#[allow(non_snake_case)]
extern "system" fn OnDefaultDeviceChanged(
    _: *mut IMMNotificationClient,
    _: EDataFlow,
    _: ERole,
    _: LPCWSTR
) -> HRESULT {
    UPDATED.store(true, Ordering::SeqCst);
    S_OK
}

macro_rules! no_op {
    ($name:ident ( $($arg:ty),* )) => {
        no_op!($name ( $($arg),* ) => HRESULT);
    };
    ($name:ident ( $($arg:ty),* ) => $ret:ty) => {
        #[allow(non_snake_case)]
        extern "system" fn $name($(_: $arg),*) -> $ret { S_OK as _ }
    };
}

no_op!(QueryInterface(*mut IUnknown, REFIID, *mut *mut c_void));
no_op!(AddRef(*mut IUnknown) => ULONG);
no_op!(Release(*mut IUnknown) => ULONG);

no_op!(OnDeviceStateChanged(*mut IMMNotificationClient, LPCWSTR, DWORD));
no_op!(OnDeviceAdded(*mut IMMNotificationClient, LPCWSTR));
no_op!(OnDeviceRemoved(*mut IMMNotificationClient, LPCWSTR));
no_op!(OnPropertyValueChanged(*mut IMMNotificationClient, LPCWSTR, PROPERTYKEY));

trait HResExt {
    fn to_res(self) -> io::Result<()>;
}

impl HResExt for HRESULT {
    fn to_res(self) -> io::Result<()> {
        if self < 0 {
            Err(io::Error::from_raw_os_error(self))
        } else {
            Ok(())
        }
    }
}

// the values are boxed to ensure they don't move
pub struct Watcher {
    client: Box<IMMNotificationClient>,
    enumerator: Box<IMMDeviceEnumerator>
}

impl crate::EventSource for Watcher {
    fn new() -> io::Result<Watcher> {
        init_com()?;

        let enumerator = enumerator()?;

        let client = Box::new(IMMNotificationClient {
            lpVtbl: &CLIENT_VTABLE,
        });

        unsafe {
            enumerator
                .RegisterEndpointNotificationCallback(client.as_ref() as *const _ as *mut _)
                .to_res()?
        }

        Ok(Watcher { enumerator, client })
    }

    fn device_updated(&self) -> bool {
        UPDATED.swap(false, Ordering::SeqCst)
    }
}

fn init_com() -> io::Result<()> {
    unsafe { 
        combaseapi::CoInitializeEx(
            ptr::null_mut(),
            combaseapi::COINITBASE_MULTITHREADED
        ).to_res()
    } 
}

fn enumerator() -> io::Result<Box<IMMDeviceEnumerator>> {
    let mut enumerator = Box::into_raw(Box::new(IMMDeviceEnumerator {
        lpVtbl: ptr::null() 
    }));

    unsafe { 
        combaseapi::CoCreateInstance(
            &MMDeviceEnumerator::uuidof(),
            ptr::null_mut(),
            combaseapi::CLSCTX_ALL,
            &IMMDeviceEnumerator::uuidof(),
            &mut enumerator as *mut *mut IMMDeviceEnumerator as *mut _,
        ).to_res()?;

        Ok(Box::from_raw(enumerator))
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        unsafe {
            self.enumerator.Release();
            self.client.Release();

            // There seem to be COMplications with this crate's
            // and the `cpal` crate's use of the COM API, most likely
            // due to `cpal` not calling `CoUninitialize` (looking at the
            // source code, it is actually present in the `Drop` impl of 
            // a thread-local value, but maybe `drop` never gets called?).
            //
            // This causes an access violation when the program exits, but
            // luckily just calling `CoUninitialize` a couple of times here
            // seems to fix it.
            for _ in 0..5 {
                combaseapi::CoUninitialize()
            }
        }
    }
}
