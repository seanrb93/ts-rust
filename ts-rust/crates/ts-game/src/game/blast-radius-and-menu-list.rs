//! The ring of a blast, a standard menu's item list, clipping a portal against a plane, and the tune of the level.

use crate::*;

/// Draw the ring of a blast at the place `at` (three floats) for the time `time` of a blast that starts at `start` and lasts `duration`, with the height `raise` added: a quad
/// that grows with the time (3 times the part gone) and fades (127 less 127 times the part gone, 0x3f6a7f the colour), in the picture at 0x3b018c; nothing outside its time (@ 0x2abd08).
pub fn specialfxBlastRadius(at: i32, time: f32, start: f32, duration: f32, raise: f32) {
    let frame = Frame::new(80);
    let sp = frame.sp();
    if !(start <= time) || !(time < ps2float::add(start, duration)) {
        return;
    }
    let top = 127.0_f32;
    let gone = (ps2float::sub(time, start)).pdiv(duration);
    let z = lwc1(at + 8);
    let y = ps2float::add(lwc1(at + 4), raise);
    let x = lwc1(at);
    let player = lw(g::currentplayer);
    swc1(sp + 68, y);
    swc1(sp + 20, y);
    swc1(sp + 36, y);
    let size = ps2float::mul(gone, 3.0_f32);
    swc1(sp + 52, y);
    let texture = lw(0x3b018c);
    let z_low = ps2float::sub(z, size);
    let x_high = ps2float::add(x, size);
    let x_low = ps2float::sub(x, size);
    swc1(sp + 72, z_low);
    let z_high = ps2float::add(z, size);
    swc1(sp + 64, x_high);
    swc1(sp + 32, x_high);
    let colour = (ps2float::sub(top, ps2float::mul(gone, top)) as i32) << 24 | 0x3f6a7f;
    swc1(sp + 48, x_low);
    sw(sp, colour);
    swc1(sp + 40, z_high);
    let matrix = lw(player + 1764);
    swc1(sp + 16, x_low);
    swc1(sp + 24, z_high);
    swc1(sp + 56, z_low);
    quadRenderAligned(texture, matrix, sp + 16, 128, 128, colour, colour, colour, 0.0_f32, 0.0_f32);
}

/// Make the menu items of the page `page` into `menus` (records of 24 bytes) like `menulistmake` but with the standard menu's items (`std_menuitemmake`): the x of each item from
/// the page's x and the item's, the y going down by the font's line, both shifted by the window's split flags; the number made is returned (@ 0x22d988).
pub fn std_menulistmake(menus: i32, page: i32) -> i32 {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let mut at = menus;
    let scroll = page + 12;
    let window = g::window;
    let first = lbu(scroll + lw(lw(g::front)));
    let count = lhu(page + 2);
    let mut i = first;
    sh(sp + 2, lh(page + 6) >> (lw(window + 92) & 31));
    let mut item = page + 16 + (first << 5);
    if i < i + count && i < lhu(page) {
        let mut base_x = lh(page + 4);
        loop {
            sh(sp, (base_x + lh(item + 4)) >> (lw(window + 88) & 31));
            std_menuitemmake(at, sp, item);
            let y = lh(sp + 2);
            let line = if (lbu(g::fontflags) & 1) == 0 {
                lw(lw(g::curfont) + 12)
            } else {
                lw(lw(g::curfont) + 12) >> 1
            };
            let next_y = (y - 1) + line;
            i += 1;
            let shown = lhu(page + 2);
            at += 24;
            let front = lw(lw(g::front));
            sh(sp + 2, next_y);
            item += 32;
            if i >= lbu(scroll + front) + shown {
                break;
            }
            if i >= lhu(page) {
                break;
            }
            base_x = lh(page + 4);
        }
    }
    ((at - menus).wrapping_mul(-0x55555555)) >> 3
}

/// Clip the polygon `from` (a list of pointers to vertices of 20 bytes, the count at +80) against a plane and put the result in `to` (the same layout, new vertices taken from the
/// pool at 0x3b0098 with the count at 0x3b009c): a vertex is kept when its outcode has none of the bits of `mask`; where an edge goes from outside to inside a new vertex is made at the
/// plane (the coordinate number `axis` times `a` plus the w times `b`); the polygon is closed by repeating the first vertex (@ 0x257f20).
pub fn bgPortalPlaneClip(from: i32, to: i32, axis: i32, mask: i32, a: f32, b: f32) {
    if lw(from + 80) <= 0 {
        return;
    }
    sw(to + 80, 0);
    let mut i = 0;
    if lw(from + 80) > 0 {
        let at = axis << 2;
        loop {
            let next_index = i + 1;
            let current = lw(from + (i << 2));
            let next = lw(from + (next_index << 2));
            let mut next_flags = lw(next + 16);
            if (lw(current + 16) & mask) != next_flags & mask {
                let used = lw(0x3b009c);
                let out_count = lw(to + 80);
                let pool = lw(0x3b0098);
                let d_current = ps2float::add(ps2float::mul(a, lwc1(current + at)), ps2float::mul(lwc1(current + 12), b));
                let d_next = ps2float::add(ps2float::mul(a, lwc1(next + at)), ps2float::mul(lwc1(next + 12), b));
                sw(0x3b009c, used + 1);
                let made = pool + used * 20;
                sw(to + (out_count << 2), made);
                sw(to + 80, out_count + 1);
                let t = (d_current).pdiv(ps2float::sub(d_current, d_next));
                let x = lwc1(current);
                swc1(made, ps2float::add(x, ps2float::mul(t, ps2float::sub(lwc1(next), x))));
                let y = lwc1(current + 4);
                swc1(made + 4, ps2float::add(y, ps2float::mul(t, ps2float::sub(lwc1(next + 4), y))));
                let z = lwc1(current + 8);
                swc1(made + 8, ps2float::add(z, ps2float::mul(t, ps2float::sub(lwc1(next + 8), z))));
                let w = lwc1(current + 12);
                swc1(made + 12, ps2float::add(w, ps2float::mul(t, ps2float::sub(lwc1(next + 12), w))));
                bgPortalCalcOutCode(made);
                next_flags = lw(next + 16);
            }
            if (next_flags & mask) == 0 {
                let n = lw(to + 80);
                sw(to + (n << 2), next);
                sw(to + 80, n + 1);
            }
            i = next_index;
            if !(i < lw(from + 80)) {
                break;
            }
        }
    }
    sw(to + (lw(to + 80) << 2), lw(to));
}

/// The number of the tune of the current level: 0 when the program parameter at 0x3ae758 is given; for a level of the table `LVINFO` (24-byte records, the number of its name text
/// at +0, the level at +4, the tune at +8) the tune of the level, 0 if it is not there; for level 100 (a map maker level) the tune the map file says (+10) or by its game kind (3, 2, 1, 5, 4) (@ 0x225950).
pub fn lvGetTuneNum() -> i32 {
    if bossCheckParm(0x3ae758) != 0 {
        return 0;
    }
    let level = lw(g::lvlevelnum);
    let language = lw(g::cur_language);
    if level != 100 {
        let texts = lw((language << 2) + g::languages);
        if lw((lh(LVINFO) << 2) + texts) == 0 {
            return 0;
        }
        let base = LVINFO;
        let mut offset = 0;
        let mut at = base;
        loop {
            let tag = lw(at + 4);
            at += 24;
            if tag == level {
                break;
            }
            offset += 24;
            if lw((lh(at) << 2) + texts) == 0 {
                return 0;
            }
        }
        return lw(base + offset + 8);
    }
    let info = mmfileGetInfo(lw(g::mmtempbuffer));
    let tune = lhu(info + 10);
    if tune != 0 {
        return tune;
    }
    let kind = lhu(info + 8);
    if (kind as u32) < 5 {
        return match kind {
            0 => 3,
            1 => 2,
            2 => 1,
            3 => 5,
            _ => 4,
        };
    }
    0
}
