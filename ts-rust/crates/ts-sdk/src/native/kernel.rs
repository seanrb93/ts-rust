//! EE kernel services: semaphores and cache maintenance.  The port runs the game on a single thread, so waiting on a
//! semaphore that has no signaller is a logic error that is reported rather than blocking forever.
#![allow(non_snake_case)]

use std::sync::Mutex;
use ts_rt::*;

static SEMAS: Mutex<Vec<(i32, i32)>> = Mutex::new(Vec::new()); // (count, max_count)

/// `CreateSema(param*)`; param = { count, max_count, init_count, ... } as written by the game.
pub fn CreateSema(a0: i32) -> i32 {
    let p = a0;
    let init = lw(p + 8);
    let max = lw(p + 4);
    let mut s = SEMAS.lock().unwrap();
    s.push((init, max));
    s.len() as i32
}

fn with_sema<R>(id: i64, f: impl FnOnce(&mut (i32, i32)) -> R) -> Option<R> {
    let mut s = SEMAS.lock().unwrap();
    s.get_mut((id - 1) as usize).map(f)
}

pub fn SignalSema(a0: i32) -> i32 {
    with_sema(a0 as i64, |s| {
        if s.0 < s.1.max(1) {
            s.0 += 1;
        }
    });
    a0
}
pub fn iSignalSema(a0: i32) -> i32 {
    SignalSema(a0)
}
pub fn WaitSema(a0: i32) -> i32 {
    let ok = with_sema(a0 as i64, |s| {
        if s.0 > 0 {
            s.0 -= 1;
            true
        } else {
            false
        }
    });
    if ok == Some(false) {
        eprintln!("[kernel] WaitSema({a0}) would block (no other thread to signal it)");
    }
    a0
}
pub fn iPollSema(a0: i32) -> i32 {
    with_sema(a0 as i64, |s| {
        if s.0 > 0 {
            s.0 -= 1;
            a0
        } else {
            -1
        }
    })
    .unwrap_or(-1)
}

pub fn FlushCache(_a0: i32) -> i32 {
    0
}
pub fn InvalidDCache(_a0: i32, _a1: i32) -> i32 {
    0
}
pub fn sceSifWriteBackDCache(_args: &[i64]) -> i32 {
    0
}
