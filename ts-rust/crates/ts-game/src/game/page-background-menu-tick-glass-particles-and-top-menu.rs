//! The background of a front end page, the tick of a standard menu, the glass particles, and the top menu.

use crate::*;

/// Draw the background of the front end page `page`: a box round its area (from `frontGetPageSize`) made wide enough for the title `title`, filled dark with a border of four lines,
/// the title above it with an outline, the highlight of the current item, and a bar under the current box that flashes (@ 0x206578).
pub fn pagedrawbg(page: i32, title: i32) {
    let frame = Frame::new(48);
    let sp = frame.sp();
    sw(sp + 24, page);
    sw(sp + 28, title);
    let old = textSelect(0);
    sw(sp + 32, old);
    frontGetPageSize(lw(sp + 24), sp);
    let left = lw(sp);
    let right = lw(sp + 8);
    let height = lw(sp + 12);
    sw(sp + 36, lw(sp + 4));
    textSize(sp + 16, sp + 20, lw(sp + 28));
    let text_w = lw(sp + 16);
    let window_x = lw(g::window);
    let text_x = (left + right - text_w) >> 1;
    let text_end = text_x + text_w;
    let top = lw(sp + 36);
    let window_y = lw(g::window + 20);
    let box_left = window_x + (if text_x < left { text_x } else { left });
    let box_right = window_x + (if right < text_end { text_end } else { right });
    let box_top = window_y + top;
    let title_y = top - lw(sp + 20) - 4 + window_y;
    let inner_left = box_left - 5;
    let inner_right = box_right + 5;
    let border_left = box_left - 6;
    let border_right = box_right + 6;
    let bottom = window_y + height;
    dlFillRectangle(inner_left, title_y + 1, inner_right, box_top - 3, 0x1e0080);
    sw(sp + 36, box_top - 2);
    dlFillRectangle(inner_left, box_top - 1, inner_right, bottom + 5, 0x1e0080);
    let border_bottom = bottom + 6;
    dlFillRectangle(border_left, title_y, border_right, title_y, 0x7f0080);
    let rule = lw(sp + 36);
    dlFillRectangle(border_left, rule, border_right, rule, 0x7f0080);
    dlFillRectangle(border_left, border_bottom, border_right, border_bottom, 0x7f0080);
    dlFillRectangle(border_left, title_y, border_left, border_bottom, 0x7f0080);
    dlFillRectangle(border_right, title_y, border_right, border_bottom, 0x7f0080);
    textOutline(text_x + window_x, title_y, lw(sp + 28), 0x7f0080, 0xa0080);
    textSelect(lw(sp + 32));
    menuitemHighlight(lw(sp + 24));
    let current = lw(sp + 24);
    let found = frontGetBox(current, lhu(current + 4));
    if found != 0 {
        let x = lhu(found);
        let y = lhu(found + 2) + lhu(found + 6);
        dlFillRectangle(x, y, x + lhu(found + 4), y + 2, lbu(g::flashalpha) << 24 | 0x7f3f80);
    }
}

/// The tick of a standard menu (`menu` its page record, `list` the menu's items): follow the current item (+4) to the next one that has the buttons the front end flags 4 shift
/// name; keep the scroll of a list in view; with the front end's flags 48 (left or right) change the value of a slider item by the step (655 for a wide one), `gvSetValue`; a
/// press of 64 selects the item (opens a list or goes to its page), a press of 16 goes to the page of the item 0x8000 (@ 0x22df40).
pub fn std_menutick(menu: i32, list: i32) {
    let current = lhu(menu + 4);
    if current == 0 {
        return;
    }
    let mut item = menuitem(list, current);
    let mut front = lw(g::front);
    if item == 0 {
        return;
    }
    let mut i = 0;
    let mut at = item + 16;
    let flags = lw(front + 12);
    loop {
        let wanted = flags & (4 << (i & 31));
        i += 1;
        if wanted != 0 {
            let target = lhu(at);
            if target != 0 {
                sh(menu + 4, target);
                item = menuitem(list, target & 0xffff);
                let front_now = lw(g::front);
                sw(front_now + 12, lw(front_now + 12) & -61);
                break;
            }
        }
        at += 2;
        if i >= 4 {
            break;
        }
    }
    let in_list = isinlist(list, lhu(menu + 4));
    front = lw(g::front);
    if in_list != 0 {
        let base = in_list + 12;
        let place = (item - (in_list + 16)) >> 5;
        let scroll_at = base + lw(front);
        let from_top = place - lbu(scroll_at);
        let shown;
        let mut in_range = false;
        if from_top > 0 {
            let rows = lhu(in_list + 2);
            if from_top < rows - 1 {
                shown = lhu(in_list);
                in_range = true;
            } else {
                sb(scroll_at, place - lbu(in_list + 2) + 2);
                front = lw(g::front);
                shown = lhu(in_list);
            }
        } else {
            let above = place - 1;
            sb(scroll_at, if -1 >= above { 0 } else { above });
            front = lw(g::front);
            shown = lhu(in_list);
        }
        let _ = in_range;
        if !(place < shown - 1) {
            sb(base + lw(front), lbu(in_list) - lbu(in_list + 2));
        }
    }
    front = lw(g::front);
    let front_flags = lw(front + 12);
    if (front_flags & 48) != 0 && (lw(item + 8) & 0x440000) == 0x40000 {
        let value = lw(item + 12);
        let mut step = lw(front + 16);
        if (lw(value + 12) & 2) != 0 {
            step *= 655;
        }
        let step = step >> 5;
        if step != 0 {
            gvSetValue(value, lw(value) + step);
            return;
        }
    }
    let joystick = lw(g::menujoy);
    if (joyNewBut(joystick) & 64) == 0 {
        let joystick = lw(g::menujoy);
        if (joyNewBut(joystick) & 16) != 0 {
            let back = menuitem(list, 0x8000);
            if back != 0 {
                frontPageSet(0, lw(back + 12));
            }
        }
    } else if (lhu(item + 10) & 64) == 0 {
        let selected = isinlist(list, lhu(menu + 4));
        if selected != 0 && (lhu(selected + 8) & 1) != 0 {
            menuitemSelect(list, lhu(menu + 4));
            sh(selected + 10, lhu(menu + 4));
            return;
        }
        let item_flags = lhu(item + 10);
        if (item_flags & 64) == 0 && (item_flags & 1) != 0 && lhu(item + 8) != 0x8000 {
            frontPageSet(0, lw(item + 12));
        }
    }
}

/// The tick of a group of glass particles `group`: for each of the pieces of the glass (the count at +0x1188 of the third level of owners, 64 bytes of matrix and 12 of speed and
/// spin each) the time at +0x13e0 goes up, the piece moves by its speed and falls, turns by its spin, its colour's alpha follows its age (fading in the end, or at once when the
/// flag 256 of the group is set), and the matrix is made from the place and turns (@ 0x29b890).
pub fn particleGlassTick(group: i32) {
    let frame = Frame::new(48);
    let sp = frame.sp();
    let step = lwc1(g::updateframesf);
    sw(sp, group + 928);
    let dt = step / 6e+01_f32;
    let owner = lw(lw(lw(group + 60) + 60) + 60);
    let base = owner + 928;
    let mut count = lw(base + 0x1188);
    let mut i = 0;
    if count > 0 {
        sw(sp + 4, group + 976);
        let times = owner + 0x13e0;
        sw(sp + 16, owner + 2208);
        sw(sp + 8, group + 980);
        sw(sp + 20, owner + 2212);
        sw(sp + 12, group + 984);
        let speed_z = owner + 2216;
        loop {
            let at12 = i * 12;
            let at64 = i << 6;
            let at4 = i << 2;
            let time_at = times + at4;
            let spin = lw(sp + 20) + i * 12;
            let time = lwc1(time_at) + dt;
            let fall_z = lwc1(speed_z + i * 12) * dt;
            let fall_y = lwc1(spin) * dt;
            let y = lwc1(lw(sp + 8) + at64);
            let z = lwc1(lw(sp + 12) + at64);
            let x = lwc1(lw(sp + 4) + at64) + lwc1(lw(sp + 16) + i * 12) * dt;
            swc1(time_at, time);
            let new_y = y + fall_y;
            let new_z = z + fall_z;
            let next = i + 1;
            if 0.0_f32 <= time {
                let turn_x = base + 2240;
                let turn_y = base + 2244;
                let turn_z = base + 2248;
                let rx = lwc1(turn_x + at12);
                let ry = lwc1(turn_y + at12);
                let rz = lwc1(turn_z + at12);
                matrixTransRotZXY(lw(sp) + at64, x, new_y, new_z, rx, ry, rz);
                let updates = lw(g::updateframesi);
                swc1(spin, lwc1(spin) + dt * -7.5_f32);
                let flags = lw(group + 8);
                let spins_x = base + 3200;
                let spins_y = base + 3204;
                let spins_z = base + 3208;
                let at16 = i << 4;
                let colour0 = base + 4;
                let colour1 = base + 8;
                let colour2 = base + 12;
                let mut k = 0;
                if updates > 0 {
                    loop {
                        let drag = lwc1(0x3ad504);
                        k += 1;
                        let a = lw(sp + 16) + i * 12;
                        let b = speed_z + i * 12;
                        swc1(a, lwc1(a) * drag);
                        swc1(b, lwc1(b) * drag);
                        if k >= updates {
                            break;
                        }
                    }
                }
                let a = turn_x + i * 12;
                let b = turn_y + i * 12;
                let c = turn_z + i * 12;
                swc1(a, lwc1(a) + lwc1(spins_x + i * 12) * dt);
                swc1(b, lwc1(b) + lwc1(spins_y + i * 12) * dt);
                swc1(c, lwc1(c) + lwc1(spins_z + i * 12) * dt);
                let mut alpha = 1.0_f32;
                if (flags & 256) == 0 {
                    alpha -= lwc1(times + at4) / lwc1(0x3ad50c);
                } else {
                    let start = lwc1(0x3ad508);
                    let age = lwc1(times + at4);
                    if start < age {
                        alpha = start - (age - alpha) * 4.0_f32;
                    }
                }
                if alpha < 0.0_f32 {
                    alpha = 0.0_f32;
                }
                let corner0 = base + at16;
                let corner1 = colour0 + at16;
                let corner2 = colour1 + at16;
                let corner3 = colour2 + at16;
                let mask = 0xffffff;
                sw(corner0, lw(corner0) & mask);
                sw(corner1, lw(corner1) & mask);
                let top = ((alpha * 127.0_f32) as i32) << 24;
                sw(corner2, lw(corner2) & mask);
                sw(corner3, lw(corner3) & mask);
                sw(corner0, lw(corner0) | top);
                sw(corner1, lw(corner1) | top);
                sw(corner2, lw(corner2) | top);
                sw(corner3, lw(corner3) | top);
                count = lw(base + 0x1188);
            }
            i = next;
            if i >= count {
                break;
            }
        }
    }
}

/// Draw the top menu (the pause menu's heading of the current player's record of 92 bytes at 0x1fb3240): the title of the page, with the player's number put in the place of
/// the "#" of the text, in the middle at the top with an outline; for a page with nothing under it, arrows to the page before and after (flashing); else the page's second
/// line (the texts at +0x1054 to +0x105c of the language) under it (@ 0x20dd58).
pub fn topmenuGfx(menu: i32) {
    let record = lw(lw(g::currentplayer)) * 92 + 0x1fb3240;
    let players = ilinkGetNumLocalPlayers();
    let old = textSelect(((players < 2) as i32) ^ 1);
    let page = lhu(record + 4);
    let at = page << 1;
    let mut language = lw(g::cur_language);
    if ((page - 1) as u32) < 2 {
        let mut k = 0;
        if lb(lw((lh(at + g::topmenus) << 2) + lw((language << 2) + g::languages))) != 0 {
            let mut found = false;
            let mut place = 0;
            loop {
                let text = lw((lh(at + g::topmenus) << 2) + lw((language << 2) + g::languages));
                place = text + k;
                k += 1;
                if lb(place) == 35 {
                    found = true;
                    break;
                }
                if lb(text + k) == 0 {
                    break;
                }
            }
            if found {
                sb(place, lbu(lw(g::currentplayer)) + 49);
                language = lw(g::cur_language);
            }
        }
    }
    let texts = lw((language << 2) + g::languages);
    let mut title = lw((lh(at + g::topmenus) << 2) + texts);
    if lh(record + 2) == 0 && (lw(record + 8) & 0x400008) == 0x400008 {
        title = lw(texts + 0x1050);
    }
    let width = textWidth(title);
    let top = lw(g::window + 20);
    let spare = lw(g::window + 12) - width;
    let y = if lw(g::window + 92) != 0 { top + 8 } else { top + 26 };
    let x = lw(g::window) + ((spare + ((spare as u32) >> 31) as i32) >> 1);
    textOutline(x, y, title, 0x7f0080, 0x180080);
    let font_flags = lbu(g::fontflags);
    if lh(record + 2) <= 0 {
        let colour = if (lw(g::tickframenum) & 32) != 0 { 0x7f0080 } else { 0x7f0040 };
        let line = if (lbu(g::fontflags) & 1) == 0 {
            lw(lw(g::curfont) + 12)
        } else {
            lw(lw(g::curfont) + 12) >> 1
        };
        let line = line - 8;
        let mut previous = lhu(record + 4);
        let bits = lhu(record);
        let current = previous;
        loop {
            let before = previous - 1;
            previous = if -1 >= before { 9 } else { before };
            if (bits >> (previous & 31) & 1) == 0 {
                break;
            }
        }
        if previous != current {
            let width = lw(g::window + 12);
            let arrow_y = (y + ((line + ((line as u32) >> 31) as i32) >> 1)) as f32;
            guiDrawImage(20, colour, (lw(g::window + 8) - width.wrapping_div(3) - 28) as f32, arrow_y);
            let width = lw(g::window + 12);
            guiDrawImage(21, colour, (lw(g::window + 8) + width.wrapping_div(3) + 12) as f32, arrow_y);
        }
    } else {
        let font = lw(g::curfont);
        let below = if (font_flags & 1) == 0 { y + lw(lw(g::curfont) + 12) } else { y + (lw(font + 12) >> 1) };
        let shown = lw(record + (lhu(record + 4) << 2) + 12);
        let text;
        if shown != g::setupcontrols_page {
            let language = lw(g::cur_language);
            if shown == g::setup_page && lhu(lw(lw(g::currentplayer) + 24)) == 0 {
                text = lw(lw((language << 2) + g::languages) + 0x1058);
            } else {
                text = lw(lw((language << 2) + g::languages) + 0x105c);
            }
        } else {
            text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 0x1054);
        }
        textSelect(1);
        let width = textWidth(text);
        let spare = lw(g::window + 12) - width;
        textOutline(lw(g::window) + ((spare + ((spare as u32) >> 31) as i32) >> 1), below, text, 0x50505080, 100);
    }
    textSelect(old);
    menuitemHighlight(menu);
}
