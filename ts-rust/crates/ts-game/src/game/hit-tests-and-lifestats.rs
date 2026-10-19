//! A line against a box, bounds of a chunk, the nearest target, and the lifetime statistics.

use crate::*;

/// Whether the line from `a2` to `a3` (points of three floats) hits the box centred at `a0` with half sizes `a1`: the slab test on each axis, in the game's
/// own order of tests (@ 0x209640).
pub fn hittestLineBox(center: i32, half: i32, from: i32, to: i32) -> i32 {
    // x
    let mut ex = lwc1(half);
    let c = lwc1(center);
    let mut p_to = lwc1(to) - c;
    let mut p_from = lwc1(from) - c;
    if ex < 0.0 {
        ex = -ex;
        p_from = -p_from;
        p_to = -p_to;
    }
    if p_from < 0.0 && p_to < 0.0 {
        return 0;
    }
    let (mut x_min, mut x_max) = (p_from, p_to);
    if p_to < p_from {
        x_min = p_to;
        x_max = p_from;
    }
    // y
    let mut ey = lwc1(half + 4);
    let c = lwc1(center + 4);
    let mut q_to = lwc1(to + 4) - c;
    let mut q_from = lwc1(from + 4) - c;
    if ey < 0.0 {
        ey = -ey;
        q_from = -q_from;
        q_to = -q_to;
    }
    if q_from < 0.0 && q_to < 0.0 {
        return 0;
    }
    let (mut y_min, mut y_max) = (q_from, q_to);
    if q_to < q_from {
        y_min = q_to;
        y_max = q_from;
    }
    let x_low_y = x_min * ey;
    let y_low_x = y_min * ex;
    let x_high_y = x_max * ey;
    let y_high_x = y_max * ex;
    let low_along;
    let low_scale;
    if !(y_low_x < x_low_y) {
        low_along = y_min;
        if x_high_y < y_low_x {
            return 0;
        }
        low_scale = ey;
    } else {
        low_along = x_min;
        if y_high_x < x_low_y {
            return 0;
        }
        low_scale = ex;
    }
    let high_along;
    let mut high_scale = ex;
    if !(x_high_y < y_high_x) {
        high_along = y_max;
        high_scale = ey;
    } else {
        high_along = x_max;
    }
    // z
    let mut ez = lwc1(half + 8);
    let c = lwc1(center + 8);
    let mut r_to = lwc1(to + 8) - c;
    let mut r_from = lwc1(from + 8) - c;
    if ez < 0.0 {
        ez = -ez;
        r_from = -r_from;
        r_to = -r_to;
    }
    if r_from < 0.0 && r_to < 0.0 {
        return 0;
    }
    let (mut z_min, mut z_max) = (r_from, r_to);
    if r_to < r_from {
        z_min = r_to;
        z_max = r_from;
    }
    let low = low_along * ez;
    if !(low < z_min * low_scale) {
        let high = z_max * low_scale;
        if !(high < low) { 1 } else { 0 }
    } else if (high_along * ez) < z_min * high_scale {
        0
    } else {
        1
    }
}

/// The bounds of the vertices of a chunk of a model (strips at `model`'s table, 16 bytes each, the vertices of 12 bytes at its list; the chunk record gives the first
/// strip at +4 and the first vertex at +12): the smallest and largest of each axis go to `out` (@ 0x25e4d8).
pub fn obCalcChunkBounds(model: i32, chunk: i32, out: i32) {
    let mut min = [lwc1(0x3acb58); 3];
    let mut max = [lwc1(0x3acb5c); 3];
    let mut strip = lw(model) + (lw(chunk + 4) << 4);
    let mut vertex = lw(model + 4) + lw(chunk + 12) * 12;
    loop {
        let header = lw(strip);
        let count = (header & 0x7fff) as i32;
        strip += 16;
        for _ in 0..count {
            for axis in 0..3 {
                let v = lwc1(vertex + 4 * axis as i32);
                if v < min[axis] {
                    min[axis] = v;
                }
                if max[axis] < v {
                    max[axis] = v;
                }
            }
            vertex += 12;
        }
        if header & 0x8000 != 0 {
            break;
        }
    }
    for axis in 0..3 {
        swc1(out + 4 * axis as i32, min[axis]);
        swc1(out + 12 + 4 * axis as i32, max[axis]);
    }
}

/// Choose a target for a computer character in the story mode: the nearest player's character, unless `kind` is 1, 3 or 4, the current target is not the first player's
/// or there are fewer than two players (then the current target `target` stays); a player-controlled one found first is taken at once (@ 0x2c5e68).
pub fn suggestTarget(chr: i32, target: i32, kind: i32) -> i32 {
    if lw(g::curgameparms + 72) == 7 && ((kind - 3) as u32) >= 2 && kind != 1 {
        let players = lw(g::players);
        let n = lw(g::lvnumplayers);
        if target != lw(players + 384) || n < 2 {
            return target;
        }
        let mut best = -1;
        let mut best_d2 = lwc1(0x3afd7c);
        let chr_info = lw(g::chrinfo);
        for i in 0..n {
            if lw(i * 1820 + players + 296) != 4 {
                let rec = i * 0x1210 + chr_info;
                if lw(rec + 484) != 0 {
                    return lw(rec + 3020);
                }
                let t = lw(rec + 3020);
                let dx = lwc1(t + 48) - lwc1(chr + 48);
                let dy = lwc1(t + 52) - lwc1(chr + 52);
                let dz = lwc1(t + 56) - lwc1(chr + 56);
                let d2 = dx * dx + dy * dy + dz * dz;
                if d2 < best_d2 {
                    best_d2 = d2;
                    best = i;
                }
            }
        }
        if best == -1 {
            let (z, x, y) = (lwc1(chr + 56), lwc1(chr + 48), lwc1(chr + 52));
            for i in 0..n {
                let t = lw(i * 0x1210 + chr_info + 3020);
                let dy = lwc1(t + 52) - y;
                let dx = lwc1(t + 48) - x;
                let dz = lwc1(t + 56) - z;
                let d2 = dx * dx + dy * dy + dz * dz;
                if d2 < best_d2 {
                    best_d2 = d2;
                    best = i;
                }
            }
        }
        return lw(best * 0x1210 + chr_info + 3020);
    }
    target
}

/// Add up the lifetime statistics of three records (`first`, `second`, `third`) into `out`: counts are summed, the three records at +16, +20 and +24 take the largest, floats are
/// summed, the table of 34 words at +64 and 34 floats at +200 are summed, and the bytes at +348 (48 of them) are those of the second (@ 0x224878).
pub fn StatsGetOverallLifestats(out: i32, first: i32, second: i32, third: i32) {
    let sum3 = |at: i32| lw(first + at) + lw(second + at) + lw(third + at);
    let largest = |at: i32| {
        let (a, b, c) = (lw(first + at), lw(second + at), lw(third + at));
        let high = if c < b { b } else { c };
        if high < a { a } else { high }
    };
    sw(out, sum3(0));
    sw(out + 4, sum3(4));
    sw(out + 8, sum3(8));
    sw(out + 12, sum3(12));
    sw(out + 16, largest(16));
    sw(out + 20, largest(20));
    sw(out + 24, largest(24));
    sw(out + 28, sum3(28));
    sw(out + 32, sum3(32));
    let fsum = |at: i32| lwc1(first + at) + lwc1(second + at) + lwc1(third + at);
    let f36 = fsum(36);
    sw(out + 52, sum3(52));
    swc1(out + 36, f36);
    let f40 = fsum(40);
    sw(out + 56, sum3(56));
    swc1(out + 40, f40);
    let f44 = fsum(44);
    sw(out + 60, sum3(60));
    swc1(out + 44, f44);
    swc1(out + 48, fsum(48));
    for i in 0..34 {
        sw(out + 64 + 4 * i, sum3(64 + 4 * i));
        swc1(out + 200 + 4 * i, fsum(200 + 4 * i));
    }
    let f344 = lwc1(first + 344) + lwc1(second + 344);
    let third_344 = lwc1(third + 344);
    sw(out + 340, -1);
    sw(out + 336, -1);
    swc1(out + 344, f344 + third_344);
    for i in 0..48 {
        sb(out + 348 + i, lbu(second + 348 + i));
    }
}
