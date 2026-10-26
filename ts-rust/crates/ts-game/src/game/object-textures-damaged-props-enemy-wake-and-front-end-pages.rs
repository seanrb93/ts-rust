//! Functions: the precalculation of an object's drawing data, loading its textures, setting up the props that can be damaged, an enemy waking up, and the number of players,
//! controls set-up, arcade, challenge and story pages of the front end.

use super::particle_spawners::random_as_float;
use crate::*;

/// Precalculate the drawing data of the object `a0`: four arrays with a word for each part are taken, and for each part the VIF and DMA data of both its sides is made
/// (@ 0x25fb88).
pub fn obPrecalcGfxData(a0: i32) {
    let mask = -16;
    let first = memAlloc((lw(a0) << 2) + 15 & mask, 1);
    let count = lw(a0);
    sw(a0 + 20, first);
    let second = memAlloc((count << 2) + 15 & mask, 1);
    let count = lw(a0);
    sw(a0 + 24, second);
    let third = memAlloc((count << 2) + 15 & mask, 1);
    let count = lw(a0);
    sw(a0 + 28, third);
    let fourth = memAlloc((count << 2) + 15 & mask, 1);
    let parts = lw(a0);
    sw(a0 + 32, fourth);
    if parts > 0 {
        let mut part = 0;
        loop {
            partPrecalcVifData(a0, part, 0);
            partPrecalcVifData(a0, part, 1);
            partPrecalcDmaData(a0, part, 0);
            partPrecalcDmaData(a0, part, 1);
            part += 1;
            if part >= lw(a0) {
                break;
            }
        }
    }
}

/// Load the textures of the object `a0`: an object that does not share its textures has each loaded by number; a shared one has its table of textures (16 bytes each, ended
/// by -1) copied to its own memory and each loaded into the first free texture buffer (@ 0x25ebe0).
pub fn obLoadTextures(a0: i32) {
    let textures = lw(a0 + 12);
    if (lw(textures + 12) & 0x10000000) == 0 {
        sw(a0 + 40, 0);
        sw(a0 + 16, 0);
        if lw(textures) != -1 {
            let mut entry = textures;
            let mut i = 1;
            loop {
                emLoadTextureNum(lw(entry), entry, entry);
                entry = (i << 4) + textures;
                i += 1;
                if lw(entry) == -1 {
                    break;
                }
            }
        }
    } else {
        let mut count = 0;
        let kept;
        let after;
        if lw(textures) == -1 {
            kept = lw(a0 + 16);
            after = count + 1;
        } else {
            kept = lw(a0 + 16);
            loop {
                count += 1;
                if lw((count << 4) + textures) == -1 {
                    break;
                }
            }
            after = count + 1;
        }
        let size = after << 4;
        if kept == 0 {
            let memory = memAlloc(size, 1);
            sw(a0 + 16, memory);
        }
        bcopy(textures, lw(a0 + 16), size);
        sw(a0 + 40, -1);
        let copy = lw(a0 + 16);
        let mut buffer = 0;
        let mut at = buffer << 2;
        loop {
            let used = lw(at + g::tbuffers);
            let next = buffer + 1;
            if used == 0 {
                sw(a0 + 40, next);
            }
            buffer = next;
            at = buffer << 2;
            if buffer >= 4 {
                break;
            }
        }
        emSelectTexBuffer(lw(a0 + 40));
        sw(((lw(a0 + 40) - 1) << 2) + g::tbuffers, 1);
        if lw(copy) != -1 {
            let mut entry = copy;
            let mut i = 1;
            loop {
                emLoadTextureAddr(lw(entry), entry, entry);
                entry = (i << 4) + copy;
                i += 1;
                if lw(entry) == -1 {
                    break;
                }
            }
        }
        emSelectTexBuffer(0);
    }
    sw(a0 + 36, 1);
}

/// Set up a prop that can be damaged by its number: the control panel (287) gets the press function; the drums (194, 197), crates (195, 200, 288, 293), the glass props
/// (201, 202) and the 291 and 292 props get their damage function (at +400) and model flags, and the auto and deer guns (420, 423) get a linked prop as well (@ 0x268d10).
pub fn initDamagedProps(a0: i32) {
    let kind = lw(a0 + 4);
    if kind == 287 {
        let model = lw(a0 + 32);
        let flags = lhu(model + 292);
        let prop_flags = lw(a0 + 16) | 4;
        sw(g::controlPanelActivated, 0);
        sw(a0 + 16, prop_flags);
        sh(model + 292, flags | 4);
        sw(a0 + 384, 0x275170);
        return;
    }
    let (function, bits) = match kind {
        194 | 197 => (0x2748a8, 4),
        195 | 200 | 288 | 293 => (0x274998, 2),
        201 | 202 => (0x274bb8, 2),
        291 | 292 => (0x274bb0, 2),
        420 | 423 => {
            let (function, linked) = if kind == 420 { (0x274e50, 421) } else { (0x274fe0, 424) };
            let x = lwc1(a0 + 48);
            let flags = lw(a0 + 16) | 16;
            let y = lwc1(a0 + 52);
            let z = lwc1(a0 + 56);
            let angle = lwc1(a0 + 76);
            sw(a0 + 16, flags);
            sw(a0 + 400, function);
            let prop = propNew(linked, x, y, z, angle);
            let linked_flags = lw(prop + 16);
            sw(a0 + 536, prop);
            sw(prop + 16, linked_flags & -34);
            return;
        }
        _ => return,
    };
    let model = lw(a0 + 32);
    let flags = lw(a0 + 16);
    let model_flags = lhu(model + 292);
    sw(a0 + 400, function);
    sw(a0 + 16, flags | 16);
    sh(model + 292, model_flags | bits);
}

/// An enemy wakes up: it goes after its target (the move mode 8, and 4 as well for the state 0x80000), the target's position is kept, a hit while it was hurt (flag 2048) or a
/// target it is aware of puts it back in the state it came from, and with several players it may switch to the other player as its target; then it takes its idle animation
/// (@ 0x2cb3e0).
pub fn enemyAwake(a0: i32) {
    let record = lw(a0 + 352);
    let target;
    if lw(record + 2724) != 0x80000 {
        enemySetMoveMode(a0, 8);
        target = lw(record + 2788);
    } else {
        enemySetMoveMode(a0, 8);
        enemySetMoveMode(a0, 4);
        target = lw(record + 2788);
    }
    'b3: {
        let mut players = lw(g::players);
        if target != 0 {
            let flags = lw(record + 2704);
            swc1(record + 2796, lwc1(target + 48));
            swc1(record + 2800, lwc1(target + 52));
            swc1(record + 2804, lwc1(target + 56));
            if (flags & 2048) != 0 {
                if lw(record + 2772) != 0 {
                    let state = lw(record + 2716);
                    let previous = lw(record + 2724);
                    sw(record + 2724, 0);
                    sw(record + 2892, 0);
                    sw(record + 2704, flags & -2049);
                    sw(record + 2720, state);
                    sw(record + 2716, previous);
                    enemyInformOthers(a0);
                    return;
                }
            }
            if awareOfTarget(a0) != 0 {
                let state = lw(record + 2716);
                let previous = lw(record + 2724);
                sw(record + 2724, 0);
                sw(record + 2892, 0);
                sw(record + 2720, state);
                sw(record + 2716, previous);
                enemyInformOthers(a0);
                return;
            }
            let current = lw(record + 2788);
            let count = lw(g::lvnumplayers);
            if current == 0 {
                players = lw(g::players);
            } else {
                players = lw(g::players);
                if count >= 2 {
                    let flags = lw(record + 2704);
                    if (flags & 0x4000) == 0 {
                        break 'b3;
                    }
                    if lw(record + 2772) != 0 {
                        break 'b3;
                    }
                    let new_target;
                    'b6: {
                        let info = lw(g::chrinfo);
                        if lw(lw(current + 352)) == 1 {
                            let first = lw(g::players);
                            if lw(info + 2708) != 32 {
                                new_target = lw(first + 384);
                                break 'b6;
                            }
                        }
                        let second = lw(g::players);
                        if lw(info + 0x1ca4) == 32 {
                            break 'b3;
                        }
                        new_target = lw(second + 2204);
                    }
                    sw(record + 2704, flags & -0x4001);
                    sw(record + 2788, new_target);
                    break 'b3;
                }
            }
        }
        sw(record + 2788, lw(players + 384));
    }
    let kind = lh(record + 4);
    if kind == 31 {
        if lw(record + 2724) == 0x2000000 {
            return;
        }
    }
    enemySetIdleAnimation(a0);
}

/// The tick of the number of players page: the pads are scanned; the count of players is limited by the pads present and the items for more than that are dimmed; the buttons
/// 64 and 2112 choose the count and go on to the sign-on page; otherwise the menu ticks (@ 0x23fd70).
pub fn nplayers_pageTick(a0: i32) {
    let ports = joyUpdatePorts();
    frontfxTick(1);
    pageRoot();
    pageSet(0, g::nplayers_page);
    demoplaybackTick();
    let mut left = 3;
    let mut item = g::i0100;
    loop {
        left -= 1;
        sh(item + 10, lhu(item + 10) & 0xffbf);
        item += 32;
        if left < 0 {
            break;
        }
    }
    let mut i = 0;
    let base = g::i0100;
    let mut under_three = (i < 3) as i32;
    loop {
        let offset = i << 5;
        let next;
        if under_three == 0 {
            next = i + 1;
        } else {
            next = i + 1;
            let at = offset + base;
            if next >= ports {
                sh(at + 22, 0);
            } else {
                sh(at + 22, i + 2);
            }
        }
        if i < ports {
            i = next;
        } else {
            let at = (i << 5) + base;
            sh(at + 10, lhu(at + 10) | 64);
            i = next;
        }
        under_three = (i < 3) as i32;
        if i >= 4 {
            break;
        }
    }
    if ports < lhu(a0 + 4) {
        sh(a0 + 4, ports);
    }
    if lhu(a0 + 4) == 0 {
        sh(a0 + 4, 1);
    }
    let finished = frontfxFinished(1);
    let player = lw(g::currentplayer);
    if finished != 0 {
        let pressed = joyNewBut(lw(player));
        let mut front = lw(g::front);
        if (pressed & 2048) != 0 {
            emPrintStats();
            front = lw(g::front);
        }
        let pressed = joyNewBut(lw(front));
        if (pressed & 2112) != 0 {
            let selected = lhu(a0 + 4);
            if ((selected - 1) as u32) < (ports as u32) {
                sw(0x3aff20, selected);
                sh(g::nplayers_page + 8, lhu(0x3aff20));
                frontPageSet(0, g::signon_page);
                pageIn(a0);
                setStates(1);
                return;
            }
        }
        let menu = g::nplayers_menu;
        menutick_230238(a0, menu);
        let made = std_menumake(lw(a0 + 8), menu);
        sw(a0 + 12, made);
    }
}

/// Copy the 76 bytes of the controls backup of the player (at 0x1fb33b0 + 76 per player) to `to` (64 bytes in blocks of 32 and then 12) (@ setupcontrols_pageTick).
fn restore_controls(from: i32, to: i32) {
    let mut from = from;
    let mut to = to;
    let end = from + 64;
    loop {
        let b = ld(from + 8);
        let c = ld(from + 16);
        let d = ld(from + 24);
        sd(to, ld(from));
        sd(to + 8, b);
        sd(to + 16, c);
        sd(to + 24, d);
        from += 32;
        to += 32;
        if from == end {
            break;
        }
    }
    let tail = lw(from + 8);
    sd(to, ld(from));
    sw(to + 8, tail);
}

/// The tick of the controls set-up page: the menu is made from the pad's custom controls; the button 16 on a custom set restores the backup of the player's settings into the
/// player and its pad; otherwise the front end's flags 48 are dropped; the menu ticks (@ 0x20f270).
pub fn setupcontrols_pageTick(a0: i32) {
    customcontrols_makemenu();
    let player = lw(g::currentplayer);
    if lhu(lw(player + 24)) == 0 {
        let pressed = joyNewBut(lw(player));
        let player = lw(g::currentplayer);
        if (pressed & 16) != 0 {
            restore_controls(lw(player) * 76 + 0x1fb33b0, player + 28);
            let player = lw(g::currentplayer);
            restore_controls(lw(player) * 76 + 0x1fb33b0, lw(player + 24) + 4);
        }
    } else {
        let front = lw(g::front);
        sw(front + 12, lw(front + 12) & -49);
    }
    ingame_menutick(a0, g::controls_menu);
}

/// Draw the arcade page: the background (unless the load page is up), the level's name (or the all levels text) with the arrows, and for the map maker's maps its name and
/// description from the info of the map in the temporary buffer (@ 0x23ab70).
pub fn arcade_pageDraw(a0: i32) {
    pagebackgroundGfx_230688(a0);
    let front = lw(g::front);
    if lw(front + 20) >= 2 {
        if lw(lw(front + 4) + 16) == g::arcade_mmload_page {
            return;
        }
    }
    let old_font = textSelect(2);
    let width = lw(g::window + 4);
    let name_box = frontGetBox(a0, 2);
    let x = lhu(name_box);
    let y = lhu(name_box + 2);
    let state = 0x1fc11f8;
    if lw(lw(g::front) + 20) < 2 {
        let mut flags = lw(state + 8);
        if (flags & 16) != 0 {
            guiDrawImage(20, 0x800080, (x - 32) as f32, (y + 5) as f32);
            flags = lw(state + 8);
        }
        if (flags & 32) != 0 {
            guiDrawImage(21, 0x800080, (width - 64) as f32, (y + 5) as f32);
        }
    }
    let language = lw(g::cur_language);
    let text = if lw(state + 36) == 18 {
        lw(lw((language << 2) + g::languages) + 1000)
    } else {
        lvGetNameFromNum(lw(state + 40))
    };
    textOutline(x, y, text, 0x40787880, 0x5080);
    if lw(0x1fc121c) == 18 {
        let buffer = lw(g::mmtempbuffer);
        if lw(buffer) == 0x1324626 {
            let info = mmfileGetInfo(buffer);
            let top = lw(g::window + 20);
            let left = lhu(name_box);
            textSelect(2);
            textOutline(left, top + 32, info + 12, 0x40787880, 0x5080);
            let description = info + 44;
            if description != 0 {
                textSelect(1);
                textOutline(left + 32, top + 50, description, lw(g::menuitem_fgcol), lw(g::menuitem_bgcol));
            }
        }
    }
    textSelect(old_font);
}

/// Draw the challenge page: the background, the title and the best score of the challenge (or the none text) at the right, the arrows (flashing colour) and the name of the
/// challenge (@ 0x23c5d0).
pub fn challenge_pageDraw(a0: i32) {
    let languages = g::languages;
    let state = 0x1fc11f8;
    pagebackgroundGfx_230688(a0);
    let old_font = textSelect(2);
    let title_x = lh(g::challenge_menu + 4);
    let title_box = frontGetBox(a0, 2);
    let y = lhu(title_box + 2) + 60;
    textOutline(title_x, y, lw(lw((lw(g::cur_language) << 2) + languages) + 1040), lw(g::menuitem_fgcol), lw(g::menuitem_bgcol));
    let stats = statsGetPlayer(0);
    let language = lw(g::cur_language);
    let score_text;
    if lw(stats + (lw(state + 44) << 3) + 1400) == 0 {
        score_text = lw(lw((language << 2) + languages) + 1044);
    } else {
        let stats = statsGetPlayer(0);
        let number = lw(state + 44);
        score_text = challengeFormatScore(number, lw(stats + (number << 3) + 1404));
    }
    let width = textWidth(score_text);
    let window = g::window;
    textOutline(lw(window + 4) - (width + 100), y, score_text, 0x40787880, 0x5080);
    let frame_box = frontGetBox(a0, 1);
    let x = lhu(frame_box);
    let frame_box = frontGetBox(a0, 1);
    let mut flags = lw(state + 8);
    let arrow_y = lhu(frame_box + 2);
    let colour = if (lw(g::tickframenum) & 32) != 0 { 0x7f7f7f80 } else { 0x40787880 };
    if (flags & 16) != 0 {
        guiDrawImage(20, 0x800080, (x - 32) as f32, (arrow_y + 5) as f32);
        flags = lw(state + 8);
    }
    if (flags & 32) != 0 {
        guiDrawImage(21, 0x800080, (lw(window + 4) - 64) as f32, (arrow_y + 5) as f32);
    }
    let setup = challengeGetSetupNum(lw(state + 44));
    let name = lw((lh(setup) << 2) + lw((lw(g::cur_language) << 2) + languages));
    textOutline(x, arrow_y, name, colour, 0x5080);
    textSelect(old_font);
}

/// The tick of a rocket smoke effect group: when it has no smoke stream it may start one at random, with a sound; the stream's rate follows a sine of the group's time for the
/// first two seconds and then drops to 0 (@ 0x2ac510).
pub fn specialfxOccasionalRocketSmokeTick(a0: i32) {
    let groups = g::fxgroups;
    let stream_at = a0 * 92 + (groups + 72);
    let base;
    'b1: {
        if lw(stream_at) == 0 {
            let groups = g::fxgroups;
            if lw(g::updateframesi) <= 0 {
                base = groups;
                break 'b1;
            }
            let roll = newrnd();
            let ticks = lw(g::updateframesi);
            let factor = lwc1(0x3ad6d8);
            if ticks == 0 {
                trap("break");
            }
            let time_at = a0 * 92 + (groups + 52);
            if (((roll as i32 as u32) % ((2040_i32).wrapping_div(ticks) as u32)) as i32) == ps2float::mul(lwc1(time_at), factor) as i32 {
                let stream = particleNewStream(stream_at, 8, a0 * 92 + (groups + 12), 0, 0, 0.0_f32);
                sw(stream_at, stream);
                let roll = random_as_float();
                let groups = g::fxgroups;
                let ticks = lw(g::updateframesi);
                swc1(time_at, ps2float::mul(roll, 2.3283064e-10_f32));
                sw(groups + a0 * 92 + 48, 0);
                if ticks > 0 {
                    let range = 4e+01_f32;
                    let position = a0 * 92 + (groups + 12);
                    soundStartRange(13, position, 0, range);
                    soundStartRange(10, position, 0, range);
                }
            }
        }
        base = g::fxgroups;
    }
    let stream_at = a0 * 92 + (base + 72);
    let stream = lw(stream_at);
    let group = base + a0 * 92;
    if stream == 0 {
        return;
    }
    let time = lwc1(group + 48);
    if !(time < 2.0_f32) {
        particleUpdateRate(stream, 0.0_f32);
        return;
    }
    let half = fptodp(ps2float::mul(time, 0.5_f32));
    let scaled = dpmul(half, ld(0x3a9738));
    let angle = dpmul(scaled, ld(0x3a9740));
    let rounded = dpadd(angle, 0x3fe0000000000000_i64);
    let index = dptoli(rounded);
    particleUpdateRate(lw(stream_at), ps2float::add(ps2float::mul(lwc1(((index & 2047) << 2) + g::g_SinTable), 25.0_f32), 1.0_f32) as i32 as f32);
}

/// Draw the story page: the background, the story level's name with arrows (flashing colour), and the best times of the three skill levels of the story level for the
/// first player or the team (@ 0x23bb90).
pub fn story_pageDraw(a0: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    pagebackgroundGfx_230688(a0);
    let old_font = textSelect(2);
    sw(sp, old_font);
    let frame_box = frontGetBox(a0, 1);
    let x = lhu(frame_box);
    let frame_box = frontGetBox(a0, 1);
    let y = lhu(frame_box + 2);
    let state = 0x1fc11f8;
    let mut flags = lw(state + 8);
    let colour = if (lw(g::tickframenum) & 32) != 0 { 0x7f7f7f80 } else { 0x40787880 };
    if (flags & 16) != 0 {
        guiDrawImage(20, 0x800080, (x - 32) as f32, (y + 5) as f32);
        flags = lw(state + 8);
    }
    if (flags & 32) != 0 {
        guiDrawImage(21, 0x800080, (lw(g::window + 4) - 64) as f32, (y + 5) as f32);
    }
    let languages = g::languages;
    let name = lw((lh((lw(state + 32) << 3) + g::storydata) << 2) + lw((lw(g::cur_language) << 2) + languages));
    textOutline(x, y, name, colour, 0x5080);
    let mut level = 0;
    textSelect(1);
    let mut story = lw(state + 32);
    loop {
        let index = lh((story << 3) + (g::storydata + 2));
        let time;
        if lw(0x3aff20) != 1 {
            let stats = statsGet();
            time = lw(stats + ((level << 2) + index * 36) + 340);
        } else {
            let stats = statsGetPlayer(0);
            time = lw(stats + ((level << 2) + index * 36) + 328);
        }
        let language = lw(g::cur_language);
        let text;
        if time <= 0 {
            text = lw(lw((language << 2) + languages) + 1028);
        } else {
            text = gameFormatTime(time);
        }
        let width = textWidth(text);
        let left = lw(g::window + 4) - (width + 100);
        let row = frontGetBox(a0, level + 2 & 0xffff);
        level += 1;
        textOutline(left, lhu(row + 2), text, 0x407f7f80, 0x5080);
        if level >= 3 {
            break;
        }
        story = lw(state + 32);
    }
    textSelect(lw(sp));
}
