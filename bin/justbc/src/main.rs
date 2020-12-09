mod state;
mod play;
mod ui;

use {
    event::*,
    fetch::Fetcher,
    bandcamp_api::Api,
    reqwest::blocking::Client,
    tui::{
        Terminal,
        backend::CrosstermBackend
    },
    std::{
        io,
        thread,
        error::Error,
        time::Duration
    }
};

fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let (api, receiver) = Fetcher::new(Api::with_client(client));

    let mut state = state::State::new(<_>::default(), api);

    let events = Events::new(receiver)?;

    loop {
        for event in events.iter() {
            match event {
                Event::Input(evt) => state.input(evt),
                Event::Response(resp) => state.set_response(resp),
                Event::DeviceUpdated => state.update_device(),
            }
        }

        state.update();

        terminal.draw(|frame| ui::draw(frame, &mut state))?;

        thread::sleep(Duration::from_millis(16))
    }
}
