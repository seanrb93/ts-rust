//! Optional textual trace of everything the front-end interprets (VIF codes, programs, GIF registers).
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

static ON: AtomicBool = AtomicBool::new(false);
thread_local! {
    static LOG: RefCell<String> = const { RefCell::new(String::new()) };
}

static VU_ON: AtomicBool = AtomicBool::new(false);

/// Per-instruction VU1 trace (very verbose; env TS_VUTRACE).
pub fn vu_enabled() -> bool {
    VU_ON.load(Ordering::Relaxed) && ON.load(Ordering::Relaxed)
}

pub fn enable_vu(on: bool) {
    VU_ON.store(on, Ordering::Relaxed);
}

pub fn enable(on: bool) {
    ON.store(on, Ordering::Relaxed);
}

#[inline]
pub fn enabled() -> bool {
    ON.load(Ordering::Relaxed)
}

pub fn line(s: String) {
    LOG.with(|l| {
        let mut l = l.borrow_mut();
        if l.len() < 8_000_000 {
            l.push_str(&s);
            l.push('\n');
        }
    });
}

pub fn take() -> String {
    LOG.with(|l| std::mem::take(&mut *l.borrow_mut()))
}

#[macro_export]
macro_rules! tlog {
    ($($arg:tt)*) => {
        if $crate::trace::enabled() {
            $crate::trace::line(format!($($arg)*));
        }
    };
}
