//! The light of the explosions nearest to a point.

use crate::*;

/// How strong the light of an explosion group `group` is at its age: the sum over its 69 particle times (from +1796) of how far each is from the group's own time (+12),
/// doubled, that fall between 0 and 1 (1 up to a half, the rest falling away twice as fast).
fn explosion_glow(group: i32) -> f32 {
    let time = lwc1(group + 12);
    let mut sum = 0.0_f32;
    let mut at = group + 1796;
    let mut left = 68;
    loop {
        let age = time - lwc1(at);
        let age = age + age;
        if !(0.0_f32 < age) {
            left -= 1;
        } else if !(age <= 1.0_f32) {
            left -= 1;
        } else {
            if !(0.5_f32 < age) {
                sum += 1.0_f32;
            } else {
                let rest = 1.0_f32 - age;
                sum += rest + rest;
            }
            left -= 1;
        }
        at += 4;
        if left < 0 {
            break;
        }
    }
    sum
}

/// Set a colour (three floats at `colour`) from the explosion kind `kind` (the table at `explosiondefs`, 8 bytes each: three colour bytes) scaled by `strength` and the three
/// game constants at `scales`.
fn explosion_colour(colour: i32, kind: i32, strength: f32, scales: i32) {
    let word = lw((kind << 3) + g::explosiondefs);
    swc1(colour, strength * lwc1(scales) * (word & 255) as f32);
    let word = lw((kind << 3) + g::explosiondefs);
    swc1(colour + 4, strength * lwc1(scales + 4) * ((word as u32 >> 8) & 255) as f32);
    let third = lhu((kind << 3) + (g::explosiondefs + 2));
    let value = strength * lwc1(scales + 8) * (third & 255) as f32;
    swc1(colour + 8, value);
}

/// Find the two explosions nearest to the point `point` (within the distance whose square is at `pgroups + 4`) and give the light of the stronger (its strength is its glow over
/// its distance): its position goes to `position`, its distance to `distance` and its colour (floats, at most 1) to `colour`. Returns 1, or 0 if there is none (@ 0x2a1e98).
pub fn particleGetClosestExplosion(point: i32, position: i32, distance: i32, colour: i32) -> i32 {
    let mut nearest_d2 = lwc1(g::pgroups + 4);
    let mut second = 0;
    let mut nearest = 0;
    let mut second_d2 = nearest_d2;
    let mut group = lw(0x1fe99f0);
    if group != 0 {
        let pz = lwc1(point + 8);
        let px = lwc1(point);
        let py = lwc1(point + 4);
        loop {
            let dx = px - lwc1(group + 28);
            let dy = py - lwc1(group + 32);
            let dz = pz - lwc1(group + 36);
            let d2 = dx * dx + dy * dy + dz * dz;
            if !(d2 < nearest_d2) {
                if d2 < second_d2 {
                    second_d2 = d2;
                    second = group;
                }
            } else {
                if !(nearest_d2 < second_d2) {
                    nearest_d2 = d2;
                } else {
                    second_d2 = nearest_d2;
                    second = nearest;
                    nearest_d2 = d2;
                }
                nearest = group;
            }
            group = lw(group + 68);
            if group == 0 {
                break;
            }
        }
    }
    let limit = lwc1(g::pgroups + 4);
    if !(nearest_d2 < limit) && !(second_d2 < limit) {
        return 0;
    }
    let mut glow_a = 0.0_f32;
    let mut dist_b = limit;
    let mut glow_b = 0.0_f32;
    let mut dist_a = limit;
    let mut falloff_a = limit;
    let mut falloff_b = limit;
    // `a` is the nearest, `b` the second
    if nearest_d2 < limit {
        let d = nearest_d2.abs().sqrt();
        glow_a = explosion_glow(nearest) * lwc1(0x3ad59c);
        dist_a = d;
        let half = d * 0.5_f32;
        falloff_a = 1.0_f32;
        if !(half <= 2.0_f32) {
            falloff_a = falloff_a / (half - falloff_a);
        }
    }
    if second_d2 < lwc1(g::pgroups + 4) {
        let d = second_d2.abs().sqrt();
        glow_b = explosion_glow(second) * lwc1(0x3ad5a0);
        dist_b = d;
        let half = d * 0.5_f32;
        falloff_b = 1.0_f32;
        if !(half <= 2.0_f32) {
            falloff_b = falloff_b / (half - falloff_b);
        }
    }
    let limit = lwc1(g::pgroups + 4);
    let from_second;
    if !(second_d2 < limit) {
        from_second = false;
    } else {
        from_second = (glow_a * falloff_a) < glow_b * falloff_b;
    }
    if from_second {
        swc1(position, lwc1(second + 28));
        swc1(position + 4, lwc1(second + 32));
        swc1(position + 8, lwc1(second + 36));
        swc1(distance, dist_b);
        explosion_colour(colour, lw(second + 928), glow_b, 0x3ad5a4);
    } else {
        swc1(position, lwc1(nearest + 28));
        swc1(position + 4, lwc1(nearest + 32));
        swc1(position + 8, lwc1(nearest + 36));
        swc1(distance, dist_a);
        explosion_colour(colour, lw(nearest + 928), glow_a, 0x3ad5b0);
    }
    if 1.0_f32 < lwc1(colour) {
        swc1(colour, 1.0_f32);
    }
    if 1.0_f32 < lwc1(colour + 4) {
        swc1(colour + 4, 1.0_f32);
    }
    if 1.0_f32 < lwc1(colour + 8) {
        swc1(colour + 8, 1.0_f32);
    }
    1
}
