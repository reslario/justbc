use rodio::{Sink, PlayError, OutputStream, OutputStreamHandle, StreamError};

pub struct Handle {
    // Never actually used, but needs to be kept around
    // so the device doesn't get dropped.
    _stream: OutputStream,
    handle: OutputStreamHandle
}

impl Handle {
    pub fn new() -> Result<Handle, StreamError> {
        OutputStream::try_default()
            .map(|(_stream, handle)| Handle {
                _stream,
                handle
            })
    }

    pub fn new_sink(&self) -> Result<Sink, PlayError> {
        Sink::try_new(&self.handle)
    }
}
