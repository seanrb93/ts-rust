//! Music streamer: `.MSC` files are raw SPU-ADPCM stereo, interleaved in 0x8000-byte chunks (left, right, ...); a mod can give an Ogg Opus
//! file (`.opus`) instead.  A background thread reads and decodes; `mix` resamples the decoded frames into the output.

use crate::adpcm::{decode_block, State};
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, TryRecvError};
use std::sync::Arc;

const CHUNK: usize = 0x8000;
const SRC_RATE: f32 = 44100.0;
/// an Opus file's rate
const OPUS_RATE: f32 = 48000.0;

/// Commands of the original stream driver (as issued by the game's `stream_RPC`).
pub enum MusicCmd {
    /// Open a file (host path); does not start playing.
    Open(PathBuf),
    Close,
    /// Start or resume playback.
    Start,
    /// Pause playback.
    Stop,
    /// Loop at end of file.
    SetLoop(bool),
    /// Left / right volume, 0..=0x3fff.
    Volume(u32, u32),
    /// Query: returns 0x5000 while playing, else 0.
    Status,
}

pub struct Music {
    rx: Option<Receiver<Vec<f32>>>,
    looping: Arc<AtomicBool>,
    playing: bool,
    ended: bool,
    chunk: Vec<f32>,
    idx: usize,
    a: [f32; 2],
    b: [f32; 2],
    frac: f32,
    vol_l: f32,
    vol_r: f32,
    /// the rate of the frames being played (44100 for a .MSC, 48000 for Opus)
    src_rate: f32,
}

/// The thread of an Opus file: its blocks as they are decoded, the file again from the start when the music loops.
fn opus_decoder(path: PathBuf, looping: Arc<AtomicBool>, tx: std::sync::mpsc::SyncSender<Vec<f32>>) {
    loop {
        let mut r = match crate::codec::OpusReader::open(&path) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[audio] {e}");
                return;
            }
        };
        let mut any = false;
        while let Some(block) = r.next_block() {
            any = true;
            // (the blocks are sent in pieces of about a tenth of a second)
            for piece in block.chunks(9600) {
                if tx.send(piece.to_vec()).is_err() {
                    return;
                }
            }
        }
        if !looping.load(Ordering::Relaxed) || !any {
            return;
        }
    }
}

/// The thread of a file in another format (read by `AnyReader`): its blocks, and the file again from the start when the music loops.
fn any_decoder(path: PathBuf, first: crate::anyfmt::AnyReader, looping: Arc<AtomicBool>, tx: std::sync::mpsc::SyncSender<Vec<f32>>) {
    let mut reader = first;
    loop {
        let mut any = false;
        while let Some(block) = reader.next_block() {
            any = true;
            for piece in block.chunks(9600) {
                if tx.send(piece.to_vec()).is_err() {
                    return;
                }
            }
        }
        if !looping.load(Ordering::Relaxed) || !any {
            return;
        }
        reader = match crate::anyfmt::AnyReader::open(&path) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[audio] {e}");
                return;
            }
        };
    }
}

fn decoder(path: PathBuf, looping: Arc<AtomicBool>, tx: std::sync::mpsc::SyncSender<Vec<f32>>) {
    let Ok(mut f) = std::fs::File::open(&path) else { return };
    let mut sl = State::default();
    let mut sr = State::default();
    let mut buf = vec![0u8; CHUNK * 2];
    loop {
        let mut n = 0;
        while n < buf.len() {
            match f.read(&mut buf[n..]) {
                Ok(0) | Err(_) => break,
                Ok(k) => n += k,
            }
        }
        if n < 32 {
            if looping.load(Ordering::Relaxed) {
                use std::io::Seek;
                if f.seek(std::io::SeekFrom::Start(0)).is_err() {
                    return;
                }
                sl = State::default();
                sr = State::default();
                continue;
            }
            return;
        }
        let half = (n / 2) & !15;
        let blocks = half / 16;
        let mut out = Vec::with_capacity(blocks * 56);
        let mut l = [0i16; 28];
        let mut r = [0i16; 28];
        for b in 0..blocks {
            decode_block(&buf[b * 16..b * 16 + 16], &mut sl, &mut l);
            decode_block(&buf[CHUNK + b * 16..CHUNK + b * 16 + 16], &mut sr, &mut r);
            for k in 0..28 {
                out.push(l[k] as f32 / 32768.0);
                out.push(r[k] as f32 / 32768.0);
            }
        }
        if tx.send(out).is_err() {
            return;
        }
        if n < buf.len() {
            // short final chunk: loop or finish on the next iteration
            continue;
        }
    }
}

impl Music {
    pub fn new() -> Music {
        Music {
            rx: None,
            looping: Arc::new(AtomicBool::new(true)),
            playing: false,
            ended: false,
            chunk: Vec::new(),
            idx: 0,
            a: [0.0; 2],
            b: [0.0; 2],
            frac: 0.0,
            vol_l: 1.0,
            vol_r: 1.0,
            src_rate: SRC_RATE,
        }
    }

    pub fn playing(&self) -> bool {
        self.playing && !self.ended
    }

    fn close(&mut self) {
        self.rx = None;
        self.playing = false;
        self.ended = false;
        self.chunk.clear();
        self.idx = 0;
        self.a = [0.0; 2];
        self.b = [0.0; 2];
    }

    pub fn command(&mut self, cmd: MusicCmd) -> i32 {
        match cmd {
            MusicCmd::Open(path) => {
                self.close();
                let (tx, rx) = sync_channel(3);
                let looping = self.looping.clone();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
                if ext == "opus" || (ext == "ogg" && crate::codec::OpusReader::open(&path).is_ok()) {
                    self.src_rate = OPUS_RATE;
                    std::thread::spawn(move || opus_decoder(path, looping, tx));
                } else if ext != "msc" {
                    // (MP3, Ogg Vorbis, FLAC, AAC, WAV ...: the file is opened here for its rate, and read on by the thread)
                    match crate::anyfmt::AnyReader::open(&path) {
                        Ok(reader) => {
                            self.src_rate = reader.rate as f32;
                            std::thread::spawn(move || any_decoder(path, reader, looping, tx));
                        }
                        Err(e) => eprintln!("[audio] {e}"),
                    }
                } else {
                    self.src_rate = SRC_RATE;
                    std::thread::spawn(move || decoder(path, looping, tx));
                }
                self.rx = Some(rx);
            }
            MusicCmd::Close => self.close(),
            MusicCmd::Start => {
                if self.rx.is_some() {
                    self.playing = true;
                }
            }
            MusicCmd::Stop => self.playing = false,
            MusicCmd::SetLoop(l) => self.looping.store(l, Ordering::Relaxed),
            MusicCmd::Volume(l, r) => {
                self.vol_l = l.min(0x3fff) as f32 / 16383.0;
                self.vol_r = r.min(0x3fff) as f32 / 16383.0;
            }
            MusicCmd::Status => return if self.playing() { 0x5000 } else { 0 },
        }
        0
    }

    /// Next source frame, or None on underrun / end.
    fn pull(&mut self) -> Option<[f32; 2]> {
        loop {
            if self.idx + 1 < self.chunk.len() {
                let f = [self.chunk[self.idx], self.chunk[self.idx + 1]];
                self.idx += 2;
                return Some(f);
            }
            let rx = self.rx.as_ref()?;
            match rx.try_recv() {
                Ok(c) => {
                    self.chunk = c;
                    self.idx = 0;
                }
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Disconnected) => {
                    self.ended = true;
                    return None;
                }
            }
        }
    }

    pub fn mix(&mut self, out: &mut [f32], rate: u32) {
        if !self.playing || self.ended || self.rx.is_none() {
            return;
        }
        let step = self.src_rate / rate as f32;
        for frame in out.chunks_exact_mut(2) {
            self.frac += step;
            while self.frac >= 1.0 {
                self.frac -= 1.0;
                self.a = self.b;
                match self.pull() {
                    Some(f) => self.b = f,
                    None => {
                        if self.ended {
                            return;
                        }
                        // underrun: hold last frame
                    }
                }
            }
            let t = self.frac;
            frame[0] += (self.a[0] + (self.b[0] - self.a[0]) * t) * self.vol_l;
            frame[1] += (self.a[1] + (self.b[1] - self.a[1]) * t) * self.vol_r;
        }
    }
}

/// Decode a whole `.MSC` file into a stereo 16-bit WAV file at 44100 Hz (for `tools/convert_music.py`, which makes Opus files of the music).
pub fn msc_to_wav(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    let d = std::fs::read(src).map_err(|e| format!("{}: {e}", src.display()))?;
    let mut pcm: Vec<i16> = Vec::with_capacity(d.len() / 16 * 28);
    let (mut sl, mut sr) = (State::default(), State::default());
    let (mut l, mut r) = ([0i16; 28], [0i16; 28]);
    for chunk in d.chunks(CHUNK * 2) {
        if chunk.len() < 32 {
            break;
        }
        let half = (chunk.len() / 2) & !15;
        // (the last chunk of a file can be a short one: its halves are the two channels)
        let right_at = if chunk.len() >= CHUNK * 2 { CHUNK } else { half };
        for b in 0..half / 16 {
            decode_block(&chunk[b * 16..b * 16 + 16], &mut sl, &mut l);
            decode_block(&chunk[right_at + b * 16..right_at + b * 16 + 16], &mut sr, &mut r);
            for k in 0..28 {
                pcm.push(l[k]);
                pcm.push(r[k]);
            }
        }
    }
    let bytes = pcm.len() * 2;
    let mut w = Vec::with_capacity(44 + bytes);
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&((36 + bytes) as u32).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&(SRC_RATE as u32).to_le_bytes());
    w.extend_from_slice(&(SRC_RATE as u32 * 4).to_le_bytes());
    w.extend_from_slice(&4u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&(bytes as u32).to_le_bytes());
    for s in pcm {
        w.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::write(dst, w).map_err(|e| format!("{}: {e}", dst.display()))
}
