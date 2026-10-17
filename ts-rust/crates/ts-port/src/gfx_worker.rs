//! Frame pipelining: the display list of frame N is interpreted and rendered on a worker thread while the game thread
//! already runs the logic of frame N+1 (the original hardware worked the same way: the EE built the next list while
//! DMA / VU1 / GS consumed the previous one).  The game thread waits for the worker before it presents the next frame
//! and before it reads back the depth buffer (`sync`).

use std::sync::mpsc::{channel, Sender};
use std::sync::{Condvar, Mutex, OnceLock};

type Req = (u64, i32, Vec<(i32, i32)>);
static TX: OnceLock<Mutex<Sender<Req>>> = OnceLock::new();
static BUSY: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());

fn start() -> Mutex<Sender<Req>> {
    let (tx, rx) = channel::<Req>();
    std::thread::Builder::new()
        .name("gfx".into())
        .spawn(move || {
            for (n, list, world) in rx {
                let t0 = std::time::Instant::now();
                // (a panic while building one frame (it is written to ts-port-crash.log) must not leave the game waiting for the worker
                // for ever: that frame is dropped and the next one is built as usual)
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::process_frame(n, list, world)));
                ts_sdk::perf::add_worker(t0.elapsed());
                let mut b = BUSY.0.lock().unwrap();
                *b = false;
                BUSY.1.notify_all();
            }
        })
        .expect("cannot start the graphics thread");
    Mutex::new(tx)
}

/// Block until the worker has finished the previous frame.
pub fn sync() {
    let mut b = BUSY.0.lock().unwrap();
    if *b {
        let t = std::time::Instant::now();
        while *b {
            b = BUSY.1.wait(b).unwrap();
        }
        ts_sdk::perf::add_sync(t.elapsed());
    }
}

/// Queue frame `n` (display list at `list`) for the worker.
pub fn submit(n: u64, list: i32, world: Vec<(i32, i32)>) {
    sync();
    let tx = TX.get_or_init(start);
    *BUSY.0.lock().unwrap() = true;
    tx.lock().unwrap().send((n, list, world)).ok();
}

// ---- second stage: depth reconstruction + drawing ------------------------------------------------------------------

type Job = (u64, ts_gfx::scene::Frame, u32, std::time::Instant);
static TX2: OnceLock<Mutex<std::sync::mpsc::SyncSender<Job>>> = OnceLock::new();

fn start2() -> Mutex<std::sync::mpsc::SyncSender<Job>> {
    let (tx, rx) = std::sync::mpsc::sync_channel::<Job>(1);
    std::thread::Builder::new()
        .name("draw".into())
        .spawn(move || {
            stage2(rx);
        })
        .expect("cannot start the drawing thread");
    Mutex::new(tx)
}

/// Hand the translated frame to the drawing thread (or finish it right here when running synchronously).
pub fn finish(n: u64, frame: ts_gfx::scene::Frame, fbp: u32) {
    if crate::pipelined() {
        let tx = TX2.get_or_init(start2);
        // stamped before the (possibly blocking) hand-over: the stamps show the pace of the game, not of the drawing thread
        tx.lock().unwrap().send((n, frame, fbp, std::time::Instant::now())).ok();
    } else {
        crate::finish_frame(n, frame, fbp);
    }
}

/// The drawing thread: every translated frame is drawn once, as it arrives.
fn stage2(rx: std::sync::mpsc::Receiver<Job>) {
    for (n, frame, fbp, _) in rx {
        // (as in the translation thread: a panic drops the frame, the drawing goes on)
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::prepare_frame(n, &frame, fbp);
            crate::show_frame(&frame);
        }));
    }
}
