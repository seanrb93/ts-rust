//! A second memory pool for the large allocations of a level of a mod.
//!
//! The game keeps its level in a pool of about 21 MB (`memAlloc(size, 1)`, reset by `memReset` at the start of a level), after the program in the
//! console's 32 MB; above the pool are the stacks, a display list buffer and the program's other static data, so the pool cannot be made longer.
//! The port has more memory than the console, so a level of a mod (the numbers 31 to 99) has its allocations of 256 KB or more (the level's
//! model, the biggest of its textures, ...) made in a pool of 32 MB of the port's own, which is emptied (and cleared, as the game's pool is) when the
//! level pool is.  The rest of the level is allocated where it always was.

use crate::*;
use std::sync::atomic::{AtomicI32, Ordering};

pub const BIG_BASE: i32 = 0x400_0000;
const BIG_SIZE: i32 = 0x200_0000;
pub const BIG_MIN: i32 = 0x4_0000;
static BIG_AT: AtomicI32 = AtomicI32::new(0);

/// Whether an allocation of this size of the level pool is made in the big pool (and there is room).
pub fn big_wants(size: i32) -> bool {
    if size < BIG_MIN || !(31..=99).contains(&lw(g::lvlevelnum)) {
        return false;
    }
    let at = (BIG_AT.load(Ordering::Relaxed) + 63) & !63;
    at.checked_add(size).map_or(false, |end| end <= BIG_SIZE)
}

/// Allocate `size` bytes (cleared) in the big pool.
pub fn big_alloc(size: i32) -> i32 {
    let at = (BIG_AT.load(Ordering::Relaxed) + 63) & !63;
    BIG_AT.store(at + size, Ordering::Relaxed);
    let p = BIG_BASE + at;
    host_slice(p, size as usize).fill(0);
    if std::env::var_os("TS_MEMLOG").is_some() {
        eprintln!("[bigpool] {size} bytes at {p:#x} (pool used {} of {})", at + size, BIG_SIZE);
    }
    p
}

/// The level pool is emptied: so is the big one.
pub fn big_reset() {
    BIG_AT.store(0, Ordering::Relaxed);
}
