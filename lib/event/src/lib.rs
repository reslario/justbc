use {
    std::{
        io,
        sync::mpsc,
    },
    snafu::{
        Snafu,
        ResultExt
    }
};

pub enum Event {
    Input(input::Key),
    Response(fetch::Response),
    DeviceUpdated,
    Terminate
}

#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("error creating device watcher: {}", source))]
    Device { source: io::Error },
    #[snafu(display("error setting termination handler: {}", source))]
    Terminate { source: terminate::Error }
}

pub struct Events {
    responses: mpsc::Receiver<fetch::Response>,
    device_watcher: device::Watcher,
}

impl Events {
    pub fn new(responses: mpsc::Receiver<fetch::Response>) -> Result<Events, Error> {
        terminate::install().context(Terminate)?;
        
        Ok(Events {
            responses,
            device_watcher: device::Watcher::new().context(Device)?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = Event> + '_ {
        input::keys()
            .map(Event::Input)
            .chain(self.responses())
            .chain(self.device_update())
            .chain(self.should_terminate())
    }

    fn responses(&self) -> impl Iterator<Item = Event> + '_ {
        self.responses
            .try_iter()
            .map(Event::Response)
    }

    fn device_update(&self) -> impl Iterator<Item = Event> {
        event_if(self.device_watcher.device_updated(), Event::DeviceUpdated)
    }

    fn should_terminate(&self) -> impl Iterator<Item = Event> {
        event_if(terminate::should(), Event::Terminate)
    }
}

fn event_if(cond: bool, event: Event) -> impl Iterator<Item = Event> {
    Some(event)
        .filter(|_| cond)
        .into_iter()
}
