/// Trait for matching byte patterns.
pub trait BytePat {
    fn matches(&self, bytes: &[u8]) -> bool;
}

impl BytePat for &[u8] {
    fn matches(&self, bytes: &[u8]) -> bool {
        *self == bytes
    }
}

impl BytePat for &str {
    fn matches(&self, bytes: &[u8]) -> bool {
        self.as_bytes() == bytes
    }
}

impl <F> BytePat for F
where F: Fn(&[u8]) -> bool {
    fn matches(&self, bytes: &[u8]) -> bool {
        (self)(bytes)
    }
}

pub fn contains(split: u8, needle: impl BytePat) -> impl BytePat {
    move |bytes: &[u8]| bytes
        .split(|b| *b == split)
        .any(|bytes| needle.matches(bytes))
}