//! Drawing the pages of the front end, restarting the particles and the decals, making a standard menu item, and the update of a prop's last place.

use crate::*;

/// Draw the front end: for each page (records of 16 bytes: the page definition at +0, the list of boxes at +8, their count at +12) call the definition's draw function (+4) with the
/// page, then the draw function (+12) of each box (24 bytes each) and the definition's end function (+12); then, if the front end has a function at +40, call it with the box
/// of the last page that is current (flag 1 at +12) or 0 (@ 0x22cd30).
pub fn frontGfx() {
    let mut front = lw(g::front);
    let mut i = 0;
    if lw(front + 20) > 0 {
        let mut table = front;
        loop {
            let page = lw(table + 4) + (i << 4);
            let definition = lw(page);
            let draw = lw(definition + 4);
            let count = lw(page + 12);
            if draw != 0 {
                call_indirect(draw, [page, front, 0, 0, 0, 0, 0, 0], [0.0; 8]);
            }
            let mut box_at = lw(page + 8);
            let mut k = 0;
            i += 1;
            if count > 0 {
                loop {
                    let box_draw = lw(box_at + 12);
                    if box_draw != 0 {
                        call_indirect(box_draw, [box_at, front, 0, 0, 0, 0, 0, 0], [0.0; 8]);
                    }
                    k += 1;
                    box_at += 24;
                    if k >= count {
                        break;
                    }
                }
            }
            let end = lw(lw(page) + 12);
            front = lw(g::front);
            if end != 0 {
                call_indirect(end, [page, front, 0, 0, 0, 0, 0, 0], [0.0; 8]);
                front = lw(g::front);
            }
            table = front;
            if i >= lw(front + 20) {
                break;
            }
        }
    }
    if lw(front + 40) != 0 {
        let pages = lw(front + 20);
        let mut current = 0;
        if pages != 0 {
            let last = lw(front + 4) + ((pages << 4) - 16);
            if last != 0 && (lw(front + 12) & 1) != 0 {
                current = frontGetBox(last, lhu(last + 4));
                front = lw(g::front);
            }
        }
        call_indirect(lw(front + 40), [current, front, 0, 0, 0, 0, 0, 0], [0.0; 8]);
    }
}

/// Restart the particles: clear the 100 groups (0x17a0 bytes each, in `pgroups`) and link them into a list (each is free, -1 at +0, with the groups before and after at +64 and +68, none
/// at the ends); clear two tables of 16 words (at 0x1fe99c0 and 0x1fe9a00), find the texture numbers of the 11 particle pictures (at +12 of 16-byte records) and keep seven of them
/// for the particles that are made often (@ 0x296928).
pub fn particleRestart() {
    let groups = lw(g::pgroups);
    memset(groups, 0, 0x93a80);
    let groups = lw(g::pgroups);
    let mut after = groups + 0x17a0;
    let mut before = groups - 0x17a0;
    let mut at = groups;
    let mut i = 0;
    loop {
        sw(at, -1);
        sw(at + 60, 0);
        sw(at + 56, 0);
        if i != 0 {
            sw(at + 64, before);
        } else {
            sw(groups + 64, 0);
        }
        if i != 99 {
            sw(at + 68, after);
        } else {
            sw(groups + 0x92324, 0);
        }
        i += 1;
        after += 0x17a0;
        before += 0x17a0;
        at += 0x17a0;
        if i >= 100 {
            break;
        }
    }
    sw(0x3b0150, groups);
    let mut first = 0x1fe99c0;
    let mut second = 0x1fe9a00;
    let mut left = 15;
    loop {
        sw(first, 0);
        left -= 1;
        sw(second, 0);
        first += 4;
        second += 4;
        if left < 0 {
            break;
        }
    }
    let mut left = 10;
    let mut record = g::particletextures;
    loop {
        let name = lw(record);
        left -= 1;
        let texture = preloadGetTexName(name);
        sw(record + 12, lw(texture + 20));
        record += 16;
        if left < 0 {
            break;
        }
    }
    let textures = g::particletextures;
    let t0 = lw(textures + 12);
    let t1 = lw(textures + 28);
    let t4 = lw(textures + 76);
    let t7 = lw(textures + 124);
    let t6 = lw(textures + 108);
    sw(0x3b0158, lw(textures + 44));
    sw(0x3b015c, t4);
    sw(0x3b0160, t7);
    sw(0x3b0164, t6);
    sw(0x3b0168, t0);
    sw(0x3b016c, t1);
    sw(0x3b0154, t1);
}

/// Make the menu box `menu` for the item `item` of the standard menu placed by `at`: like `menuitemmake`, but an item with flag 4 is at least 300 wide, an item with the value 0x8000
/// has 32 more width and starts 32 earlier, and an item with flag 16 is put in the middle of the window (@ 0x22d7c0).
pub fn std_menuitemmake(menu: i32, at: i32, item: i32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let y = lh(at + 2);
    let mut text = lw(item);
    sw(sp, 0);
    sw(sp + 4, 0);
    let mut x = lh(at);
    if text != 0 {
        let mut old = -1;
        if (lhu(item + 10) & 256) != 0 {
            old = textSelect(lw(g::menuitem_bigfont));
            text = lw(item);
        }
        let language = lw(g::cur_language);
        if text >= 1246 {
            textSize(sp, sp | 4, text);
        } else {
            textSize(sp, sp | 4, lw((text << 2) + lw((language << 2) + g::languages)));
        }
        if old != -1 {
            textSelect(old);
        }
    }
    let flags = lhu(item + 10);
    let width = lw(sp);
    if (flags & 4) != 0 && width < 300 {
        sw(sp, 300);
    }
    let mut centred = flags & 16;
    if lhu(item + 8) == 0x8000 {
        x -= 32;
        sw(sp, lw(sp) + 32);
        centred = flags & 16;
    }
    if centred != 0 {
        let spare = lw(g::window + 12) - lw(sp);
        x = ((spare + ((spare as u32) >> 31) as i32) >> 1) + lh(at);
    }
    makebox(menu, (x << 16) >> 16, y, lh(sp), (lhu(sp + 4) << 16) >> 16, lhu(item + 8), 0, 0x22d230);
    let text = lw(item);
    sw(menu + 16, item);
    if text >= 1246 {
        sw(menu + 20, text);
    } else {
        sw(menu + 20, lw((text << 2) + lw((lw(g::cur_language) << 2) + g::languages)));
    }
}

/// Restart the decals: the groups are one for each eight rooms (at least 1; `s_NumGroups`), each of up to 75 decals of `s_MaxNumDecals` shared out; clear the group heads, the
/// lists, the decals (-1 at +204, no list links) and the decal pictures' texture numbers (12 bytes each from `s_DecalTextures`, the name at +8), and the 15 create records (@ 0x2a24e0).
pub fn decalRestart() {
    let rooms = lw(g::bgNumRooms);
    let padded = rooms + 8;
    let groups = (if -1 < padded { padded } else { rooms + 15 }) >> 3;
    sw(g::s_MovingPropRoom, rooms);
    sw(g::s_NumGroups, groups);
    if groups <= 0 {
        sw(g::s_NumGroups, 1);
    }
    let max = lw(g::s_MaxNumDecals);
    let group_count = lw(g::s_NumGroups);
    if group_count == 0 {
        trap("break");
    }
    sw(g::s_DecalsToGroup, max.wrapping_div(group_count));
    if max.wrapping_div(group_count) >= 76 {
        sw(g::s_DecalsToGroup, 75);
    }
    let mut i = 0;
    if group_count > 0 {
        loop {
            let at = (i << 2) + 0x1fe9cd8;
            i += 1;
            sw(at, 0);
            if i >= group_count {
                break;
            }
        }
    }
    let mut left = 211;
    let mut at = 0x1fea094;
    loop {
        sw(at, 0);
        left -= 1;
        at -= 4;
        if left < 0 {
            break;
        }
    }
    let mut i = 0;
    if max > 0 {
        let decals = lw(g::s_Decals);
        loop {
            let decal = i * 240 + decals;
            i += 1;
            sw(decal + 204, -1);
            sw(decal + 208, 0);
            sw(decal + 192, 0);
            sw(decal + 196, 0);
            if i >= max {
                break;
            }
        }
    }
    let mut record = g::s_DecalTextures;
    let end = record + 144;
    loop {
        let texture = preloadGetTexName(lw(record + 8));
        sw(record, lw(texture + 20));
        record += 12;
        if record >= end {
            break;
        }
    }
    let mut left = 14;
    let mut at = 0x1fe9cac;
    loop {
        sw(at, 0);
        left -= 1;
        at -= 44;
        if left < 0 {
            break;
        }
    }
    sw(g::s_DecalCreateListRead, 0);
    sw(g::s_DecalCreateListWrite, 0);
}

/// Keep the prop's place for the next tick: when it has not been moved since its last place was kept (and is not flagged 0x40000000) nothing is done about its rooms, else
/// `propCalcRooms` is run; a prop of kind 8 (a character's) adds the distance it moved to the player's statistic 46; the place, the flags, the angle and the other three floats
/// (+72, +76, +88) are kept at +112 to +132 and +20 (@ 0x269d78).
pub fn propUpdate(prop: i32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let mut x = lwc1(prop + 48);
    let mut y;
    let mut z;
    let kind;
    'moved: {
        if (lw(prop + 16) & 0x40000000) == 0 {
            if x == lwc1(prop + 112) {
                y = lwc1(prop + 52);
                if y == lwc1(prop + 116) {
                    z = lwc1(prop + 56);
                    if z == lwc1(prop + 120) && lwc1(prop + 76) == lwc1(prop + 128) && lw(prop + 184) != 0 {
                        kind = lw(prop + 8);
                        break 'moved;
                    }
                }
            }
            propCalcRooms(prop);
            x = lwc1(prop + 48);
        }
        y = lwc1(prop + 52);
        z = lwc1(prop + 56);
        kind = lw(prop + 8);
    }
    let turn;
    if kind != 8 {
        turn = lwc1(prop + 72);
    } else {
        let dx = ps2float::sub(x, lwc1(prop + 112));
        let dy = ps2float::sub(y, lwc1(prop + 116));
        let dz = ps2float::sub(z, lwc1(prop + 120));
        swc1(sp, dx);
        swc1(sp + 4, dy);
        swc1(sp + 8, dz);
        let (ax, ay, az) = (lwc1(sp), lwc1(sp + 4), lwc1(sp + 8));
        let moved = dptofp(sqrt(fptodp(ps2float::add(ps2float::add(ps2float::mul(ax, ax), ps2float::mul(ay, ay)), ps2float::mul(az, az)))));
        if 0.0_f32 < moved {
            StatsAdd(46, lw(lw(prop + 352)), moved);
        }
        x = lwc1(prop + 48);
        y = lwc1(prop + 52);
        z = lwc1(prop + 56);
        turn = lwc1(prop + 72);
    }
    let angle = lwc1(prop + 76);
    let other = lwc1(prop + 88);
    let flags = lw(prop + 16);
    swc1(prop + 112, x);
    sw(prop + 20, flags);
    swc1(prop + 116, y);
    swc1(prop + 120, z);
    swc1(prop + 124, turn);
    swc1(prop + 128, angle);
    swc1(prop + 132, other);
}
