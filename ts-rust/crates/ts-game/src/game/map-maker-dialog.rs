//! Making the items of the map maker's tool dialog.

use crate::*;

/// The string of the current language at `offset` of its record.
fn language_text(offset: i32) -> i32 {
    lw(lw((lw(g::cur_language) << 2) + g::languages) + offset)
}

/// Make the boxes (records of 24 bytes from `items`: the text at +16 and the image at +20) of the map maker's dialog `dialog`: three mode buttons in a row, a button each side of the
/// bottom to go back and forward, and for the mode kept at `editmode` (0: the tiles from the scroll position, 16 at most, of the list at 0x3a23b0, each box as wide as its tile
/// and a Next Page box; 1: nothing more; 2: the light dialog with 16 colour boxes, a Next Page box and a list of the 11 light types); returns the number of items made (@ 0x251f10).
pub fn makedialog(dialog: i32, items: i32) -> i32 {
    let frame = Frame::new(272);
    let sp = frame.sp();
    let mut cur = items;
    let mut descs = g::editmode_desc;
    let mut mode_number = 0;
    let mut x = 50;
    loop {
        makebox(cur, (x << 16) >> 16, 27, 173, 15, mode_number + 10 & 0xffff, 0, 0x24fb00);
        let mode = lw(g::editmode);
        sw(cur + 16, lw((lh(descs) << 2) + lw((lw(g::cur_language) << 2) + g::languages)));
        let mut image = g::nt_editmodeon;
        if mode != mode_number {
            image = g::nt_editmodeoff;
        }
        sw(cur + 20, image);
        mode_number += 1;
        cur += 24;
        descs += 2;
        x += 183;
        if mode_number >= 3 {
            break;
        }
    }
    makebox(cur, ((lhu(g::window + 8) - 80) << 16) >> 16, 182, 0, 12, 0, 1, 0x24fb00);
    let text = if (lhu(dialog + 4) - 64) != lw(g::curlightcol) { language_text(3500) } else { language_text(3496) };
    sw(cur + 16, text);
    sw(cur + 20, g::nt_center);
    cur += 24;
    makebox(cur, ((lhu(g::window + 8) + 80) << 16) >> 16, 182, 0, 12, 0, 1, 0x24fb00);
    sw(cur + 20, g::nt_center);
    let mode = lw(g::editmode);
    sw(cur + 16, language_text(3564));
    cur += 24;
    if mode == 2 {
        let mut id = 0;
        let mut y = 50;
        loop {
            let row_y = y;
            y += 24;
            let mut column_x = 50;
            loop {
                makebox(cur, (column_x << 16) >> 16, (row_y << 16) >> 16, 40, 20, id + 64 & 0xffff, 0, 0x251e70);
                column_x += 48;
                cur += 24;
                id += 1;
                if column_x >= 242 {
                    break;
                }
            }
            if y >= 146 {
                break;
            }
        }
        makebox(cur, 50, 150, 192, 12, 2, 0, 0x24fb00);
        sw(cur + 20, g::nt_nextpage);
        sw(cur + 16, language_text(3508));
        cur += 24;
        makebox(cur, 306, 50, 280, 110, 0, 1, 0x24fb00);
        sw(cur + 16, 0);
        sw(cur + 20, g::nt_listbg);
        cur += 24;
        let mut types = g::lighttypes;
        let mut y = 50;
        for k in 0..11 {
            makebox(cur, 306, (y << 16) >> 16, 280, 10, k + 100 & 0xffff, 0, 0x24fb00);
            sw(cur + 16, lw((lh(types) << 2) + lw((lw(g::cur_language) << 2) + g::languages)));
            let mut image = g::nt_listoff;
            if lbu(g::lighttype) == lbu(types + 2) {
                image = g::nt_liston;
            }
            sw(cur + 20, image);
            cur += 24;
            types += 4;
            y += 10;
        }
    } else if mode == 0 {
        let mut from = 0x3a23b0;
        let mut to = sp;
        let first = lw(0x3b005c);
        let tiles = lw(g::numtiles);
        for _ in 0..2 {
            let b = ld(from + 8);
            let c = ld(from + 16);
            let d = ld(from + 24);
            sd(to, ld(from));
            sd(to + 8, b);
            sd(to + 16, c);
            sd(to + 24, d);
            from += 32;
            to += 32;
        }
        let b = ld(from + 8);
        let c = lw(from + 16);
        sd(to, ld(from));
        sd(to + 8, b);
        sw(to + 16, c);
        let mut x = 50;
        let mut y = 50;
        let mut index = first;
        let mut tiles_now = tiles;
        if index < tiles_now {
            loop {
                let tile = lw(sp + (index << 2));
                let width = lw(lw((tile << 2) + g::tiles));
                let box_width = if 3 >= width { 48 } else { width << 4 };
                let box_height = box_width >> 1;
                let id = if tile >= tiles_now { 0 } else { tile + 200 & 0xffff };
                let real = tile < tiles_now;
                makebox(
                    cur,
                    (x << 16) >> 16,
                    (y << 16) >> 16,
                    (box_width << 16) >> 16,
                    (box_height << 16) >> 16,
                    id,
                    real as i32 ^ 1,
                    if real { 0x251be0 } else { 0 },
                );
                cur += 24;
                if index.wrapping_rem(4) != 3 {
                    x += box_width + 10;
                } else {
                    x = 50;
                    y += box_height + 5;
                }
                index += 1;
                tiles_now = lw(g::numtiles);
                let start = lw(0x3b005c);
                if index >= tiles_now {
                    break;
                }
                if (index - start) >= 16 {
                    break;
                }
            }
        }
        makebox(cur, 50, 166, 222, 12, 5, 0, 0x24fb00);
        sw(cur + 20, g::nt_nextpage);
        sw(cur + 16, language_text(3504));
        cur += 24;
    }
    ((cur - items).wrapping_mul(-0x55555555)) >> 3
}
