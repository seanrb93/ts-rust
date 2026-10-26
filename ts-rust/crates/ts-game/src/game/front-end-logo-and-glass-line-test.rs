//! The two halves of the logo of the front end, and a line against the panes of glass of a room.

use crate::*;

/// Draw the 256 columns of one half of the logo: the texture `texture`, a column of 1 by 100 pixels at (`x` + column, 40) for each, with the alpha of the column from the table of
/// `frontfxalpha` at `alpha_offset`.
fn logo_columns(texture_at: i32, x: i32, alpha_offset: i32, sp: i32) {
    dlSelectTextureKick(lw(texture_at));
    let mut column = 0;
    let mut texture = lw(texture_at);
    loop {
        let next = column + 1;
        let u0 = column << 4;
        let height = lhu(texture * 40 + lw(lw(g::thistexbuf) + 4) + 2);
        sw(sp, 0 << 24 | lbu(column + alpha_offset));
        dlTextureRectangle(column + x, 40, column + x + 1, 140, u0, 0, next << 4, height << 4);
        column = next;
        texture = lw(texture_at);
        if column >= 256 {
            break;
        }
    }
}

/// Draw the logo of the front end's goodies page: two halves, each 256 columns of a texture with the alpha of its own column (@ 0x22c068).
pub fn frontfxGoodiesGfx() {
    let frame = Frame::new(128);
    let sp = frame.sp();
    logo_columns(g::logotop1num, 64, g::frontfxalpha, sp);
    logo_columns(g::logotop2num, 320, g::frontfxalpha + 256, sp);
}

/// Test the line from `start` along `direction` against the panes of glass of the room `room` (a null-ended list of pointers to panes, the pane's own corners at +16, +56, +96
/// and +136, a float at +0 that is negative while broken): for each pane that is hit, up to 5, a record of 44 bytes at 0x1fc4748 is added (the point, the direction, the normal,
/// the room and the number of the pane); the count is kept at `count`; a pane with flag 256 sets `glasshitreverse` (@ 0x259eb8).
pub fn bgGlassLineTestRoom(room: i32, start: i32, direction: i32, count: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let list = lw(room * 44 + lw(g::levelrooms) + 16);
    if list != 0 {
        let dx = lwc1(direction);
        let mut index = 0;
        let dy = lwc1(direction + 4);
        let dz = lwc1(direction + 8);
        if lw(list) != 0 {
            let mut at = list;
            loop {
                let pane = lw(at);
                if 0.0_f32 <= lwc1(pane) {
                    let limit2 = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz));
                    let hit = bgHittestDistGlass(start, direction, pane + 16, pane + 56, pane + 96, pane + 136, sp, sp + 16, limit2);
                    if hit == 0 {
                        // no hit: the glass flag is not looked at
                    } else {
                        let n = lw(count);
                        if n < 5 {
                            let base = 0x1fc4748;
                            let hx = lwc1(sp);
                            let hy = lwc1(sp + 4);
                            let hz = lwc1(sp + 8);
                            let nx = lwc1(sp + 16);
                            sw(base + n * 44 + 36, room);
                            let ny = lwc1(sp + 20);
                            let nz = lwc1(sp + 24);
                            sw(lw(count) * 44 + base + 40, index);
                            let slot = lw(count);
                            sw(count, slot + 1);
                            let record = base + slot * 44;
                            swc1(slot * 44 + base, hx);
                            swc1(record + 4, hy);
                            swc1(record + 8, hz);
                            swc1(record + 12, lwc1(direction));
                            swc1(record + 16, lwc1(direction + 4));
                            swc1(record + 20, lwc1(direction + 8));
                            swc1(record + 24, nx);
                            swc1(record + 28, ny);
                            swc1(record + 32, nz);
                        }
                        if (lw(lw(at) + 192) & 256) != 0 {
                            sw(g::glasshitreverse, 1);
                        }
                    }
                }
                index += 1;
                at = (index << 2) + list;
                if lw((index << 2) + list) == 0 {
                    break;
                }
            }
        }
    }
}
