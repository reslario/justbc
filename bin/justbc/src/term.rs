use {
    std::io::{self, Stdout},
    tui::backend::CrosstermBackend,
};

pub type Terminal = tui::Terminal<CrosstermBackend<Stdout>>;

pub fn terminal() -> io::Result<Terminal> {
    tui::Terminal::new(CrosstermBackend::new(io::stdout()))
}
