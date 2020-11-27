use std::sync::mpsc;

pub enum Event {
    Input(input::Key),
    Response(fetch::Response)
}

pub struct Events {
    responses: mpsc::Receiver<fetch::Response>
}

impl Events {
    pub fn new(responses: mpsc::Receiver<fetch::Response>) -> Events {
        Events { responses }
    }

    pub fn iter(&self) -> impl Iterator<Item = Event> + '_ {
        input::keys()
            .map(Event::Input)
            .chain(self.responses())
    }

    fn responses(&self) -> impl Iterator<Item = Event> + '_ {
        self.responses
            .try_iter()
            .map(Event::Response)
    }
}
