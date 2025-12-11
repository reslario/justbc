use {
    snafu::{ResultExt, Snafu},
    std::{io, sync::mpsc},
};

pub enum Event {
    Input(input::Key),
    Response(fetch::Response),
    MediaKey(media_keys::MediaKey),
    DeviceUpdated,
    Terminate,
}

#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("error creating device watcher: {}", source))]
    Device { source: io::Error },
    #[snafu(display("error creating media key listener: {}", source))]
    MediaKeys { source: media_keys::Error },
    #[snafu(display("error setting termination handler: {}", source))]
    Terminate { source: terminate::Error },
}

pub struct Events {
    responses: mpsc::Receiver<fetch::Response>,
    device_watcher: device::Watcher,
    media_key_listener: media_keys::Listener,
}

impl Events {
    pub fn new(responses: mpsc::Receiver<fetch::Response>) -> Result<Events, Error> {
        terminate::install().context(Terminate)?;

        Ok(Events {
            responses,
            device_watcher: device::Watcher::new().context(Device)?,
            media_key_listener: media_keys::Listener::new().context(MediaKeys)?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = Event> + '_ {
        input::keys()
            .map(Event::Input)
            .chain(self.responses())
            .chain(self.media_keys())
            .chain(self.device_update())
            .chain(self.should_terminate())
    }

    fn responses(&self) -> impl Iterator<Item = Event> + '_ {
        self.responses.try_iter().map(Event::Response)
    }

    fn media_keys(&self) -> impl Iterator<Item = Event> + '_ {
        self.media_key_listener.keys().map(Event::MediaKey)
    }

    fn device_update(&self) -> impl Iterator<Item = Event> {
        event_if(self.device_watcher.device_updated(), Event::DeviceUpdated)
    }

    fn should_terminate(&self) -> impl Iterator<Item = Event> {
        event_if(terminate::should(), Event::Terminate)
    }
}

fn event_if(cond: bool, event: Event) -> impl Iterator<Item = Event> {
    Some(event).filter(|_| cond).into_iter()
}
