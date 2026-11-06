//! A journal of the memory a tick changes, so that a network game can go back some ticks (rollback).
//!
//! While it is on, the first store to a page (4 KiB) of the game's RAM in a tick first copies the page as it was.  `begin_tick` starts the record of a tick
//! and `undo` puts back the pages of the last ticks, newest first, which gives the memory as it was before the first of them.  Only the main RAM is kept; the
//! scratchpad and the registers are not part of the game's state.

use crate::mem::RAM_SIZE;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;

const PAGE: usize = 4096;
const PAGES: usize = (RAM_SIZE as usize) >> 12;

static ON: AtomicBool = AtomicBool::new(false);
/// the number of the tick that is recorded now; a page whose mark is this one is already saved
static NOW: AtomicU32 = AtomicU32::new(1);
static mut MARKS: *mut u32 = std::ptr::null_mut();

struct Tick {
    pages: Vec<(u32, Box<[u8; PAGE]>)>,
}

struct Journal {
    ticks: VecDeque<Tick>,
    spare: Vec<Box<[u8; PAGE]>>,
    /// the most ticks kept
    keep: usize,
    saved_last: usize,
}

static JOURNAL: Mutex<Option<Journal>> = Mutex::new(None);

/// Start keeping the journal of the last `keep` ticks.
pub fn enable(keep: usize) {
    unsafe {
        if MARKS.is_null() {
            MARKS = Box::leak(vec![0u32; PAGES].into_boxed_slice()).as_mut_ptr();
        }
    }
    *JOURNAL.lock().unwrap() = Some(Journal { ticks: VecDeque::new(), spare: Vec::new(), keep, saved_last: 0 });
    ON.store(true, Ordering::Relaxed);
}

pub fn disable() {
    ON.store(false, Ordering::Relaxed);
    *JOURNAL.lock().unwrap() = None;
}

#[inline(always)]
pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

/// A store of `n` bytes at the game address `a` is about to be made.
#[inline(always)]
pub fn touch(a: i32, n: usize) {
    if on() {
        // (nearly every store is to a page that this tick has already kept)
        let u = a as u32;
        if u < RAM_SIZE && (u & 0xfff) as usize + n <= PAGE && unsafe { *MARKS.add((u >> 12) as usize) } == NOW.load(Ordering::Relaxed) {
            return;
        }
        touch_slow(u, n);
    }
}

#[inline(never)]
fn touch_slow(a: u32, n: usize) {
    let a = if a < RAM_SIZE {
        a
    } else {
        // (the mirrors of the main RAM)
        let m = a & 0x0fff_ffff;
        if matches!(a >> 28, 2 | 3 | 8 | 9 | 0xa | 0xb) && m < RAM_SIZE {
            m
        } else {
            return;
        }
    };
    let first = (a >> 12) as usize;
    let last = ((a as usize + n.max(1) - 1) >> 12).min(PAGES - 1);
    for pg in first..=last {
        unsafe {
            let mark = MARKS.add(pg);
            if *mark != NOW.load(Ordering::Relaxed) {
                *mark = NOW.load(Ordering::Relaxed);
                save_page(pg);
            }
        }
    }
}

fn save_page(pg: usize) {
    let mut g = JOURNAL.lock().unwrap();
    let Some(j) = g.as_mut() else { return };
    let Some(t) = j.ticks.back_mut() else { return };
    let mut buf = j.spare.pop().unwrap_or_else(|| Box::new([0u8; PAGE]));
    unsafe {
        std::ptr::copy_nonoverlapping(crate::mem::host_ptr((pg << 12) as i32), buf.as_mut_ptr(), PAGE);
    }
    t.pages.push((pg as u32, buf));
}

/// A new tick begins: what is stored from here on is the record of it.
pub fn begin_tick() {
    let mut g = JOURNAL.lock().unwrap();
    let Some(j) = g.as_mut() else { return };
    NOW.fetch_add(1, Ordering::Relaxed);
    if let Some(t) = j.ticks.back() {
        j.saved_last = t.pages.len();
    }
    j.ticks.push_back(Tick { pages: Vec::new() });
    while j.ticks.len() > j.keep {
        if let Some(old) = j.ticks.pop_front() {
            for (_, b) in old.pages {
                j.spare.push(b);
            }
        }
    }
}

/// The ticks that can be undone (including the one being recorded).
pub fn depth() -> usize {
    JOURNAL.lock().unwrap().as_ref().map_or(0, |j| j.ticks.len())
}

/// The pages the last finished tick saved (a measure of how much a tick changes).
pub fn saved_last() -> usize {
    JOURNAL.lock().unwrap().as_ref().map_or(0, |j| j.saved_last)
}

/// Put the memory back as it was before the last `n` ticks of the journal (the one being recorded is the first of them): their records are dropped.  False
/// when the journal does not go back so far.
pub fn undo(n: usize) -> bool {
    let mut g = JOURNAL.lock().unwrap();
    let Some(j) = g.as_mut() else { return false };
    if n > j.ticks.len() {
        return false;
    }
    // (the stores of the undo itself are not recorded: they go straight to the memory)
    for _ in 0..n {
        let Some(t) = j.ticks.pop_back() else { break };
        for (pg, buf) in t.pages.into_iter().rev() {
            unsafe {
                std::ptr::copy_nonoverlapping(buf.as_ptr(), crate::mem::host_ptr((pg << 12) as i32), PAGE);
            }
            j.spare.push(buf);
        }
    }
    // (the marks of the dropped ticks are stale: a new tick number makes every page unsaved again)
    NOW.fetch_add(1, Ordering::Relaxed);
    true
}
