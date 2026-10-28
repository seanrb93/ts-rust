//! Functions: building the halls of the pads, the credits background tick, the story page's tick, the sound tick, explosion damage, setting a player's start position, a remote
//! gun's fire and the particle groups' tick.

use super::particle_spawners::random_as_float;
use crate::*;

/// Reset the pads of the level (@ 0x264248): load the pad file (or link the pads of the tile set in the map maker's level), check the counts, then build the table of halls:
/// each hall lists its pads, the links that start at its pads and the links that end at them.
pub fn padReset() {
    let pads;
    if lw(g::lvlevelnum) != 100 {
        let name = padFileInName();
        padLoad(name);
        pads = lw(g::numpads);
    } else {
        padLinkTileSetPads();
        pads = lw(g::numpads);
    }
    let small = (pads < 2401) as i32;
    if pads <= 0 {
        return;
    }
    let links = lw(g::numlinks);
    if !(small != 0 && links < 0x1131) {
        exit(-1);
    }
    padCheckLinks();
    let pad_count = lw(g::numpads);
    let mut i = 0;
    sw(g::numhalls, 0);
    let mut halls;
    let first;
    let second;
    let third;
    let sizes;
    if pad_count <= 0 {
        halls = 0;
        first = lw(0x3b00a8);
        second = lw(0x3b00a0);
        sizes = lw(0x3b00a4) << 2;
        third = 0;
    } else {
        let level_pads = lw(g::levelpads);
        halls = i;
        let size = lw(0x3b00a4);
        first = lw(0x3b00a8);
        second = lw(0x3b00a0);
        loop {
            let hall = lh(i * 28 + level_pads + 8);
            i += 1;
            if hall >= halls {
                halls = hall + 1;
                sw(g::numhalls, halls);
            }
            if i >= pad_count {
                break;
            }
        }
        sizes = size << 2;
        third = 0;
    }
    let _ = third;
    let bytes = (((halls << 4) + sizes + ((first << 3) + (second << 2)) + 15) & -16) as i32;
    let table = memAlloc(bytes, 1);
    let hall_count = lw(g::numhalls);
    sw(g::halls, table);
    let mut hall = 0;
    let mut entry = table + (hall_count << 2);
    if hall_count > 0 {
        let link_table = lw(g::levellinks);
        let pad_table = lw(g::levelpads);
        loop {
            sw(entry, 0);
            sw(entry + 4, 0);
            sw(entry + 8, 0);
            sw((hall << 2) + table, entry);
            let mut count = lw(g::numpads);
            let mut p = 0;
            let next_hall = hall + 1;
            let mut own = 0;
            let mut link_count;
            if count <= 0 {
                link_count = lw(g::numlinks);
            } else {
                let level_pads = lw(g::levelpads);
                let mut offset = p * 28;
                loop {
                    let pad = offset + level_pads;
                    if (lw(pad + 4) & 1) == 0 {
                        p += 1;
                    } else if lh(pad + 8) != hall {
                        p += 1;
                    } else {
                        let slot = own + 1;
                        let at = entry + (own << 2);
                        own = slot;
                        sw(at + 12, p);
                        sw(entry, slot);
                        count = lw(g::numpads);
                        p += 1;
                    }
                    offset = p * 28;
                    if p >= count {
                        break;
                    }
                }
                link_count = lw(g::numlinks);
            }
            let mut l = 0;
            let mut starting;
            if link_count <= 0 {
                starting = lw(entry + 4);
            } else {
                starting = lw(entry + 4);
                let mut offset = l * 20;
                loop {
                    'link: {
                        let link = offset + link_table;
                        if (lw(link) & 1) == 0 {
                            l += 1;
                        } else {
                            let to = lh(lw(link + 8) * 28 + pad_table + 8);
                            let from = lh(lw(link + 4) * 28 + pad_table + 8);
                            if from == to {
                                l += 1;
                            } else {
                                let position = own + starting;
                                if from != hall {
                                    if to != hall {
                                        l += 1;
                                        break 'link;
                                    }
                                }
                                let slot = starting + 1;
                                starting = slot;
                                sw(entry + (position << 2) + 12, l);
                                sw(entry + 4, slot);
                                own = lw(entry);
                                link_count = lw(g::numlinks);
                                l += 1;
                            }
                        }
                    }
                    offset = l * 20;
                    if l >= link_count {
                        break;
                    }
                }
                l = 0;
            }
            let mut ending = lw(entry + 8);
            let total;
            if link_count <= 0 {
                total = own + starting;
            } else {
                let mut offset = l * 20;
                loop {
                    'link: {
                        let link = offset + link_table;
                        if (lw(link) & 1) == 0 {
                            l += 1;
                        } else {
                            let to = lh(lw(link + 8) * 28 + pad_table + 8);
                            if lh(lw(link + 4) * 28 + pad_table + 8) == hall {
                                if hall != to {
                                    l += 1;
                                    break 'link;
                                }
                                let slot = ending + 1;
                                let at = (own + starting + ending) << 2;
                                ending = slot;
                                sw(entry + at + 12, l);
                                sw(entry + 8, slot);
                                own = lw(entry);
                                link_count = lw(g::numlinks);
                                starting = lw(entry + 4);
                            }
                            l += 1;
                        }
                    }
                    offset = l * 20;
                    if l >= link_count {
                        break;
                    }
                }
                total = own + starting;
            }
            hall = next_hall;
            entry = entry + (((total + ending) << 2) + 12);
            if hall >= hall_count {
                break;
            }
        }
    }
    padSetRoomNums();
    padRestart();
}

/// Tick of the credits background (@ 0x2aeac8): the camera moves along the tunnel and looks a little ahead of itself with a smoothed direction, the layers rotate, a random
/// wave shakes the tunnel and the tunnel sections that fell behind the camera are recycled to the front and given their quads and texture coordinates.
pub fn creditsBgTick() {
    let position = g::creditscampos;
    let limit = lwc1(0x3ad75c);
    let moved = ps2float::add(lwc1(position + 8), ps2float::mul(lwc1(g::updateframesf), lwc1(0x3ad758)));
    swc1(position + 8, moved);
    if limit < moved {
        swc1(position + 8, ps2float::sub(moved, limit));
        creditsRestartTunnel();
    }
    let along = lwc1(position + 8);
    let ahead = 3e+01_f32;
    getCreditTunnelPos(position, position + 4, along);
    let direction = g::creditscamdir;
    getCreditTunnelPos(direction, direction + 4, ps2float::add(lwc1(position + 8), ahead));
    let y = lwc1(position + 4);
    let x = lwc1(g::creditscampos);
    let dir_x = lwc1(g::dirvec);
    let dir_y = lwc1(g::dirvec + 4);
    let smoothing = lwc1(0x3ad760);
    let new_y = ps2float::add(dir_y, ps2float::mul(ps2float::sub(ps2float::sub(lwc1(direction + 4), y), dir_y), smoothing));
    let new_x = ps2float::add(dir_x, ps2float::mul(ps2float::sub(ps2float::sub(lwc1(g::creditscamdir), x), dir_x), smoothing));
    swc1(g::dirvec + 4, new_y);
    swc1(g::dirvec, new_x);
    let squared = ps2float::add(ps2float::mul(new_x, new_x), ps2float::mul(new_y, new_y));
    swc1(g::creditscamdir, ps2float::add(x, new_x));
    swc1(direction + 4, ps2float::add(y, new_y));
    let length = sqrt(fptodp(squared));
    let length = dptofp(dpmul(length, ld(0x3a9888)));
    let step = lwc1(g::updateframesf);
    let wave_time = lwc1(g::randomsinewavetime);
    let advance = ps2float::mul(step, lwc1(0x3ad764)).pdiv(6e+01_f32);
    let roll = ps2float::add(lwc1(g::creditscamrotz), step);
    swc1(direction + 8, ps2float::add(lwc1(position + 8), ahead));
    swc1(g::creditscamrotz, roll);
    let wave_time = ps2float::add(wave_time, advance);
    swc1(g::randomsinewavetime, wave_time);
    if 1.0_f32 < wave_time {
        let random = ps2float::mul(random_as_float(), 2.3283064e-10_f32);
        sw(g::randomsinewavetime, 0);
        let centred = ps2float::sub(ps2float::add(random, random), 1.0_f32);
        swc1(g::randomsinewaveintensity, ps2float::mul(ps2float::mul(centred, centred), centred));
    }
    let one = 1.0_f32;
    let layers = g::layerrot;
    let half_turn = ld(0x3a9890);
    let first = sinf(ps2float::mul(gameframe_x(), lwc1(0x3ad768)));
    let second_angle = ps2float::mul(gameframe_x(), lwc1(0x3ad76c));
    swc1(g::layerrot, ps2float::add(first, one));
    let second = sinf(second_angle);
    let third_angle = ps2float::mul(gameframe_x(), lwc1(0x3ad770));
    swc1(layers + 4, ps2float::add(second, one));
    let third = sinf(third_angle);
    let wave = lwc1(g::randomsinewavetime);
    let doubled = ps2float::add(wave, wave);
    let angle = dptofp(dpsub(dpmul(fptodp(doubled), half_turn), half_turn));
    let cosine = cosf(angle);
    let scale = 5.0_f32;
    let mut data = lw(g::currenttunneldata);
    let mut offset = 0;
    let mut section_offset = 0;
    let mut tested = 0;
    let mut replaced = 0;
    let mut found = false;
    swc1(layers + 8, ps2float::add(ps2float::add(third, one), ps2float::mul(ps2float::mul(ps2float::mul(ps2float::mul(ps2float::add(cosine, one), 0.5_f32), lwc1(g::randomsinewaveintensity)), length), length)));
    loop {
        let section = offset + data;
        tested += 1;
        let more = (tested < 40) as i32;
        if lw(section + 0x1440) == 0 {
            replaced = data + section_offset;
            if ps2float::mul(lw(section + 0x1444) as f32, scale) < ps2float::sub(lwc1(g::creditscampos + 8), scale) {
                found = true;
                break;
            }
        }
        offset += 0x1450;
        section_offset += 0x1450;
        if more == 0 {
            break;
        }
    }
    if found {
        let section_number = lw(g::currenttunnelsection);
        let head = lw(g::tunnelDataHead);
        let mut at = data;
        let mut left = 39;
        loop {
            left -= 1;
            if lw(at + 0x1440) == replaced {
                sw(at + 0x1440, 0);
            }
            at += 0x1450;
            if left < 0 {
                break;
            }
        }
        let section = offset + data;
        sw(section + 0x1440, head);
        let mut current = section;
        sw(section + 0x1444, section_number + 40);
        sw(g::currenttunnelsection, section_number + 1);
        let mut quad = 0;
        sw(g::tunnelDataHead, current);
        loop {
            let at = current;
            let number = quad;
            let id = lw(at + 0x1444);
            quad += 1;
            creditsSetQuad(at, id, section_number + 41, number);
            current = lw(g::tunnelDataHead);
            if quad >= 12 {
                break;
            }
        }
        data = lw(g::currenttunneldata);
    }
    let mut i = 0;
    loop {
        let section = i * 0x1450;
        i += 1;
        let at = section + data;
        let id = lw(at + 0x1444);
        creditsSetST(at, id, id + 1);
        data = lw(g::currenttunneldata);
        if i >= 40 {
            break;
        }
    }
}

/// Tick of the story page (@ 0x23bee8): the difficulty items are enabled by what is unlocked, the left and right buttons step through the levels of the story that are
/// available, the cheat button opens the cheats page and accept opens the character select for the picked difficulty.
pub fn story_pageTick_23bee8(a0: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let item = menuitem(g::story_menu, lhu(a0 + 4));
    sw(sp, item);
    let table = (lw(g::cur_language) << 2) + g::languages;
    sw(((lw(0x3aff2c) - 1) << 3) + 0x1fc1308, 0);
    pageSet(lw(lw(table) + 484), g::story_page_354e08);
    frontfxTick(0);
    let items = g::i0440;
    let flags = if storyDifficultyAvail(lw(0x1fc1218), 1) != 0 { lhu(items + 74) & 0xffbf } else { lhu(items + 74) | 64 };
    sh(items + 74, flags);
    let available = storyDifficultyAvail(lw(0x1fc1218), 2);
    let items = g::i0440;
    let flags = if available != 0 { lhu(items + 106) & 0xffbf } else { lhu(items + 106) | 64 };
    sh(items + 106, flags);
    let current = lhu(a0 + 4);
    let mut text = 0;
    if current < 5 && current >= 2 {
        if (lhu(((current - 1) << 5) + (g::i0440 + 10)) & 64) == 0 {
            text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 512);
        }
    }
    let items = g::i0440;
    let state = 0x1fc11f8;
    sw(items + 160, text);
    let cheats = cheatsStory();
    let option = cheatOption(cheats);
    let level = lw(state + 32);
    let previous = if level >= 1 { level - 1 } else { 8 };
    let data = g::storydata + 2;
    let next_level = level + 1;
    let cleared = lw(state + 8) & -49;
    let previous_id = lh((previous << 3) + data);
    let next = if 8 >= next_level { next_level } else { 0 };
    let solo = ((lw(0x3aff20) < 2) as i32) ^ 1;
    sw(items + 192, option);
    sw(state + 8, cleared);
    if levelAvailSTORY(previous_id, solo) != 0 {
        sw(state + 8, lw(state + 8) | 16);
    }
    let next_id = lh((next << 3) + data);
    let solo = ((lw(0x3aff20) < 2) as i32) ^ 1;
    if levelAvailSTORY(next_id, solo) != 0 {
        sw(state + 8, lw(state + 8) | 32);
    }
    let current = lhu(a0 + 4);
    let front = lw(g::front);
    if current != 0 {
        let buttons = lw(front + 12);
        if (buttons & 16) == 0 {
            if (buttons & 32) != 0 {
                if (lw(state + 8) & 32) == 0 {
                    soundStart(142, 0);
                } else {
                    sw(state + 32, next);
                    soundStart(140, 0);
                }
            }
        } else if (lw(state + 8) & 16) == 0 {
            soundStart(142, 0);
        } else {
            sw(state + 32, previous);
            soundStart(140, 0);
        }
    }
    'buttons: {
        if cheatsStory() != 0 {
            if (joyNewBut(0) & 32) != 0 {
                let state = 0x1fc11f8;
                sw(state + 4, lw(state + 4) & -65);
                frontPageSet(0, g::cheats_page);
                pageIn(a0);
                break 'buttons;
            }
        }
        if (joyNewBut(0) & 2112) != 0 {
            if lhu(a0 + 4) != 1 {
                let picked = lhu(a0 + 4);
                let item = lw(sp);
                if ((picked - 2) as u32) < (3 as u32) {
                    if (lhu(item + 10) & 64) == 0 {
                        sw((lw(0x3aff2c) << 3) + 0x1fc1308, lw(lw((lw(g::cur_language) << 2) + g::languages) + 1032));
                        frontPageSet(0, g::storychr_page);
                        setStates(1);
                        pageIn(a0);
                        sw(0x1fc18e4, lhu(a0 + 4) - 2);
                    }
                }
            } else {
                let mut best = 4;
                'best: {
                    if (lhu(g::i0440 + 106) & 64) != 0 {
                        best = 3;
                        loop {
                            let lower = best - 1;
                            if best < 2 {
                                break 'best;
                            }
                            if (lhu((lower << 5) + (g::i0440 + 10)) & 64) == 0 {
                                break 'best;
                            }
                            best = lower;
                        }
                    }
                }
                sh(a0 + 4, best);
            }
        }
    }
    let menu = g::story_menu;
    menutick_230238(a0, menu);
    let made = std_menumake(lw(a0 + 8), menu);
    sw(a0 + 12, made);
}

/// Tick of the sounds (@ 0x204ec0): announcements are ticked, then each of the 10 sound requests in the loading queue goes through its states (1 waiting, 2 transfer to the
/// sound chip, 3 and 4 loading and starting the data, 5 done, 6 start the sound), and the volume of every positional sound is updated and sent as a batch.
pub fn soundTick() {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let mut i = 0;
    announceTick();
    let mut offset = i << 5;
    loop {
        'request: {
            let request = offset + 0x1fb1870;
            let state = lw(request);
            if (state as u32) < (7 as u32) {
                match state {
                    0 => {}
                    1 => {
                        let left = lw(request + 24) - lw(g::tickframesi);
                        sw(request + 24, left);
                        if left <= 0 {
                            if (lhu(lw(request + 4) * 36 + (g::sounds + 22)) & 18) == 0 {
                                sw(request, 6);
                            } else {
                                sw(request, 2);
                            }
                        }
                    }
                    2 => {
                        if lw(g::transinprogress) != 0 {
                            i += 1;
                            break 'request;
                        }
                        let mut sound = lw(request + 4);
                        let at = sound * 36;
                        let sounds = g::sounds;
                        let data = sounds + 28;
                        sw(g::transinprogress, 1);
                        if lw(sound * 36 + data) == 0 {
                            let names = sounds + 4;
                            let memory = fileGetMem(lw(at + names));
                            if memory == 0 {
                                sh(request + 10, 0);
                                sound = lw(request + 4);
                            } else {
                                let name = lw(lw(request + 4) * 36 + names);
                                sw(lw(request + 4) * 36 + data, memory);
                                let size = fileSize(name);
                                sound = lw(request + 4);
                                sw(sound * 36 + sounds + 32, size);
                            }
                        }
                        let address = lw(0x3afe20);
                        let entry = sound * 36 + g::sounds;
                        let source = lw(entry + 28);
                        let size = lw(entry + 32) + 64;
                        sceSdTransToIOP(&[source as i64, address as i64, size as i64, 1]);
                        sw(request, 3);
                    }
                    3 => {
                        sw(request, 4);
                    }
                    4 => {
                        let buffer = lw(g::demandbuf_138);
                        let sound;
                        if (lhu(lw(request + 4) * 36 + (g::sounds + 22)) & 16) == 0 {
                            let other = 1 - buffer;
                            sw(g::demandbuf_138, other);
                            soundSelectBuffer(other + 1);
                            soundResetBuffer(lw(g::demandbuf_138) + 1);
                            sound = lw(request + 4);
                        } else {
                            soundSelectBuffer(3);
                            soundResetBuffer(3);
                            sound = lw(request + 4);
                        }
                        let length = lw(lw(g::thissoundbuf) + 8);
                        let entry = sound * 36 + g::sounds;
                        sw(entry + 24, length);
                        let size = lw(entry + 32);
                        let address = lw(0x3afe20);
                        if size > 0 {
                            sceSdRemote(&[1, 0x80d0, 1, 0, (address + 64) as i64, length as i64, size as i64]);
                            sw(request, 5);
                        } else {
                            sw(request, 0);
                        }
                        soundSelectBuffer(0);
                        i += 1;
                        break 'request;
                    }
                    5 => {
                        if lw(g::lvlevelnum) == 102 {
                            sceSdRemote(&[1, 0x80f0, 1, 1]);
                        }
                        sw(g::transinprogress, 0);
                        sw(request, 6);
                    }
                    _ => {
                        let flags = lhu(lw(request + 4) * 36 + (g::sounds + 22));
                        let half;
                        if (flags & 2) == 0 {
                            if (flags & 16) != 0 {
                                soundSetChannel(0, 4);
                            }
                            half = lhu(request + 8);
                        } else {
                            let channel = if lw(g::demandbuf_138) == 0 { 3 } else { 2 };
                            soundSetChannel(0, channel);
                            half = lhu(request + 8);
                        }
                        let sound = lw(request + 4);
                        let parameters = if half == 0 { 0 } else { request + 12 };
                        let started = soundStart(sound, parameters);
                        let sound = lw(request + 4);
                        sw(request + 28, started);
                        if (lhu(sound * 36 + (g::sounds + 22)) & 18) != 0 {
                            soundSetChannel(-1, -1);
                        }
                        sw(request, 0);
                    }
                }
            }
            i += 1;
        }
        offset = i << 5;
        if i >= 10 {
            break;
        }
    }
    let count = lw(0x3afe28);
    let mut i = 0;
    if count > 0 {
        let mut count = count;
        loop {
            let sound = i * 24 + 0x1fb17e0;
            if lw(sound + 4) != 0 {
                soundCalcVol(lw(sound), sound + 8, sp, sp | 4, 0, lwc1(sound + 20));
                let channel = (i + 5) << 1;
                soundBatchAdd(1, channel & 0xfffe, lw(sp));
                soundBatchAdd(1, (channel | 256) & 0xfffe, lw(sp + 4));
                count = lw(0x3afe28);
            }
            i += 1;
            if i >= count {
                break;
            }
        }
    }
    soundBatchExec();
    soundBatchReset();
}

/// Damage the props within the radius of an explosion at the position `a1` (@ 0x2751c0). The size of the explosion (by the kind `a2` in 8 to 18) sets the damage and the
/// radius, the age (`f12` seconds) fades the damage over four 30 frame stages with a bit of randomness, and every prop and character with a clear line gets `propDamage` by
/// its distance. Mines nearby are set off in the first frames.
pub fn explosionDamage(_a0: i32, a1: i32, a2: i32, a3: i32, _a4: i32, a5: i32, f12: f32) {
    let frame = Frame::new(288);
    let sp = frame.sp();
    let kind = a2 - 8;
    let mut strength = 0.0_f32;
    let age = ps2float::mul(f12, 6e+01_f32) as i32;
    sw(sp + 64, 0);
    let scale;
    let radius;
    let radius_squared;
    if (kind as u32) < (11 as u32) && kind <= 4 {
        if kind <= 1 {
            scale = lwc1(0x3acdb4);
            radius = 3.0_f32;
        } else {
            scale = lwc1(0x3acdb8);
            radius = 4.0_f32;
        }
        radius_squared = radius * radius;
    } else {
        scale = lwc1(0x3acdbc);
        radius = 5.0_f32;
        radius_squared = radius * radius;
    }
    let frames = lw(g::updateframesi);
    let mut due;
    'mines: {
        let mut frames = frames;
        if age >= 18 {
            due = ((age + 24).wrapping_rem(30) < frames) as i32;
            if age >= frames + 18 {
                break 'mines;
            }
            bulletExplodeMines(a1, 4.5_f32);
            frames = lw(g::updateframesi);
        }
        due = ((age + 24).wrapping_rem(30) < frames) as i32;
    }
    if due != 0 {
        let stage = age.wrapping_div(30);
        if stage == 1 {
            strength = ps2float::mul(scale, lwc1(0x3acdc4));
        } else if stage >= 2 {
            if stage == 2 {
                strength = ps2float::mul(scale, lwc1(0x3acdc8));
            } else if stage == 3 {
                strength = 0.0_f32;
            }
        } else if stage == 0 {
            strength = lwc1(0x3acdc0);
            sw(sp + 64, 1);
        }
        let random = random_as_float();
        let strength = ps2float::mul(strength, ps2float::add(ps2float::mul(ps2float::mul(random, 2.3283064e-10_f32), lwc1(0x3acdcc)), lwc1(0x3acdd0)));
        if !(strength <= 0.0_f32) {
            let shrink = lwc1(0x3acdd4);
            let mut list = 0;
            loop {
                let count;
                let table;
                if list != 0 {
                    count = lw(0x3b00ec);
                    table = 0x1fd3410;
                } else {
                    count = lw(0x3b00e8);
                    table = 0x1fd2f60;
                }
                let mut i = 0;
                let next_list = list + 1;
                if count <= 0 {
                    list += 1;
                } else {
                    loop {
                        'prop: {
                            let prop = lw((i << 2) + table);
                            if prop != 0 {
                                if (ld(prop + 16) & 48_i64) != 48_i64 {
                                    i += 1;
                                    break 'prop;
                                }
                                let mut damage = strength;
                                let mut hit = 0;
                                if lw(prop + 8) != 8 {
                                    let id = lw(prop + 4);
                                    let height = lwc1(prop + 52);
                                    let offset = lwc1(prop + 512);
                                    hit = 1;
                                    let factor = if id != 420 && id != 423 { 0.5_f32 } else { lwc1(0x3acdd8) };
                                    damage = ps2float::mul(damage, factor);
                                    swc1(sp + 36, ps2float::add(height, offset));
                                } else {
                                    let mode = lw(lw(prop + 352) + 2716);
                                    if (mode & 0x2010) == 0 {
                                        if mode != 0 {
                                            hit = 1;
                                            swc1(sp + 36, ps2float::add(lwc1(prop + 52), 1.5_f32));
                                        }
                                    }
                                }
                                let y = lwc1(sp + 36);
                                if hit != 0 {
                                    let x = lwc1(prop + 48);
                                    let centre_y = lwc1(a1 + 4);
                                    let dx = ps2float::sub(lwc1(a1), x);
                                    swc1(sp + 32, x);
                                    let centre_z = lwc1(a1 + 8);
                                    let dy = ps2float::sub(centre_y, y);
                                    let z = lwc1(prop + 56);
                                    swc1(sp + 16, dx);
                                    let dz = ps2float::sub(centre_z, z);
                                    swc1(sp + 40, z);
                                    swc1(sp + 20, dy);
                                    let distance = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz));
                                    swc1(sp + 24, dz);
                                    if distance < radius_squared {
                                        sw(sp, 0);
                                        sw(sp + 8, 0);
                                        swc1(sp + 16, ps2float::mul(dx, shrink));
                                        swc1(sp + 20, ps2float::mul(dy, shrink));
                                        swc1(sp + 24, ps2float::mul(dz, shrink));
                                        let blocked = bgLineTest(lw(prop + 12), sp + 32, sp + 16, 0, 0, 0, 0, -3);
                                        if blocked != 0 {
                                            i += 1;
                                            break 'prop;
                                        }
                                        let ny = -lwc1(sp + 20);
                                        let stage_flag = lw(sp + 64);
                                        let nz = -lwc1(sp + 24);
                                        swc1(sp + 48, -lwc1(sp + 16));
                                        swc1(sp + 52, ny);
                                        let power = ps2float::mul(damage, ps2float::sub(1.0_f32, (distance.abs().psqrt()).pdiv(radius)));
                                        swc1(sp + 56, nz);
                                        propDamage(prop, a5, prop + 48, a2, a3, sp + 48, 0, stage_flag, power);
                                    }
                                }
                            }
                            i += 1;
                        }
                        if i >= count {
                            break;
                        }
                    }
                    list = next_list;
                }
                if list >= 2 {
                    break;
                }
            }
        }
    }
}

/// Put the player `a0` at his start position (@ 0x27f138): a pad is chosen (the player's number in the story, the challenge's pad or a random spawn pad), the position, heading
/// and room are set, and the spawn effect is made unless the game is the story. Finally the player's start gun is set.
pub fn playerSetStartPos(a0: i32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let number = lw(a0);
    let position = a0 + 140;
    if number >= lw(g::numpads) {
        sw(a0 + 144, 0);
        sw(a0 + 148, 0);
        swc1(a0 + 140, number as f32);
    } else {
        let mut pad = 0;
        'chosen: {
            let params = g::curgameparms;
            if lw(params + 72) != 7 {
                pad = -1;
                if (lw(params + 80) & 32) == 0 {
                    pad = lvGetRandomPlayerSpawn(lw(lw(a0 + 384) + 352));
                } else {
                    let owner = lw(lw(a0 + 384) + 352);
                    let setup = challengeGetSetup();
                    let extref = lw(setup + 52);
                    if extref >= 0 {
                        pad = padNumFromExtref(extref);
                    }
                    if pad == -1 {
                        pad = lvGetRandomPlayerSpawn(owner);
                    }
                }
            } else {
                let setup = lvGetLevelSetup(lw(g::lvlevelnum));
                let mut i = 0;
                let mut remaining = lw(a0);
                let mut offset;
                loop {
                    offset = i << 4;
                    if i >= lw(setup + 36) {
                        break 'chosen;
                    }
                    let flags = lw(offset + lw(setup + 32));
                    if lvCheckCreateFlags(flags) == 0 {
                        i += 1;
                    } else {
                        remaining -= 1;
                        i += 1;
                        if remaining == -1 {
                            break;
                        }
                    }
                }
                pad = lw(offset + lw(setup + 32) + 12);
            }
        }
        let at = pad * 28 + lw(g::levelpads);
        swc1(a0 + 140, lwc1(at + 12));
        swc1(a0 + 144, lwc1(at + 16));
        swc1(a0 + 148, lwc1(at + 20));
        swc1(a0 + 276, lwc1(at + 24));
    }
    let x;
    if lw(g::lvlevelnum) != 6 {
        x = lwc1(a0 + 140);
    } else {
        swc1(a0 + 148, 4.0_f32);
        x = lwc1(a0 + 140);
    }
    'start: {
        let owner = lw(a0 + 384);
        sw(a0 + 284, 0);
        swc1(owner + 48, x);
        sw(a0 + 288, 0);
        let y = lwc1(a0 + 144);
        swc1(owner + 52, y);
        let z = lwc1(a0 + 148);
        swc1(owner + 56, z);
        let heading = lwc1(a0 + 276);
        swc1(owner + 120, z);
        swc1(owner + 76, heading);
        swc1(owner + 112, x);
        swc1(owner + 116, y);
        let eye_y = lwc1(a0 + 144);
        let eye_x = lwc1(a0 + 140);
        let eye_height = ps2float::add(eye_y, lwc1(a0 + 116));
        let eye_z = lwc1(a0 + 148);
        swc1(a0 + 348, eye_y);
        swc1(a0 + 356, eye_x);
        swc1(a0 + 360, eye_height);
        swc1(a0 + 364, eye_z);
        swc1(a0 + 152, eye_x);
        swc1(a0 + 156, eye_height);
        swc1(a0 + 160, eye_z);
        swc1(a0 + 344, eye_x);
        swc1(a0 + 352, eye_z);
        let nearest = closestpad(position, 0.0_f32);
        let level = lw(g::lvlevelnum);
        sw(lw(lw(a0 + 384) + 352) + 680, nearest);
        if ((level - 101) as u32) >= (2 as u32) {
            if level != 104 {
                if level != 105 {
                    let room = moveFindRoom(position, lwc1(a0 + 104), lwc1(a0 + 132));
                    let mut ex = lwc1(a0 + 152);
                    let mut ey = lwc1(a0 + 156);
                    let mut ez = lwc1(a0 + 160);
                    let mut previous = 0;
                    let mut crossed = 0;
                    let mut current = room;
                    let dy = ps2float::sub(ey, lwc1(a0 + 144));
                    let player = lw(a0 + 384);
                    let dz = ps2float::sub(ez, lwc1(a0 + 148));
                    swc1(sp, ps2float::sub(ex, lwc1(a0 + 140)));
                    let data = lw(player + 352);
                    swc1(sp + 4, dy);
                    swc1(sp + 8, dz);
                    if room != 0 {
                        loop {
                            crossed = bgPortalCrossed(position, sp, current, previous, 0);
                            previous = current;
                            current = crossed;
                            if crossed == previous {
                                break;
                            }
                        }
                        ex = lwc1(a0 + 152);
                        ey = lwc1(a0 + 156);
                        ez = lwc1(a0 + 160);
                    }
                    let spawn = lw(data + 0x11a4);
                    swc1(a0 + 884, ex);
                    swc1(a0 + 888, ey);
                    swc1(a0 + 892, ez);
                    sw(a0 + 900, crossed);
                    sw(a0 + 796, crossed);
                    sw(a0 + 896, crossed);
                    sw(a0 + 904, 0);
                    sw(a0 + 824, 0);
                    if spawn != 0 {
                        specialfxFreeSpawn(spawn);
                        sw(data + 0x11a4, 0);
                    }
                    if lw(g::curgameparms + 72) == 7 {
                        break 'start;
                    }
                    let spawn_type = specialfxGetBestSpawnType(lw(a0), position, room);
                    let made = specialfxNewSpawn(spawn_type, position, -1, data, 0);
                    sw(data + 0x11a4, made);
                    break 'start;
                }
            }
        }
        sw(a0 + 796, 0);
    }
    let value = lwc1(g::viewprop_mode + 4);
    sw(a0 + 1792, 0);
    sw(a0 + 1808, 0);
    sw(a0 + 1812, 0);
    swc1(a0 + 1816, value);
    playerSetStartGun(a0);
}

/// Fire the remote controlled gun (the prop `a0`, of the gun combo `a1`, in the hand `a2`) once its reload time has passed (@ 0x290010). The muzzle position comes from the
/// prop's matrix and the bone of the gun; for the dual barrel gun of the kind 425 the barrels alternate and the shot is spread randomly around the direction of the aim. The
/// shot goes through `gunFire`, then its sound.
pub fn gunRemoteFire(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32) {
    let frame = Frame::new(352);
    let sp = frame.sp();
    memset(sp + 144, 0, 12);
    sw(sp + 160, a4);
    let model = lw(a0 + 32);
    let definition = lw(model);
    let frames = running_x();
    let data = lw(a0 + 352);
    let gun = lw(a1) * 400 + GUNS_TABLE;
    let clip = lw(gun + 20);
    let base = definition - lw(definition) * 80;
    sw(a1 + (lw(gun + 16) << 2) + 4, if clip == 0 { 1 } else { clip });
    let stats = if a2 == 0 { gun + 52 } else { gun + 188 };
    if frames > 0 {
        let timer = lw(a1 + 68);
        let delay = if a3 == 0 { lw(stats + 8) } else { lw(stats + 12) };
        if timer >= delay {
            let flags = lhu(model + 292);
            let muzzle;
            let kind = lw(a0 + 4);
            if kind != 425 {
                sw(sp + 164, 1);
                sh(model + 292, flags | 1);
                let mut at = sp + 144;
                if lb(base + 7) > 0 {
                    at = lw(base + 72);
                }
                muzzle = at;
            } else {
                sw(sp + 164, 3);
                let cleared = flags & 0xfffc;
                let barrel = lw(data + 104);
                sh(model + 292, cleared);
                let low = cleared & 0xffff;
                let with_barrel = if barrel != 0 { low | 1 } else { low | 2 };
                sh(model + 292, with_barrel);
                if barrel != 0 {
                    swc1(data + 128, lwc1(0x3ad320));
                } else {
                    swc1(data + 124, lwc1(0x3ad31c));
                }
                let muzzles = lw(base + 72);
                let next = barrel + 1;
                swc1(data + 136, 1.0_f32);
                sw(data + 104, next);
                muzzle = muzzles + (barrel * 12 + 12);
                if next >= 2 {
                    sw(data + 104, 0);
                }
            }
            'aim: {
                let heading = ps2float::mul(lwc1(a0 + 76), lwc1(0x3ad324));
                sw(a1 + 68, 0);
                let x = lwc1(a0 + 48);
                let y = lwc1(a0 + 52);
                let degrees = (heading).pdiv(1.8e+02_f32);
                let z = lwc1(a0 + 56);
                matrixTransRotY(sp + 16, x, y, z, degrees);
                sceVu0MulMatrix(sp + 80, sp + 16, lw(lw(a0 + 32) + 4));
                sceVu0MulMatrix(sp + 80, sp + 80, lw(lw(a0 + 32) + 4) + (lw(sp + 164) << 6));
                let mz = lwc1(muzzle + 8);
                let mx = lwc1(muzzle);
                let my = lwc1(muzzle + 4);
                matrixTrans(sp + 16, mx, my, mz);
                sceVu0MulMatrix(sp + 80, sp + 80, sp + 16);
                let py = lwc1(sp + 132);
                let pz = lwc1(sp + 136);
                swc1(a1 + 72, lwc1(sp + 128));
                swc1(a1 + 76, py);
                swc1(a1 + 80, pz);
                let direction_x;
                if kind != 425 {
                    direction_x = lwc1(sp + 112);
                } else {
                    let owner = lw(lw(data + 96) + 356);
                    direction_x = lwc1(sp + 112);
                    if owner != 0 {
                        let first = ps2float::mul(random_as_float(), 2.3283064e-10_f32);
                        let spread_a = ps2float::mul(ps2float::sub(ps2float::add(first, first), 1.0_f32), lwc1(stats + 32));
                        let second = ps2float::mul(random_as_float(), 2.3283064e-10_f32);
                        let spread_b = ps2float::mul(ps2float::sub(ps2float::add(second, second), 1.0_f32), lwc1(stats + 32));
                        if lw(owner + 828) == 0 {
                            let dy = lwc1(sp + 116);
                            let dz = lwc1(sp + 120);
                            swc1(sp, lwc1(sp + 112));
                            swc1(sp + 4, dy);
                            swc1(sp + 8, dz);
                        } else {
                            let cy = lwc1(a1 + 76);
                            let cz = lwc1(a1 + 80);
                            swc1(sp, ps2float::sub(lwc1(owner + 832), lwc1(a1 + 72)));
                            swc1(sp + 4, ps2float::sub(lwc1(owner + 836), cy));
                            swc1(sp + 8, ps2float::sub(lwc1(owner + 840), cz));
                        }
                        let fx = lwc1(sp);
                        let fy = lwc1(sp + 4);
                        let fz = lwc1(sp + 8);
                        let inverse = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(fx, fx), ps2float::mul(fy, fy)), ps2float::mul(fz, fz)).abs().psqrt());
                        swc1(sp, ps2float::mul(ps2float::add(ps2float::add(ps2float::mul(fx, inverse), ps2float::mul(spread_a, lwc1(owner + 240))), ps2float::mul(spread_b, lwc1(owner + 228))), lwc1(stats + 28)));
                        swc1(sp + 4, ps2float::mul(ps2float::add(ps2float::add(ps2float::mul(fy, inverse), ps2float::mul(spread_a, lwc1(owner + 244))), ps2float::mul(spread_b, lwc1(owner + 232))), lwc1(stats + 28)));
                        swc1(sp + 8, ps2float::mul(ps2float::add(ps2float::add(ps2float::mul(fz, inverse), ps2float::mul(spread_a, lwc1(owner + 248))), ps2float::mul(spread_b, lwc1(owner + 236))), lwc1(stats + 28)));
                        break 'aim;
                    }
                }
                let dy = lwc1(sp + 116);
                let dz = lwc1(sp + 120);
                swc1(sp, direction_x);
                swc1(sp + 4, dy);
                swc1(sp + 8, dz);
            }
            let mut owner_arg = lw(sp + 160);
            if lw(a0 + 4) == 425 {
                StatsAdd(8, a5, 1.0_f32);
                StatsAdd(62, a5, 1.0_f32);
                owner_arg = lw(sp + 160);
            }
            gunFire(a0, a1, 0, a2, sp, owner_arg, a5, 1);
            gunFireSfx(a1, a2, a0 + 48);
        }
    }
}

/// Tick of the particle groups (@ 0x29bc58): every group in use ages by the frame time, is freed when its life is over (for the types that free themselves), has the
/// players that can see its rooms recorded in its flags and then ticks by its type (a pulse group moves, burst, stream, trail, glass, manual and explosion groups call their
/// own tick, a spinning group of kind 14 turns).
pub fn particleTick() {
    let mut groups = lw(g::pgroups);
    let mut i = 0;
    let mut frame_time = lwc1(g::updateframesf);
    loop {
        let group = i * 0x17a0 + groups;
        let mut kind = lw(group);
        let first_kind = kind;
        if kind != -1 {
            let remaining = lwc1(group + 52);
            let age = ps2float::add(lwc1(group + 12), (frame_time).pdiv(6e+01_f32));
            swc1(group + 12, age);
            if 0.0_f32 < remaining {
                if !(remaining <= age) {
                } else {
                    swc1(group + 52, 0.0_f32);
                    swc1(group + 12, ps2float::sub(age, remaining));
                }
            }
            let now = lwc1(group + 12);
            if lwc1(group + 16) <= now {
                if lw((first_kind << 4) + (g::particletypeinfos + 8)) != 0 {
                    particleFree(group);
                    kind = lw(group);
                }
            }
            if kind != -1 {
                groups = lw(g::pgroups);
                if lwc1(group + 52) == 0.0_f32 {
                    if (ld(group + 8) & 6_i64) != 0_i64 {
                        let rooms = group + 72;
                        sw(group + 8, lw(group + 8) & -121);
                        if bgIsAnyRoomsVisibleByPlayer(rooms, 0) != 0 {
                            sw(group + 8, lw(group + 8) | 8);
                        }
                        if bgIsAnyRoomsVisibleByPlayer(rooms, 1) != 0 {
                            sw(group + 8, lw(group + 8) | 16);
                        }
                        if bgIsAnyRoomsVisibleByPlayer(rooms, 2) != 0 {
                            sw(group + 8, lw(group + 8) | 32);
                        }
                        let visible = bgIsAnyRoomsVisibleByPlayer(rooms, 3);
                        if visible != 0 {
                            sw(group + 8, lw(group + 8) | 64);
                        }
                    }
                    let flags = lw(group + 8);
                    sw(group + 8, flags | 128);
                    if (ld(group + 8) & 122_i64) != 2_i64 {
                        let type_index = lw(group) - 3;
                        if (type_index as u32) < (12 as u32) {
                            match type_index {
                                0 => {
                                    frame_time = lwc1(g::updateframesf);
                                    let mut tick = true;
                                    if !(((frame_time).pdiv(6e+01_f32)) < lwc1(group + 12)) {
                                        let word = lw(group + 48);
                                        groups = lw(g::pgroups);
                                        if (word & (word - 1)) != 0 {
                                            tick = false;
                                        }
                                    }
                                    if tick {
                                        let speed = lwc1(group + 940);
                                        let y = ps2float::add(lwc1(group + 932), ps2float::mul(lwc1(group + 944), frame_time));
                                        let z = ps2float::add(lwc1(group + 936), ps2float::mul(lwc1(group + 948), frame_time));
                                        swc1(group + 928, ps2float::add(lwc1(group + 928), ps2float::mul(speed, frame_time)));
                                        swc1(group + 932, y);
                                        swc1(group + 936, z);
                                    }
                                }
                                1 => particleBurstTick(group),
                                2 => particleStreamTick(group),
                                3 => particleTrailTick(group),
                                4 => particleGlassTick(group),
                                5 | 6 | 7 | 10 => {}
                                8 => particleManualTick(),
                                9 => particleExplosionTick(group),
                                _ => {
                                    frame_time = lwc1(g::updateframesf);
                                    let mut tick = true;
                                    if !(((frame_time).pdiv(6e+01_f32)) < lwc1(group + 12)) {
                                        let word = lw(group + 48);
                                        if (word & (word - 1)) != 0 {
                                            tick = false;
                                        }
                                    }
                                    if tick {
                                        let speed = lwc1(group + 944);
                                        let y = ps2float::add(lwc1(group + 936), ps2float::mul(lwc1(group + 948), frame_time));
                                        let frames = lw(g::updateframesi);
                                        let z = ps2float::add(lwc1(group + 940), ps2float::mul(lwc1(group + 952), frame_time));
                                        swc1(group + 932, ps2float::add(lwc1(group + 932), ps2float::mul(speed, frame_time)));
                                        swc1(group + 936, y);
                                        swc1(group + 940, z);
                                        if frames > 0 {
                                            let mut angle = group + 956;
                                            let mut left = 19;
                                            loop {
                                                let random = ps2float::mul(random_as_float(), 2.3283064e-10_f32);
                                                left -= 1;
                                                swc1(angle, ps2float::mul(random, 3.6e+02_f32));
                                                angle += 4;
                                                if left < 0 {
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                groups = lw(g::pgroups);
            } else {
                groups = lw(g::pgroups);
            }
            frame_time = lwc1(g::updateframesf);
        }
        i += 1;
        if i >= 100 {
            break;
        }
    }
}
