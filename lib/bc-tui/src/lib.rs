pub mod tracks;
pub mod releases;
pub mod nav;
mod search;
mod outlets;
mod symbols;

fn fmt_release(artist: &str, title: &str) -> String {
    format!("{} — {}", artist, title)
}
