pub struct None;

impl crate::EventSource for None {
    fn new() -> std::io::Result<Self> {
        Ok(None)
    }

    fn device_updated(&self) -> bool {
        false
    }
}
