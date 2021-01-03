mod buf;
mod tracked;

use {
    buf::Buf,
    tracked::Tracked,
    minimp3_sys as minimp3,
    crate::{
        Frame,
        samples::{Sample, Samples}
    },
    std::{
        mem::MaybeUninit,
        io::{self, Read, Seek}
    },
};

const MAX_FRAME_SAMPLES: usize = minimp3::MINIMP3_MAX_SAMPLES_PER_FRAME as _;

impl Frame {
    fn new(samples: Vec<Sample>, info: minimp3::mp3dec_frame_info_t, pos: u64) -> Frame {
        Frame {
            samples: Samples::new(samples),
            channels: info.channels as _,
            sample_rate: info.hz as _,
            pos
        }
    }
}

pub struct Decoder<R> {
    reader: Tracked<R>,
    decoder: Box<minimp3::mp3dec_t>,
    buf: Buf
}

impl <R: Read> Decoder<R> {
    pub fn new(reader: R) -> Decoder<R> {
        let mut decoder = MaybeUninit::uninit();

        let decoder = unsafe {
            minimp3::mp3dec_init(decoder.as_mut_ptr());
            decoder.assume_init().into()
        };

        Decoder {
            reader: Tracked::new(reader),
            decoder,
            buf: Buf::new()
        }
    }

    pub fn next_frame(&mut self) -> io::Result<Frame> {
        self.buf.fill(&mut self.reader)?;

        let (samples, frame_info) = self.decode_frame();

        self.buf.consume(frame_info.frame_bytes as _);
        let pos = self.pos() + frame_info.frame_offset as u64;

        Ok(Frame::new(samples, frame_info, pos))
    }

    fn pos(&self) -> u64 {
        self.reader.pos() - self.buf.len() as u64
    }

    fn decode_frame(&mut self) -> (Vec<Sample>, minimp3::mp3dec_frame_info_t) {
        let mut frame_info = MaybeUninit::uninit();
        let mut samples = Vec::with_capacity(MAX_FRAME_SAMPLES);
        let data = self.buf.as_slice();

        unsafe {
            let num = minimp3::mp3dec_decode_frame(
                self.decoder.as_mut(),
                data.as_ptr(),
                data.len() as _,
                samples.as_mut_ptr(),
                frame_info.as_mut_ptr()
            ) as usize;
    
            let frame_info = frame_info.assume_init();
    
            samples.set_len(num * frame_info.channels as usize);
    
            (samples, frame_info)
        }
    }
}

impl <R: Seek> Seek for Decoder<R> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.buf.clear();
        self.reader.seek(pos)
    }
}
