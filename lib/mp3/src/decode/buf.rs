use std::io;

// going off https://www.mars.org/pipermail/mad-dev/2002-January/000425.html,
// this should be enough to fit any possible frame
const SIZE: usize = 3000;

pub struct Buf {
    bytes: Vec<u8>,
    len: usize
}

impl Buf {
    pub fn new() -> Buf {
        Buf {
            bytes: uninit_buf(SIZE),
            len: 0
        }
    }

    pub fn fill(&mut self, mut reader: impl io::Read) -> io::Result<()> {
        let mut read = self.len;

        while read < SIZE {
            read += match reader.read(&mut self.bytes[read..])? {
                0 => break,
                n => n
            }
        }

        self.len = read;

        Ok(())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub fn consume(&mut self, amount: usize) {
        self.bytes.copy_within(amount.., 0);
        self.len -= amount
    }

    pub fn clear(&mut self) {
        self.len = 0
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

fn uninit_buf(len: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(len);
    unsafe { buf.set_len(len) }
    buf
}
