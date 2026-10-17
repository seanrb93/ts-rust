//! Store tracing for differential testing (feature `trace`).
use crate::mem::host_ptr;
use std::cell::{Cell, RefCell};

#[derive(Clone, Copy, Debug)]
pub struct Store {
    pub addr: u32,
    pub size: u8,
    pub old: [u8; 16],
    pub new: [u8; 16],
    /// Tag set by the interpreter (program counter of the storing instruction); 0 for native code.
    pub tag: u32,
    pub bulk: bool,
}

thread_local! {
    static LOG: RefCell<Option<Vec<Store>>> = const { RefCell::new(None) };
    static TAG: Cell<u32> = const { Cell::new(0) };
}

pub fn set_tag(t: u32) {
    TAG.with(|c| c.set(t));
}

/// Start recording stores.
pub fn start() {
    LOG.with(|l| *l.borrow_mut() = Some(Vec::new()));
}

/// Record a typed store of `size` bytes whose new contents are at `newp` (called just before the write).
#[inline]
pub fn record(a: i32, size: usize, newp: *const u8) {
    LOG.with(|l| {
        if let Some(v) = l.borrow_mut().as_mut() {
            let mut s = Store { addr: a as u32, size: size as u8, old: [0; 16], new: [0; 16], tag: TAG.with(|c| c.get()), bulk: false };
            for i in 0..size.min(16) {
                s.old[i] = unsafe { *host_ptr(a.wrapping_add(i as i32)) };
                s.new[i] = unsafe { *newp.add(i) };
            }
            v.push(s);
        }
    });
}

/// Record a bulk write (memset / memcpy): only the old bytes are kept, in 16-byte chunks.
pub fn record_bulk(a: i32, n: usize) {
    LOG.with(|l| {
        if let Some(v) = l.borrow_mut().as_mut() {
            let mut off = 0;
            while off < n {
                let len = (n - off).min(16);
                let mut s = Store { addr: a.wrapping_add(off as i32) as u32, size: len as u8, old: [0; 16], new: [0; 16], tag: 0, bulk: true };
                for i in 0..len {
                    s.old[i] = unsafe { *host_ptr(a.wrapping_add((off + i) as i32)) };
                }
                v.push(s);
                off += len;
            }
        }
    });
}

/// Stop recording and return the stores in order.
pub fn stop() -> Vec<Store> {
    LOG.with(|l| l.borrow_mut().take().unwrap_or_default())
}

/// Per-byte (address, original byte) list for every touched byte, in write order, for rolling back.
pub fn undo_list(log: &[Store]) -> Vec<(u32, u8)> {
    let mut v = Vec::new();
    for s in log {
        for i in 0..s.size as usize {
            v.push((s.addr.wrapping_add(i as u32), s.old[i]));
        }
    }
    v
}

/// Undo all recorded writes (newest first).
pub fn rollback(log: &[Store]) {
    for s in log.iter().rev() {
        for i in (0..s.size as usize).rev() {
            unsafe { *host_ptr(s.addr.wrapping_add(i as u32) as i32) = s.old[i] };
        }
    }
}
