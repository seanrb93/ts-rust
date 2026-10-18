//! Runtime support for the decompiled TimeSplitters code.
//!
//! The decompiled functions are ordinary native Rust.  The game's original data structures still live at their
//! original 32-bit addresses inside one flat arena (`mem`), exactly like the pointers stored in the game's data
//! tables and level files do.  This is *not* an emulator: there is no CPU model, no instruction interpretation and
//! no emulated PS2 hardware.  Hardware (GS/VU/DMA/SPU/IOP) is replaced by native services in `ts-sdk` / `ts-port`.
//!
//! Over time, individual structures are lifted out of the arena into typed Rust structs.

pub mod mem;
pub mod frame;
pub mod int;
pub mod loader;
pub mod texnames;
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
pub use mem::*;

/// Hit when the original code executed a `break` instruction (divide-by-zero checks, assertions).
#[cold]
#[inline(never)]
pub fn trap(what: &str) {
    panic!("trap: {what}");
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
