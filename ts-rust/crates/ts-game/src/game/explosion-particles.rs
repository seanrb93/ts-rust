//! Starting the particles of an explosion.

use super::particle_spawners::random_as_float;
use crate::*;

/// Start an explosion (a group of kind 12) of the kind `kind` (an entry of the explosion table, 8 bytes each) at the point `at`: `tag` and `kind` are kept at +932 and +928, the
/// vectors `first` and `second` at +936 and +948, `a6` at +960, and `owner_prop` at +964 when it has flag 16; the game's number of particles (kind 69) get a random time and a
/// place round the point (the first third spread wider, by the game's constants), and the rooms of a box round the point are found (@ 0x2978c0).
pub fn particleNewExplosion(owner: i32, kind: i32, at: i32, tag: i32, first: i32, second: i32, a6: i32, owner_prop: i32) {
    let group = getgroup(12);
    let life = lwc1(0x3ad4ac);
    sw(group + 12, 0);
    sw(group + 20, owner);
    let x = lwc1(at);
    sw(group + 8, 4);
    swc1(group + 28, x);
    sw(group + 4, -1);
    swc1(group + 32, lwc1(at + 4));
    let z = lwc1(at + 8);
    sw(group + 52, 0);
    swc1(group + 36, z);
    swc1(group + 16, life);
    let count = particleGetNum(69);
    sw(group + 932, tag);
    sw(group + 928, kind);
    sw(group + 24, count);
    let f = lwc1(first);
    sw(group + 44, 0);
    swc1(group + 936, f);
    swc1(group + 940, lwc1(first + 4));
    swc1(group + 944, lwc1(first + 8));
    swc1(group + 948, lwc1(second));
    swc1(group + 952, lwc1(second + 4));
    let g_ = lwc1(second + 8);
    sw(group + 960, a6);
    swc1(group + 956, g_);
    sw(group + 964, 0);
    if owner_prop != 0 && lw(owner_prop + 16) & 16 != 0 {
        sw(group + 964, owner_prop);
    }
    let total = lw(group + 24);
    let size = (lwc1((kind << 3) + (g::explosiondefs + 4))).pdiv(500.0_f32);
    let mut i = 0;
    let mut more = i < total;
    if total.wrapping_div(3) > 0 {
        loop {
            let r = random_as_float();
            swc1(group + 1796 + (i << 2), ps2float::mul(ps2float::mul(r, 2.3283064e-10_f32), 0.5_f32));
            let r = random_as_float();
            swc1(group + 968 + i * 12, ps2float::add(lwc1(at), ps2float::mul(ps2float::mul(ps2float::sub(ps2float::mul(r, 2.3283064e-10_f32), 0.5_f32), lwc1(0x3ad4b0)), size)));
            let r = random_as_float();
            swc1(group + 972 + i * 12, ps2float::add(lwc1(at + 4), ps2float::mul(ps2float::mul(ps2float::sub(ps2float::mul(r, 2.3283064e-10_f32), 0.5_f32), lwc1(0x3ad4b4)), size)));
            let r = random_as_float();
            i += 1;
            swc1(group + 976 + i * 12 - 12, ps2float::add(lwc1(at + 8), ps2float::mul(ps2float::mul(ps2float::sub(ps2float::mul(r, 2.3283064e-10_f32), 0.5_f32), lwc1(0x3ad4b8)), size)));
            let total_now = lw(group + 24);
            more = i < total_now;
            if i >= total_now.wrapping_div(3) {
                break;
            }
        }
    }
    if more {
        loop {
            let r = random_as_float();
            swc1(group + 1796 + (i << 2), ps2float::mul(r, 2.3283064e-10_f32));
            let r = random_as_float();
            swc1(group + 968 + i * 12, ps2float::add(lwc1(at), ps2float::mul(ps2float::sub(ps2float::mul(r, 2.3283064e-10_f32), 0.5_f32), size)));
            let r = random_as_float();
            swc1(group + 972 + i * 12, ps2float::add(lwc1(at + 4), ps2float::mul(ps2float::sub(ps2float::mul(r, 2.3283064e-10_f32), 0.5_f32), size)));
            let r = random_as_float();
            let place = i * 12 + (group + 976);
            i += 1;
            swc1(place, ps2float::add(lwc1(at + 8), ps2float::mul(ps2float::sub(ps2float::mul(r, 2.3283064e-10_f32), 0.5_f32), size)));
            if i >= lw(group + 24) {
                break;
            }
        }
    }
    let x = lwc1(at);
    let y = lwc1(at + 4);
    let z = lwc1(at + 8);
    bgCalcRoomArray(group + 72, ps2float::sub(ps2float::add(x, -0.5_f32), 3.0_f32), ps2float::add(ps2float::add(x, 0.5_f32), 3.0_f32), ps2float::sub(ps2float::add(y, -0.5_f32), 3.0_f32), ps2float::add(ps2float::add(y, 0.5_f32), 3.0_f32), ps2float::sub(ps2float::add(z, -0.5_f32), 3.0_f32), ps2float::add(ps2float::add(z, 0.5_f32), 3.0_f32));
    enemyAddToExplosionList();
}
