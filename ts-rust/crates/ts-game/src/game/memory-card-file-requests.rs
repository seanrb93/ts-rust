//! Functions: the memory card library setup and the requests of the game for the files on a memory card (they are only recorded here, the state machine of the memory card does the work).

use crate::*;

/// Start the memory card library (@ 0x207a60).
pub fn mcardMake() {
    sceMcInit(&[]);
}

/// Allocate the memory for a file of a memory card (@ 0x208ec0): `a0` bytes and the header, rounded to 16.
pub fn memcardFileAlloc(a0: i32, a1: i32) -> i32 {
    memAlloc(a0 + 23 & -16, a1)
}

/// Ask to delete the file named `a0` on the memory card (@ 0x208e80).
pub fn mcardFileDelete(a0: i32) {
    strcpy(0x1fb30c0, a0);
    sw(g::mcardresult, 0);
    sw(g::mcardstate, 48);
}

/// Ask whether the file named `a0` exists on the memory card (@ 0x208d70).
pub fn mcardFileExists(a0: i32) {
    strcpy(0x1fb3140, a0);
    sw(g::mcardresult, 0);
    sw(g::mcardstate, 10);
}

/// Ask to read the file named `a0` into `a1`, `a2` bytes, and keep `a3` (@ 0x208da0).
pub fn mcardFileRead(a0: i32, a1: i32, a2: i32, a3: i32) {
    strcpy(0x1fb2fc0, a0);
    sw(g::read_buffer, a1);
    sw(g::request_read_len, a2);
    sw(0x3afe70, a3);
    sw(g::mcardstate, 32);
    sw(g::mcardresult, 0);
}

/// Ask to write the `a2` bytes at `a1` to the file named `a0` (replacing a file already there if `a3` says it may be) and keep `a4` (@ 0x208e08).
pub fn mcardFileWrite(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32) {
    strcpy(0x1fb3040, a0);
    sw(g::write_buffer, a1);
    sw(g::write_len, a2);
    sw(g::write_overwriteok, a3);
    sw(0x3afe74, a4);
    sw(g::mcardstate, 40);
    sw(g::mcardresult, 0);
}

/// Unformat the memory card of the port and slot in use, and wait for it (@ 0x208be8).
pub fn mcunformat() {
    let port = lw(g::port);
    let slot = lw(g::slot);
    sceMcUnformat(&[port as i64, slot as i64]);
    sceMcSync(&[0, 0, 0]);
}
