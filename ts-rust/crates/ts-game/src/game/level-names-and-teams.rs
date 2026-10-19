//! Level names, controls, teams, the map maker's tiles and cursor, and more.

use crate::*;

/// @ 0x20f988
pub fn prefs_pageGfx() {
    prefs_vars();
}

/// Remember which item of a menu page is highlighted: the value at +16 of the page's current box, or 0 (@ 0x22de68).
pub fn menuitemHighlight(page: i32) {
    let id = lhu(page + 4);
    let item = frontGetBox(page, id);
    let front = lw(g::front);
    sw((lw(front) << 2) + g::highlightitem, if item == 0 { 0 } else { lw(item + 16) });
}

/// Counted the profiles once a second and did nothing (@ 0x2209d8).
pub fn debugsignon() {}

/// The memory card clock (BCD bytes: second +1, minute +2, hour +3, day +5, month +6, year +7) as a number of seconds, with months of 31 days and years of 12 months (@ 0x2b39c8).
pub fn clock2u32(clock: i32) -> i32 {
    let year = bcd2int(lbu(clock + 7));
    let month = bcd2int(lbu(clock + 6));
    let day = bcd2int(lbu(clock + 5));
    let hour = bcd2int(lbu(clock + 3));
    let minute = bcd2int(lbu(clock + 2));
    let second = bcd2int(lbu(clock + 1));
    year * 0x1ea6e00 + second + (hour * 3600 + day * 0x15180) + (month * 0x28de80 + minute * 60)
}

/// A controls menu item (40 bytes, from 0x1fb34e0) changed: store the new button for its action on the current player, and in the
/// player's saved settings when the configuration is the first (@ 0x20ec10).
pub fn controlOnChange(item: i32) {
    let index = (item - 0x1fb34e0).wrapping_mul(-0x33333333) >> 3;
    let action = lw((index << 2) + g::controlsorder);
    let rel = action - 10;
    let value = if (rel as u32) >= 8 {
        lw((lw(item) << 2) + g::gc_buttons)
    } else if rel % 2 == 0 {
        if lw(item) != 0 { 3 } else { 1 }
    } else if lw(item) == 0 {
        0
    } else {
        2
    };
    let player = lw(g::currentplayer);
    let at = action << 2;
    sw(player + at + 28, value);
    if lw(g::gv_config) == 0 {
        sw(lw(player + 24) + at + 4, value);
    }
}

/// Work out which of the 8 controller records (384 bytes, state at +272, 99 = ready) belong to players: the first four ready ones; returns how many are ready (@ 0x203030).
pub fn joyUpdatePorts() -> i32 {
    let mut ready = 0;
    for i in 0..8 {
        if lw(g::controllers + 272 + 384 * i) == 99 {
            if ready < 4 {
                sw(g::playercontrollers + 4 * ready, i);
            }
            ready += 1;
        }
    }
    ready
}

/// Begin adding glow points to the current depth test slot: 1 if there is room (72 tests), after clearing its 9 points (@ 0x2a6db0).
pub fn zbtestStartAddGlow() -> i32 {
    let slot = lw(0x3b0170) << 2;
    sw(g::CurrentAddIndex, 0);
    let tests = lw(slot + 0x1fea0a0);
    if tests >= 72 {
        return 0;
    }
    let first = lw(slot + g::zbval) + tests * 216;
    for i in 0..9 {
        sw(first + 24 * i, -1);
    }
    1
}

/// The mean of a statistic over the first `n` slots (@ 0x2213d8).
pub fn StatsGetAverage(stat: i32, n: i32) -> f32 {
    let mut sum = 0.0_f32;
    for slot in 0..n.max(0) {
        sum += StatsGet(stat, slot);
    }
    sum / n as f32
}

/// The text of the level list entry (24 bytes; text number at +0, level number at +4) of level `level` in the current language, or 0 (@ 0x2258b8).
pub fn lvGetNameFromNum(level: i32) -> i32 {
    let texts = lw((lw(g::cur_language) << 2) + g::languages);
    let text_of = |entry: i32| lw((lh(entry) << 2) + texts);
    let mut entry = LVINFO;
    if text_of(entry) == 0 {
        return 0;
    }
    while lw(entry + 4) != level {
        entry += 24;
        if text_of(entry) == 0 {
            return 0;
        }
    }
    text_of(entry)
}

/// The background colour of the level list entry of the current level (+20), or a default (@ 0x225bb0).
pub fn lvGetBgCol() -> i32 {
    let texts = lw((lw(g::cur_language) << 2) + g::languages);
    let text_of = |entry: i32| lw((lh(entry) << 2) + texts);
    if text_of(LVINFO) == 0 {
        return -0xffff80;
    }
    let level = lw(g::lvlevelnum);
    let mut i = 0;
    loop {
        let entry = LVINFO + i * 24;
        i += 1;
        if lw(entry + 4) == level {
            return lw(entry + 20);
        }
        if text_of(i * 24 + LVINFO) == 0 {
            return -0xffff80;
        }
    }
}

/// Set zoom box 0 to the memory card dialog's window: the screen's rectangle in eighths and quarters (@ 0x243f28).
pub fn mcseqWindowSet() {
    let rect = Frame::new(16);
    let r = rect.sp();
    let window = g::window;
    let height = lw(window + 12);
    let top = lw(window + 32);
    let x = height / 8;
    let width = lw(window + 4);
    let y = top / 4;
    sw(r, x);
    sw(r + 12, lw(window + 24) - y);
    sw(r + 8, width - x);
    sw(r + 4, y);
    zoomboxSet(0, r, 0x2880);
}

/// Reserve `count` tiles of `w` x `h` in the map maker's tile memory: a header of 116 bytes with the address of each tile; returns the header (@ 0x248f18).
pub fn mmAllocTile(w: i32, h: i32, count: i32) -> i32 {
    let used = lw(0x3b0024);
    let header = lw(0x3b0020) + used;
    sw(header + 12, header + 116);
    for i in 1..count {
        let previous = lw(header + 12 + 4 * (i - 1));
        sw(header + 12 + 4 * i, previous + w * h);
    }
    sw(0x3b0024, used + (w * h * count + 119 & -4));
    header
}

/// Go back one level in the map maker's cursor stack, putting the current position in the slot above (@ 0x24fde8).
pub fn mmPopCursorPos() {
    let n = lw(0x3b0060);
    if n > 0 {
        let front = lw(g::front);
        let base = 0x1fc4080;
        sw(n * 12 + base + 4, lw(front + 32));
        let cursor = lw(g::mmcursor);
        let y = lw(front + 36);
        sw(0x3b0060, n - 1);
        sw(n * 12 + base + 8, y);
        sw(n * 12 + base, cursor);
        let previous = lw((n - 1) * 12 + base);
        sw(front + 32, lw((n - 1) * 12 + base + 4));
        sw(g::mmcursor, previous);
        sw(front + 36, lw((n - 1) * 12 + base + 8));
    }
}

/// Clear the override records (24 bytes; two a part) of a prop (@ 0x274400).
pub fn nullOverridePointers(prop: i32) {
    let model = lw(prop + 32);
    let mut i = 0;
    while i < (lw(lw(model)) << 1) {
        let rec = i * 24 + lw(model + 248);
        i += 1;
        for at in [20, 0, 4, 8, 12, 16] {
            sw(rec + at, 0);
        }
    }
}

/// Whether a particle group is drawn for the current player: no hold (+52 is 0), its viewport mask (+48) lets this player see it, and its flags (+8) do not hide it (@ 0x2a18a0).
pub fn particleGroupIsVisible(group: i32, flags: i32) -> i32 {
    if lwc1(group + 52) == 0.0 {
        let mask = lw(group + 48);
        let player = lw(g::currentplayer);
        if mask != 0 && (mask >> (lw(player) & 31)) & 1 == 0 {
            return 0;
        }
        let f = lw(group + 8);
        if f & 4 != 0 && f & flags == 0 && f & 128 != 0 {
            return 0;
        }
        return 1;
    }
    0
}

/// A new external reference number for a prop: one more than the largest in the level (at least 1001) (@ 0x2692f0).
pub fn propGetNewExtref() -> i32 {
    let mut largest = 1000;
    for i in 0..lw(g::num_props) {
        let extref = lw(i * 592 + lw(g::levelprops));
        if extref != -1 && largest < extref {
            largest = extref;
        }
    }
    largest + 1
}

/// The first condition (20 bytes) of challenge `n` with flag 8, which gives its score, or 0 (@ 0x21f2b0).
pub fn challengeGetScoreCondition(n: i32) -> i32 {
    let setup = challengeGetSetupNum(n);
    let count = lw(setup + 20);
    let list = lw(setup + 16);
    for i in 0..count.max(0) {
        let cond = i * 20 + list;
        if lhu(cond) & 8 != 0 {
            return cond;
        }
    }
    0
}

/// Give a character its team number (+24) and colour (+20) for the mode being played (@ 0x288710).
pub fn chrSelectTeam(chr: i32) {
    let number = lw(chr);
    let params = g::curgameparms;
    let mode = lw(params + 72);
    if lw(g::lvlevelnum) != 25 && mode != 6 && mode != 7 {
        if mode > 0 && mode < 7 {
            if lw(params + 80) & 1 != 0 {
                let team = lbu(number + (params + 35));
                sw(chr + 24, team);
                sw(chr + 20, lw((team << 3) + (g::storydata + 68)));
                return;
            }
            sw(chr + 24, number + 1);
            sw(chr + 20, lw((number.wrapping_rem(14) << 2) + g::charcolours));
        }
        return;
    }
    sw(chr + 24, 5);
    sw(chr + 20, lw(g::teamdata + 36));
}

/// The viewport bits props are drawn for: one for each local player, from bit 24 (@ 0x268f00).
pub fn getPropViewportDrawFlags() -> i32 {
    match lw(g::lvnumplayers) {
        1 => 0x1000000,
        2 => 0x3000000,
        3 => 0x7000000,
        4 => 0xf000000,
        _ => 0,
    }
}

/// Whether all the bases of Last Stand are destroyed: there are some, and every one has its value at +532 below 1 (@ 0x21df90).
pub fn gmLastStandAllBasesDestroyed() -> i32 {
    let mut i = 0;
    while i < enemyGetNumBases() {
        if lwc1(enemyGetBase(i) + 532) < 1.0 {
            break;
        }
        i += 1;
    }
    (i != 0 && i == enemyGetNumBases()) as i32
}

/// Follow the route table from `start` to the entry whose pad is in hall `hall`; the value at +6 of that entry, or -1 (@ 0x266000).
pub fn hr_findroutetohall(start: i32, hall: i32) -> i32 {
    let mut at = lw(start);
    let pads = lw(g::levelpads);
    while at >= 0 {
        let entry = (at << 4) + 0x1fc6490;
        if lh(lh(entry + 4) * 28 + pads + 8) == hall {
            return lh(entry + 6);
        }
        at = lh(entry);
    }
    -1
}
