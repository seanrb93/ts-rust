//! Which character are available, the rooms of pads and props, and the background of a front end page.

use crate::*;

/// Which of the 64 characters (in the order of `characterorder`) are available, as a bit each in the three words at `out` (@ 0x240380).
pub fn goodiesGetAvailableChrs(out: i32) {
    sw(out, 0);
    sw(out + 4, 0);
    sw(out + 8, 0);
    let mut i = 0;
    loop {
        let chr = lbu(i + g::characterorder);
        let available = chrAvail(chr);
        let word = ((i >> 5) << 2) + out;
        let bit = 1 << (i & 31 & 31);
        i += 1;
        if available != 0 {
            sw(word, lw(word) | bit);
        }
        if !((i as u32) < 64) {
            break;
        }
    }
}

/// Find the room of each pad that has flag 1 (a word at +4 of the pad of 28 bytes): the room (a halfword at +10) of the point of the pad raised by 1.25, and the height at +16 as
/// the floor's height plus the game's constant (@ 0x265790).
pub fn padSetRoomNums() {
    let mut pads_count = lw(g::numpads);
    let mut pad = 0;
    if pads_count > 0 {
        let mut pads = lw(g::levelpads);
        loop {
            let offset = pad * 28;
            let record = pad * 28 + pads;
            if (lw(record + 4) & 1) != 0 {
                let room = moveFindRoom(record + 12, 1.25_f32, 0.0_f32);
                pads = lw(g::levelpads);
                let height = ps2float::add(lwc1(g::lastfloorheight), lwc1(0x3acba4));
                pads_count = lw(g::numpads);
                let target = offset + pads;
                sh(target + 10, room);
                swc1(target + 16, height);
            }
            pad += 1;
            if pad >= pads_count {
                break;
            }
        }
    }
}

/// Find the rooms of a prop (unless flag 0x40000000 is set; in the levels 102 and 103 it has none): a prop of kind 2 is in the room under its number (+4) less 512; a prop that is not
/// of kind 4 or 256 is first looked for by the room under its centre (+508 offset from its position), and if no room holds it, or for the other kinds, the rooms its box
/// (the half sizes at +524 and +528) touches (10 at most, at +144, their number at +184) (@ 0x269b30).
pub fn propCalcRooms(prop: i32) {
    let frame = Frame::new(80);
    let sp = frame.sp();
    if (lw(prop + 16) & 0x40000000) != 0 {
        return;
    }
    if ((lw(g::lvlevelnum) - 102) as u32) < 2 {
        sw(prop + 184, 0);
        sw(prop + 144, 0);
        return;
    }
    let kind = lw(prop + 8);
    if kind == 2 {
        let number = lw(prop + 4);
        sw(prop + 184, 1);
        sw(prop + 144, number - 512);
        return;
    }
    let search_box;
    if kind != 256 && kind != 4 {
        let x_offset = lwc1(prop + 508);
        let y = ps2float::add(lwc1(prop + 52), lwc1(prop + 512));
        let radius = lwc1(0x3acbec);
        let z = ps2float::add(lwc1(prop + 56), lwc1(prop + 516));
        swc1(sp + 32, ps2float::add(lwc1(prop + 48), x_offset));
        swc1(sp + 36, y);
        swc1(sp + 40, z);
        let room = moveFindRoom(sp + 32, radius, radius);
        sw(prop + 144, room);
        sw(prop + 184, 1);
        search_box = room == 0;
    } else {
        search_box = true;
    }
    if search_box {
        let x = ps2float::add(lwc1(prop + 48), lwc1(prop + 508));
        let y = ps2float::add(lwc1(prop + 52), lwc1(prop + 512));
        let half_x = lwc1(prop + 528);
        let z = ps2float::add(lwc1(prop + 56), lwc1(prop + 516));
        let half_y = lwc1(prop + 524);
        swc1(sp + 12, ps2float::add(x, half_x));
        swc1(sp + 16, ps2float::add(y, half_y));
        swc1(sp + 20, ps2float::add(z, half_x));
        swc1(sp, ps2float::sub(x, half_x));
        swc1(sp + 4, ps2float::sub(y, half_y));
        swc1(sp + 8, ps2float::sub(z, half_x));
        let count = bgRoomBBIntersection(sp, prop + 144, 10);
        sw(prop + 184, count);
    }
}

/// Draw the background of a front end page: a zoom box round the page (sized by the page), darker unless it is the page the front end has open, and for the open page a flashing
/// bar behind the selected box, its highlight and an arrow beside it (@ 0x24c670).
pub fn pagebackgroundGfx_24c670(page: i32) {
    let selected = frontGetBox(page, lhu(page + 4));
    if lw(page + 12) > 0 {
        let size = 0x1fc39f0;
        frontGetPageSize(page, size);
        let front = lw(g::front);
        let bottom = lw(size + 12) + 2;
        sw(size + 12, bottom);
        let left;
        let right;
        if (lw(front + 12) & 2) == 0 {
            left = lw(0x1fc39f0);
            right = lw(size + 8);
        } else {
            right = lw(size + 8);
            let margins = g::pagemargins;
            left = lw(0x1fc39f0);
            sw(g::pagemargins, lw(size + 4) << 16);
            sw(margins + 12, right << 16);
            sw(margins + 4, bottom << 16);
            sw(margins + 8, left << 16);
        }
        let new_left = left - 36;
        let new_right = right + 10;
        sw(0x1fc39f0, new_left);
        sw(size + 8, new_right);
        let bottom_now = lw(size + 12);
        let top = lw(size + 4);
        let colour = if (((page - lw(front + 4)) >> 4) ^ (lw(front + 20) - 1)) != 0 { 0x3f3f3f7f } else { 0x7f7f7f7f };
        zoomboxDraw(new_left, top, new_right, bottom_now, 0x2880, colour);
    }
    let front = lw(g::front);
    if ((page - lw(front + 4)) >> 4) == lw(front + 20) - 1 {
        if selected != 0 {
            let window_top = lw(g::window + 20);
            let box_y = lhu(selected + 2);
            let left = lw(0x1fc39f0) + 2;
            let top = window_top + box_y + 1;
            let right = lw(0x1fc39f8) - 2;
            let bottom = window_top + box_y + lhu(selected + 6) - 1;
            dlFillRectangle(left, top, right, bottom, lbu(g::flashalpha) | 0x8000);
        }
        menuitemHighlight(page);
        if selected != 0 {
            let alpha = lbu(g::flashalpha);
            if lhu(page + 4) != 0x8000 {
                let y = (lw(g::window + 20) + lhu(selected + 2) + 2) as f32;
                let x = (lw(g::window) + lhu(selected) - 30) as f32;
                guiDrawImage(3, alpha | 0x7f7f7f00, x, y);
            }
        }
    }
}
