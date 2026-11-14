//! Runtime support for the decompiled TimeSplitters code.
//!
//! The decompiled functions are ordinary native Rust.  The game's original data structures still live at their
//! original 32-bit addresses inside one flat arena (`mem`), exactly like the pointers stored in the game's data
//! tables and level files do.  This is *not* an emulator: there is no CPU model, no instruction interpretation and
//! no emulated PS2 hardware.  Hardware (GS/VU/DMA/SPU/IOP) is replaced by native services in `ts-sdk` / `ts-port`.
//!
//! Over time, individual structures are lifted out of the arena into typed Rust structs.

pub mod depth;
pub mod mem;
#[cfg(feature = "journal")]
pub mod journal;
pub mod frame;
pub mod capture;
pub mod int;
pub mod ps2float;
pub mod loader;
pub mod texnames;
pub mod gswide;
#[cfg(feature = "trace")]
pub mod trace;
#[cfg(feature = "calltrace")]
pub mod calltrace;
#[cfg(any(feature = "calltrace", feature = "watch"))]
pub mod watch;
#[cfg(feature = "shadow")]
pub mod shadow;

pub use frame::Frame;
pub use int::*;
pub use ps2float::Ps2F;
pub use mem::*;

/// Hit when the original code executed a `break` instruction (divide-by-zero checks, assertions).
#[cold]
#[inline(never)]
pub fn trap(what: &str) {
    panic!("trap: {what}");
}

/// Set by the verifier: `exit` then ends the run like a trap (a panic starting with "trap:") instead of ending the process.
pub static VERIFYING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

thread_local! {
    static DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// The depth of a recursive function of the game, while the verifier runs it: past `limit` the run ends with a fault (a panic, which the verifier
/// takes as the original's fault on the same data), as random data can make the structure a recursion walks endless and the host's stack would
/// overflow and end the whole check.  In play it does nothing.  Hold the guard for the length of the call.
pub struct DepthGuard(bool);

impl DepthGuard {
    #[inline]
    pub fn enter(limit: u32) -> DepthGuard {
        if !VERIFYING.load(std::sync::atomic::Ordering::Relaxed) {
            return DepthGuard(false);
        }
        let d = DEPTH.with(|d| {
            d.set(d.get() + 1);
            d.get()
        });
        if d > limit {
            DEPTH.with(|d| d.set(0));
            panic!("recursion deeper than {limit}");
        }
        DepthGuard(true)
    }
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        if self.0 {
            DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
        }
    }
}

/// The budget of memory accesses of a run (feature `fuel`, the verifier's build only).
#[cfg(feature = "fuel")]
pub mod fuel {
    thread_local! {
        static LEFT: std::cell::Cell<u64> = const { std::cell::Cell::new(u64::MAX) };
    }
    /// Give the next run `n` accesses (`u64::MAX`: no limit).
    pub fn refill(n: u64) {
        LEFT.with(|l| l.set(n));
    }
    /// A bulk copy or fill of `n` bytes costs a word's access for each 4 bytes (the original's code does it a word at a time).
    pub fn burn_bytes(n: usize) {
        let cost = (n / 4) as u64;
        LEFT.with(|l| {
            let v = l.get();
            if v < cost {
                l.set(u64::MAX);
                panic!("out of fuel: a copy or fill of {n} bytes");
            }
            l.set(v - cost);
        });
    }
    #[inline(always)]
    pub fn burn() {
        LEFT.with(|l| {
            let v = l.get();
            if v == 0 {
                l.set(u64::MAX);
                panic!("out of fuel: a loop that does not end");
            }
            l.set(v - 1);
        });
    }
}

/// Called by auto-generated stubs of SDK functions that have no native implementation yet.
/// Logs each distinct name once; the stub then returns 0.
pub fn stub_called(name: &str) {
    use std::collections::HashSet;
    use std::sync::Mutex;
    static SEEN: Mutex<Option<HashSet<String>>> = Mutex::new(None);
    let mut g = SEEN.lock().unwrap();
    let set = g.get_or_insert_with(HashSet::new);
    #[cfg(feature = "calltrace")]
    crate::calltrace::poll();
    if set.insert(name.to_string()) {
        eprintln!("[stub] {name}() called - not implemented yet, returning 0");
    }
}
