//! The custom controls menu, an enemy's ambush, the colour page of the map maker, the check before a game starts, and the pad an enemy defends from.

use crate::*;

/// Make the items of the custom controls menu (records of 32 bytes at `controlsmenu_items`, ended by the type 0x8000 at +8; those with a type get flags 4 at +10 and a record of 40
/// bytes at 0x1fb34e0 at +12): for the controls 10 to 17 (the analog sticks) the record says whether the player's setting is 2 or more, for the others the index of the setting
/// among the game controller's buttons (`gc_buttons`); the drawing function 0x20ec10 is set at +20 of the record. The list's height (halfword at +2) is 5, or 3 when the window is
/// split (@ 0x20ece0).
pub fn customcontrols_makemenu() {
    let mut item = g::controlsmenu_items;
    let mut kind = lhu(item + 8);
    let mut index = 0;
    let window;
    if kind == 0x8000 {
        window = g::window;
    } else {
        loop {
            let next_item = item + 32;
            if kind != 0 {
                sh(item + 10, 4);
                let record = index * 40 + 0x1fb34e0;
                sw(item + 12, record);
                memset(record, 0, 40);
                let control = lw((index << 2) + g::controlsorder);
                let setting = lw(lw(g::currentplayer) + (control << 2) + 28);
                if control < 18 && control >= 10 {
                    sw(record, if (setting as u32) >= 2 { 1 } else { 0 });
                    let at = index * 40 + 0x1fb34e0;
                    sw(at + 8, 1);
                    sw(at + 16, g::disp_analogstick);
                } else {
                    let mut place = 0;
                    let first = lw(g::gc_buttons);
                    if first >= 0 && first != setting {
                        place += 1;
                        loop {
                            let button = lw((place << 2) + g::gc_buttons);
                            if button < 0 {
                                break;
                            }
                            if button == setting {
                                break;
                            }
                            place += 1;
                        }
                    }
                    let at = 0x1fb34e0 + index * 40;
                    sw(index * 40 + 0x1fb34e0, place);
                    sw(at + 16, g::disp_buttons);
                    sw(at + 8, 13);
                }
                let offset = index * 40;
                index += 1;
                sw(offset + 0x1fb34f4, 0x20ec10);
            }
            item = next_item;
            kind = lhu(item + 8);
            if kind == 0x8000 {
                break;
            }
        }
        window = g::window;
    }
    let height = if lw(window + 92) != 0 { 3 } else { 5 };
    sh(g::controls_list + 2, height);
}

/// The ambush from the right: the enemy `enemy` stays where it is (its place kept at +2796 of its record), and once it is triggered by a player's nearness it plays the animation
/// 447, or 448 when its gun has flag 2, to step out; when the animation is done, or when it is not triggered and becomes aware of its target, its state (+2716) goes to 2 (the
/// old state kept at +2720; flags 6 added to +2704 after the animation) (@ 0x2c86c8).
pub fn enemyAmbushRight(enemy: i32) {
    let record = lw(enemy + 352);
    enemySetMoveMode(enemy, 8);
    swc1(record + 2796, lwc1(enemy + 48));
    swc1(record + 2800, lwc1(enemy + 52));
    let z = lwc1(enemy + 56);
    sw(record + 2780, 0);
    swc1(record + 2804, z);
    if enemyTriggered(enemy) != 0 && curAnim(enemy, 447) == 0 && curAnim(enemy, 448) == 0 {
        if (lw(lw(record + 376) * 400 + (GUNS_TABLE + 28)) & 2) == 0 {
            setAnim(enemy, 447, 0.0_f32, 1.0_f32);
        } else {
            setAnim(enemy, 448, 0.0_f32, 1.0_f32);
        }
        return;
    }
    if curAnim(enemy, 447) == 0 && curAnim(enemy, 448) == 0 {
        enemySetMoveMode(enemy, 8);
        enemySetIdleAnimation(enemy);
        if awareOfTarget(enemy) == 0 {
            return;
        }
        let old = lw(record + 2716);
        sw(record + 2716, 2);
        sw(record + 2720, old);
    } else {
        if curAnimComplete(lw(enemy + 32)) == 0 {
            return;
        }
        let old = lw(record + 2716);
        let flags = lw(record + 2704) | 6;
        sw(record + 2716, 2);
        sw(record + 2704, flags);
        sw(record + 2720, old);
    }
    sw(record + 2892, 0);
}

/// The tick of the colour page of the map maker (`page`): with the button 16 newly pressed the colour (`lighthue`, `lightsaturation`, `lightvolume`) is stored in the current
/// light's colour and the page closed; otherwise a new press of 64 on the colour wheel (page box 1) or on the volume slider (box 2) starts that drag (flags 0x40000 or 0x80000 of
/// the edit flags), and while 64 is held the wheel's cursor sets hue and saturation from where it is (kept on the wheel's edge), or the slider's sets the volume (0 to 1); the
/// colour dialog is made (@ 0x2518a8).
pub fn mapmakercolour_pageTick(page: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let pressed = joyNewBut(0);
    let flags = lw(g::editflags);
    if (pressed & 16) != 0 {
        let hue = lwc1(g::lighthue);
        let saturation = lwc1(g::lightsaturation);
        let volume = lwc1(g::lightvolume);
        sw(g::editflags, flags & -0xc0001);
        let colour = hsv2col(hue, saturation, volume);
        let front = lw(g::front);
        sw(((lw(g::curlightcol) & 15) << 2) + 0x1fc4040, colour);
        sw(front + 20, lw(front + 20) - 1);
        return;
    }
    if (joyNewBut(0) & 64) != 0 {
        let flags = lw(g::editflags);
        let box_number = lhu(page + 4);
        let cleared = flags & -0xc0001;
        sw(g::editflags, cleared);
        let mut bit = 0x40000;
        let mut start = true;
        if box_number != 1 {
            bit = 0x80000;
            if box_number != 2 {
                start = false;
            }
        }
        if start {
            sw(g::editflags, cleared | bit);
        }
    }
    let held = joyBut(0);
    let flags = lw(g::editflags);
    if (held & 64) != 0 {
        let front = lw(g::front);
        if (flags & 0x40000) == 0 {
            if (flags & 0x80000) != 0 {
                sw(sp, g::loadbuf + 576);
                sw(sp + 4, 0x970000);
                sw(sp + 8, 0x10a0000);
                sw(sp + 12, 0x1280000);
                mmClampMouse(sp);
                let volume = ps2float::sub(1.0_f32, ((lh(lw(g::front) + 38) - 56) as f32).pdiv(96.0_f32));
                swc1(g::lightvolume, volume);
                if volume < 0.0_f32 {
                    swc1(g::lightvolume, 0.0_f32);
                }
                if 1.0_f32 < lwc1(g::lightvolume) {
                    swc1(g::lightvolume, 1.0_f32);
                }
            }
        } else {
            let y = ps2float::sub(ps2float::mul(lw(front + 36) as f32, 1.5258789e-05_f32), 103.0_f32);
            let x = ps2float::sub(ps2float::mul(lw(front + 32) as f32, 1.5258789e-05_f32), 1.6e+02_f32);
            let y2 = ps2float::add(y, y);
            let mut saturation = (ps2float::add(ps2float::mul(x, x), ps2float::mul(y2, y2)).abs().psqrt()).pdiv(92.0_f32);
            let hue = lwc1(g::lighthue);
            if 1.0_f32 < saturation {
                saturation = 1.0_f32;
                let scale = lwc1(0x3aca94);
                let front = lw(g::front);
                let angle = ps2float::add(ps2float::mul(ps2float::mul(hue, lwc1(0x3aca8c)).pdiv(1.8e+02_f32), lwc1(0x3aca90)), 0.5_f32) as i32;
                let new_x = (ps2float::add(ps2float::mul(ps2float::mul(lwc1(((angle + 512 & 2047) << 2) + g::g_SinTable), scale), 92.0_f32), 1.6e+02_f32) as i32) << 16;
                sw(front + 36, (ps2float::sub(103.0_f32, ps2float::mul(ps2float::mul(lwc1(((angle & 2047) << 2) + g::g_SinTable), scale), 46.0_f32)) as i32) << 16);
                sw(front + 32, new_x);
            }
            let turn = atan2f(y2, -x);
            let half = lwc1(0x3aca98);
            swc1(g::lightsaturation, saturation);
            swc1(g::lighthue, ps2float::mul(ps2float::add(turn, half), 1.8e+02_f32).pdiv(half));
        }
    }
    let dialog = makecolourdialog(lw(page + 8));
    sw(page + 12, dialog);
}

/// Whether a game may start from the set-up page, else show the message about what is missing and say 0: in the mode 4 a team of bots needs the other team to have bots as well
/// (texts at +1716 to +1720 of the language); in the other modes but 6 team play needs a team made (text +1724 for fewer than two players of 0x3aff20 without bots) and a map
/// that has items for at least two teams (+1728) (@ 0x235788).
pub fn checkOkToGo() -> i32 {
    let mode = lw(0x1fc1228);
    if mode == 4 {
        let message;
        if countBots(2) != 0 {
            if countBots(1) != 0 {
                return 1;
            }
            message = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1720);
        } else {
            let language = lw(g::cur_language);
            message = lw(lw((language << 2) + g::languages) + 1716);
        }
        frontStartMessage(0, message, 0x6464647f, 0);
        return 0;
    }
    if mode == 6 || lw(g::gv_teamplay) == 0 {
        return 1;
    }
    let state = 0x1fc11f8;
    if countBots(0) == 0 {
        let language = lw(g::cur_language);
        if lw(0x3aff20) < 2 {
            let message = lw(lw((language << 2) + g::languages) + 1724);
            frontStartMessage(0, message, 0x6464647f, 0);
            return 0;
        }
    }
    if lw(state + 36) != 18 {
        return 1;
    }
    if lw(lw(g::mmtempbuffer)) != 0x1324626 {
        return 1;
    }
    let kind = lw(state + 48);
    let team_starts = (kind == 3 || kind == 1) && lw(g::gv_teamplay) != 0 && lw(g::gv_useteamstarts) != 0;
    if !team_starts && lw(lw(state + 48) * 12 + (g::gamemodedata + 4)) == 0 {
        return 1;
    }
    let mut teams = 0;
    let mut team = 1;
    loop {
        let items = mmfileGetItemCount(lw(g::mmtempbuffer), team);
        teams = if items != 0 { teams + 1 } else { teams };
        let more = team < 4;
        team += 1;
        if !more {
            break;
        }
    }
    let language = lw(g::cur_language);
    if teams < 2 {
        let message = lw(lw((language << 2) + g::languages) + 1728);
        frontStartMessage(0, message, 0x6464647f, 0);
        return 0;
    }
    1
}

/// Choose the pad the enemy `enemy` defends from: the pads linked to the pad of the drop spawn that its record (+24) names, taken as a direction from the pad, are scored by how
/// far in front of the spawn's direction (turned by the pad's angle) they are, and the best one that the enemy is not on is the answer (-1 for none) (@ 0x2c0b90).
pub fn enemyChooseDefendPad(enemy: i32) -> i32 {
    let frame = Frame::new(192);
    let sp = frame.sp();
    let mut drop = -1;
    let drops = lw(g::numPickupDrops);
    let record = lw(enemy + 352);
    sd(sp + 96, ld(0x3a9d30));
    sw(sp + 104, lw(0x3a9d38));
    if drops > 0 {
        let wanted = lw(record + 24);
        if lw(lw(lw(g::dropspawn + 4) + 352) + 16) != wanted {
            let mut i = 1;
            loop {
                if i >= drops {
                    break;
                }
                if lw(lw(lw((i << 3) + (g::dropspawn + 4)) + 352) + 16) == wanted {
                    drop = i;
                    break;
                }
                i += 1;
            }
        } else {
            drop = 0;
        }
    }
    let mut best_pad = -1;
    let mut best = -1.0_f32;
    let mut i = 0;
    let pad = lw(lw(lw((drop << 3) + (g::dropspawn + 4)) + 352) + 36);
    let count = findpadslinked2pad(pad, sp, 20);
    let angle = ps2float::mul(lwc1(pad * 28 + lw(g::levelpads) + 24), lwc1(0x3ad8b4)).pdiv(1.8e+02_f32);
    matrixRotY(sp + 112, angle);
    matrixVecRotAligned(&[(sp + 112) as i64, (sp + 96) as i64]);
    if count > 0 {
        loop {
            let slot = sp + (i << 2);
            let pads = lw(g::levelpads);
            let from = pad * 28 + pads;
            let to = lw(slot) * 28 + pads;
            let dx = ps2float::sub(lwc1(to + 12), lwc1(from + 12));
            swc1(sp + 80, dx);
            let dy = ps2float::sub(lwc1(to + 16), lwc1(from + 16));
            swc1(sp + 84, dy);
            let dz = ps2float::sub(lwc1(to + 20), lwc1(from + 20));
            let inverse = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt());
            let nx = ps2float::mul(lwc1(sp + 80), inverse);
            let ny = ps2float::mul(lwc1(sp + 84), inverse);
            swc1(sp + 88, ps2float::mul(lwc1(sp + 88), inverse));
            swc1(sp + 80, nx);
            swc1(sp + 84, ny);
            let score = checkInFrontOfDir(sp + 96, sp + 80, 0.0_f32);
            if score == -1.0_f32 {
                i += 1;
            } else {
                let candidate = lw(slot);
                let on = enemyOnPad(enemy, candidate);
                i += 1;
                if on == 0 && best < score {
                    best_pad = lw(slot);
                    best = score;
                }
            }
            if i >= count {
                break;
            }
        }
    }
    best_pad
}
