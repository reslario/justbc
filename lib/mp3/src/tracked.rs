use std::io::{self, Read, Seek};

pub struct Tracked<R> {
    reader: R,
    pos: u64
}

impl <R> Tracked<R> {
    pub fn new(reader: R) -> Tracked<R> {
        Tracked {
            reader,
            pos: 0
        }
    }

    pub fn pos(&self) -> u64 {
        self.pos
    }
}

impl <R: Read> Read for Tracked<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.reader.read(buf)?;
        self.pos += read as u64;

        Ok(read)
    }
}

impl <R: Seek> Seek for Tracked<R> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let pos = self.reader.seek(pos)?;
        self.pos = pos;

        Ok(pos)
    }
}
