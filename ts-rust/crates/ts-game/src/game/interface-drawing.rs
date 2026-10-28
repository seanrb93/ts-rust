//! Drawing the menus: the pad action page, the map maker level box, the team lists and the profile bars.

use crate::*;

/// Draw the pad action page: a flashing bar behind the selected box, the highlight, and an arrow image beside it (@ 0x24f418).
pub fn padact_pageDraw(page: i32) {
    let box_ = frontGetBox(page, lhu(page + 4));
    if box_ != 0 {
        let x = lhu(box_);
        let y = lw(g::window + 20) + lhu(box_ + 2) - (lw(g::rolloffset) - 1);
        dlFillRectangle(x + 2, y, x + 252, y + lhu(box_ + 6) - 2, lbu(g::flashalpha) | 0x8000);
    }
    menuitemHighlight(page);
    if box_ != 0 && lhu(page + 4) != 0x8000 {
        let x = (lw(g::window) + lhu(box_) - 30) as f32;
        let y = (lw(g::window + 20) + lhu(box_ + 2) - (lw(g::rolloffset) - 2)) as f32;
        guiDrawImage(3, lbu(g::flashalpha) | 0x7f7f7f00, x, y);
    }
}

/// Draw the level box of the map maker: seven level images going up, coloured by whether the level is the one being edited (flashing green), a warning level (flashing
/// red, while flag 1024 is set), the other level, or none; and two images at the ends (@ 0x24b240).
pub fn mmDrawLevelBox() {
    let x = (lw(g::window) + lw(g::w_x) + 32) as f32;
    let top = lw(g::window + 20) + lw(g::w_y);
    let mut y = top + 51;
    let mut flags = lw(g::editflags);
    for i in 0..7 {
        let mut image = 11;
        let level = i - 3;
        let colour;
        if flags & 1024 != 0 && level == lw(g::warnlevel) {
            colour = lbu(g::quickflash) | 0x7f3f3f00;
        } else if level != lw(g::editlevel) {
            if level != lw(g::otherlevel) {
                image = 10;
                colour = 0x3f3f3f7f;
            } else {
                colour = 0x3f3f50;
            }
        } else {
            colour = lbu(g::flashalpha) | 0x3f7f3f00;
        }
        guiDrawImage(image, colour, x, y as f32);
        y -= 5;
        flags = lw(g::editflags);
    }
    guiDrawImage(12, 0x7f7f7f7f, x, (top + 9) as f32);
    guiDrawImage(13, 0x7f7f7f7f, x, (top + 66) as f32);
}

/// Draw a profile bar at (`x`, `y`): tick lines every 240 pixels (three of them), and the part from `from` to `to` (in units of 240 pixels, at most 2) in a colour (@ 0x2b6460).
pub fn profileTimeBar(x: i32, y: i32, colour: i32, from: f32, to: f32) {
    let step = lwc1(0x3ad7c4);
    let mut position = 0.0_f32;
    let mut offset = ps2float::mul(0.0_f32, 240.0);
    loop {
        position = ps2float::add(position, step);
        let at = x + offset as i32;
        dlFillRectangle(at, y, at, y + 8, 0x7f7f7fff);
        if !(position <= 2.0) {
            break;
        }
        offset = ps2float::mul(position, 240.0);
    }
    for k in 0..3 {
        dlFillRectangle(x - 1 + 240 * k, y - 1, x + 1 + 240 * k, y + 9, 127);
        dlFillRectangle(x + 240 * k, y, x + 240 * k, y + 8, -129);
    }
    let start = if from <= 2.0 { from } else { 2.0 };
    let end = if to <= 2.0 { to } else { 2.0 };
    dlFillRectangle(x + ps2float::mul(start, 240.0) as i32, y + 2, x + ps2float::mul(end, 240.0) as i32, y + 6, colour);
}

/// Draw the images of the players of team `team` (those with a sign-on, a byte of 4 or more at 0x3aff28) along a row at the position kept in the word `x_ptr`, at height `y`
/// (a negative `y` only moves the position on by 1): a shadow and the image, the image by the character (@ 0x235998).
pub fn drawTeamPlayers(team: i32, x_ptr: i32, y: i32) {
    let mut count = lw(0x3aff20);
    let mut i = 0;
    while i < count {
        let team_play = lw(g::gv_teamplay);
        let in_team = team_play == 0 || lw((i << 2) + 0x1fc2c88) + 1 == team;
        if in_team && lbu(i + 0x3aff28) >= 4 {
            if y < 0 {
                sw(x_ptr, lw(x_ptr) + 1);
            } else {
                let sign_on = signonGet(i);
                let chr = lw(sign_on * 2936 + lw(g::signonarray) + 116);
                let image = if (lw(chr * 44 + (CHRS_TABLE + 16)) ^ 1) == 0 { 23 } else { 22 };
                guiDrawImage(image, 64, (lw(x_ptr) + 3) as f32, (y + 1) as f32);
                guiDrawImage(image, -0x7f7fff80, lw(x_ptr) as f32, y as f32);
                sw(x_ptr, lw(x_ptr) + 16);
            }
            count = lw(0x3aff20);
        }
        i += 1;
    }
}

/// The same for the bots of the team (the 10 slots at 0x1fc2a88): in a team game with `coloured` set the colour of the second image is the team's from the story data (@ 0x235b00).
pub fn drawTeamBots(team: i32, x_ptr: i32, y: i32, coloured: i32) {
    let team_colour_at = (team << 3) + (g::storydata + 64);
    let mut x = lw(x_ptr);
    for i in 0..10 {
        let slot = 0x1fc2a88 + 16 * i;
        let team_play = lw(g::gv_teamplay);
        if team_play != 0 {
            let t = lw(slot + 4);
            if t + 1 != team || lbu(t + 0x1fc1234) == 0 {
                continue;
            }
        }
        if lw(slot + 12) != 0 {
            if lw(slot) < 0 {
                continue;
            }
            if y < 0 {
                x += 1;
                sw(x_ptr, x);
            } else {
                let mut colour = -0x7f7f7f80;
                if team_play != 0 && coloured != 0 {
                    colour = lw(team_colour_at + 4);
                }
                let image = if (lw(lw(slot) * 44 + (CHRS_TABLE + 16)) ^ 1) == 0 { 23 } else { 22 };
                guiDrawImage(image, 64, (x + 3) as f32, (y + 1) as f32);
                guiDrawImage(image, colour, lw(x_ptr) as f32, y as f32);
                x = lw(x_ptr) + 16;
                sw(x_ptr, x);
            }
        }
    }
}

/// The portals whose boxes the box `bb` overlaps (the portal records of 48 bytes after the first): their two rooms (from the portal pointer list, entry number + 1) are
/// written to `out` without repeats, at most `max` (@ 0x2573b0).
pub fn bgPortalBBIntersection(bb: i32, out: i32, max: i32) {
    let mut count = 0;
    let portals = lw(g::bgNumPortals);
    if portals > 0 {
        let records = lw(g::portaldata);
        let max_x = lwc1(bb + 12);
        let mut i = 0;
        let mut n = portals;
        loop {
            let rec = records + (i * 48 + 48);
            let mut next = i + 1;
            if !(max_x < lwc1(rec))
                && !(lwc1(rec + 12) < lwc1(bb))
                && !(lwc1(bb + 16) < lwc1(rec + 4))
                && !(lwc1(rec + 16) < lwc1(bb + 4))
                && !(lwc1(bb + 20) < lwc1(rec + 8))
                && !(lwc1(rec + 20) < lwc1(bb + 8))
            {
                let all = lw(g::levelportals);
                let portal = lw((i << 2) + all + 4);
                let room_b = lw(portal + 4);
                let room_a = lw(portal);
                let (mut has_a, mut has_b) = (false, false);
                for k in 0..count {
                    let v = lw((k << 2) + out);
                    has_b |= v == room_b;
                    has_a |= v == room_a;
                }
                let full = count >= max;
                if !has_a && !full {
                    sw((count << 2) + out, room_a);
                    count += 1;
                }
                n = lw(g::bgNumPortals);
                if !has_b && count < max {
                    sw((count << 2) + out, room_b);
                    count += 1;
                    n = lw(g::bgNumPortals);
                }
                next = i + 1;
            }
            i = next;
            if i >= n {
                break;
            }
        }
    }
}

/// Set the blend mode of the display list (-1 keeps the current one); modes 0 to 2 take a prepared packet, higher ones make one; returns the mode set before (@ 0x2b7ea8).
pub fn dlSetBlend(mode: i32) -> i32 {
    let previous = lw(g::dlBlendMode);
    if mode != previous && mode != -1 {
        sw(g::dlBlendMode, mode);
        if mode >= 3 {
            let colour = lw(0x3b01a4);
            let mut words = vec![0x8003, 0x10000000, 14, 0];
            if mode == 3 {
                words.push(100);
            } else if mode == 4 {
                words.push(104);
            }
            words.extend([colour, 66, 0, 0, 0, 73, 0, 0, 0, 59, 0, 0x140007fc, 0x11000000, 0]);
            super::display_list_shapes::push_packet(80, 0x6c048000, 5, &words);
        } else {
            sb(lw(g::gfxtag) + 3, 48);
            let tag = lw(g::gfxtag);
            sw(tag + 4, mode * 80 + g::dlSetBlendVif);
            sh(tag, 5);
            sw(g::gfxtag, tag + 16);
        }
    }
    previous
}

/// Draw the tick lines of a profile bar (three, 240 pixels apart, with a dark frame around each) at (`x`, `y`); `step` is the game's constant that moves the first line.
fn profile_ticks(x: i32, y: i32, step: f32) {
    let mut position = 0.0_f32;
    let mut offset = ps2float::mul(0.0_f32, 240.0);
    loop {
        position = ps2float::add(position, step);
        let at = x + offset as i32;
        dlFillRectangle(at, y, at, y + 8, 0x7f7f7fff);
        if !(position <= 2.0) {
            break;
        }
        offset = ps2float::mul(position, 240.0);
    }
    for k in 0..3 {
        dlFillRectangle(x - 1 + 240 * k, y - 1, x + 1 + 240 * k, y + 9, 127);
        dlFillRectangle(x + 240 * k, y, x + 240 * k, y + 8, -129);
    }
}

/// The simple profile bar: the ticks, the filled part up to `from` (at most 2) in a colour, and a line at `to` if it is beyond `from` (@ 0x2b6058).
pub fn profileSimpleTimeBar(x: i32, y: i32, colour: i32, from: f32, to: f32) {
    profile_ticks(x, y, lwc1(0x3ad7c0));
    let end = if from <= 2.0 { from } else { 2.0 };
    dlFillRectangle(x, y + 2, x + ps2float::mul(end, 240.0) as i32, y + 6, colour);
    if !(from < to) {
        return;
    }
    let offset = ps2float::mul(to, 240.0) as i32;
    dlFillRectangle(x + offset, y, x + (offset + 1), y + 8, 0x7f7f7fff);
}
