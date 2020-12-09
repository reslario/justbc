use std::{
    io,
    sync::mpsc
};

pub enum Event {
    Input(input::Key),
    Response(fetch::Response),
    DeviceUpdated,
}

pub struct Events {
    responses: mpsc::Receiver<fetch::Response>,
    device_watcher: device::Watcher
}

impl Events {
    pub fn new(responses: mpsc::Receiver<fetch::Response>) -> io::Result<Events> {
        Ok(Events {
            responses,
            device_watcher: device::Watcher::new()?
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = Event> + '_ {
        input::keys()
            .map(Event::Input)
            .chain(self.responses())
            .chain(self.device_update())
    }

    fn responses(&self) -> impl Iterator<Item = Event> + '_ {
        self.responses
            .try_iter()
            .map(Event::Response)
    }

    fn device_update(&self) -> impl Iterator<Item = Event> {
        event_if(self.device_watcher.device_updated(), Event::DeviceUpdated)
    }
}

fn event_if(cond: bool, event: Event) -> impl Iterator<Item = Event> {
    Some(event)
        .filter(|_| cond)
        .into_iter()
}
