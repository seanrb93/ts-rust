//! Functions: choosing the animation of a computer character (and the speed it plays at) and drawing the page of the character sets.

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    random_as_float() * 2.3283064e-10_f32
}

/// Set the animation `anim` of the character `chr` with the usual blend (0.25) when none of the animations in `playing` is on.
fn set_unless_playing(chr: i32, playing: &[i32], anim: i32) {
    for check in playing {
        if curAnim(chr, *check) != 0 {
            return;
        }
    }
    setAnim(chr, anim, 0.25_f32, 1.0_f32);
}

/// Choose the animation of the computer character `a0` from what it is doing (@ 0x2bece0): the death-like spin (555) of a character hit hard, the walk, run, back-step and
/// turning animations by its speed and the turning needed, and the idle animations, with the sound of some; the speed of the animation is then set from the movement of its root.
pub fn setTimeSplitterAnimation(a0: i32) {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let mind = lw(a0 + 352);
    sw(a0 + 344, 0);
    if (lw(mind + 2704) & 2) != 0 {
        if curAnim(a0, 555) == 0 {
            let mut size = fptodp(lwc1(mind + 2780));
            if dpcmp(size, 0_i64) < 0 {
                size = dpsub(0_i64, size);
            }
            if dpcmp(size, 0x4000000000000000_i64) < 0 {
                setAnim(a0, 555, 0.25_f32, 2.0_f32);
                return;
            }
        }
        'b5: {
            if curAnim(a0, 555) != 0 {
                if curAnimComplete(lw(a0 + 32)) != 0 {
                    sw(mind + 2704, lw(mind + 2704) & -3);
                    break 'b5;
                }
            }
            if curAnim(a0, 555) != 0 {
                let progress = lwc1(lw(a0 + 32) + 100);
                if 1.25_f32 < progress {
                    if progress < lwc1(0x3ad878) {
                        gunChrFire(a0, mind + 376, 0);
                    }
                }
            }
        }
    }
    let flags = lw(mind + 2708);
    if (flags & 8) == 0 {
        let speed = lwc1(mind + 2876);
        let top = lwc1(mind + 2860);
        if 0.0_f32 <= speed {
            let crouching = (flags & 2048) != 0;
            let sideways = (flags & 4) != 0;
            if !(top < speed) {
                if !crouching {
                    if !sideways {
                        set_walk_chain(a0, 538);
                    } else {
                        set_unless_playing(a0, &[533], 533);
                    }
                } else {
                    set_unless_playing(a0, &[534], 534);
                }
            } else if !crouching {
                if sideways {
                    set_unless_playing(a0, &[533], 533);
                } else {
                    set_walk_chain(a0, 530);
                }
            } else {
                set_unless_playing(a0, &[534], 534);
            }
        } else if speed < -top {
            set_walk_chain(a0, 535);
        }
    } else {
        let turn = anglediff(lwc1(a0 + 80), lwc1(a0 + 76));
        'b12: {
            if !(45.0_f32 < turn) {
                if !(turn < -45.0_f32) {
                    if curAnim(a0, 560) == 0 && curAnim(a0, 561) == 0 {
                        let mut idle = true;
                        for check in [556, 557, 558, 553, 554, 555] {
                            if curAnim(a0, check) != 0 {
                                idle = false;
                                break;
                            }
                        }
                        if idle {
                            setAnim(a0, lw(a0 + 344) + 556, 0.0_f32, 1.0_f32);
                            break 'b12;
                        }
                        if curAnimComplete(lw(a0 + 32)) != 0 {
                            let style = lw(a0 + 344);
                            if style == 0 {
                                let random = random_unit();
                                if !(random < lwc1(0x3ad87c)) {
                                    if !(random < lwc1(0x3ad880)) {
                                        setAnim(a0, 554, 0.25_f32, 1.0_f32);
                                        if lw(g::updateframesi) > 0 {
                                            soundDelayStartEx(170, a0 + 48, 65, 0);
                                        }
                                    } else {
                                        setAnim(a0, 553, 0.25_f32, 1.0_f32);
                                        if lw(g::updateframesi) > 0 {
                                            soundDelayStartEx(170, a0 + 48, 65, 0);
                                        }
                                    }
                                } else {
                                    setAnim(a0, 556, 0.25_f32, 1.0_f32);
                                }
                            } else {
                                setAnim(a0, style + 556, 0.25_f32, 1.0_f32);
                            }
                        }
                    }
                } else {
                    start_turn(a0, 560);
                }
            } else {
                start_turn(a0, 561);
            }
        }
        let mut nothing_more = false;
        'b20: {
            if curAnimSlot1(a0, 560) != 0 {
                if lw(lw(a0 + 32) + 152) != 0 {
                    break 'b20;
                }
            }
            if curAnimSlot1(a0, 561) == 0 {
                nothing_more = true;
            } else if lw(lw(a0 + 32) + 152) == 0 {
                nothing_more = true;
            }
        }
        if !nothing_more {
            let mut idle_done = false;
            if curAnimSlot2(a0, 560) != 0 || curAnimSlot2(a0, 561) != 0 {
                idle_done = true;
            }
            if !idle_done {
                let mut idle = true;
                for check in [556, 557, 558, 553, 554, 555] {
                    if curAnim(a0, check) != 0 {
                        idle = false;
                        break;
                    }
                }
                if idle {
                    setAnim(a0, lw(a0 + 344) + 556, 0.25_f32, 1.0_f32);
                } else if curAnimComplete(lw(a0 + 32)) != 0 {
                    let style = lw(a0 + 344);
                    if style != 0 {
                        setAnim(a0, style + 556, 0.25_f32, 1.0_f32);
                    } else {
                        let random = random_unit();
                        if random < lwc1(0x3ad884) {
                            setAnim(a0, 556, 0.25_f32, 1.0_f32);
                        } else {
                            let anim = if !(random < lwc1(0x3ad888)) { 554 } else { 553 };
                            setAnim(a0, anim, 0.25_f32, 1.0_f32);
                            if lw(g::updateframesi) > 0 {
                                soundDelayStartEx(170, a0 + 48, 65, 0);
                            }
                        }
                    }
                }
            }
        }
    }
    extractAnimRoot(a0, sp, sp | 4, sp | 8, 2);
    let mut distance = fptodp(lwc1(sp + 4) - lwc1(sp));
    if dpcmp(distance, 0_i64) < 0 {
        distance = dpsub(0_i64, distance);
    }
    let travelled = dptofp(distance);
    let flags_now;
    'b26: {
        if curAnim(a0, 555) == 0 {
            if curAnim(a0, 561) == 0 {
                if curAnim(a0, 560) == 0 {
                    if !(travelled == 0.0_f32) {
                        let speed = lwc1(mind + 2876);
                        if !(speed == 0.0_f32) {
                            let mut magnitude = fptodp(speed);
                            if dpcmp(magnitude, 0_i64) < 0 {
                                magnitude = dpsub(0_i64, magnitude);
                            }
                            let scaled = dpmul(magnitude, fptodp(lwc1(sp + 8)));
                            let per_frame = fptodp(travelled * 6e+01_f32);
                            let ratio = dpdiv(scaled, per_frame);
                            let ratio = dpdiv(ratio, ld(0x3a9d00));
                            curAnimSpeed(lw(a0 + 32), dptofp(ratio));
                            flags_now = lw(mind + 2708);
                            break 'b26;
                        }
                    }
                    curAnimSpeed(lw(a0 + 32), 1.0_f32);
                    flags_now = lw(mind + 2708);
                    break 'b26;
                }
            }
            curAnimSpeed(lw(a0 + 32), 1.0_f32);
            flags_now = lw(mind + 2708);
        } else {
            curAnimSpeed(lw(a0 + 32), 2.0_f32);
            flags_now = lw(mind + 2708);
        }
    }
    sw(mind + 2712, flags_now);
}

/// Start the animation of a chain of three (`first`, +1, +2) with the style of the character at +344 added, when none of the three is playing.
fn set_walk_chain(chr: i32, first: i32) {
    if curAnim(chr, first) != 0 {
        return;
    }
    if curAnim(chr, first + 1) != 0 {
        return;
    }
    if curAnim(chr, first + 2) != 0 {
        return;
    }
    setAnim(chr, lw(chr + 344) + first, 0.25_f32, 1.0_f32);
}

/// Start the turning animation `anim` (560 or 561) when no turn is playing, or when the turn on the first slot has come to its end and the second slot is not the same turn.
fn start_turn(chr: i32, anim: i32) {
    let other = if anim == 560 { 561 } else { 560 };
    let _ = other;
    if curAnim(chr, anim) != 0 || curAnim(chr, if anim == 560 { 561 } else { 560 }) != 0 {
        if curAnimSlot1(chr, anim) == 0 {
            return;
        }
        if lw(lw(chr + 32) + 152) == 0 {
            return;
        }
        if curAnimSlot2(chr, anim) != 0 {
            return;
        }
    }
    setAnim(chr, anim, 0.25_f32, 1.0_f32);
}

/// Draw the page of the character sets (@ 0x233810) for the page `a0`: the lists of the characters of the set and of the set to choose from, with the cursor, the colours
/// of the teams, the arrows to scroll and the warning when a team that is not played has characters.
pub fn chrsets_pageDraw(a0: i32) {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let mut s7 = 0;
    sw(sp + 16, a0);
    pagebackgroundGfx_230688(a0);
    sw(sp + 24, 0);
    sw(sp + 28, 0);
    let mut state = lhu(lw(sp + 16) + 4);
    'b1: {
        'b2: {
            if (state & 0xffff) != 2 {
                if (state & 0xffff) != 3 {
                    break 'b2;
                }
                dlFillRectangle(288, 47, 575, 168, 0x2040);
            } else {
                dlFillRectangle(64, 47, 263, 168, 0x2040);
            }
            state = lhu(lw(sp + 16) + 4);
            if state == 3 {
                let column = lw(0x3aff60);
                s7 = if column <= 0 { 288 } else { (column - 1) * 22 + 509 };
                let top = lw(0x3aff5c) * 11 + 59;
                let width = if column > 0 { 22 } else { (lw(0x3aff6c) - 1) * 22 + 221 };
                dlFillRectangle(s7, top, s7 + (width - 1), top + 10, lbu(g::flashalpha) | 0x8000);
                break 'b1;
            }
        }
        if state == 2 {
            dlFillRectangle(64, lw(0x3aff68) * 11 + 59, 263, lw(0x3aff68) * 11 + 69, lbu(g::flashalpha) | 0x8000);
        }
    }
    let saved_font = textSelect(1);
    sw(sp + 20, saved_font);
    let part = lhu(lw(sp + 16) + 4) ^ 3;
    let s5 = if part == 0 { 0x2040 } else { 128 };
    let s3 = if part == 0 { 0x6e6e6e80 } else { 0x40404080 };
    zoomboxDraw(288, 47, 575, 168, s5, s3);
    let language = lw(g::cur_language);
    let title = if lw(g::gv_chrset) >= 8 {
        lw(lw((language << 2) + g::languages) + 1676)
    } else {
        lw(lw((language << 2) + g::languages) + 1672)
    };
    let width = textWidth(title);
    let free = 288 - width;
    textOutline(((free + ((free as u32) >> 31) as i32) >> 1) + 288, 47, title, s3, s5);
    let mut n = 0;
    let mut s1_2 = 0;
    loop {
        let table = 0x1fc2a88;
        let offset = n << 4;
        let entry = offset + table;
        s1_2 = n * 11 + 59;
        dlDrawLine(288, s1_2, 575, s1_2, s3);
        let id = lw(entry);
        if id >= 0 {
            let selected = lw(table + offset + 12);
            let strings = lw((lw(g::cur_language) << 2) + g::languages);
            let name = lh(id * 44 + g::chrs) << 2;
            sw(sp, 290);
            let text = lw(name + strings);
            let colour = if selected != 0 { s3 } else { 0x20202040 };
            sw(sp + 4, s1_2);
            textPrint(sp, sp | 4, text, colour);
            if lw(entry) >= 0 {
                'b10: {
                    if lw(g::gv_teamplay) != 0 {
                        let team_at = offset + (table + 4);
                        guiTriGradient(510, n * 11 + 60, 530, n * 11 + 69, lw((lw(team_at) << 3) + (g::teamdata + 4)), 128, 64);
                        if lbu(lw(team_at) + 0x1fc1234) != 0 {
                            break 'b10;
                        }
                        let alpha = lbu(g::flashalpha);
                        guiDrawImage(36, alpha << 24 | alpha << 16 | 128, lwc1(0x3ac998), (n * 11 + 61) as f32);
                    }
                }
                'b12: {
                    'b13: {
                        let shape = lw(table + offset + 8);
                        if (shape as u32) < (5 as u32) {
                            match shape {
                                0 => {
                                    guiDrawImage(17, -0x7fafd780, lwc1(0x3ac99c), (s1_2 + 3) as f32);
                                    break 'b12;
                                }
                                1 => {
                                    let y = (s1_2 + 3) as f32;
                                    guiDrawImage(17, 0x60504080, lwc1(0x3ac9a0), y);
                                    guiDrawImage(17, 0x60500000 | 0x4080, 544.0_f32, y);
                                    break 'b12;
                                }
                                2 => {
                                    guiDrawImage(17, 0x60606080, lwc1(0x3ac9a4), (s1_2 + 1) as f32);
                                    guiDrawImage(17, 0x60606080, lwc1(0x3ac9a8), (s1_2 + 3) as f32);
                                    guiDrawImage(17, 0x60606080, lwc1(0x3ac9ac), (s1_2 + 6) as f32);
                                    break 'b12;
                                }
                                3 => {
                                    let left = lwc1(0x3ac9b0);
                                    let top = (s1_2 + 1) as f32;
                                    guiDrawImage(17, -0x7f7fbf80, left, top);
                                    let bottom = (s1_2 + 6) as f32;
                                    guiDrawImage(17, -0x7f7fbf80, left, bottom);
                                    guiDrawImage(17, -0x7f7fbf80, 544.0_f32, top);
                                    guiDrawImage(17, -0x7f800000 | 0x4080, 544.0_f32, bottom);
                                    break 'b12;
                                }
                                _ => {
                                    let left = lwc1(0x3ac9b4);
                                    let top = (s1_2 + 1) as f32;
                                    guiDrawImage(17, -0x7f7fdf80, left, top);
                                    let bottom = (s1_2 + 6) as f32;
                                    guiDrawImage(17, -0x7f7fdf80, left, bottom);
                                    guiDrawImage(17, -0x7f7fdf80, 544.0_f32, top);
                                    guiDrawImage(17, -0x7f7fdf80, 544.0_f32, bottom);
                                    guiDrawImage(17, -0x7f7fdf80, lwc1(0x3ac9b8), (s1_2 + 3) as f32);
                                    break 'b13;
                                }
                            }
                        }
                    }
                }
                s7 = 555;
                let marked = lw(table + offset + 12);
                let (image, colour) = if marked == 0 { (19, -0x7fffff80) } else { (18, 0x800080) };
                guiDrawImage(image, colour, s7 as f32, (s1_2 + 2) as f32);
            }
        }
        n += 1;
        if n >= 10 {
            break;
        }
    }
    let mut column_n = (lw(g::gv_teamplay) as u32 <= 0) as i32;
    if column_n < 3 {
        let mut x = column_n * 22 + 509;
        loop {
            s7 = x;
            dlDrawLine(s7, 59, s7, 168, s3);
            column_n += 1;
            x = s7 + 22;
            if column_n >= 3 {
                break;
            }
        }
    }
    let state = lhu(lw(sp + 16) + 4);
    let s3_2;
    if state == 2 {
        s3_2 = 0x6e6e6e80;
    } else {
        s3_2 = if 7 >= lw(g::gv_chrset) { 0x20202040 } else { 0x40404080 };
    }
    let s5_3 = if (state ^ 2) == 0 { 0x2040 } else { 128 };
    zoomboxDraw(64, 47, 263, 168, s5_3, s3_2);
    let mut row = 0;
    let mut y = 59;
    let title = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1668);
    let width = textWidth(title);
    let free = 200 - width;
    textOutline(((free + ((free as u32) >> 31) as i32) >> 1) + 64, 47, title, s3_2, s5_3);
    loop {
        sw(sp + 8, 66);
        sw(sp + 12, y);
        dlDrawLine(64, y, 263, y, s3_2);
        let index = lw(0x3aff64) + row;
        if index < lw(0x3aff50) {
            let name = (lh(lw((index << 2) + 0x1fc2b48) * 44 + (g::chrs + 2)) << 2) + lw((lw(g::cur_language) << 2) + g::languages);
            textPrint(sp + 8, sp + 12, lw(name), s3_2);
        }
        row += 1;
        y += 11;
        if row >= 10 {
            break;
        }
    }
    let status = 0x1fc11f8;
    let mut s1_3 = 170;
    if lw(status + 48) == 4 {
        let teams = g::teamdata;
        sb(g::fontflags, lbu(g::fontflags) | 1);
        guiTriGradient(293, 171, 313, 180, lw(teams + 4), 128, 64);
        s7 = 436;
        let strings = g::languages;
        textOutline(324, 173, lw(lw((lw(g::cur_language) << 2) + strings) + 1700), 0x40787880, 0x5080);
        guiTriGradient(437, 171, 457, 180, lw(teams + 12), 128, 64);
        textOutline(468, 173, lw(lw((lw(g::cur_language) << 2) + strings) + 1704), 0x40787880, 0x5080);
        let font = lw(g::curfont);
        sb(g::fontflags, lbu(g::fontflags) & 254);
        s1_3 = lw(font + 12) + 170;
    }
    let teams_at = status + 60;
    let mut team = 0;
    let mut warn = false;
    loop {
        if lbu(team + teams_at) != 0 {
            team += 1;
        } else {
            let mut found = false;
            if lw(0x1fc2a88 + 4) == team {
                if lw(0x1fc2a88) >= 0 {
                    found = true;
                }
            }
            if !found {
                let mut i = 1;
                while i < 10 {
                    if lw(i * 16 + 0x1fc2a8c) == team {
                        if lw(i * 16 + 0x1fc2a88) >= 0 {
                            found = true;
                            break;
                        }
                    }
                    i += 1;
                }
            }
            if found {
                warn = true;
                break;
            }
            team += 1;
        }
        if team >= 4 {
            break;
        }
    }
    if warn {
        let language = lw(g::cur_language);
        let text = if lw(status + 48) != 4 {
            lw(lw((language << 2) + g::languages) + 1712)
        } else {
            lw(lw((language << 2) + g::languages) + 1708)
        };
        drawWarning(292, s1_3, text);
    }
    let mode = lw(0x3aff70);
    let alpha = lbu(g::flashalpha);
    let s3_3;
    if (mode & 1) == 0 {
        s3_3 = (((alpha as u32) >> 1) as i32) << 24 | 128 | (alpha << 8 | alpha << 16);
    } else {
        s3_3 = (((alpha as u32) >> 1) as i32 + 64) << 16 | 128;
    }
    let shown = lhu(lw(sp + 16) + 4);
    'b34: {
        if (shown & 0xffff) != 3 {
            if (shown & 0xffff) != 2 {
                break 'b34;
            }
            let row = lw(0x3aff68);
            sw(sp + 28, 11);
            sw(sp + 24, 200);
            s7 = 64;
            s1_3 = row * 11 + 59;
        } else {
            let column = lw(0x3aff60);
            s7 = if column <= 0 { 288 } else { (column - 1) * 22 + 509 };
            s1_3 = lw(0x3aff5c) * 11 + 59;
            let width = if column > 0 { 22 } else { (lw(0x3aff6c) - 1) * 22 + 221 };
            sw(sp + 24, width);
            sw(sp + 28, 11);
        }
    }
    let height = lw(sp + 28);
    let mut mode = mode;
    if ((shown - 2) as u32) < (2 as u32) {
        zoomboxDraw(s7 + 1, s1_3 + 1, s7 + lw(sp + 24), s1_3 + height - 1, 0, s3_3);
        mode = lw(0x3aff70);
    }
    let kind = lw(0x3aff58);
    if (mode & 1) == 0 {
        let alpha = lbu(g::flashalpha);
        if lw(g::gv_chrset) >= 8 {
            let mut scroll = lw(0x3aff64);
            let colour = (((alpha as u32) >> 1) as i32) << 24 | 128 | (alpha << 8 | alpha << 16);
            if (lw(0x3aff50) - scroll) >= 11 {
                guiDrawImage(5, colour, 32.0_f32, 159.0_f32);
                scroll = lw(0x3aff64);
            }
            if scroll > 0 {
                guiDrawImage(6, colour, 32.0_f32, 61.0_f32);
            }
        }
    } else {
        let alpha;
        'b41: {
            if kind != 0 {
                alpha = lbu(g::flashalpha);
                let _ = alpha;
                if kind != 1 {
                    break 'b41;
                }
                s7 = 288;
                let width = (lw(0x3aff6c) - 1) * 22;
                let top = lw(0x3aff54) * 11;
                sw(sp + 28, 11);
                s1_3 = top + 59;
                sw(sp + 24, width + 221);
            } else {
                let top = lw(0x3aff54) * 11;
                s7 = 64;
                sw(sp + 24, 200);
                sw(sp + 28, 11);
                s1_3 = top + 59;
            }
        }
        let alpha = lbu(g::flashalpha);
        zoomboxDraw(s7, s1_3 + 1, s7 + lw(sp + 24) - 1, s1_3 + lw(sp + 28) - 1, 0, (((alpha as u32) >> 1) as i32 + 64) << 16 | 128);
        let colour = (((lbu(g::flashalpha) as u32) >> 1) as i32 + 64) << 16 | 128;
        guiDrawImage(3, colour, 264.0_f32, (lw(0x3aff5c) * 11 + 61) as f32);
    }
    textSelect(lw(sp + 20));
}
