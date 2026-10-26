//! Debug aid (features `watch` / `calltrace`): report every store to one game address together with the Rust call
//! stack, to find which decompiled function writes a value.
use std::sync::atomic::{AtomicU32, Ordering};

static WATCH: AtomicU32 = AtomicU32::new(0);

pub fn set(addr: u32) {
    WATCH.store(addr, Ordering::Relaxed);
}

#[inline]
pub fn check(a: i32, size: usize, v: u64) {
    let w = WATCH.load(Ordering::Relaxed);
    if w != 0 && (a as u32) <= w && w < a as u32 + size as u32 {
        report(a as u32, size, v);
    }
}

#[cold]
fn report(a: u32, size: usize, v: u64) {
    eprintln!("[watch] store {size} bytes at {a:#x} = {v:#x}");
    let bt = std::backtrace::Backtrace::force_capture().to_string();
    for l in bt.lines().filter(|l| l.contains("ts_game::dec") || l.contains("ts_game::game") || l.contains("ts_game::manual") || l.contains("ts_sdk")) {
        eprintln!("   {}", l.trim());
    }
}
