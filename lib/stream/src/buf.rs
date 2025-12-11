use std::{io, ops::Range};

/// Tracks the start, end and position of a
/// [StreamBuffer](StreamBuffer) within a stream.
struct StreamCursor {
    range: Range<usize>,
    pos: usize,
}

impl Default for StreamCursor {
    fn default() -> Self {
        StreamCursor {
            range: 0..0,
            pos: 0,
        }
    }
}

impl StreamCursor {
    fn start(&self) -> usize {
        self.range.start
    }

    fn end(&self) -> usize {
        self.range.end
    }

    fn shift(&mut self, by: usize) {
        self.range.start += by;
        self.range.end += by;

        self.pos = self.pos.max(self.start())
    }

    fn extend(&mut self, by: usize) {
        self.range.end += by;
    }

    fn ready(&self) -> usize {
        self.end().checked_sub(self.pos).unwrap_or_default()
    }

    fn consumed(&self) -> usize {
        self.pos - self.start()
    }

    fn done(&self) -> bool {
        self.pos >= self.end()
    }
}

/// Internal buffer used in an [AudioStream](crate::AudioStream).
pub struct StreamBuf {
    buf: Vec<u8>,
    cursor: StreamCursor,
}

impl StreamBuf {
    pub const SIZE: usize = super::CHUNK * 20;

    pub fn new() -> StreamBuf {
        StreamBuf {
            buf: Vec::with_capacity(Self::SIZE),
            cursor: <_>::default(),
        }
    }

    fn space(&self) -> usize {
        Self::SIZE - self.len()
    }

    fn move_left(&mut self, by: usize) {
        self.buf.copy_within(by.., 0);
        self.cursor.shift(by);
    }

    fn advance(&mut self, by: usize) -> &mut [u8] {
        self.move_left(by);
        self.slice_from_end(by)
    }

    fn advance_extend(&mut self, by: usize) -> &mut [u8] {
        let space = self.space();

        if by <= space {
            self.lengthen(by);
        } else {
            let rem = by - space;
            self.move_left(rem);
            self.lengthen(space);
        }

        self.slice_from_end(by)
    }

    fn lengthen(&mut self, by: usize) {
        self.buf.resize(self.len() + by, 0);
        self.cursor.extend(by);
    }

    fn slice_from_end(&mut self, num: usize) -> &mut [u8] {
        let start = self.len() - num;
        &mut self.buf[start..]
    }

    pub fn append(&mut self, bytes: &[u8]) {
        let space = self.space();

        if bytes.len() <= space {
            self.extend(bytes)
        } else {
            let split = bytes.len().min(space);
            let (a, b) = bytes.split_at(split);
            self.extend(a);
            let rem = b.len();
            self.advance(rem).copy_from_slice(b)
        }
    }

    fn extend(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
        self.cursor.extend(bytes.len());
    }

    pub fn read_from(&mut self, reader: impl io::Read, bytes: usize) -> io::Result<usize> {
        let new = self.advance_extend(bytes);
        super::try_fill_buf(reader, new)
    }

    pub fn fill(&mut self, buf: &mut [u8]) -> usize {
        let start = self.cursor.consumed();
        let num = self.cursor.ready().min(buf.len());

        buf[..num].copy_from_slice(&self[start..][..num]);
        self.cursor.pos += num;
        num
    }

    pub fn ready(&self) -> usize {
        self.cursor.ready()
    }

    pub fn exhausted(&self) -> bool {
        self.cursor.done()
    }

    pub fn seek(&mut self, pos: usize) -> bool {
        if self.cursor.range.contains(&pos) {
            self.cursor.pos = pos;
            true
        } else {
            false
        }
    }

    pub fn pos(&self) -> usize {
        self.cursor.pos
    }

    pub fn restart_from(&mut self, pos: usize) {
        self.buf.clear();
        self.cursor = <_>::default();
        self.cursor.shift(pos)
    }
}

impl std::ops::Deref for StreamBuf {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}
