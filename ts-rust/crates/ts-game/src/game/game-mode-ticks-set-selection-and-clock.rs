//! Small functions: the clock, the level reset, the character and weapon set selection and two game mode ticks.

use crate::*;

/// Read the console clock into the hours (`a0`), minutes (`a1`) and seconds (`a2`): all three are 0 when the clock cannot be read (@ 0x2b3b88).
pub fn getHoursMinutesSeconds(a0: i32, a1: i32, a2: i32) {
    let frame = Frame::new(96);
    let sp = frame.sp();
    sw(a0, 0);
    sw(a1, 0);
    sw(a2, 0);
    let read = sceCdReadClock(&[sp as i64, a1 as i64, a2 as i64]);
    let hours = lbu(sp + 3);
    if read != 0 {
        let h = bcd2int(hours);
        let minutes = lbu(sp + 2);
        sw(a0, h.wrapping_rem(24));
        let m = bcd2int(minutes);
        let seconds = lbu(sp + 1);
        sw(a1, m.wrapping_rem(60));
        let s = bcd2int(seconds);
        sw(a2, s.wrapping_rem(60));
    }
}

/// Reset everything before a level: the game number is 1 for the level 101 and 2 otherwise, and the parts reset in the order the game does (@ 0x225cd8).
pub fn lvResetBefore() {
    let game = if lw(g::lvlevelnum) != 101 { 2 } else { 1 };
    sw(g::lvgamenum, game);
    packfileReset();
    animReset();
    memdbReset(0);
    emReset();
    preloadReset();
    zbtestReset();
    if ((lw(g::lvlevelnum) - 102) as u32) < 2 {
        mcardReset();
    }
    timerReset();
    obReset();
    propReset();
    chrReset();
    camReset();
    windowReset();
    frontReset();
    enemyReset();
    soundReset();
    bglightReset();
}

/// Preload the fonts: three always, and two more except in the level 104 (@ 0x201fa8).
pub fn textPreload() {
    preloadAddTexName(lw(g::s_font07), 0);
    preloadAddTexName(lw(g::s_newsfont), 0);
    preloadAddTexName(lw(g::s_squarefont), 0);
    if lw(g::lvlevelnum) == 104 {
        return;
    }
    preloadAddTexName(lw(g::s_font09), 0);
    preloadAddTexName(lw(g::s_itckabelfont), 0);
}

/// Work out the angles of the clock hands from the time of the day: the hours, minutes and seconds are kept, and the angles in radians are made from the 12 hour, 60 minute
/// and 60 second turns (@ 0x279240).
pub fn calculateClockTime() {
    let frame = Frame::new(32);
    let sp = frame.sp();
    getHoursMinutesSeconds(sp, sp | 4, sp | 8);
    let hours = lw(sp);
    let minutes = lw(sp + 4);
    let seconds = lw(sp + 8);
    let sixty = 6e+01_f32;
    let turn = 3.6e+02_f32;
    let pi = lwc1(0x3aceb0);
    let half_turn = 1.8e+02_f32;
    sw(0x3b0104, hours);
    sw(0x3b0108, minutes);
    sw(0x3b010c, seconds);
    swc1(0x3b00f8, hours as f32 / 12.0_f32 * turn * pi / half_turn);
    swc1(0x3b0100, seconds as f32 / sixty * turn * pi / half_turn);
    swc1(0x3b00fc, minutes as f32 / sixty * turn * pi / half_turn);
}

/// Step the character set setting down (wrapping from 0 to 10) until one that is available is found (sets 8 and above always are), with the sound 141 (@ 0x234510).
pub fn chrsetDec() {
    soundStart(141, 0);
    let mut effective = lw(g::gv_chrset);
    loop {
        let value = effective - 1;
        sw(g::gv_chrset, value);
        if value < 0 {
            sw(g::gv_chrset, 10);
        }
        let set = lw(g::gv_chrset);
        if set >= 8 {
            break;
        }
        let available = botsetAvail(set);
        effective = lw(g::gv_chrset);
        if available != 0 {
            break;
        }
    }
}

/// Step the character set setting up (wrapping from 10 to 0) until one that is available is found (@ 0x234588).
pub fn chrsetInc() {
    soundStart(141, 0);
    let mut effective = lw(g::gv_chrset);
    loop {
        let value = effective + 1;
        sw(g::gv_chrset, value);
        if value >= 11 {
            sw(g::gv_chrset, 0);
        }
        let set = lw(g::gv_chrset);
        if set >= 8 {
            break;
        }
        let available = botsetAvail(set);
        effective = lw(g::gv_chrset);
        if available != 0 {
            break;
        }
    }
}

/// Preload the water textures in the levels 2, 21 and 24 (sixteen of them) (@ 0x277b00).
pub fn specialPreload() {
    let level = lw(g::lvlevelnum);
    if level != 2 {
        if level != 21 {
            if level != 24 {
                return;
            }
        }
    }
    let mut at = g::watertexnames;
    for _ in 0..16 {
        let name = lw(at);
        at += 4;
        preloadAddTexName(name, 0);
    }
}

/// Change the prop being edited by `a1` steps (the number is wrapped through 0 to 713 and only valid prop numbers count): the old edit prop is deleted and the new one
/// created where the old was, and the number is kept (@ 0x27c400).
pub fn propEditChange(a0: i32, a1: i32) {
    let position = lw(a0);
    let mut number = lw(a0 + 4) + a1;
    loop {
        if propEditValidPropnum(number) != 0 {
            break;
        }
        let next = number + a1;
        let low = if -1 >= next { 713 } else { next };
        number = if low >= 714 { 0 } else { low };
    }
    propEditDelete(a0);
    propEditCreate(number, position);
    sw(g::lasteditpropnum, number);
}

/// The tick of the capture the flag game: the level is completed when a team's score reaches the goal (the number at +96 of the game parameters, when it is not 0), or when a
/// combo is unlocked (@ 0x21d938).
pub fn gmCtfTick() {
    'b1: {
        let parms = g::curgameparms;
        if lw(parms + 96) != 0 {
            let mut team = 0;
            loop {
                let next = team + 1;
                if team >= 5 {
                    break 'b1;
                }
                let score = teamGetScore(next);
                team = next;
                if score == lw(parms + 96) {
                    break;
                }
            }
            gameLevelCompleted();
        }
    }
    if UnlockCombo() == 0 {
        return;
    }
    gameLevelCompleted();
}

/// The tick of the story game: the level is completed (the flag 0x10000000 of the game parameters set) when a character has a positive count at +248, or when a combo is
/// unlocked (@ 0x21d3d0).
pub fn gmStoryTick() {
    let mut i = 0;
    let players = lw(g::lvnumplayers);
    let chars = lw(g::numactivechars);
    'b3: {
        loop {
            if i >= players + chars {
                break 'b3;
            }
            if lw(i * 0x1210 + lw(g::chrinfo) + 248) > 0 {
                break;
            }
            i += 1;
        }
        let parms = g::curgameparms;
        sw(parms + 80, lw(parms + 80) | 0x10000000);
        gameLevelCompleted();
    }
    if UnlockCombo() == 0 {
        return;
    }
    let parms = g::curgameparms;
    sw(parms + 80, lw(parms + 80) | 0x10000000);
    gameLevelCompleted();
}

/// Whether a position is outdoors: a line test from `a1` to `a0` hits nothing (@ 0x2ae1d8).
pub fn specialfxIsOutdoors(a0: i32, a1: i32) -> i32 {
    let frame = Frame::new(48);
    let sp = frame.sp();
    sw(sp, 0);
    sw(sp + 8, 0);
    let t4 = lw(0x3a9800 + 8);
    sd(sp + 16, ld(0x3a9800));
    sw(sp + 24, t4);
    let hit = bgLineTest(a1, a0, sp + 16, 1, 0, 0, 0, 4089);
    ((hit as u32) < 1) as i32
}

/// Step the weapon set setting down (wrapping from 0 to 9) until one that is available is found (set 0 always is), with the sound 141 (@ 0x232df0).
pub fn weaponSetDec() {
    soundStart(141, 0);
    let mut effective = lw(g::gv_weaponset);
    loop {
        let value = effective - 1;
        sw(g::gv_weaponset, value);
        if value < 0 {
            sw(g::gv_weaponset, 9);
        }
        let set = lw(g::gv_weaponset);
        if set == 0 {
            break;
        }
        let available = weaponsetAvail(set - 1);
        effective = lw(g::gv_weaponset);
        if available != 0 {
            break;
        }
    }
}

/// Step the weapon set setting up (wrapping from 9 to 0) until one that is available is found (@ 0x232e68).
pub fn weaponSetInc() {
    soundStart(141, 0);
    let mut effective = lw(g::gv_weaponset);
    loop {
        let value = effective + 1;
        sw(g::gv_weaponset, value);
        if value >= 10 {
            sw(g::gv_weaponset, 0);
        }
        let set = lw(g::gv_weaponset);
        if set == 0 {
            break;
        }
        let available = weaponsetAvail(set - 1);
        effective = lw(g::gv_weaponset);
        if available != 0 {
            break;
        }
    }
}
