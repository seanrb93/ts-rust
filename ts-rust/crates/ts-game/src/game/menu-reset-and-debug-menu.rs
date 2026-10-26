//! The in-game menu's reset, the levels of the goodies, the skeleton slots, the debug overlays and the z buffer copy.

use crate::*;

/// Mark the first item of the main set-up menu with flags 64 and 512 and restart the in-game menu (@ 0x212520).
pub fn ingamemenuReset() {
    let items = g::mainsetupmenu_items;
    sh(items + 10, lhu(items + 10) | 576);
    ingamemenuRestart();
}

/// The levels from 2 to 30 that are available (`levelAvail`) as a bit mask, one bit per level number (@ 0x240320).
pub fn goodiesGetLevels() -> i32 {
    let mut mask = 0;
    let mut level = 2;
    loop {
        let available = levelAvail(level);
        let bit = 1 << (level & 31);
        if available != 0 {
            mask |= bit;
        }
        level += 1;
        if level >= 31 {
            break;
        }
    }
    mask
}

/// Take the memory of the slot `slot` (20 bytes) of the skeleton table: 1152 bytes at +0 and 512 bytes at +4 from the eternal area (2), the slot number at +12, `file` at +16 and
/// the used flag at +8 (@ 0x215cb8).
pub fn loadSkeleton(slot: i32, file: i32) {
    let at = slot * 20 + g::skeletons;
    let bones = memAlloc(1152, 2);
    sw(at, bones);
    let extra = memAlloc(512, 2);
    sw(at + 16, file);
    sw(at + 12, slot);
    sw(at + 4, extra);
    sw(at + 8, 1);
}

/// Draw the debug overlays whose flags are set: the speed, the performance statistics and the position; and the text box of the message being typed (@ 0x20db90).
pub fn debugmenuGfx() {
    if lw(g::displayspeed) != 0 {
        displayspeedGfx();
    }
    let mut position = lw(g::displaypos);
    if lw(g::displayperfstats) != 0 {
        displayperfstatsGfx();
        position = lw(g::displaypos);
    }
    let mut typing = lw(g::in_message);
    if position != 0 {
        displayposGfx();
        typing = lw(g::in_message);
    }
    if typing == 0 {
        return;
    }
    menuTextBoxGfx();
}

/// Copy the z buffer (640 by 224 at 2560, 10 and 50) to the screen image record (112 bytes) of the number `gs_simagenum`, which then flips between 0 and 1 (@ 0x2a70a0).
pub fn zbtestCopyZB() {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let record = lw(g::gs_simagenum) * 112;
    sw(sp, lw(g::zbuffer));
    sjeStoreImage(record + 0x1fea860, 2560, 10, 50, 0, 0, 640, 224);
    sw(g::gs_simagenum, 1 - lw(g::gs_simagenum));
}
