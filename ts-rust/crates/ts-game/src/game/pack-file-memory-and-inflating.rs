//! Functions: the sector of a file in the pack, the memory taken for loading, and the start and the flush of the inflating of a compressed file.

use crate::*;

/// The sector on the disc of the file `a0` of the pack file (@ 0x21a2f8), or 0 if it is not in it.
pub fn fileGetSector(a0: i32) -> i32 {
    let entry = packfileSeek(a0);
    let pack = lw(0x3afeb0);
    if entry == 0 {
        return 0;
    }
    lw(pack + 4) + (lw(entry + 48) >> 11)
}

/// Inflate the compressed file at `a0` of `a1` bytes to `a2` (@ 0x21c1b0), through the scratch buffer; gives the number of bytes written.
pub fn fileInflate(a0: i32, a1: i32, a2: i32) -> i32 {
    sw(0x3afed0, a2);
    sw(0x3afeb8, a0);
    sw(0x3afec4, g::scratchbuffer);
    sw(0x3afec0, a1);
    sw(0x3afecc, 0);
    sw(0x3afec8, 0);
    sw(0x3afebc, 0);
    GetHeader();
    Inflate();
    lw(0x3afecc)
}

/// Forget the pack file in use (@ 0x21a6f8); the pack of the level 102 is loaded again.
pub fn packfileReset() {
    let level = lw(g::lvlevelnum);
    sw(0x3afeac, 0);
    if level != 102 {
        return;
    }
    packfileLoad(0x3a0768, 1, 1);
}

/// Take `a0` bytes (rounded to 4) of the memory for the loading of a file (@ 0x21a750); the game ends if the memory runs out.
pub fn AllocMem(a0: i32) -> i32 {
    let at = lw(0x3afeb4);
    let end = at + (a0 + 3 & -4);
    sw(0x3afeb4, end);
    if 0x70004000 < end {
        exit(1);
    }
    at
}

/// Write the bytes the inflating made in its window to the output (@ 0x21be88).
pub fn FlushOut() {
    let count = lw(0x3afec8);
    if count != 0 {
        let from = lw(0x3afec4);
        let to = lw(0x3afed0) + lw(0x3afecc);
        memcpy(to, from, count);
        let written = lw(0x3afecc);
        let flushed = lw(0x3afec8);
        sw(0x3afec8, 0);
        sw(0x3afecc, written + flushed);
    }
}
