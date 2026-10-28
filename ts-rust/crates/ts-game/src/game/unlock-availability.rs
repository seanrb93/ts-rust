//! Whether levels, characters, weapon sets, bot sets and difficulties are available (unlocked).

use crate::*;

/// The pad's external reference: the number `reference` itself, or, when it is negative, a pad chosen by its flags (@ 0x228be8).
pub fn lvGetPadExtrefFromData(reference: i32) -> i32 {
    let mut result = reference;
    if result < 0 {
        result = lvGetPseudoRandomPadExtref(reference);
    }
    result
}

/// Whether challenge mode is unlocked (@ 0x225010).
pub fn challengeModeAvail() -> i32 {
    unlockedEx(g::unlock_challengemode, 0)
}

/// Whether character `chr` can be played: always with the flag at +1 of the player array, else when its unlock item (16 bytes each) is unlocked (@ 0x224bb0).
pub fn chrPlayable(chr: i32) -> i32 {
    // (a character that a mod added is always playable; the table of unlocks has the game's characters only)
    if crate::manual::chartable::is_added(chr) {
        return 1;
    }
    // (BlueRobot has no front end model: he cannot be chosen as a player; when he was playable every texture of the front end broke)
    #[cfg(feature = "extensions")]
    if chr == 79 {
        return 0;
    }
    let item = chr << 4;
    if lw(item + g::unlock_playable_chrs) != 0 && lb(g::pcarray + 1) != 0 {
        return 1;
    }
    unlockedEx(item + g::unlock_playable_chrs, 0)
}

/// Whether weapon set `set` is available: all are except the one numbered 8 (9 + 8 = 17), which needs its item (kept at 0x3a1b70) unlocked (@ 0x232d90).
pub fn weaponsetAvail(set: i32) -> i32 {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let b = ld(0x3a1b70 + 8);
    sd(sp, ld(0x3a1b70));
    sd(sp + 8, b);
    let mut result = 1;
    if set + 9 == 17 {
        result = unlockedEx(sp, 0);
    }
    result
}

/// Whether every item of the list `set` (a count, then items of 16 bytes) is unlocked (@ 0x225138).
pub fn unlockedset(set: i32) -> i32 {
    let mut i = 0;
    if lw(set) > 0 {
        let mut item = set + 4;
        loop {
            if unlockedEx(item, 0) == 0 {
                return 0;
            }
            let count = lw(set);
            i += 1;
            item += 16;
            if i >= count {
                break;
            }
        }
    }
    1
}

/// Whether the difficulty `difficulty` is available for the story part `part` (from 3 up, the second and third parts of the story: the three levels of the part, from the story
/// data, must all be unlocked on that difficulty, for the saved or, with two players or more, the current statistics); always for the first parts and with the player array flag 0 set otherwise (@ 0x23be08).
pub fn storyDifficultyAvail(part: i32, difficulty: i32) -> i32 {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let flag = lb(g::pcarray);
    let b = ld(0x3a1dd8 + 8);
    sd(sp, ld(0x3a1dd8));
    sd(sp + 8, b);
    let mut result = 1;
    if flag == 0 && part >= 3 {
        let mut level = if 5 < part { 3 } else { 0 };
        let end = level + 3;
        let mut more;
        loop {
            more = level < end;
            if !more {
                break;
            }
            let current = (lw(0x3aff20) < 2) as i32 ^ 1;
            sw(sp + 4, lh((level << 3) + (g::storydata + 2)) << 16 | difficulty & 0xffff);
            let unlocked = unlockedEx(sp, current);
            level += 1;
            if unlocked == 0 {
                break;
            }
        }
        result = more as i32 ^ 1;
    }
    result
}

/// Whether bot set `set` (0 to 7) is available: the first always, the others when their item is unlocked (@ 0x224f20).
pub fn botsetAvail(set: i32) -> i32 {
    if (set as u32) >= 8 {
        return 0;
    }
    match set {
        0 => 1,
        1 => unlockedEx(g::unlock_botset1, 0),
        2 => unlockedEx(g::unlock_botset2, 0),
        3 => unlockedEx(g::unlock_botset3, 0),
        4 => unlockedEx(g::unlock_botset4, 0),
        5 => unlockedEx(g::unlock_botset5, 0),
        6 => unlockedEx(g::unlock_botset6, 0),
        7 => unlockedEx(g::unlock_botset7, 0),
        _ => unreachable!(),
    }
}

/// Whether story level `level` is available: with the player array flag 0 set, levels 2, 5 and 9 need the item of the first part and levels 8, 24 and 27 that of the second;
/// all others always (@ 0x224d58).
pub fn levelAvailSTORY(level: i32, current: i32) -> i32 {
    if lb(g::pcarray) == 0 {
        let offset = level - 2;
        if (offset as u32) < 26 {
            match offset {
                0 | 3 | 7 => return unlockedEx(g::unlock_story123, current),
                6 | 22 | 25 => return unlockedEx(g::unlock_story456, current),
                _ => {}
            }
        }
    }
    1
}
