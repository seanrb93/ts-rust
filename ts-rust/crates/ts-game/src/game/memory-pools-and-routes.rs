//! Memory pools, stars, animation state tests, route searches and more short functions.

use super::controllers::controller_slot;
use crate::*;

/// How many sign-on profiles are used, counting from the start until the first unused one (up to 144) (@ 0x220890).
pub fn signonCount() -> i32 {
    let mut at = lw(g::signonarray);
    let mut n = 0;
    if lbu(at) != 0 {
        n += 1;
        loop {
            at += 2936;
            if n >= 144 || lbu(at) == 0 {
                break;
            }
            n += 1;
        }
    }
    n
}

/// Put the map maker's cursor in the middle of its window and reset the 10 saved cursor positions (12 bytes: 16, x, y) (@ 0x24fcf0).
pub fn mmResetCursorPos() {
    let front = lw(g::front);
    let y = (lw(g::w_h) << 16) >> 1;
    sw(front + 32, (lw(g::w_w) << 16) >> 1);
    sw(front + 36, y);
    for i in 0..10 {
        let slot = 0x1fc4080 + 12 * i;
        sw(slot, 16);
        sw(slot + 4, lw(front + 32));
        sw(slot + 8, lw(front + 36));
    }
}

/// Looked for a prop in the prop list and did nothing with the result (@ 0x272918).
pub fn propInPropList(_prop: i32) {}

/// Whether a gun is kept when it has no ammo: only the mines (gun 19), and only while some are out to detonate (@ 0x295050).
pub fn gunShouldKeepWithNoAmmo(chr: i32, gun: i32) -> i32 {
    (gun == 19 && gunCanDetonateMines(chr) != 0) as i32
}

/// Set up the game's memory: `size` bytes at `base`, the first 0x72b000 for things that last (eternal), the rest for the level, and the local
/// block starting at the top.  Both blocks are marked with "mem_mark" (@ 0x201b38).
pub fn memMake(base: i32, size: i32) {
    let level_at = base + 0x72b000;
    let end = base + size;
    sw(g::mem_eternal_free, level_at - base);
    sw(g::mem_level_free, end - level_at);
    sw(g::mem_local_at, end);
    sw(g::memBuffer, base);
    sw(g::memBufferSize, size);
    sw(g::mem_eternal_at, base);
    sw(g::mem_eternal_end, level_at);
    sw(g::mem_level_at, level_at);
    sw(g::mem_level_end, end);
    memMark(base, level_at);
    memMark(lw(g::mem_level_at), lw(g::mem_level_end));
}

/// Start animation `anim` on an object instance at speed `speed`, clearing its blend state (@ 0x260a78).
pub fn obInstSetAnim(inst: i32, anim: i32, speed: f32) {
    sw(inst + 96, anim);
    swc1(inst + 140, speed);
    sw(inst + 220, -1);
    sw(inst + 156, -1);
    for at in [188, 100, 128, 132, 136, 104, 108, 112, 116, 120, 124, 192, 196, 200, 168, 172, 176, 180, 184] {
        sw(inst + at, 0);
    }
}

/// The first item of an object's list (head at +52) whose kind is `kind` or whose group is `group`, or 0 (@ 0x248b98).
pub fn objFindItem(obj: i32, kind: i32, group: i32) -> i32 {
    let mut item = lw(obj + 52);
    while item != 0 {
        if lhu(item) == kind || lhu(item + 2) == group {
            return item;
        }
        item = lw(item + 4);
    }
    0
}

/// Give star `n` (28 bytes) a random direction (-1 or 1 on two axes) and a bluish-white colour (@ 0x2a8670).
pub fn setStar(n: i32) {
    let star = n * 28 + g::stars;
    swc1(star + 8, if newrnd() & 1 != 0 { 1.0 } else { -1.0 });
    swc1(star + 12, if newrnd() & 1 != 0 { 1.0 } else { -1.0 });
    let r = (newrnd() as i32 as u32 % 28) as i32;
    let g_ = (newrnd() as i32 as u32 % 28) as i32;
    let b = (newrnd() as i32 as u32 % 28) as i32;
    sw(star + 24, ((r + 100) << 16) + ((g_ + 100) << 8) + 100 + b);
}

/// The volume (44 bytes) with number `id`, or 0 (@ 0x257638).
pub fn bgGetVolume(id: i32) -> i32 {
    for i in 0..lw(g::bgNumVolumes) {
        let volume = i * 44 + lw(g::levelvolumes);
        if lw(volume) == id {
            return volume;
        }
    }
    0
}

/// Clear all of a player's controls: buttons, and sticks to the centre (128) (@ 0x203a88).
pub fn joyClearAll(player: i32) {
    let slot = controller_slot(player);
    joyClearButs(player, 0xffff);
    let c = lw((slot << 2) + g::playercontrollers) * 384 + g::controllers;
    sw(c + 312, 128);
    sw(c + 300, 128);
    sw(c + 304, 128);
    sw(c + 308, 128);
}

/// How many map maker objects have kind `kind` (@ 0x248bd8).
pub fn mmObjCount(kind: i32) -> i32 {
    if lw(g::nummapobjects) == 0 {
        return 0;
    }
    let mut n = 0;
    let mut object = lw(g::mapobjects);
    while object != 0 {
        if lw(object) == kind {
            n += 1;
        }
        object = lw(object + 56);
    }
    n
}

/// The defaults of a player's preferences (@ 0x21fbf0).
pub fn playerprefsNew(prefs: i32) {
    sh(prefs + 2, 253);
    sh(prefs, 0);
    for (at, value) in [
        (8, 8), (16, 16), (32, 4), (24, 0x1000), (28, 0x4000), (12, 64), (36, 0x2000), (40, 0x8000), (60, 1), (68, 3), (72, 2),
        (84, 0), (80, 0), (88, 2), (92, 3), (4, 2), (20, 1), (44, 1), (52, 3), (48, 0), (56, 2), (64, 0),
    ] {
        sw(prefs + at, value);
    }
}

/// Remember the game mode options of the menus for mode `mode` (32 bytes a mode) (@ 0x238fe8).
pub fn storeGameModeOptions(mode: i32) {
    let friendly_fire = lw(g::gv_friendfire);
    let radar = lw(g::gv_dispradar);
    let time = lw(g::gv_timelimit);
    let rec = (mode << 5) + 0x1fc2c98;
    let kills = lw(g::gv_killlimit);
    let score = lw(g::gv_scorelimit);
    let method = lw(g::gv_scoremethod);
    let one_shot = lw(g::gv_oneshotkill);
    sw(rec, lw(g::gv_teamplay));
    sw(rec + 28, one_shot);
    sw(rec + 4, friendly_fire);
    sw(rec + 8, radar);
    sw(rec + 12, time);
    sw(rec + 16, kills);
    sw(rec + 20, score);
    sw(rec + 24, method);
}

/// Find the data after the third line break from `at`, within the first 32 bytes after the big buffer's start (@ 0x2b9130).
pub fn FindRawData(at: i32) -> i32 {
    let limit = lw(g::bigbuffer) + 32;
    let mut p = at;
    let mut lines = 0;
    if (p as u32) < (limit as u32) {
        let mut c = lbu(p);
        loop {
            if c == 10 {
                lines += 1;
            }
            p += 1;
            if lines >= 3 || (p as u32) >= (limit as u32) {
                break;
            }
            c = lbu(p);
        }
    }
    p
}

/// Whether the animation `slot` of a character matches animation `n`, as stored at `at` of its animation state: the number is moved
/// by the character's variant (+344) for the ranges 3 to 13 and 113 to 123 of a character of kind 0, and by 38 for some of the range from 443 (@ 0x215a10, 0x215a88).
fn anim_matches(chr: i32, n: i32, at: i32) -> i32 {
    let mut n = n;
    let mut range = n - 443;
    if lw(lw(chr + 352) + 8) == 0 && (((n - 3) as u32) < 11 || ((n - 113) as u32) < 11) {
        n += lw(chr + 344) * 11;
        range = n - 443;
    }
    let state = lw(chr + 32);
    if (range as u32) < 32 && lw(lw(state + 88) + 12) == 1 {
        n += 38;
    }
    (lw(state + at) == n) as i32
}

/// @ 0x215a10
pub fn curAnimSlot1(chr: i32, n: i32) -> i32 {
    anim_matches(chr, n, 96)
}

/// @ 0x215a88
pub fn curAnimSlot2(chr: i32, n: i32) -> i32 {
    anim_matches(chr, n, 160)
}

/// The box (24 bytes) of a front end page with id `id`, or 0 (@ 0x22c470).
pub fn frontGetBox(page: i32, id: i32) -> i32 {
    let count = lw(page + 12);
    let mut box_ = lw(page + 8);
    for _ in 0..count {
        if lhu(box_ + 8) == id & 0xffff {
            return box_;
        }
        box_ += 24;
    }
    0
}

/// Whether a front end effect of type `n` is over: types 0, 2 and 4 are always, type 1 when its time has passed a limit (@ 0x22a5b0).
pub fn frontfxFinished(n: i32) -> i32 {
    if n == 1 && lwc1(g::fronttime) < lwc1(0x3ac8dc) {
        0
    } else {
        1
    }
}

/// Take a free head of the 15 (112 bytes each) and mark it used; 0 if none (@ 0x268678).
pub fn getfreehead() -> i32 {
    for i in 0..15 {
        if lw(g::headused + 4 * i) == 0 {
            sw(g::headused + 4 * i, 1);
            return lw(g::levelheads) + 112 * i;
        }
    }
    0
}

/// Follow a route table (16-byte entries, the entry's next at +0, its target at +6) from the entry in `start`'s first word to the one that leads to `target`; -1 if none.
fn find_route(start: i32, target: i32, table: i32) -> i32 {
    let mut at = lw(start);
    while at >= 0 {
        let entry = (at << 4) + table;
        if lh(entry + 6) == target {
            return at;
        }
        at = lh(entry);
    }
    -1
}

/// @ 0x265fc0
pub fn hr_findroute(start: i32, target: i32) -> i32 {
    find_route(start, target, 0x1fc6490)
}

/// @ 0x266680
pub fn r_findroute(start: i32, target: i32) -> i32 {
    find_route(start, target, 0x1fca310)
}

/// Add a map file entry (40 bytes) to the first directory (100 at most); its number, or -1 if full (@ 0x253a98).
pub fn mmfileEntryNew(entry: i32) -> i32 {
    let dir = lw(g::mapdir1);
    let n = lw(dir + 8);
    if n >= 100 {
        return -1;
    }
    let rec = n * 40 + dir;
    sw(dir + 8, n + 1);
    let (a, b, c) = (ld(entry + 8), ld(entry + 16), ld(entry + 24));
    sd(rec + 12, ld(entry));
    sd(rec + 20, a);
    sd(rec + 28, b);
    sd(rec + 36, c);
    sd(rec + 44, ld(entry + 32));
    n
}

/// Load the current player's options into the preferences menu's variables (@ 0x20f8d8).
pub fn prefs_vars() {
    let settings = lw(lw(g::currentplayer) + 24);
    let flags = lhu(settings + 2);
    let vibration_bits = flags & 96;
    sw(g::gv_autoaim, flags >> 7 & 1);
    let weapon_change = lw(settings + 88);
    sw(g::gv_vibration, flags >> 4 & 1);
    sw(g::gv_wpchange, weapon_change);
    sw(g::gv_vibmode, match vibration_bits {
        64 => 1,
        96 => 3,
        32 => 2,
        _ => 0,
    });
}

/// How many profiles with a memory card file (info 8) lie in the used run at the start of the sign-on array (@ 0x2208c8).
pub fn signonMemCount() -> i32 {
    let array = lw(g::signonarray);
    let mut n = 0;
    let mut i = 0;
    if lbu(array) != 0 {
        loop {
            let info = i << 3;
            i += 1;
            if lw(info + g::signoninfo) == 8 {
                n += 1;
            }
            if i >= 144 || lbu(i * 2936 + array) == 0 {
                break;
            }
        }
    }
    n
}

/// Read back the depth buffer at each point of the current depth test slot (9 points a test) (@ 0x2a7108).
pub fn zbtestDoTest() {
    let slot = lw(0x3b0174);
    let points = lw((slot << 2) + 0x1fea0a0) * 9;
    for i in 0..points.max(0) {
        let point = lw((slot << 2) + g::zbval) + i * 24;
        let buffer = lw(g::zbuffer);
        if lw(point) >= 0 {
            sh(point + 18, lhu(buffer + lw(point + 12)));
        }
    }
}
