pub struct None;
pub type Error = std::convert::Infallible;

impl None {
    pub fn new() -> Result<None, Error> {
        Ok(None)
    }

    pub fn keys(&self) -> impl Iterator<Item = crate::MediaKey> {
        std::iter::empty()
    }
}
