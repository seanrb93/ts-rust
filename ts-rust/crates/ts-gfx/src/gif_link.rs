//! Between the vector unit and the GIF: the packets a frame's microprograms send are worked into draws on a thread of their own, while the next microprograms run.  Nothing comes back from the GIF to
//! the vector unit, so the only wait is for the end of the list (`GifLink::finish`).  With tracing on (the lines of both sides must come in order) or on a machine with few cores the packets are worked
//! at once on the calling thread.

use crate::gif::Gif;
use crate::scene::Frame;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};

/// About how many bytes of packets go to the thread at a time (a busy frame has some megabytes).  The cost of a hand-over is that of waking a thread; the delay does not matter, as nothing waits for a packet.
const BATCH: usize = 256 * 1024;
/// How many batches may wait: what the thread cannot keep up with holds the vector unit back here.
const QUEUE: usize = 16;
/// The bytes before a packet in a batch: program, kick, world, size.
const HEADER: usize = 16;

enum Message {
    Start(Box<(Gif, Frame)>),
    Batch(Vec<u8>),
    Finish,
}

struct Worker {
    to: SyncSender<Message>,
    done: Receiver<(Gif, Frame)>,
    spare: Receiver<Vec<u8>>,
}

fn work_batch(gif: &mut Gif, frame: &mut Frame, batch: &[u8]) {
    let mut at = 0;
    while at + HEADER <= batch.len() {
        let word = |o: usize| u32::from_le_bytes(batch[at + o..at + o + 4].try_into().unwrap());
        (gif.prog, gif.kick, gif.world) = (word(0), word(4), word(8) != 0);
        let size = word(12) as usize;
        gif.process(frame, &batch[at + HEADER..at + HEADER + size]);
        at += HEADER + size;
    }
}

fn start_worker() -> Option<Worker> {
    let (to, from) = sync_channel::<Message>(QUEUE);
    let (done_to, done) = channel();
    let (spare_to, spare): (Sender<Vec<u8>>, _) = channel();
    std::thread::Builder::new()
        .name("gif".into())
        .spawn(move || {
            // (as on the thread of the vector unit: tiny numbers are zero)
            crate::vu::enter_vu_mode();
            let mut current: Option<Box<(Gif, Frame)>> = None;
            for message in from {
                match message {
                    Message::Start(state) => current = Some(state),
                    Message::Batch(mut batch) => {
                        if let Some(state) = &mut current {
                            let (gif, frame) = &mut **state;
                            let t = crate::programs::tstart();
                            // (a panic loses the rest of this batch, as it lost the rest of the frame before the thread: the game goes on)
                            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| work_batch(gif, frame, &batch)));
                            crate::programs::tadd(&crate::programs::GIF_NS, t);
                        }
                        batch.clear();
                        spare_to.send(batch).ok();
                    }
                    Message::Finish => {
                        if let Some(state) = current.take() {
                            done_to.send(*state).ok();
                        }
                    }
                }
            }
        })
        .ok()?;
    Some(Worker { to, done, spare })
}

/// How many bytes of `d` the GIF reads for the packet at its start: as `Gif::process` goes through it, without working it.
pub fn packet_size(d: &[u8]) -> usize {
    let mut p = 0usize;
    while p + 16 <= d.len() {
        let lo = u64::from_le_bytes(d[p..p + 8].try_into().unwrap());
        let nloop = (lo & 0x7fff) as usize;
        let mut nreg = ((lo >> 60) & 0xf) as usize;
        if nreg == 0 {
            nreg = 16;
        }
        p += 16;
        match (lo >> 58) & 3 {
            0 => p += nloop * nreg * 16,
            1 => p = (p + nloop * nreg * 8 + 15) & !15,
            _ => p += nloop * 16,
        }
        if (lo >> 15) & 1 != 0 {
            break;
        }
    }
    p.min(d.len())
}

pub struct GifLink {
    /// entry of the VU1 microprogram running
    pub prog: u32,
    /// the packet belongs to a world-projected sprite function
    pub world: bool,
    /// counts XGKICKs
    pub kick: u32,
    batch: Vec<u8>,
    /// between lists
    idle: Option<Gif>,
    /// during a list, when the packets are worked on this thread
    here: Option<(Gif, Frame)>,
    worker: Option<Worker>,
    /// during a list, when they go to the thread
    sending: bool,
    /// no thread will be started
    no_thread: bool,
}

impl GifLink {
    pub fn new() -> GifLink {
        let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
        GifLink {
            prog: 0,
            world: false,
            kick: 0,
            batch: Vec::new(),
            idle: Some(Gif::new()),
            here: None,
            worker: None,
            sending: false,
            no_thread: cores < 4 || std::env::var_os("TS_GIF_INLINE").is_some(),
        }
    }

    /// A list begins: its draws go into `frame`.
    pub fn begin(&mut self, frame: Frame) {
        // (the last list was cut short by a panic)
        if self.sending || self.here.is_some() {
            drop(self.finish());
        }
        let gif = self.idle.take().unwrap_or_default();
        if !self.no_thread && !crate::trace::enabled() {
            if self.worker.is_none() {
                self.worker = start_worker();
                self.no_thread = self.worker.is_none();
            }
            if let Some(worker) = &self.worker {
                if worker.to.send(Message::Start(Box::new((gif, frame)))).is_ok() {
                    self.sending = true;
                    return;
                }
            }
            // (the thread is gone)
            self.worker = None;
            self.no_thread = true;
            self.here = Some((Gif::new(), Frame::default()));
            return;
        }
        self.here = Some((gif, frame));
    }

    fn send_batch(&mut self) {
        let Some(worker) = &self.worker else { return };
        let next = worker.spare.try_recv().unwrap_or_else(|_| Vec::with_capacity(BATCH + BATCH / 4));
        worker.to.send(Message::Batch(std::mem::replace(&mut self.batch, next))).ok();
    }

    /// A packet of the bytes `d`, from an XGKICK or the direct path.
    #[inline]
    pub fn packet(&mut self, d: &[u8]) {
        if let Some((gif, frame)) = &mut self.here {
            (gif.prog, gif.kick, gif.world) = (self.prog, self.kick, self.world);
            gif.process(frame, d);
            return;
        }
        let size = packet_size(d);
        if self.batch.len() + HEADER + size > BATCH && !self.batch.is_empty() {
            self.send_batch();
        }
        self.batch.extend_from_slice(&self.prog.to_le_bytes());
        self.batch.extend_from_slice(&self.kick.to_le_bytes());
        self.batch.extend_from_slice(&(self.world as u32).to_le_bytes());
        self.batch.extend_from_slice(&(size as u32).to_le_bytes());
        self.batch.extend_from_slice(&d[..size]);
    }

    /// The list is over: all its packets have been worked, and this is its frame.
    pub fn finish(&mut self) -> Frame {
        if let Some((gif, frame)) = self.here.take() {
            self.idle = Some(gif);
            return frame;
        }
        self.sending = false;
        if !self.batch.is_empty() {
            self.send_batch();
        }
        if let Some(worker) = &self.worker {
            if worker.to.send(Message::Finish).is_ok() {
                if let Ok((gif, frame)) = worker.done.recv() {
                    self.idle = Some(gif);
                    return frame;
                }
            }
        }
        self.worker = None;
        self.no_thread = true;
        Frame::default()
    }
}

impl Default for GifLink {
    fn default() -> Self {
        GifLink::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The size is what `Gif::process` goes through, for all sorts of tags and data that ends early.
    #[test]
    fn size_is_what_the_gif_reads() {
        let mut seed = 7u32;
        let mut r = move |m: u32| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) % m
        };
        for _ in 0..3000 {
            let mut d = Vec::new();
            for _ in 0..r(4) + 1 {
                let flg = r(4) as u64;
                let (nloop, nreg) = (r(5) as u64, r(5) as u64);
                let eop = (r(5) == 0) as u64;
                // (registers that make no primitive: only the walk is being compared)
                d.extend_from_slice(&(nloop | eop << 15 | flg << 58 | nreg << 60).to_le_bytes());
                d.extend_from_slice(&0x0a0a_0a0a_0a0a_0a0au64.to_le_bytes());
                let room = if nreg == 0 { 16 } else { nreg } * nloop;
                let data = match flg {
                    0 => room * 16,
                    1 => (room * 8 + 15) & !15,
                    _ => nloop * 16,
                };
                d.extend(std::iter::repeat(0u8).take(data as usize));
            }
            d.truncate(d.len() - (r(4) as usize * 8).min(d.len()));
            d.truncate(d.len() & !7);
            let read = Gif::new().process(&mut Frame::default(), &d);
            // (when the data ends in the middle, the GIF stops at the last whole item: the bytes after it are passed on and not read)
            let (size, read) = (packet_size(&d), read.min(d.len()));
            assert!(size == read || (size == d.len() && read < size), "{size} {read} {d:?}");
        }
    }
}
