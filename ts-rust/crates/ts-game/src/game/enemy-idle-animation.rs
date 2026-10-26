//! The idle animation of an enemy.

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    ps2float::mul(random_as_float(), 2.3283064e-10_f32)
}

/// Choose what an enemy does when it stands about (not with flag 4 at +2708 of its state): in the story it starts animation 3 at a random speed when it is not playing; else when
/// it is not playing animation 450 or 451 (or animation 3 has finished) it picks by a random number animation 3, 450 or 451 with a random speed, and when it plays one of them it
/// goes back to animation 3 when that has finished (@ 0x2cb648).
pub fn enemySetIdleAnimation(chr: i32) {
    if (lw(lw(chr + 352) + 2708) & 4) != 0 {
        return;
    }
    if lw(g::curgameparms + 72) == 7 {
        if curAnim(chr, 3) == 0 {
            let r = random_unit();
            setAnim(chr, 3, 0.0_f32, ps2float::add(ps2float::mul(r, lwc1(0x3ada70)), 1.0_f32));
        }
        return;
    }
    let mut pick = false;
    let playing_special = curAnim(chr, 450) != 0 || curAnim(chr, 451) != 0;
    if !playing_special {
        pick = true;
    } else {
        if curAnim(chr, 3) != 0 && curAnimComplete(lw(chr + 32)) != 0 {
            pick = true;
        }
        if !pick {
            if curAnimComplete(lw(chr + 32)) == 0 {
                return;
            }
            let r = random_unit();
            setAnim(chr, 3, 0.0_f32, ps2float::add(ps2float::mul(r, lwc1(0x3ada6c)), 1.0_f32));
            return;
        }
    }
    let first = random_unit();
    let (anim, scale, base);
    if !(first < lwc1(0x3ada4c)) {
        if !(first < lwc1(0x3ada58)) {
            let r = random_unit();
            scale = lwc1(0x3ada64);
            base = lwc1(0x3ada68);
            anim = 451;
            setAnim(chr, anim, 0.0_f32, ps2float::add(ps2float::mul(r, scale), base));
        } else {
            let r = random_unit();
            scale = lwc1(0x3ada5c);
            base = lwc1(0x3ada60);
            anim = 450;
            setAnim(chr, anim, 0.0_f32, ps2float::add(ps2float::mul(r, scale), base));
        }
    } else {
        let r = random_unit();
        scale = lwc1(0x3ada50);
        base = lwc1(0x3ada54);
        anim = 3;
        setAnim(chr, anim, 0.0_f32, ps2float::add(ps2float::mul(r, scale), base));
    }
    let _ = pick;
}
