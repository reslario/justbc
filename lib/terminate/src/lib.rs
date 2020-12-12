use std::sync::atomic::{AtomicBool, Ordering};

static TERMINATE: AtomicBool = AtomicBool::new(false);

pub type Error = ctrlc::Error;

pub fn install() -> Result<(), Error> {
    ctrlc::set_handler(|| TERMINATE.store(true, Ordering::SeqCst))
}

#[inline]
pub fn should() -> bool {
    TERMINATE.load(Ordering::SeqCst)
}
