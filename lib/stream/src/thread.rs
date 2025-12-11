use {
    super::Synced,
    std::{
        io::{self, Read},
        sync::mpsc,
        thread,
    },
};

type FetchResult = io::Result<FetchResponse>;

/// Communicates with a background thread in order
/// to fetch new data from an audio source in a
/// non-blocking fashion.
pub struct FetchThread<R>
where
    R: Read + Send + Sync + 'static,
{
    receiver: mpsc::Receiver<FetchResult>,
    sender: mpsc::Sender<Message<R>>,
    pending: bool,
}

enum Message<R> {
    Fetch(Synced<R>, usize),
    End,
}

impl<R> FetchThread<R>
where
    R: Read + Send + Sync + 'static,
{
    pub fn new() -> io::Result<FetchThread<R>> {
        let (res_sender, receiver) = mpsc::sync_channel(0);
        let (sender, msg_receiver) = mpsc::channel();
        thread::Builder::new()
            .name("audio stream fetching".into())
            .stack_size(64_000)
            .spawn(fetch_thread(res_sender, msg_receiver))?;

        Ok(FetchThread {
            receiver,
            sender,
            pending: false,
        })
    }

    fn recv<F, E>(&mut self, f: F) -> Option<FetchResult>
    where
        F: Fn(&mpsc::Receiver<FetchResult>) -> Result<FetchResult, E>,
    {
        if self.pending {
            let result = f(&self.receiver).ok();
            self.pending = result.is_none();
            result
        } else {
            None
        }
    }

    pub fn get_fetched(&mut self) -> Option<FetchResult> {
        self.recv(mpsc::Receiver::try_recv)
    }

    pub fn await_fetched(&mut self) -> Option<FetchResult> {
        self.recv(mpsc::Receiver::recv)
    }

    pub fn fetch(&mut self, response: Synced<R>, bytes: usize) {
        if !self.pending {
            self.pending = true;
            self.send(Message::Fetch(response, bytes))
        }
    }

    fn send(&self, msg: Message<R>) {
        self.sender.send(msg).unwrap();
    }
}

impl<R> Drop for FetchThread<R>
where
    R: Read + Send + Sync + 'static,
{
    fn drop(&mut self) {
        self.send(Message::End)
    }
}

pub struct FetchResponse {
    pub bytes: Vec<u8>,
}

fn fetch_thread<R>(
    sender: mpsc::SyncSender<FetchResult>,
    receiver: mpsc::Receiver<Message<R>>,
) -> impl FnOnce()
where
    R: Read + 'static,
{
    move || loop {
        let res = match receiver.recv().unwrap() {
            Message::Fetch(resp, bytes) => fetch(resp, bytes),
            Message::End => return,
        };

        sender.send(res).ok();
    }
}

fn fetch(reader: Synced<impl Read>, bytes: usize) -> FetchResult {
    let mut buf = vec![0; bytes];
    let mut reader = super::lock(&reader);

    let read = super::try_fill_buf(&mut *reader, &mut buf)?;

    buf.truncate(read);

    Ok(FetchResponse { bytes: buf })
}
