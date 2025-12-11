use {
    reqwest::{
        blocking::{Client, Response},
        header::{self, HeaderMap},
        Url,
    },
    std::io::{self, Read, Seek},
};

/// Allows reading from a Bandcamp track stream and seeking within it.
pub struct TrackStream {
    url: Url,
    client: Client,
    pos: u64,
    length: Option<u64>,
    seek: Option<u64>,
    response: Response,
}

impl TrackStream {
    /// Creates a new `TrackStream` by fetching a response from the provided URL
    /// using the provided client.
    pub fn new(url: Url, client: Client) -> reqwest::Result<TrackStream> {
        client.get(url.clone()).send().map(|response| TrackStream {
            url,
            client,
            pos: 0,
            length: content_length(response.headers()),
            seek: None,
            response,
        })
    }

    fn maybe_seek(&mut self) -> io::Result<()> {
        if let Some(pos) = self.seek.take() {
            self.response = self
                .client
                .get(self.url.clone())
                .header(header::RANGE, format!("bytes={}-", pos))
                .send()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            self.pos = pos;
        }

        Ok(())
    }
}

fn content_length(headers: &HeaderMap) -> Option<u64> {
    headers
        .get(header::CONTENT_LENGTH)?
        .to_str()
        .ok()?
        .parse()
        .ok()
}

impl Read for TrackStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.maybe_seek()?;
        self.response.read(buf)
    }
}

const LEN_UNKNOWN: &str = "cannot seek from end as stream length is unknown";

/// Seeking performs no I/O, only subsequent calls to
/// [read](std::io::Read::read) will.
impl Seek for TrackStream {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let pos = match pos {
            io::SeekFrom::Start(pos) => pos,
            io::SeekFrom::Current(offs) => offset(self.pos, offs),
            io::SeekFrom::End(offs) => self
                .length
                .map(|len| offset(len, offs))
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, LEN_UNKNOWN))?,
        };

        let pos = self.seek.unwrap_or_default() + pos;

        self.seek = if self.pos == pos { None } else { Some(pos) };

        Ok(pos)
    }
}

fn offset(pos: u64, offs: i64) -> u64 {
    if offs < 0 {
        pos - -offs as u64
    } else {
        pos + offs as u64
    }
}
