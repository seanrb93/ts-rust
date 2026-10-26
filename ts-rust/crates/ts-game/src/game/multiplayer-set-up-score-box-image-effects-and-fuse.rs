//! The multiplayer game set-up, the HUD box of the team score, the image effects of a player and the sparks of a fuse.

use crate::*;

/// Make the front end's multiplayer game set-up: one player, the state record (64 bytes at 0x1fc11f8) cleared, the first arcade level listed as 14 found, the ten list records of
/// the set-up page given their start (the first six a row, the rest in pairs), the 30 bytes of team choices at 0x1fc2b28 set to 255, the three sign-on choices to the sign-on page's
/// first value, and the game options made (@ 0x240f38).
pub fn frontmpgMake() {
    sw(0x3aff20, 1);
    memset(0x1fc11f8, 0, 64);
    sw(0x1fc1224, 0);
    sw(0x1fc11f8, 0);
    let mut index = 0;
    sw(0x1fc1218, 0);
    if lw(g::arcadelevels) != 14 {
        index += 1;
        while index < 18 {
            if lw((index << 2) + g::arcadelevels) == 14 {
                break;
            }
            index += 1;
        }
    }
    let level = lw((index << 2) + g::arcadelevels);
    sw(0x1fc121c, index);
    sw(0x1fc1220, level);
    let mut at = 0x1fc2a94;
    let mut i = 0;
    loop {
        let first_row = (i < 6) as i32;
        let past = i - 6;
        if first_row == 0 {
            sw(at - 8, ((past + ((past as u32) >> 31) as i32) >> 1) + 2);
        } else {
            sw(at - 8, i.wrapping_div(3));
        }
        i += 1;
        sw(at, first_row);
        sw(at - 4, 1);
        at += 16;
        if i >= 10 {
            break;
        }
    }
    memset(0x1fc2b28, 255, 30);
    let value = lhu(g::signon_page + 8);
    let mut slot = 0x3aff88;
    let mut left = 3;
    loop {
        sh(slot, value);
        left -= 1;
        slot += 2;
        if left < 0 {
            break;
        }
    }
    gameoptionsMake();
}

/// Draw the score box of the team of `team` at height `y` and `across` more to the right: for the bag tag (mode 1) the time (`gameFormatBagTagTime`) and the language's text
/// at +3240, for the other modes 2 to 7 the number (red when below 0) and the text at +3244; the box's width is the score width and the colours are the team's with the
/// share of the width in the low byte (@ 0x2b15b8).
pub fn hudScoreGfx(team: i32, y: i32, across: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let x = lw(g::window) + (32 >> (lw(g::window + 88) & 31)) + across;
    let width = hudScoreWidth();
    let total = width + 4;
    if total == 0 {
        trap("break");
    }
    let mode = lw(g::curgameparms + 72);
    let share = ((across + width + 4) * 100).wrapping_div(total) & 255;
    let light = share | 0x7f7f7f00;
    if mode != 1 {
        if mode > 0 && mode < 8 {
            let score = teamGetScore(lw(team + 24));
            let title = lw(lw((lw(g::cur_language) << 2) + g::languages) + 3244);
            let text = vafmt(&[0x3afba8, score as i64]);
            let value_colour = if -1 < score { light } else { 0x7f000080 };
            let colour = lw(team + 20) & -256 | share;
            sw(sp, 1);
            drawValueBox(title, text, x, y, width, 22, colour, value_colour);
        }
    } else {
        let key = lw(team + 24);
        let title = lw(lw((lw(g::cur_language) << 2) + g::languages) + 3240);
        let score = teamGetScore(key);
        let text = gameFormatBagTagTime(score);
        sw(sp, 1);
        drawValueBox(title, text, x, y, width, 22, share | 0x5a5a5a00, light);
    }
}

/// Do the image effects of the current player (defocus, invert, motion blur, the fish-eye pulsing with the game's time, and the view of the z buffer), each when its flag is set
/// in the player's table (@ 0x296678).
pub fn ipGfx() {
    let mut player = lw(g::currentplayer);
    let mut number = lw(player);
    if lw((number << 2) + g::ipdefocus) != 0 {
        ipDefocus();
        player = lw(g::currentplayer);
        number = lw(player);
    }
    if lw((number << 2) + g::ipinvert) != 0 {
        ipInvertScreen();
        player = lw(g::currentplayer);
        number = lw(player);
    }
    let blur = (number << 2) + g::ipblur;
    if lw(blur) != 0 {
        ipMotionBlur(lbu(blur));
        player = lw(g::currentplayer);
        number = lw(player);
    }
    let _ = player;
    if lw((number << 2) + g::ipfisheye) != 0 {
        let to_radians = lwc1(0x3ad484);
        let half_turn = 1.8e+02_f32;
        let strength = sinf(ps2float::mul(gameframe_x(), to_radians).pdiv(half_turn));
        let slow = sinf(ps2float::mul(ps2float::mul(gameframe_x(), 0.5_f32), to_radians).pdiv(half_turn));
        let mut ratio = fptodp(slow);
        if dpcmp(ratio, 0_i64) < 0 {
            ratio = dpsub(0_i64, ratio);
        }
        ipFishEye(strength, dptofp(ratio));
        number = lw(lw(g::currentplayer));
    }
    if lw((number << 2) + g::ipviewzb) == 0 {
        return;
    }
    ipViewZB();
}

/// Make the sparks of the fuse of the grenade or mine `fuse` (its model at +12): the point on the model's bone between its two end points (by `amount` from 0 to 1) is moved to
/// the world by the model's matrix and to the view by the player's (at +1772), and on two ticks of three a spark (`particleNewGunSparks`, seen by the current player only) is made
/// there (@ 0x292118).
pub fn updateFuse(fuse: i32, amount: f32) {
    let frame = Frame::new(144);
    let sp = frame.sp();
    let to_radians = lwc1(0x3ad3ac);
    let model = lw(fuse + 12);
    let skeleton = lw(lw(model + 32));
    let z = lwc1(model + 56);
    let roll = ps2float::mul(lwc1(model + 72), to_radians).pdiv(1.8e+02_f32);
    let pitch = ps2float::mul(lwc1(model + 76), to_radians).pdiv(1.8e+02_f32);
    let y = lwc1(model + 52);
    let yaw = ps2float::mul(lwc1(model + 88), to_radians).pdiv(1.8e+02_f32);
    let x = lwc1(model + 48);
    let bones = skeleton - lw(skeleton) * 80;
    matrixTransRotZXY(sp, x, y, z, roll, pitch, yaw);
    let start = lw(bones + 72);
    let x0 = lwc1(start);
    swc1(sp + 64, x0);
    let y0 = lwc1(start + 4);
    swc1(sp + 68, y0);
    let z0 = lwc1(start + 8);
    let end = start + 12;
    swc1(sp + 72, z0);
    let x1 = lwc1(end);
    swc1(sp + 80, x1);
    let dx = ps2float::sub(x1, x0);
    let y1 = lwc1(end + 4);
    swc1(sp + 84, y1);
    let dy = ps2float::sub(y1, y0);
    let z1 = lwc1(end + 8);
    swc1(sp + 96, dx);
    let dz = ps2float::sub(z1, z0);
    swc1(sp + 88, z1);
    swc1(sp + 100, dy);
    swc1(sp + 112, ps2float::add(x0, ps2float::mul(dx, amount)));
    swc1(sp + 104, dz);
    swc1(sp + 116, ps2float::add(y0, ps2float::mul(dy, amount)));
    swc1(sp + 120, ps2float::add(z0, ps2float::mul(dz, amount)));
    matrixVecMulAligned(&[sp as i64, (sp + 112) as i64]);
    matrixVecMulAligned(&[lw(lw(g::currentplayer) + 1772) as i64, (sp + 112) as i64]);
    if lw(g::gameframe).wrapping_rem(3) != 0 {
        memset(sp + 128, 0, 12);
        let sparks = particleNewGunSparks(0, sp + 112, sp + 128);
        particleSetViewmask(sparks, 1 << (lw(lw(g::currentplayer)) & 31));
    }
}
