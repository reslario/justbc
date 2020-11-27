mod thread;
mod buf;

use {
    thread::*,
    buf::StreamBuf,
    std::{
        sync,
        io::{self, Read}
    },
};

/// The size of an [AudioStream](AudioStream)'s internal buffer.
/// Reading more than this amount at once is probably a bad idea.
pub const BUF_SIZE: usize = StreamBuf::SIZE;

/// This is the size of the buffer [rodio's Decoder]
/// passes to read calls, but other sizes work as well.
///
/// [rodio's Decoder]: https://docs.rs/rodio/0.12.0/rodio/decoder/struct.Decoder.html
const CHUNK: usize = 11520;

const REFILL: usize = BUF_SIZE / 2 - CHUNK;

type Synced<R> = sync::Arc<sync::RwLock<R>>;

/// Since audio playback is very delicate, fetching new data
/// from an audio source should never block, otherwise stuttering
/// may occur.
///
/// This type uses a background thread to fetch data from an
/// underlying reader, thus performing less (potentially blocking)
/// work on each read, which helps avoid that problem.
pub struct AudioStream<R>
where R: Read + Send + Sync + 'static {
    fetch_thread: FetchThread<R>,
    reader: Synced<R>,
    buf: StreamBuf,
    done: bool
}

impl <R> AudioStream<R>
where R: Read + Send + Sync + 'static {
    /// Creates a new `AudioStream` and fetches some data
    /// on its background thread.
    pub fn new(reader: R) -> io::Result<Self> {
        let mut stream = AudioStream {
            fetch_thread: FetchThread::new()?,
            reader: sync(reader),
            buf: StreamBuf::new(),
            done: false
        };

        stream.pre_fetch(REFILL);

        Ok(stream)
    }

    fn pre_fetch(&mut self, bytes: usize) {
        self.fetch_thread
            .fetch(self.reader.clone(), bytes)
    }

    fn maybe_append_pre_fetched(&mut self) -> io::Result<()> {
        if let Some(res) = self.fetch_thread.get_fetched() {
            self.buf.append(&res?.bytes)
        }

        Ok(())
    }

    fn append_pre_fetched(&mut self) -> Option<io::Result<usize>> {
        self.fetch_thread
            .await_fetched()
            .map(|res| {
                let bytes = res?.bytes;
                let len = bytes.len();
                self.buf.append(&bytes);
                Ok(len)
            })
    }

    fn fetch(&mut self, bytes: usize) -> io::Result<()> {
        let bytes = match self.append_pre_fetched().transpose()? {
            Some(n) => match n {
                0 => {
                    self.done = true;
                    return Ok(())
                },
                n if n >= bytes => return Ok(()),
                n => bytes - n,
            },
            None => bytes
        };

        if 0 == self.buf.read_from(&mut *lock(&self.reader), bytes)? {
            self.done = true
        }

        Ok(())
    }
}

fn sync<R>(reader: R) -> Synced<R> {
    sync::RwLock::new(reader).into()
}

fn lock<R>(reader: &Synced<R>) -> sync::RwLockWriteGuard<R> {
    reader.write().unwrap()
}

impl <R> Read for AudioStream<R>
where R: Read + Send + Sync + 'static {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.done && self.buf.exhausted() {
            return Ok(0)
        }

        self.maybe_append_pre_fetched()?;

        let ready = self.buf.ready();

        if ready < buf.len() {
            self.fetch(buf.len() - ready)?;
            self.pre_fetch(REFILL)
        } else if BUF_SIZE - ready <= CHUNK {
            self.pre_fetch(CHUNK)
        }

        Ok(self.buf.fill(buf))
    }
}

/// Implementing `Seek` is required by [rodio's Decoder],
/// since it tries to guess the audio format, which requires seeking to
/// the start again after each failed attempt.
/// However, since our use case only requires the mp3 format (and disables the others),
/// no seeking is performed in practice.
///
/// So for now, this simply returns an error.
///
/// [rodio's Decoder]: https://docs.rs/rodio/0.12.0/rodio/decoder/struct.Decoder.html
impl <R> io::Seek for AudioStream<R>
where R: Read + Send + Sync + 'static {
    fn seek(&mut self, _: io::SeekFrom) -> io::Result<u64> {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "seeking is not supported"
        ))
    }
}

fn try_fill_buf(mut reader: impl Read, buf: &mut [u8]) -> io::Result<usize> {
    let mut read = 0;
    while read < buf.len() {
        read += match reader.read(&mut buf[read..])? {
            0 => break,
            n => n
        }
    }

    Ok(read)
}

#[cfg(test)]
mod test {
    use {
        super::*,
        std::io::{self, Read}
    };

    const NUM_BYTES: usize = 1_000_000;

    fn bytes() -> Vec<u8> {
        (u8::MIN..u8::MAX)
            .cycle()
            .take(NUM_BYTES)
            .collect()
    }

    #[test]
    fn output_matches() {
        let stream = AudioStream::new(io::Cursor::new(bytes())).unwrap();
    
        let mut iter = stream
            .bytes()
            .map(Result::unwrap)
            .zip(bytes());

        assert!(
            iter.all(|(l, r)| l == r),
            "byte {} didn't match", NUM_BYTES - iter.count()
        )
    }

    #[test]
    fn beeg_reads() {
        let b = bytes();
        let mut streamed = vec![0; b.len()];
        let mut stream = AudioStream::new(io::Cursor::new(b)).unwrap();

        let mut read = 0;

        loop {
            let end = streamed[read..].len().min(BUF_SIZE - CHUNK);
            read += match stream.read(&mut streamed[read..][..end]).unwrap() {
                0 => break,
                n => n
            }
        }

        compare_chunks(&bytes(), &streamed)
    }

    fn compare_chunks(a: &[u8], b: &[u8]) {
        // compare in small chunks to avoid flooding the terminal
        let size = 100;
        for (i, (bytes, streamed)) in a
            .chunks(size)
            .zip(b.chunks(size))
            .enumerate()
        {
            assert_eq!(bytes, streamed, "chunk {} (pos {}) didn't match", i, i * size)
        }
    }

    #[test]
    fn latency() {
        struct ShoddyConnection<R> {
            reader: R,
            state: f32
        }
    
        impl <R: Read> Read for ShoddyConnection<R> {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                self.state += 1.;
    
                let ms = self
                    .state
                    .sin()
                    .abs()
                    * 300.;
    
                std::thread::sleep(std::time::Duration::from_millis(ms as _));
                
                self.reader.read(buf)
            }
        }

        let mut stream = AudioStream::new(
            ShoddyConnection {
                reader: std::io::Cursor::new(bytes()),
                state: 0.
            }
        ).unwrap();

        let mut buf = vec![0; NUM_BYTES];
        let mut read = 0;

        loop {
            let end = buf[read..].len().min(CHUNK);
            read += match stream.read(&mut buf[read..][..end]).unwrap() {
                0 => break,
                n => n
            }
        }

        compare_chunks(&bytes(), &buf)
    }
}
