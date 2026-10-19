//! Which characters and teams a level uses.

use crate::*;

/// Whether team `team` (1 to 5, the team numbers the menus use) can be chosen: in modes 4 and 6 only team 1; otherwise the team must be allowed, and there
/// must be someone on another team, counting the 10 character slots at 0x1fc2a88 and the entries (0x3aff20 of them) at 0x1fc2c88 (@ 0x235608).
pub fn selectableTeam(team: i32) -> i32 {
    let menu = 0x1fc11f8;
    let mode = lw(menu + 48);
    if mode == 4 || mode == 6 {
        return (team == 1) as i32;
    }
    if lbu(team + (menu + 60) - 1) == 0 {
        return 0;
    }
    let entries = lw(0x3aff20);
    let mut others = 0;
    let mut skipped = 0;
    for i in 0..10 {
        let slot = 0x1fc2a88 + 16 * i;
        if lw(slot + 12) == 0 || lw(slot) < 0 {
            continue;
        }
        let t = lw(slot + 4);
        if lbu(t + menu + 60) != 0 && (t + 1 ^ team) != 0 {
            others += 1;
        }
    }
    for i in 0..entries.max(0) {
        let v = lw((i << 2) + 0x1fc2c88);
        if v < 0 {
            skipped += 1;
        } else if (v + 1 ^ team) != 0 {
            others += 1;
        }
    }
    if skipped == 0 { (0 < others) as i32 } else { 1 }
}

/// The characters (a record of the character presets, or a set of the game's) that level `level` uses; challenge 26 has a set of its own (@ 0x227568).
pub fn lvGetLevelChrTypeNums(level: i32) -> i32 {
    let params = g::curgameparms;
    if lw(params + 80) & 32 != 0 && lw(params + 68) == 26 {
        return g::challenge9c_chrset_100;
    }
    let v = level - 2;
    if (v as u32) < 104 {
        return match v {
            0 | 11 | 20 => g::chrpresets + 44,
            3 | 13 => g::chrpresets + 176,
            6 => g::chrpresets + 88,
            7 | 15 => g::chrpresets + 308,
            10 | 25 => g::chrpresets + 352,
            12 | 16 | 26 => g::chrpresets + 264,
            14 | 21 | 24 => g::chrpresets,
            19 => g::chrpresets + 132,
            22 => g::chrpresets + 220,
            _ => g::default_chrset_99,
        };
    }
    g::default_chrset_99
}
