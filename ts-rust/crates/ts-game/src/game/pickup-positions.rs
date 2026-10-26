//! The marks where pickups lie (drawn as small squares).

use crate::*;

/// Add a pickup mark at the point `pos` (three floats): a square of four corners at the height of the game's constant 0x3acd6c, half a side of the constant
/// 0x3acd68 (corners at 0x1fce190, 12 floats each), texture coordinates (at 0x1fcf450) and four colours (at 0x1fd0710, 0x177f7f7f).  100 at most;
/// returns the number of the mark, or 99 when full (@ 0x272ac0).
pub fn pickupposAdd(pos: i32) -> i32 {
    let mut n = lw(g::num_pickuppos);
    if n < 100 {
        let half = lwc1(0x3acd68);
        let height = lwc1(0x3acd6c);
        let corners = 0x1fce190 + n * 48;
        let texture = 0x1fcf450 + n * 48;
        let (x, y, z) = (lwc1(pos), lwc1(pos + 4), lwc1(pos + 8));
        let signs = [(-1.0_f32, -1.0_f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)];
        for (i, (sx, sz)) in signs.iter().enumerate() {
            let at = corners + 12 * i as i32;
            swc1(at, if *sx < 0.0 { ps2float::sub(x, half) } else { ps2float::add(x, half) });
            swc1(at + 4, ps2float::add(y, height));
            swc1(at + 8, if *sz < 0.0 { ps2float::sub(z, half) } else { ps2float::add(z, half) });
        }
        for (i, v) in [0.0_f32, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0].iter().enumerate() {
            swc1(texture + 4 * i as i32, *v);
        }
        let colours = 0x1fd0710 + (n << 4);
        for i in 0..4 {
            sw(colours + 4 * i, 0x177f7f7f);
        }
        n += 1;
        sw(g::num_pickuppos, n);
    }
    n - 1
}
