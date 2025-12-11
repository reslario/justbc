mod fans;
pub mod nav;
mod outlets;
pub mod releases;
mod search;
mod symbols;
pub mod tracks;

fn fmt_release(artist: &str, title: &str) -> String {
    release_fmt(artist, title).to_string()
}

fn release_fmt<'a>(artist: &'a str, title: &'a str) -> impl std::fmt::Display + 'a {
    use std::fmt;

    struct Fmt<'a> {
        artist: &'a str,
        title: &'a str,
    }

    impl<'a> fmt::Display for Fmt<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{} — {}", self.artist, self.title)
        }
    }

    Fmt { artist, title }
}
