//! The memory of the props, the text of a failed challenge, the water, and an enemy defending a pad.

use crate::*;

/// Get the memory of the props (up to 300 of 592 bytes, 20 on level 102), doors, cars, remotes, character remotes, heads, pick-ups and graphics pointers from the level area, clear the
/// 713 entries of the prop table (28 bytes) and of the table of walls and floors (16 bytes) and the 80 bytes at 0x1fd0d50, restart the wall memory and the props (@ 0x267020).
pub fn propReset() {
    let max = if (lw(g::lvlevelnum) ^ 102) == 0 { 20 } else { 300 };
    sw(g::max_props, max);
    let props = memAlloc(max * 592, 1);
    sw(g::levelprops, props);
    let doors = memAlloc(608, 1);
    sw(g::leveldoors, doors);
    let cars = memAlloc(560, 1);
    sw(g::levelcars, cars);
    let remotes = memAlloc(3504, 1);
    sw(g::levelremotes, remotes);
    let chr_remotes = memAlloc(2704, 1);
    sw(g::levelchrremotes, chr_remotes);
    let heads = memAlloc(1680, 1);
    sw(g::levelheads, heads);
    let pickups = memAlloc(0x3840, 1);
    sw(g::levelpickups, pickups);
    let pointers = memAlloc(0x12c0, 1);
    sw(g::levelgfxpointers, pointers);
    let mut left = 712;
    let mut at = PROPTABLE + 20;
    loop {
        sw(at - 16, 0);
        left -= 1;
        sw(at, 0);
        at += 28;
        if left < 0 {
            break;
        }
    }
    let mut left = 712;
    let mut at = PWFTAB;
    loop {
        sw(at, 0);
        left -= 1;
        sw(at + 8, 0);
        at += 16;
        if left < 0 {
            break;
        }
    }
    propResetWallMem();
    propRestart();
    let mut left = 79;
    let mut at = 0x1fd0d9f;
    loop {
        sb(at, 0);
        left -= 1;
        at -= 1;
        if left < 0 {
            break;
        }
    }
}

/// Make the texts of the challenge's failed screen into the three outputs: `*title`, `*score` and `*reason` (all cleared first). The escort mode shows the reason (+2660 of the
/// language) with the escort's team when it is dead; for challenges 0, 1, 8 to 11, 15 and 17 the display score, for 12 to 14 (survival) the player's rank and kills, and if nothing was
/// written the language's text at +1852 and the formatted score (@ 0x21f540).
pub fn challengeGetDisplayFailed(title: i32, score: i32, reason: i32) {
    let parms = g::curgameparms;
    sw(title, 0);
    sw(score, 0);
    let mode = lw(parms + 72);
    sw(reason, 0);
    if mode == 4 {
        let escort = lw(g::escort);
        let language = lw(g::cur_language);
        if lwc1(escort + 520) <= 0.0_f32 {
            let team = lw(lw(escort + 352) + 16);
            let text = vafmt(&[0x3ae5c0, team as i64, lw(lw((language << 2) + g::languages) + 2660) as i64]);
            sw(reason, text);
        }
        return;
    }
    let kind = lw(parms + 68);
    if (kind as u32) < 18 {
        match kind {
            0 | 1 | 8 | 9 | 10 | 11 | 15 | 17 => {
                challengeGetDisplayScore(title, score);
            }
            12 | 13 | 14 => {
                let characters = lw(g::chrinfo);
                let language = lw(g::cur_language);
                if !(lwc1(lw(characters + 0x1ddc) + 520) <= 0.0_f32) {
                    let mut language_now = lw(g::cur_language);
                    let mut characters = characters;
                    let mut dead_end = false;
                    if lw(g::curgameparms + 68) == 14 {
                        let kills = chrCountKills(characters);
                        characters = lw(g::chrinfo);
                        if kills >= 4 {
                            sw(reason, lw(lw((lw(g::cur_language) << 2) + g::languages) + 2668));
                            dead_end = true;
                        } else {
                            language_now = lw(g::cur_language);
                        }
                    }
                    if !dead_end {
                        let languages = g::languages;
                        let key = lw(characters + 24);
                        sw(title, lw(lw((language_now << 2) + languages) + 1852));
                        let points = teamGetScore(key);
                        let suffix = lw(lw((lw(g::cur_language) << 2) + languages) + 2672);
                        let text = vafmt(&[(g::gameranksizechr + 4) as i64, points as i64, suffix as i64]);
                        sw(score, text);
                    }
                } else {
                    sw(reason, lw(lw((language << 2) + g::languages) + 2664));
                }
            }
            _ => {}
        }
    }
    let written = lw(reason);
    if written == 0 && lw(title) == 0 && lw(score) == 0 {
        sw(title, lw(lw((lw(g::cur_language) << 2) + g::languages) + 1852));
        let points = challengeGetScore();
        let text = challengeFormatScore(lw(g::curgameparms + 68), points);
        sw(score, text);
    }
}

/// The tick of the water: the two override chunk lists (the points and the texture coordinates) of every part of the model are taken and made by `vtxlistWaterxyz` and
/// `vtxlistWaterst`, and the model's texture is chosen by the game's time (16 pictures, one every 5 ticks) (@ 0x278bb0).
pub fn waterPropTick(prop: i32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let mut i = 0;
    let mut model = lw(prop + 32);
    let skeleton = lw(model);
    let count = lw(skeleton);
    let mut bones = skeleton - count * 80;
    let mut skeleton_now = skeleton;
    if count > 0 {
        loop {
            allocateOverrideChunk(prop, i, 1, 1);
            allocateOverrideChunk(prop, i, 1, 2);
            i += 1;
            model = lw(prop + 32);
            skeleton_now = lw(model);
            if i >= lw(skeleton_now) {
                break;
            }
        }
    }
    let _ = skeleton_now;
    let mut j = 0;
    let mut ticks = lw(g::gameframe);
    if lw(lw(model)) > 0 {
        loop {
            let points = getOverrideChunk(prop, j, 1, 1, sp);
            if points != 0 {
                vtxlistWaterxyz(lw(bones + 48), points, lw(sp), 1);
            }
            let coordinates = getOverrideChunk(prop, j, 1, 2, sp);
            if coordinates != 0 {
                vtxlistWaterst(lw(bones + 52), coordinates, lw(sp));
            }
            model = lw(prop + 32);
            j += 1;
            bones += 80;
            if j >= lw(lw(model)) {
                break;
            }
        }
        ticks = lw(g::gameframe);
    }
    let step = ticks.wrapping_div(5);
    let picture = step - (((if -1 < step { step } else { step + 15 }) >> 4) << 4);
    sw(lw(lw(model) + 12), lw((picture << 4) + g::watertexinfo));
}

/// What the enemy `enemy` does to defend a pad: it chooses one (`enemyChooseDefendPad`) when it has none (and goes to state 128 when there is none) and walks towards it along the
/// pads (`findNextPad`; -2 sends it to state 128); at the pad it stands (move mode 8) and takes a target (`enemyCycleThroughTargets`, or the state 2 when it has one that sees it);
/// otherwise it follows the route and, with its time past the constant at 0x3ad8b0 and flag 1 of its flags, goes to state 256 (@ 0x2c0908).
pub fn enemyDefend(enemy: i32) {
    let record = lw(enemy + 352);
    let old_pad = lw(record + 684);
    let mut pad = old_pad;
    let new_state;
    let old_state;
    if old_pad == -1 {
        pad = enemyChooseDefendPad(enemy);
        sw(record + 684, pad);
        if pad == -1 {
            old_state = lw(record + 2716);
            new_state = 128;
            sw(record + 2716, new_state);
            sw(record + 2720, old_state);
            sw(record + 2892, 0);
            return;
        }
    }
    let distance_index = lw(g::updatedistindex);
    if pad != old_pad {
        if distance_index != lw(record) {
            sw(record + 684, old_pad);
        } else {
            sw(record + 1688, 0);
            sw(record + 2692, 0);
            sw(record + 684, old_pad);
        }
    } else {
        sw(record + 684, old_pad);
    }
    let next = findNextPad(record + 680);
    if next != -2 {
        if next == -1 && lw(record + 680) == lw(record + 684) {
            enemySetMoveMode(enemy, 8);
            if lw(record + 2788) != 0 {
                if lw(record + 2772) != 0 {
                    let old = lw(record + 2716);
                    sw(record + 2716, 2);
                    sw(record + 2720, old);
                    sw(record + 2892, 0);
                    return;
                }
                if lw(record) != lw(g::updatedistindex) {
                    return;
                }
            }
            let target = enemyCycleThroughTargets(enemy);
            if target != 0 {
                sw(record + 2788, target);
            }
            return;
        }
        enemySetMoveMode(enemy, 1);
        let pads = lw(g::levelpads);
        swc1(record + 2872, lwc1(record + 2856));
        let range = lwc1(0x3ad8ac);
        let at = next * 28 + pads;
        let px = lwc1(at + 12);
        swc1(record + 2796, px);
        let py = lwc1(at + 16);
        swc1(record + 2800, py);
        let pz = lwc1(at + 20);
        swc1(record + 2804, pz);
        let dx = ps2float::sub(lwc1(enemy + 48), px);
        let dy = ps2float::sub(lwc1(enemy + 52), py);
        let dz = ps2float::sub(lwc1(enemy + 56), pz);
        if ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)) < range {
            let count = lw(record + 1688);
            sw(record + 680, next);
            sw(record + 1688, count - 1);
        }
        let time = lwc1(record + 2892);
        if !(lwc1(0x3ad8b0) < time) {
            return;
        }
        if (lw(record + 2704) & 1) == 0 {
            return;
        }
        old_state = lw(record + 2716);
        new_state = 256;
    } else {
        old_state = lw(record + 2716);
        new_state = 128;
    }
    sw(record + 2716, new_state);
    sw(record + 2720, old_state);
    sw(record + 2892, 0);
}
