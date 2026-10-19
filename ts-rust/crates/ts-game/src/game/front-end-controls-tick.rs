//! Turning the stick and the direction buttons of a pad into the front end's menu moves.

use crate::*;

/// Read pad `pad` for the front end: the four directions (the stick past half, or the button; up and down also move the repeat counter at +16 of the front end record),
/// each with a counter of ticks that sets a flag (64, 128, 256, 512 at +12) every time it counts out; while the stick is nearly still and no direction button is down
/// the repeat counter is reset; the four flags 4, 8, 16, 32 are set when the counter of the direction passes 12 (and the counter then starts again) and cleared otherwise (@ 0x22c4b0).
pub fn frontControlsTick(pad: i32) {
    let (stick_y, stick_x);
    if lw(lw(g::front) + 12) & 2 == 0 {
        stick_x = joyGetAxis(pad, 0);
        stick_y = joyGetAxis(pad, 1);
    } else {
        stick_y = 0.0_f32;
        stick_x = stick_y;
    }
    let buttons = joyBut(pad);
    joyNewBut(pad);
    let front = lw(g::front);
    sw(front + 12, lw(front + 12) & -961);
    let up = 0.5_f32 < stick_y;
    let ticks = (pad << 2) + g::btnticks_105;
    if buttons & 0x1000 == 0 && !up {
        sb(ticks, 12);
    } else {
        sw(front + 12, lw(front + 12) | 64);
        sb(ticks, lbu(ticks) + 1);
        if lw(front + 16) == 0 {
            sw(front + 16, 0);
        }
        sw(front + 16, lw(front + 16) - 1);
    }
    let down = stick_y < -0.5_f32;
    if buttons & 0x4000 == 0 && !down {
        sb(ticks + 1, 12);
    } else {
        sw(front + 12, lw(front + 12) | 128);
        sb(ticks + 1, lbu(ticks + 1) + 1);
        let counter;
        if lw(front + 16) != 0 {
            counter = lw(front + 16);
        } else {
            sw(front + 16, 31);
            counter = lw(front + 16);
        }
        sw(front + 16, counter + 1);
    }
    let left = stick_x < -0.5_f32;
    if buttons & 0x8000 == 0 && !left {
        sb(ticks + 2, 12);
    } else {
        sw(front + 12, lw(front + 12) | 256);
        sb(ticks + 2, lbu(ticks + 2) + 1);
        if lw(front + 16) == 0 {
            sw(front + 16, 0);
        }
        sw(front + 16, lw(front + 16) - 1);
    }
    let right = 0.5_f32 < stick_x;
    if buttons & 0x2000 == 0 && !right {
        sb(ticks + 3, 12);
    } else {
        sw(front + 12, lw(front + 12) | 512);
        sb(ticks + 3, lbu(ticks + 3) + 1);
        let counter;
        if lw(front + 16) != 0 {
            counter = lw(front + 16);
        } else {
            sw(front + 16, 31);
            counter = lw(front + 16);
        }
        sw(front + 16, counter + 1);
    }
    if buttons & 0xa000 == 0 {
        let mut x_still = false;
        if !(0.0_f32 <= stick_x) {
            let flipped = -stick_x;
            if flipped <= 0.25_f32 {
                x_still = true;
            }
        } else if stick_x <= 0.25_f32 {
            x_still = true;
        }
        if x_still {
            if !(0.0_f32 <= stick_y) {
                let flipped = -stick_y;
                if flipped <= 0.25_f32 {
                    sw(front + 16, 0);
                }
            } else if stick_y <= 0.25_f32 {
                sw(front + 16, 0);
            }
        }
    }
    let mut at = ticks;
    let mut direction = 0;
    loop {
        let bit = 4 << (direction & 31);
        direction += 1;
        let more = direction < 4;
        let flags;
        if lb(at) < 13 {
            flags = lw(front + 12) & !(0 | bit);
        } else {
            sb(at, 0);
            flags = lw(front + 12) | bit;
        }
        sw(front + 12, flags);
        at += 1;
        if !more {
            break;
        }
    }
}
