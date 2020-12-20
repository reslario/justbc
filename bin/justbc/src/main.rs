mod state;
mod play;
mod ui;
mod cfg;
mod term;

use {
    event::*,
    cfg::Config,
    state::State,
    term::Terminal,
    fetch::Fetcher,
    bandcamp_api::Api,
    input::binds::Bindings,
    directories::ProjectDirs,
    std::{
        thread,
        error::Error,
        time::Duration
    }
};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

fn main() {
    if let Err(e) = run() {
        eprint!("{}", e)
    }
}

fn run() -> Result {
    let dirs = ProjectDirs::from("io.github", "reslario", "justbc")
        .ok_or("couldn't get program directories")?;

    let cfg_file = dirs.config_dir().join("config.toml");

    let mut config = Config::load(&cfg_file)?;

    let bindings = config
        .bindings
        .map(Bindings::patched)
        .unwrap_or_default();

    let mut terminal = term::terminal()?;

    terminal.clear()?;

    let (api, receiver) = Fetcher::new(Api::new());

    let mut state = State::new(&bindings, api);

    state.core.player.set_volume(config.general.volume);

    let events = Events::new(receiver)?;

    main_loop(events, &mut state, &mut terminal, &config.gfx)?;

    terminal.clear()?;

    config.general.volume = state.core.player.volume();
    config.bindings = bindings.into();

    config.save(cfg_file)
}

fn main_loop(
    events: Events,
    state: &mut State,
    terminal: &mut Terminal,
    gfx: &cfg::Graphics
) -> Result {
    let refresh = Duration::from_millis(gfx.refresh);

    'main: loop {
        for event in events.iter() {
            match event {
                Event::Input(evt) => state.input(evt),
                Event::Response(resp) => state.set_response(resp),
                Event::MediaKey(key) => state.handle_media_key(key),
                Event::DeviceUpdated => state.update_device(),
                Event::Terminate => break 'main Ok(())
            }
        }

        state.update();

        terminal.draw(|frame| ui::draw(frame, state, gfx))?;

        thread::sleep(refresh)
    }
}
