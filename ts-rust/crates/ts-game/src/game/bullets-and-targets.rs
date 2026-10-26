//! Finding the nearest bullet, and the next character an enemy can target.

use crate::*;

/// The nearest of the first 20 bullets (nodes of 276 bytes of the bullet pool; the types 2 to 8, 10, 11, 17 and 18, by the type at +0) to the
/// point `pos`, and its distance written to `out`; 0 when none is nearer than the limit at 0x3afa60 (@ 0x28d100).
pub fn bulletGetClosest(pos: i32, out: i32) -> i32 {
    let nodes = lw(0x3b0138);
    let mut best = 0;
    let mut best_d2 = lwc1(0x3afa60);
    for i in 0..20 {
        let node = i * 276 + nodes;
        let t = lw(node) - 2;
        if (t as u32) < 17 {
            match t {
                0..=6 | 8 | 9 | 15 | 16 => {
                    let dx = ps2float::sub(lwc1(node + 24), lwc1(pos));
                    let dy = ps2float::sub(lwc1(node + 28), lwc1(pos + 4));
                    let dz = ps2float::sub(lwc1(node + 32), lwc1(pos + 8));
                    let d2 = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz));
                    if d2 < best_d2 {
                        best_d2 = d2;
                        best = node;
                    }
                }
                _ => {}
            }
        }
    }
    if best != 0 {
        swc1(out, best_d2.abs().psqrt());
    }
    best
}

/// The next character after an enemy's target (going round, over the characters in play) that is not on its team (or is hidden by flag 16 not being
/// set): the word at +3020 of its record; 0 if there is none (@ 0x2c0ae0).
pub fn enemyCycleThroughTargets(enemy: i32) -> i32 {
    let data = lw(enemy + 352);
    let target = lw(data + 2788);
    let count = lw(g::numactivechars) + lw(g::lvnumplayers);
    let last = count - 1;
    let (mut index, stop);
    if target == 0 {
        index = 0;
        stop = last;
    } else {
        let current = lw(lw(target + 352));
        if current != last {
            stop = current;
            index = current + 1;
        } else {
            index = 0;
            stop = last;
        }
    }
    let info = lw(g::chrinfo);
    let team = lw(data + 24);
    let mut chr = index * 0x1210 + info;
    loop {
        if lw(chr + 24) != team && lw(chr + 2716) & 16 == 0 {
            return lw(chr + 3020);
        }
        if index == last && stop != index {
            chr = info;
            index = 0;
            continue;
        }
        chr += 0x1210;
        if index == stop {
            return 0;
        }
        index += 1;
    }
}
