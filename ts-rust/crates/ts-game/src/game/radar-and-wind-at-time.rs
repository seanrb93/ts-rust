//! The radar of the HUD and the wind of the weather at a time.

use crate::*;

/// The radar of the HUD: the distance (over 50), the angle (turned by the player's own) and the colour of every character other than the player that is alive (the team colour in
/// a team game, else the first team's), then, in the modes 1 and 5, of the bags that are shown (flashing), and in mode 4 of the escort; the circle of the radar is drawn as a
/// triangle strip of 17 points with its marker at the top, and every entry as a small bar at its angle and distance (a distance over 1 is clamped and the colour greyed). The
/// diameter of the radar is returned (@ 0x2b2258).
pub fn hudRadarGfx() -> i32 {
    let frame = Frame::new(640);
    let sp = frame.sp();
    let mut player = lw(g::currentplayer);
    let mut count = 0;
    let mut players = lw(g::lvnumplayers);
    let mut characters = lw(g::numactivechars);
    let facing = (lwc1(player + 276) * lwc1(0x3ad79c)).pdiv(1.8e+02_f32);
    let mut strip = sp + 16;
    if players + characters > 0 {
        loop {
            let offset = count * 0x1210;
            let character = count * 0x1210 + lw(g::chrinfo);
            let thing = lw(character + 3020);
            let mut set = false;
            if count != lw(player) && lw(character + 2716) != 0 {
                let back = lwc1(player + 148);
                let slot = count * 12;
                let dx = lwc1(thing + 48) - lwc1(player + 140);
                let dz = -(lwc1(thing + 56) - back);
                let entry = count * 12 + 0x1fea944;
                swc1(entry, ((dx * dx + dz * dz).abs().psqrt()).pdiv(5e+01_f32));
                let angle = atan2f(dz, dx);
                let team_game = lw(g::curgameparms + 80) & 1;
                swc1(slot + 0x1fea940, angle - facing);
                if team_game == 0 {
                    players = lw(g::lvnumplayers);
                    sw(slot + 0x1fea948, lw(g::teamdata + 4));
                    characters = lw(g::numactivechars);
                } else {
                    players = lw(g::lvnumplayers);
                    characters = lw(g::numactivechars);
                    sw(slot + 0x1fea948, lw(offset + lw(g::chrinfo) + 20));
                }
                set = true;
            }
            if !set {
                sw(count * 12 + 0x1fea948, 0);
            }
            count += 1;
            player = lw(g::currentplayer);
            if count >= players + characters {
                break;
            }
        }
    }
    let mode = lw(g::curgameparms + 72);
    if mode == 1 || mode == 5 {
        let mut bags = lw(g::numPickupBags);
        let mut i = 0;
        if bags > 0 {
            let mut offset = i << 3;
            loop {
                let bag = lw(offset + (g::bagspawn + 4));
                let player = lw(g::currentplayer);
                if (lw(bag + 16) & 1) != 0 {
                    let slot = count * 12;
                    let dx = lwc1(bag + 48) - lwc1(player + 140);
                    let dz = -(lwc1(bag + 56) - lwc1(player + 148));
                    let entry = count * 12 + 0x1fea944;
                    count += 1;
                    swc1(entry, ((dx * dx + dz * dz).abs().psqrt()).pdiv(5e+01_f32));
                    let angle = atan2f(dz, dx);
                    sw(slot + 0x1fea948, if (lw(g::tickframenum) & 16) == 0 { 0 } else { 0x7f7f7f80 });
                    swc1(slot + 0x1fea940, angle - facing);
                    bags = lw(g::numPickupBags);
                }
                i += 1;
                offset = i << 3;
                if i >= bags {
                    break;
                }
            }
        }
    }
    if lw(g::curgameparms + 72) == 4 {
        let escort = lw(g::escort);
        let player = lw(g::currentplayer);
        if escort != 0 {
            let slot = count * 12;
            let dx = lwc1(escort + 48) - lwc1(player + 140);
            let dz = -(lwc1(escort + 56) - lwc1(player + 148));
            let entry = count * 12 + 0x1fea944;
            count += 1;
            swc1(entry, ((dx * dx + dz * dz).abs().psqrt()).pdiv(5e+01_f32));
            let angle = atan2f(dz, dx);
            let colour = if (lw(g::tickframenum) & 16) == 0 { 0 } else { 0x7f7f7f80 };
            swc1(slot + 0x1fea940, angle - facing);
            sw(slot + 0x1fea948, colour);
        }
    }
    let mut size = 5e+01_f32;
    let top = lw(g::window + 20);
    if lw(g::window + 88) != 0 {
        size = 4e+01_f32;
    }
    let half_size = size * 0.5_f32;
    let degrees = lwc1(0x3ad7a0);
    let mut turn = 0.0_f32;
    let centre_y = (top + 4) as f32 + half_size;
    let centre_x = (lw(g::window + 4) - 8) as f32 - size;
    let factor = lwc1(0x3ad7a4);
    let mut point = 0;
    loop {
        let index = ((turn * degrees).pdiv(1.8e+02_f32) * factor + 0.5_f32) as i32;
        let sine = lwc1(((index & 2047) << 2) + g::g_SinTable);
        let cosine = lwc1((((index + 512) & 2047) << 2) + g::g_SinTable);
        if point != 16 {
            sw(strip, 0x64007f);
            sw(strip + 4, centre_x as i32);
            sw(strip + 8, centre_y as i32);
            sw(strip + 12, 0x640000);
            swc1(strip + 16, f32::from_bits((centre_x + cosine * size) as i32 as u32));
            swc1(strip + 20, f32::from_bits((centre_y - sine * half_size) as i32 as u32));
            strip += 24;
            point += 1;
        } else {
            let second = ld(sp + 24);
            let third = ld(sp + 32);
            sd(strip, ld(sp + 16));
            sd(strip + 8, second);
            sd(strip + 16, third);
            point += 1;
        }
        turn += 22.5_f32;
        if point >= 17 {
            break;
        }
    }
    dlDrawTriangleStrip(sp + 16, 34, 0);
    dlSelectTextureKick(lw(0x3b0194));
    sw(sp, 0x7f7f7f80);
    dlTextureRectangle(
        (centre_x - 2.0_f32) as i32,
        (centre_y - 1.0_f32) as i32,
        (centre_x + 2.0_f32) as i32,
        (centre_y + 1.0_f32) as i32,
        0,
        0,
        80,
        80,
    );
    if count > 0 {
        let mut i = 0;
        loop {
            let slot = i * 12;
            let mut colour = lw(i * 12 + 0x1fea948);
            if colour != 0 {
                let distance_at = slot + 0x1fea944;
                let mut distance = lwc1(distance_at);
                if 1.0_f32 <= distance {
                    swc1(distance_at, 1.0_f32);
                    colour = 0x5050507f;
                    distance = 1.0_f32;
                }
                let angle = (lwc1(slot + 0x1fea940) * lwc1(0x3ad7a8) + 0.5_f32) as i32;
                let y = centre_y + lwc1(((angle & 2047) << 2) + g::g_SinTable) * distance * half_size;
                let x = centre_x - lwc1((((angle + 512) & 2047) << 2) + g::g_SinTable) * distance * size;
                dlTextureRectangleFloat(0, 0, 80, 80, colour, x - 2.0_f32, y - 1.0_f32, x + 2.0_f32, y + 1.0_f32);
            }
            i += 1;
            if i >= count {
                break;
            }
        }
    }
    (half_size + half_size) as i32
}

/// The sine (or with `quarter` the cosine) of the wind phase for the time `time` in the table of sines: the time times the constant at 0x3a95c8, doubled, times `rate` and the
/// constant at 0x3a95d8, rounded.
fn wind_sine(time: f32, rate: i64, quarter: bool) -> f32 {
    let scaled = dpmul(fptodp(time), ld(0x3a95c8));
    let doubled = dpadd(scaled, scaled);
    let index = dptoli(dpadd(dpmul(dpmul(doubled, rate), ld(0x3a95d8)), 0x3fe0000000000000_i64));
    let at = if quarter { index + 512 } else { index };
    lwc1(((at & 2047) << 2) + g::g_SinTable)
}

/// Calculate the wind vector `wind` (three floats) of the weather at the time `time`: the sines of five slow phases are taken, the first two scaled by the constant at 0x3ad628 give
/// the x and z, and the other four (each only counting when above 0, else the vector is cleared) gust-scale the x and the z (@ 0x2a80c8).
pub fn weatherCalcWindAtTime(wind: i32, time: f32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let scale = lwc1(0x3ad628);
    let first = wind_sine(time, ld(0x3a95d0), false);
    sw(wind + 4, 0);
    swc1(wind, first * scale);
    swc1(wind + 8, wind_sine(time, ld(0x3a95d0), true) * scale);
    sw(sp + 4, 0);
    swc1(sp, wind_sine(time, ld(0x3a95e0), false));
    swc1(sp + 8, wind_sine(time, 0x3fd0000000000000_i64, true));
    sw(sp + 20, 0);
    swc1(sp + 16, wind_sine(time, ld(0x3a95e8), false));
    swc1(sp + 24, wind_sine(time, ld(0x3a95f0), true));
    let gust_a = lwc1(sp);
    let x = lwc1(wind);
    let x = if !(0.0_f32 < gust_a) { x * 0.0_f32 } else { x * gust_a };
    let gust_b = lwc1(sp + 16);
    swc1(wind, x);
    let x = if !(0.0_f32 < gust_b) { x * 0.0_f32 } else { x * gust_b };
    let gust_c = lwc1(sp + 8);
    swc1(wind, x);
    let z = lwc1(wind + 8);
    let z = if !(0.0_f32 < gust_c) { z * 0.0_f32 } else { z * gust_c };
    let gust_d = lwc1(sp + 24);
    swc1(wind + 8, z);
    let z = if !(0.0_f32 < gust_d) { z * 0.0_f32 } else { z * gust_d };
    swc1(wind + 8, z);
}
