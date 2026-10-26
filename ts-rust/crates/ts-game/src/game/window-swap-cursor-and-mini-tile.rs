//! Swapping the window with the saved one, the map maker cursor, and turning the small tile of the map maker.

use crate::*;

/// Copy 96 bytes from `src` to `dst` by doublewords, four at a time (all four read before they are written).
fn copy_96(src: i32, dst: i32) {
    for k in 0..3 {
        let at = k * 32;
        let b = ld(src + at + 8);
        let c = ld(src + at + 16);
        let d = ld(src + at + 24);
        sd(dst + at, ld(src + at));
        sd(dst + at + 8, b);
        sd(dst + at + 16, c);
        sd(dst + at + 24, d);
    }
}

/// Swap the window (96 bytes) with the one saved at 0x1fb31c0 (@ 0x20c5d0).
pub fn windowSwitch() {
    let frame = Frame::new(96);
    let sp = frame.sp();
    copy_96(g::window, sp);
    copy_96(0x1fb31c0, g::window);
    copy_96(sp, 0x1fb31c0);
}

/// Draw the map maker cursor: a frame round the object `object` (not for objects with flag 1 or the two kinds of the routines at 0x2512b0 and 0x251600), then the image of the
/// cursor, moved by an amount that depends on its kind, in a flashing colour for some (@ 0x24fe68).
pub fn mmDrawCursor(object: i32) {
    let mut colour = lbu(g::flashalpha) | 0x7f7f7f00;
    if lw(lw(g::front) + 12) & 2 == 0 {
        return;
    }
    dlSetBlend(1);
    let mut front = lw(g::front);
    let mut cursor = 0;
    let mut have_cursor = false;
    if object != 0 {
        if lhu(object + 10) & 1 != 0 {
            cursor = lw(g::mmcursor);
            have_cursor = true;
        } else {
            let routine = lw(object + 12);
            if routine != 0x2512b0 {
                let alpha = lbu(g::flashalpha);
                if routine != 0x251600 {
                    let left = lhu(object);
                    let top = lhu(object + 2);
                    let right = left + lhu(object + 4);
                    let outer_left = left - 2;
                    let line = alpha << 24 | 128 | (alpha << 8 | alpha << 16);
                    let above = top - 1;
                    let outer_right = right + 2;
                    let bottom = top + lhu(object + 6);
                    dlDrawLine(outer_left, above, outer_right, above, line);
                    dlDrawLine(outer_left, bottom + 1, outer_right, bottom + 1, line);
                    dlFillRectangle(outer_left, top, left - 1, bottom, line);
                    dlFillRectangle(right, top, right + 1, bottom, line);
                }
            }
            front = lw(g::front);
        }
    }
    if !have_cursor {
        cursor = lw(g::mmcursor);
    }
    let mut y = lh(front + 38);
    let mut x = lh(front + 34);
    if (cursor as u32) < 17 {
        match cursor {
            0 | 2 => {
                x -= 8;
                y -= 6;
            }
            1 => {
                y -= 3;
                x -= 6;
                colour = lbu(g::flashalpha) | 0x7f3f0000;
            }
            3..=6 => {
                y -= 4;
                x -= 8;
            }
            7 => {
                x -= 5;
                y -= 3;
            }
            8 => {
                x -= 6;
                colour = lbu(g::flashalpha) | 0x7f7f00;
                y -= 3;
            }
            9 => {
                colour = lbu(g::flashalpha) | 0x7f7f00;
            }
            10..=15 => {}
            16 => {
                x -= 2;
            }
            _ => unreachable!(),
        }
    }
    let image = lw(g::mmcursor);
    let px = (lw(g::window) + x) as f32;
    let py = (lw(g::window + 20) + y) as f32;
    guiDrawImage(image, colour, px, py);
}

/// Turn the small tile of the map maker with the stick: the spin (axis 2, scaled by 4) either adds at once or, when flag 4 of the tile is set, builds a speed that slows down
/// below a limit; the tilt (axis 3) is held to -45 and 45, the spin to 0 and 360; two buttons move the tile away and nearer, held to 1 and 50 (@ 0x251068).
pub fn rotateMiniTile() {
    let axis = joyGetAxis(0, 2);
    let four = 4.0_f32;
    let turn = ps2float::mul(-axis, four);
    let mut size = turn;
    if !(0.0_f32 <= turn) {
        size = -turn;
    }
    let spin;
    if lw(g::zoomtileflags) & 4 == 0 {
        let rotation = lwc1(g::zoomtileroty);
        spin = ps2float::add(rotation, turn);
    } else {
        let floor = lwc1(0x3aca68);
        let speed = lwc1(0x3b007c);
        let mut now;
        if !(size < floor) {
            now = size;
            swc1(0x3b007c, now);
            if four < now {
                swc1(0x3b007c, four);
            }
            now = lwc1(0x3b007c);
            if now < 0.0_f32 {
                now = 0.0_f32;
                swc1(0x3b007c, 0.0_f32);
            }
            if !(turn < 0.0_f32) {
                swc1(0x3b0080, 1.0_f32);
            } else {
                swc1(0x3b0080, -1.0_f32);
            }
        } else {
            now = ps2float::sub(speed, lwc1(0x3aca6c));
            let below = now < lwc1(0x3aca70);
            swc1(0x3b007c, now);
            if below {
                now = floor;
                swc1(0x3b007c, now);
            }
        }
        spin = ps2float::add(lwc1(g::zoomtileroty), ps2float::mul(now, lwc1(0x3b0080)));
    }
    swc1(g::zoomtileroty, spin);
    let tilt_axis = joyGetAxis(0, 3);
    let limit = 45.0_f32;
    let mut tilt = ps2float::add(lwc1(g::zoomtiletilt), tilt_axis);
    swc1(g::zoomtiletilt, tilt);
    if limit < tilt {
        tilt = limit;
        swc1(g::zoomtiletilt, tilt);
    }
    let low = -45.0_f32;
    if tilt < low {
        swc1(g::zoomtiletilt, low);
    }
    let mut rotation = lwc1(g::zoomtileroty);
    let full = 360.0_f32;
    let mut held = rotation;
    if full <= rotation {
        loop {
            rotation = ps2float::sub(held, full);
            held = rotation;
            if !(full <= rotation) {
                break;
            }
        }
        swc1(g::zoomtileroty, rotation);
    }
    if rotation < 0.0_f32 {
        let mut wrapped;
        loop {
            wrapped = ps2float::add(rotation, 360.0_f32);
            rotation = wrapped;
            if !(wrapped < 0.0_f32) {
                break;
            }
        }
        swc1(g::zoomtileroty, wrapped);
    }
    let buttons = joyBut(0);
    let distance = lwc1(g::zoomtiledist);
    let mut distance_now;
    if buttons & 2 == 0 {
        let buttons = joyBut(0);
        distance_now = lwc1(g::zoomtiledist);
        if buttons & 8 != 0 {
            distance_now = ps2float::add(distance_now, lwc1(0x3aca78));
            swc1(g::zoomtiledist, distance_now);
        }
    } else {
        distance_now = ps2float::sub(distance, lwc1(0x3aca74));
        swc1(g::zoomtiledist, distance_now);
    }
    let one = 1.0_f32;
    if distance_now < one {
        distance_now = one;
        swc1(g::zoomtiledist, one);
    }
    let far = 50.0_f32;
    if far < distance_now {
        swc1(g::zoomtiledist, far);
    }
}
