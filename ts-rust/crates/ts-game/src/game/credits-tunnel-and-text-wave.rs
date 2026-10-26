//! The tunnel of the credits and the wave of their text.

use crate::*;

/// Work out where the credits' camera is in the tunnel at time `time`, into `out`: the tunnel is 54 points of 48 bytes (`tunnel`, x, y and z first, integers at +12 to +44);
/// the time (540 to a round, 10 to a point) gives a point and an ease between it and the next (a cosine over the fraction); the place (x, y, z at +0 to +8 of `out`) is the
/// point moved the ease of the way to the next, and the integers at `out` +12 to +52 (three of 12 bytes each, from the integers of the points at +12) are half of the point's and
/// the ease times the difference to the next point's (@ 0x2aefe0).
pub fn getCreditTunnelSection(out: i32, time: i32) {
    let frame = Frame::new(48);
    let sp = frame.sp();
    let two_pi = ld(0x3a98a0);
    let step = time.wrapping_rem(540);
    let fraction = (step as f32).pdiv(1e+01_f32) - step.wrapping_div(10) as f32;
    let angle = dptofp(dpadd(dpmul(fptodp(fraction), two_pi), two_pi));
    let cosine = cosf(angle);
    let tunnel = g::tunnel;
    let y_base = tunnel + 4;
    let z_base = tunnel + 8;
    let point = step.wrapping_div(10);
    let y_at = point * 48 + y_base;
    let x = lwc1(point * 48 + tunnel);
    let ease = cosine * 0.5_f32 + 0.5_f32;
    let rows = tunnel + 20;
    let z_at = point * 48 + z_base;
    let next = (point + 1).wrapping_rem(54);
    let dx = lwc1(next * 48 + tunnel) - x;
    let z = lwc1(z_at);
    let dy = lwc1(next * 48 + y_base) - lwc1(y_at);
    let next_z = lwc1(next * 48 + z_base);
    let mut third = out + 20;
    swc1(sp, dx);
    let dz = next_z - z;
    swc1(sp + 4, dy);
    let mut mine = point * 48 + rows;
    let mut second = out + 16;
    swc1(sp + 8, dz);
    let along = ease * dz;
    let mut first = out + 12;
    let mut theirs = next * 48 + rows;
    swc1(out, x + ease * dx);
    swc1(out + 4, lwc1(y_at) + ease * dy);
    swc1(out + 8, lwc1(z_at) + along);
    swc1(out + 8, lwc1(z_at) + along);
    loop {
        let a = lw(mine - 8);
        let da = (lw(theirs - 8) - a) as f32;
        let c = lw(theirs);
        let db = (lw(theirs - 4) - lw(mine - 4)) as f32;
        theirs += 12;
        let dc = (c - lw(mine)) as f32;
        swc1(sp + 16, da);
        swc1(sp + 20, db);
        swc1(sp + 24, dc);
        sw(first, ((a as f32 + ease * da) * 0.5_f32) as i32);
        first += 12;
        sw(second, ((lw(mine - 4) as f32 + ease * db) * 0.5_f32) as i32);
        second += 12;
        sw(third, ((lw(mine) as f32 + ease * dc) * 0.5_f32) as i32);
        third += 12;
        mine += 12;
        if third >= out + 56 {
            break;
        }
    }
}

/// The offset in x of the text of a credits page, by its place `place` (a distance in the page's width): 0 while the page is not there or the text is still, a rise by the
/// sine waves of `sinwavetimestart`/`sinwaveintensity` while it comes in (700 before it starts), and a fall to -700 as it goes out (@ 0x2b00e8).
pub fn creditsCalcTextWave(place: i32) -> i32 {
    let wait = lwc1(g::creditswaittime);
    let four = 4.0_f32;
    let page = lwc1(g::creditspagetime);
    let come_in_end = wait + four;
    let ratio = (place as f32).pdiv(lw(g::window + 32) as f32);
    let go_out_start = come_in_end + lwc1(g::creditstexttime);
    if page < wait {
        return 0;
    }
    if (go_out_start + four) <= page {
        return 0;
    }
    if come_in_end <= page && page < go_out_start {
        return 0;
    }
    if !(page < come_in_end) {
        let t = page - go_out_start - (ratio + ratio);
        if !(0.0_f32 <= t) {
            return 0;
        }
        if t < 1.0_f32 {
            let angle = dptofp(dpmul(dpmul(fptodp(t), ld(0x3a98b8)), 0x3fe0000000000000_i64));
            return (sinf(angle) * -1.2e+03_f32) as i32;
        }
        return -700;
    }
    let t = page - wait - (ratio + ratio);
    if !(0.0_f32 <= t) {
        return 700;
    }
    let mut index = 0;
    loop {
        let table = g::sinwavetimestart;
        let low = lwc1((index << 2) + table);
        let at = index << 2;
        index += 1;
        let u = ((t - low)).pdiv((lwc1((index << 2) + table) - low));
        if 0.0_f32 <= u && u < 1.0_f32 {
            let angle = dptofp(dpmul(fptodp(u), ld(0x3a98b0)));
            return (sinf(angle) * lwc1(at + g::sinwaveintensity)) as i32;
        }
    }
}
