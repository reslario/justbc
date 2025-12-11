use {
    super::{hook, Result},
    once_cell::sync::Lazy,
    std::sync::{RwLock, RwLockReadGuard},
    winapi::shared::windef::HWND,
};

type InstanceLock = Lazy<RwLock<Option<hook::Instance>>>;

static INSTANCE: InstanceLock = Lazy::new(<_>::default);

pub struct Ref {
    instance: RwLockReadGuard<'static, Option<hook::Instance>>,
}

impl Ref {
    pub fn new() -> Result<Ref> {
        // if we can get a write access, no references exist,
        // meaning that the instance is not initialised
        if let Ok(mut instance) = INSTANCE.try_write() {
            instance.replace(hook::Instance::new()?);
        }

        Ok(Ref {
            instance: INSTANCE.read().unwrap(),
        })
    }

    pub fn get_window(&self) -> HWND {
        self.instance
            .as_ref()
            .map(|init| init.window)
            .expect("not initialised")
    }
}

impl Drop for Ref {
    fn drop(&mut self) {
        static DUMMY: InstanceLock = Lazy::new(<_>::default);

        self.instance = DUMMY.read().unwrap();

        // if we can get write access, no references exist,
        // meaning we can deinitialise the instance
        if let Ok(mut instance) = INSTANCE.try_write() {
            instance
                .take()
                .map(drop)
                .expect("deinitialised without initialising first");
        }
    }
}
