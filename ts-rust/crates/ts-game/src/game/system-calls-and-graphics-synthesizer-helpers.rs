//! Functions: the entry and the exit of the program, the system call stubs, the interrupts of the graphics synthesizer, and small helpers of the graphics and the DMA.

use crate::*;

/// The system call that gets the setup of the console (@ 0x2b3c40).
pub fn GetOsdConfigParam2() {
    trap("syscall");
}

/// The entry of the program (@ 0x2000b8).
pub fn _root() {
    trap("syscall");
}

/// Leave the program (@ 0x2000b0).
pub fn _exit() {
    Exit(0);
}

/// The interrupt of the graphics synthesizer (@ 0x201330): clear it and tell the waiting thread.
pub fn gsIntHandler() {
    let semaphore = lw(g::gsdone_sid);
    sd(0x12001000, 1_i64);
    iSignalSema(semaphore);
}

/// The alarm that fires when the frame takes too long (@ 0x200f10): note it and tell the waiting thread.
pub fn lockupalarm(_a0: i32, _a1: i32, a2: i32) {
    let semaphore = lw(g::gsdone_sid);
    sw(g::alarm_activated, a2);
    iSignalSema(semaphore);
}

/// Swap the double buffer of the graphics synthesizer (@ 0x200330).
pub fn sjeGsSwapDBuff(a0: i32) {
    sceGsSwapDBuff(&[g::db as i64, a0 as i64]);
}

/// Set the half offset of the drawing environment `a0` of the double buffer to `a1` (@ 0x200300).
pub fn sjeGsSetHalfOffset(a0: i32, a1: i32) {
    let env = if a0 == 0 { g::db + 96 } else { g::db + 336 };
    sceGsSetHalfOffset(&[env as i64, 2048, 2048, ((a1 << 16) >> 16) as i64]);
}

/// Start a DMA from the scratch pad to `a1`, `a2` bytes, from the address `a0` (@ 0x200608).
pub fn bossDmaFromSpr(a0: i32, a1: i32, a2: i32) {
    let channel = lw(g::dmaSpr);
    let quads = ((a2 as u32) >> 4) as i32;
    sw(channel + 128, a0 & 0xfffffff);
    sceDmaSendN(&[channel as i64, a1 as i64, quads as i64]);
}
