mod state;
mod play;
mod ui;

use {
    event::*,
    fetch::Fetcher,
    bandcamp_api::Api,
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

fn main() {
    if let Err(e) = run() {
        eprint!("{}", e)
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let (api, receiver) = Fetcher::new(Api::new());

    let mut state = state::State::new(<_>::default(), api);

    let events = Events::new(receiver)?;

    'main: loop {
        for event in events.iter() {
            match event {
                Event::Input(evt) => state.input(evt),
                Event::Response(resp) => state.set_response(resp),
                Event::MediaKey(key) => state.handle_media_key(key),
                Event::DeviceUpdated => state.update_device(),
                Event::Terminate => break 'main
            }
        }

        state.update();

        terminal.draw(|frame| ui::draw(frame, &mut state))?;

        thread::sleep(Duration::from_millis(16))
    }

    terminal.clear()?;
    
    Ok(())
}
