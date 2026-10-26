//! The turning of a character towards what it faces.

use crate::*;

/// Turn `current` towards the angle that is `diff` away, by at most `limit`.
fn turn_step(current: f32, diff: f32, limit: f32) -> f32 {
    let mut size = fptodp(diff);
    if dpcmp(size, 0_i64) < 0 {
        size = dpsub(0_i64, size);
    }
    if dpcmp(size, fptodp(limit)) <= 0 {
        ps2float::add(current, diff)
    } else if !(0.0_f32 < diff) {
        ps2float::sub(current, limit)
    } else {
        ps2float::add(current, limit)
    }
}

/// The size of the angle `diff` as a double.
fn angle_size(diff: f32) -> i64 {
    let mut size = fptodp(diff);
    if dpcmp(size, 0_i64) < 0 {
        size = dpsub(0_i64, size);
    }
    size
}

/// Take the angle at `at` back into 0 to 360.
fn wrap_angle(at: i32) {
    let mut angle = lwc1(at);
    if 3.6e+02_f32 < angle {
        let wrapped = ps2float::sub(angle, 3.6e+02_f32);
        swc1(at, wrapped);
        angle = wrapped;
    }
    if angle < 0.0_f32 {
        swc1(at, ps2float::add(angle, 3.6e+02_f32));
    }
}

/// The angle in degrees of the heading from (`x`, `z`) given as the pair of doubles (the first as the x of the pair, the second the z), before any mirror: the angle of `atan2`
/// scaled by the constants at the two addresses.
fn heading(x: i64, z: i64, scale_a: i32, scale_b: i32) -> f32 {
    let angle = atan2(x, z);
    let scaled = dpmul(angle, ld(scale_a));
    dptofp(dpdiv(scaled, ld(scale_b)))
}

/// Turn the character `a0` for a tick: the facing at +76 and +80 are taken into 0 to 360, the angles to the target (+2780) and to the point it walks to (+2784) are worked out
/// (the target's own angle, mirrored for the modes with bit 0x200000 or 128), and when the character's animation lets it (not for the animations 566 to 606, 471, 472, 509, 510
/// and 524 to 529) the body at +84 and the facing at +80 turn towards them by at most the character's turn rate times the frames, the pad faced by the character at the defend pad
/// or pickup drop modes setting the angle wanted; then the pitch at +2968 (aim up or down at the target) is turned by a quarter of the rate, kept above a floor unless the
/// aiming animations 477 and 478 are on, and the facing is kept in +2972 (@ 0x2bc5d0).
pub fn rotateChr(a0: i32) {
    let frame = Frame::new(128);
    let sp = frame.sp();
    let mut wanted = lwc1(0x3ad818);
    let mut facing = lwc1(a0 + 76);
    let s2 = lw(a0 + 352);
    let mut flags = lw(s2 + 2708);
    if facing < 0.0_f32 {
        loop {
            let next = ps2float::add(facing, 3.6e+02_f32);
            facing = next;
            swc1(a0 + 76, next);
            if !(facing < 0.0_f32) {
                break;
            }
        }
    }
    if (flags & 0x1000) != 0 {
        return;
    }
    let target = lw(s2 + 2788);
    if target == 0 {
        swc1(s2 + 2780, lwc1(0x3ad81c));
    } else {
        let dx = ps2float::sub(lwc1(target + 48), lwc1(a0 + 48));
        let dz = ps2float::sub(lwc1(target + 56), lwc1(a0 + 56));
        let (first, second);
        if (lw(s2 + 2716) & 0x600000) == 0 {
            first = fptodp(dx);
            second = dz;
        } else {
            first = fptodp(-dx);
            second = -dz;
        }
        let second = fptodp(second);
        wanted = heading(first, second, 0x3a9cc0, 0x3a9cc8);
        let facing_now = lwc1(a0 + 80);
        if wanted < 0.0_f32 {
            wanted = ps2float::add(wanted, 3.6e+02_f32);
        }
        swc1(s2 + 2780, anglediff(facing_now, wanted));
        flags = lw(s2 + 2708);
    }
    let dx = ps2float::sub(lwc1(s2 + 2796), lwc1(a0 + 48));
    let dz = ps2float::sub(lwc1(s2 + 2804), lwc1(a0 + 56));
    let (first, second);
    if (flags & 128) == 0 {
        first = fptodp(dx);
        second = dz;
    } else {
        first = fptodp(-dx);
        second = -dz;
    }
    let second = fptodp(second);
    let mut walk = heading(first, second, 0x3a9cd0, 0x3a9cd8);
    if walk < 0.0_f32 {
        walk = ps2float::add(walk, 3.6e+02_f32);
    }
    let facing_now = lwc1(a0 + 76);
    swc1(s2 + 2784, anglediff(facing_now, walk));
    let animation = lw(lw(a0 + 32) + 96);
    if ((animation - 566) as u32) < 41 || animation == 472 || animation == 471 || animation == 510 || animation == 509 || ((animation - 524) as u32) < 6 {
        return;
    }
    let mut mode = lw(s2 + 2716);
    if mode == 0x20000 {
        let facing_now = lwc1(a0 + 80);
        let pad_angle = lwc1(lw(s2 + 680) * 28 + lw(g::levelpads) + 24);
        let diff = anglediff(facing_now, pad_angle);
        let pads = lw(g::levelpads);
        let offset = lw(s2 + 680) * 28;
        swc1(s2 + 2784, diff);
        swc1(s2 + 2780, diff);
        mode = lw(s2 + 2716);
        wanted = lwc1(offset + pads + 24);
    }
    let facing_now;
    if mode != 4 {
        facing_now = lwc1(a0 + 76);
    } else {
        let drops = lw(g::numPickupDrops);
        if (lw(s2 + 2708) & 8) != 0 {
            let mut found = -1;
            let pads;
            'search: {
                if drops <= 0 {
                    pads = lw(g::levelpads);
                } else {
                    let team = lw(s2 + 24);
                    if team != lw(lw(lw(g::dropspawn + 4) + 352) + 16) {
                        pads = lw(g::levelpads);
                        let mut i = 1;
                        loop {
                            let at = i << 3;
                            if i >= drops {
                                break 'search;
                            }
                            if team == lw(lw(lw(at + (g::dropspawn + 4)) + 352) + 16) {
                                break;
                            }
                            i += 1;
                        }
                        found = i;
                    } else {
                        found = 0;
                        pads = lw(g::levelpads);
                    }
                }
            }
            let drop_at = (found << 3) + (g::dropspawn + 4);
            let facing_now = lwc1(a0 + 80);
            let pad_angle = lwc1(lw(lw(lw(drop_at) + 352) + 36) * 28 + pads + 24);
            let diff = anglediff(facing_now, pad_angle);
            let levelpads = lw(g::levelpads);
            let record = lw(lw(drop_at) + 352);
            swc1(s2 + 2784, diff);
            let pad = lw(record + 36);
            swc1(s2 + 2780, diff);
            wanted = lwc1(pad * 28 + levelpads + 24);
        }
        facing_now = lwc1(a0 + 76);
    }
    // the body at +84
    let body_diff = anglediff(facing_now, wanted);
    let flags = lw(s2 + 2708);
    let mut turn = true;
    if (flags & 1411) == 0 {
        if (flags & 520) == 0 {
            turn = false;
        } else if dpcmp(angle_size(body_diff), 0x4047000000000000_i64) < 0 {
            turn = false;
        }
    }
    if turn {
        let aim_diff = lwc1(s2 + 2784);
        let limit = ps2float::mul(lwc1(s2 + 2880), lwc1(g::updateframesf));
        let current = lwc1(a0 + 84);
        swc1(a0 + 84, turn_step(current, aim_diff, limit));
        wrap_angle(a0 + 84);
    }
    // the facing at +80
    let target = lw(s2 + 2788);
    let mut face_wanted = lwc1(a0 + 84);
    if target != 0 && lw(s2 + 2772) != 0 && (lw(s2 + 2716) & 0x61c000) == 0 {
        let diff = anglediff(lwc1(a0 + 76), wanted);
        face_wanted = wanted;
        if dpcmp(angle_size(diff), 0x4047000000000000_i64) > 0 {
            let current = lwc1(a0 + 76);
            if !(0.0_f32 < diff) {
                face_wanted = ps2float::sub(current, 46.0_f32);
                if face_wanted < 0.0_f32 {
                    face_wanted = ps2float::add(face_wanted, 3.6e+02_f32);
                }
            } else {
                face_wanted = ps2float::add(current, 46.0_f32);
                if 3.6e+02_f32 < face_wanted {
                    face_wanted = ps2float::sub(face_wanted, 3.6e+02_f32);
                }
            }
        }
    }
    let diff = anglediff(lwc1(a0 + 80), face_wanted);
    let limit = ps2float::mul(lwc1(s2 + 2880), lwc1(g::updateframesf));
    let current = lwc1(a0 + 80);
    swc1(a0 + 80, turn_step(current, diff, limit));
    wrap_angle(a0 + 80);
    // the pitch
    let target = lw(s2 + 2788);
    let pitch_wanted;
    if target != 0 && lw(s2 + 2772) != 0 && lw(s2 + 2716) != 0x20000 {
        let kind = lw(target + 8);
        let x;
        let y;
        if kind == 8 {
            let state = lw(s2 + 376);
            let near = state == 8 || state == 11 || state == 13 || state == 25 || ((state - 14) as u32) < 5;
            if near && lwc1(target + 52) < ps2float::sub(lwc1(a0 + 52), lwc1(0x3ad820)) {
                swc1(sp, ps2float::sub(lwc1(target + 48), lwc1(s2 + 3008)));
                swc1(sp + 4, ps2float::sub(lwc1(target + 52), lwc1(s2 + 3012)));
                swc1(sp + 8, ps2float::sub(lwc1(target + 56), lwc1(s2 + 3016)));
            } else {
                let from_x = lwc1(s2 + 3008);
                let record = lw(target + 352);
                swc1(sp, ps2float::sub(lwc1(target + 48), from_x));
                let eye = lwc1(s2 + 3012);
                let point = lwc1(record + 3012);
                swc1(sp + 4, ps2float::sub(point, eye));
                swc1(sp + 8, ps2float::sub(lwc1(target + 56), lwc1(s2 + 3016)));
            }
            x = lwc1(sp);
            y = lwc1(sp + 4);
        } else if kind >= 9 && kind != 64 {
            if kind == 256 {
                swc1(sp, ps2float::sub(lwc1(target + 48), lwc1(s2 + 3008)));
                let eye = lwc1(s2 + 3012);
                let point = ps2float::add(lwc1(target + 52), 0.5_f32);
                swc1(sp + 4, ps2float::sub(point, eye));
                swc1(sp + 8, ps2float::sub(lwc1(target + 56), lwc1(s2 + 3016)));
            }
            x = lwc1(sp);
            y = lwc1(sp + 4);
        } else if kind < 9 && kind != 1 {
            x = lwc1(sp);
            y = lwc1(sp + 4);
        } else {
            if ((lw(target + 4) - 201) as u32) >= 2 {
                let half = lwc1(target + 508);
                swc1(sp, ps2float::sub(ps2float::add(lwc1(target + 48), half), lwc1(s2 + 3008)));
                swc1(sp + 4, ps2float::sub(ps2float::add(lwc1(target + 52), lwc1(target + 512)), lwc1(s2 + 3012)));
            } else {
                swc1(sp, ps2float::sub(ps2float::add(lwc1(target + 48), lwc1(target + 508)), lwc1(s2 + 3008)));
                swc1(sp + 4, ps2float::sub(lwc1(target + 52), lwc1(s2 + 3012)));
            }
            let eye = lwc1(s2 + 3016);
            let point = ps2float::add(lwc1(target + 56), lwc1(target + 516));
            swc1(sp + 8, ps2float::sub(point, eye));
            x = lwc1(sp);
            y = lwc1(sp + 4);
        }
        let z = lwc1(sp + 8);
        let length = ps2float::add(x * x + y * y, ps2float::mul(z, z)).abs().psqrt();
        let inverse = (1.0_f32).pdiv(length);
        let z_unit = ps2float::mul(z, inverse);
        let x_unit = ps2float::mul(x, inverse);
        swc1(sp + 8, z_unit);
        swc1(sp, x_unit);
        swc1(sp + 4, ps2float::mul(y, inverse));
        let rise = lwc1(sp + 4);
        let run = ps2float::add(ps2float::mul(x_unit, x_unit), ps2float::mul(z_unit, z_unit)).abs().psqrt();
        pitch_wanted = ps2float::mul(atan2f(rise, run), 1.8e+02_f32).pdiv(lwc1(0x3ad824));
    } else {
        pitch_wanted = 0.0_f32;
    }
    let pitch_now = lwc1(s2 + 2968);
    let diff = anglediff(pitch_now, pitch_wanted);
    let limit = ps2float::mul(ps2float::mul(lwc1(s2 + 2880), 0.25_f32), lwc1(g::updateframesf));
    let current = lwc1(s2 + 2968);
    swc1(s2 + 2968, turn_step(current, diff, limit));
    let floor = lwc1(0x3ad828);
    if lwc1(s2 + 2968) < floor {
        if curAnim(a0, 477) != 0 {
            swc1(s2 + 2968, floor);
        } else if curAnim(a0, 478) != 0 {
            swc1(s2 + 2968, floor);
        }
    }
    swc1(s2 + 2972, lwc1(a0 + 80));
}
