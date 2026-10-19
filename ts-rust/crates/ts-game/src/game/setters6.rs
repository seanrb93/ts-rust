//! Map maker items, the sign-on table, preload, the zoom boxes and more short functions.

use super::setters4::controller;
use crate::*;

/// @ 0x20f5d8
pub fn setup_pageGfx() {
    setup_vars();
}

/// Reads the first player's buttons and does nothing with them (@ 0x220ba8).
pub fn StatsTick() {
    let _ = joyBut(0);
}

/// Prepare the memory card dialog: no cards known, no window, no controller, no options (@ 0x244e18).
pub fn mcseqMake() {
    mcseqUnvalidateCards();
    sb(0x3affb8, 0);
    sw(0x3affdc, 0);
    sw(0x3affd4, 0);
}

/// How many particles to make out of `n`, for the number of players (@ 0x2968a0).
pub fn particleGetNum(n: i32) -> i32 {
    (particleGetScale() * n as f32) as i32
}

/// The cost of a link between pads for the route finder (@ 0x266188).
pub fn hr_calclinkcost(link: i32) -> f32 {
    hr_calcestcost(lw(link + 4), lw(link + 8))
}

/// The cost of a link for the other route finder (@ 0x2667f0).
pub fn r_calclinkcost(link: i32) -> f32 {
    r_calcestcost(lw(link + 4), lw(link + 8))
}

/// Stop a sound after its delay: stop its voice (the word at +28) if it has one and clear the pending flag at +10 (@ 0x2054c0).
pub fn soundDelayStop(n: i32) {
    if n >= 0 {
        let rec = 0x1fb1870 + (n << 5);
        let voice = lw(rec + 28);
        if voice >= 0 {
            soundStop(voice);
        }
        sh(rec + 10, 0);
    }
}

/// Whether a character can pick up a bag: it is not in a state of 0x1020, not a player-controlled one (+484), not of kind 4, and not on the same team as the bag's owner (except in mode 1) (@ 0x269f70).
pub fn propChrCanPickupBag(chr: i32, bag: i32) -> i32 {
    let owner = lw(bag + 352);
    if lw(chr + 2708) & 0x1020 == 0 && lw(chr + 484) == 0 && lw(owner + 68) != 4 {
        if lw(g::curgameparms + 72) != 1 {
            return (lw(chr + 24) != lw(owner + 16)) as i32;
        }
        return 1;
    }
    0
}

/// The first unused sign-on file in a table of 16 (records of 2936 bytes; the first byte is 0 when unused), or -1 (@ 0x220210).
pub fn signonFileGetFree(table: i32) -> i32 {
    let mut at = table;
    let mut i = 0;
    while lbu(at) != 0 {
        i += 1;
        at += 2936;
        if i >= 16 {
            return -1;
        }
    }
    i
}

/// The first unused profile of the 144 in the sign-on array, or -1 (@ 0x220240).
pub fn signonGetFree() -> i32 {
    let mut at = lw(g::signonarray);
    let mut i = 0;
    while lbu(at) != 0 {
        i += 1;
        at += 2936;
        if i >= 144 {
            return -1;
        }
    }
    i
}

/// Empty loops in the build (@ 0x220010).
pub fn statsDebug() {}

/// Pause (`pause` non-zero) or resume the game; resuming gives -1, which marks that the game was paused (@ 0x2b6818).
pub fn timePause(pause: i32) {
    if lw(g::gamepaused) == 0 {
        if pause != 0 {
            sw(g::gamepaused, 1);
        }
    } else if pause == 0 {
        sw(g::gamepaused, -1);
    }
}

/// Set up zoom box `n` (40 bytes) to show the rectangle `rect` (x0, y0, x1, y1): the rectangle, then its origin and size (@ 0x22ef58).
pub fn zoomboxSet(n: i32, rect: i32, value: i32) {
    let b = n * 40 + g::zoomboxes;
    let second = ld(rect + 8);
    sd(b, ld(rect));
    sd(b + 8, second);
    sw(b + 20, lw(rect));
    sw(b + 24, lw(rect + 4));
    sw(b + 28, lw(rect + 8) - lw(rect) + 1);
    let y0 = lw(rect + 4);
    let y1 = lw(rect + 12);
    sw(b + 16, value);
    sw(b + 36, 0);
    sw(b + 32, y1 - y0 + 1);
}

/// Work out which cheats are allowed: none when flags 0x2020 are set, else the unlocked ones for the mode (mode 7 is Story) (@ 0x225680).
pub fn cheatsReset() {
    let params = g::curgameparms;
    if lw(params + 80) & 0x2020 == 0 {
        if lw(params + 72) != 7 {
            sw(g::allowedcheats, cheatsArcade());
        } else {
            sw(g::allowedcheats, cheatsStory());
        }
    } else {
        sw(g::allowedcheats, 0);
    }
}

/// Select depth buffer mode `mode` in the display list (-1 keeps the current one); returns the mode that was set before (@ 0x2b7cd0).
pub fn dlSetZB(mode: i32) -> i32 {
    let previous = lw(g::zbmode);
    if mode != previous && mode != -1 {
        sb(lw(g::gfxtag) + 3, 48);
        let tag = lw(g::gfxtag);
        sw(g::zbmode, mode);
        sw(tag + 4, (mode << 6) + g::zbvifpackets);
        sh(tag, 4);
        sw(g::gfxtag, tag + 16);
    }
    previous
}

/// Treat the buttons in `mask` as handled: not new any more, and masked from being held (@ 0x2039e8).
pub fn joyClearButs(player: i32, mask: i32) {
    let c = controller(player);
    let new = lw(c + 292) & !mask;
    sw(c + 340, lw(c + 340) | mask);
    sw(c + 292, new);
}

/// Start the debug memory tracker's buffer: for modes 1 and 2 the memory the game provides (2 MB for mode 2, else 256 KB), otherwise a fixed 10 KB one (@ 0x201e98).
pub fn memdbRestart(mode: i32) {
    if ((mode - 1) as u32) >= 2 {
        let buffer = 0x1fac5f0;
        sw(g::buflen, 0x2800);
        sw(0x3afe14, buffer);
        sw(0x3afe10, buffer + 0x2800);
        sw(0x3afe0c, buffer);
        return;
    }
    let buffer = lw(g::memdbbuffer);
    let len = if mode == 2 { 0x200000 } else { 0x40000 };
    sw(0x3afe14, buffer);
    sw(g::buflen, len);
    sw(0x3afe10, buffer + len);
    sw(0x3afe0c, buffer);
}

/// Take an item (16 bytes, kind and group both 0xffff) from the map maker's free list, or 0 (@ 0x2489e0).
pub fn mmAllocItem() -> i32 {
    let item = lw(0x3b0018);
    if item == 0 {
        return 0;
    }
    let next = lw(item + 4);
    let free = lw(0x3b001c) - 1;
    sh(item + 2, 0xffff);
    sw(0x3b0018, next);
    sw(0x3b001c, free);
    sh(item, 0xffff);
    item
}

/// Count the items of a list by their group: `counts` is an array of words indexed by the group of each item's definition (@ 0x248d50).
pub fn mmItemGroupCount(list: i32, counts: i32) {
    let mut item = list;
    while item != 0 {
        let kind = lhu(item + 2);
        let next = lw(item + 4);
        let at = (lw((kind << 4) + (g::itemdefs + 4)) << 2) + counts;
        sw(at, lw(at) + 1);
        item = next;
    }
}

/// The quit confirmation of the map maker: bit 64 = yes (1, and flag 0x8000 is set), bit 16 = no (2), else 0 (@ 0x24ee50).
pub fn quitconfirmfunc() -> i64 {
    let buttons = joyNewBut(0);
    let flags = lw(g::editflags);
    if buttons & 64 == 0 {
        ((joyNewBut(0) as i64 as u64) >> 3) as i64 & 2
    } else {
        sw(g::editflags, flags | 0x8000);
        1
    }
}

/// Add a torch to the depth test list (32 bytes each); the count is checked as being below 0, so nothing is added (@ 0x2a6fa8).
pub fn zbtestDrawTorch(a: i32, b: i32, pos: i32, dir: i32) {
    let n = lw(0x3b0184);
    if n < 0 {
        sw(0x3b0184, n + 1);
        let x = lwc1(pos);
        let torch = (n << 5) + 0x1fea860;
        sw(torch, a);
        swc1(torch + 8, x);
        sw(torch + 4, b);
        swc1(torch + 12, lwc1(pos + 4));
        swc1(torch + 16, lwc1(pos + 8));
        swc1(torch + 20, lwc1(dir));
        swc1(torch + 24, lwc1(dir + 4));
        swc1(torch + 28, lwc1(dir + 8));
    }
}

/// Find the end of the table of levels (entries of 8 bytes, the first word 0 at the end) and note its index (@ 0x22fef8).
pub fn frontredReset() {
    sw(0x3aff1c, 0);
    if lw(g::levels) != 0 {
        let mut i = 1;
        loop {
            sw(0x3aff1c, i);
            if lw((i << 3) + g::levels) == 0 {
                break;
            }
            i += 1;
        }
    }
}

/// Printed a matrix and did nothing (@ 0x2b5690).
pub fn matrixPrintf() {}

/// Printed the particle lists and did nothing (@ 0x296c40).
pub fn particledebug() {}

/// How many of the 8 memory card slots hold a card (a state of 4 or more) (@ 0x244de0).
pub fn mcseqGetNumCards() -> i32 {
    (0..8).filter(|&i| lw(0x1fc3710 + 4 * i) >= 4).count() as i32
}

/// Clear the preload table (500 entries of 24 bytes) and its counters (@ 0x21c1f8).
pub fn preloadReset() {
    for i in 0..(0x2ee0 / 24) {
        let entry = 0x1fb4c60 + 24 * i;
        sw(entry, 0);
        sw(entry + 12, 0);
        sw(entry + 16, 0);
        sw(entry + 20, 0);
    }
    sw(g::numpreload, 0);
    sw(0x3afedc, 0);
    sw(0x3afee0, 0);
    sw(0x3afee4, 0);
}

/// Move the pointers of a texture info table (16-byte entries, ended by a first word of -1) that are flagged (0x10000000 at +12) by `delta` (@ 0x25eb30).
pub fn texinfoRelocate(table: i32, delta: i32) {
    let mut at = table;
    if lw(at) != -1 {
        let mut i = 0;
        loop {
            i += 1;
            if lw(at + 12) & 0x10000000 != 0 {
                sw(at, lw(at) + delta);
            }
            at = (i << 4) + table;
            if lw(at) == -1 {
                break;
            }
        }
    }
}

/// On the retry screen (game state 4): bit 64 restarts the level, bit 16 quits (@ 0x2108b8).
pub fn tickRetryQuit() {
    if lw(g::gamestate) == 4 {
        if joyNewBut(0) & 64 != 0 {
            gameLevelRestart();
            return;
        }
        if joyNewBut(0) & 16 != 0 {
            gameLevelQuit();
        }
    }
}

/// Cycle the prop editor's camera views: modes 4 to 5 go to 9, everything else to 4; the player's followed character (+904) is set (@ 0x20d5c8).
pub fn cyclepropview() {
    let mode = camGetMode();
    let player = lw(g::currentplayer);
    if mode >= 4 && mode < 6 {
        sw(player + 904, lw(player + 384));
        camSetMode(9);
        return;
    }
    sw(player + 904, lw(player + 384));
    camSetMode(4);
}
