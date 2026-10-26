//! The choice of a zombie's animation.

use super::particle_spawners::random_as_float;
use crate::*;

/// Start the plain walk (anim 518, or 519 when the zombie carries the weapon flag at +440) when the speed at +2876 is no more than the animation's own speed for this frame.
fn start_plain_walk(chr: i32, record: i32, blend: f32) {
    if lwc1(record + 2876) <= ps2float::mul(ps2float::mul(ps2float::mul(lwc1(lw(chr + 32) + 140), blend), 6e+01_f32), lwc1(record + 2868)) {
        if lw(record + 440) == 0 {
            if curAnim(chr, 518) == 0 {
                setAnim(chr, 518, blend, 2.0_f32);
            }
        } else if curAnim(chr, 519) == 0 {
            setAnim(chr, 519, blend, 2.0_f32);
        }
    }
}

/// Choose the animation of the zombie `a0` (@ 0x2be338): it stands up from the 526 to 529 animations by finishing and starting the idle (518); a rising animation (577, 579)
/// moves into the get-up animations 524 and 525 when it is done and the zombie is not on the ground (the height from the record at +0x1198); when still (the walking flag 8 of
/// +2708 off) it picks among the idle animations by chance (513 to 515, or 516 and 517 with the weapon flag), and when walking it turns (520 and 522 to the left, 521 and 523
/// to the right, by the angle between the facing at +80 and the wanted +76 beyond 45 degrees) or goes straight (518, 519). Then the speed of the animation is set from the root
/// movement of the animation and the walking speed at +2876.
pub fn setZombieAnimation(a0: i32) {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let s3 = lw(a0 + 352);
    if curAnim(a0, 526) == 0 && curAnim(a0, 527) == 0 && curAnim(a0, 528) == 0 && curAnim(a0, 529) == 0 {
        'rise: {
            if curAnim(a0, 577) != 0 {
                if curAnimComplete(lw(a0 + 32)) != 0 && (lw(a0 + 16) & 0x10000000) == 0 {
                    setAnim(a0, 524, 0.0_f32, 1.0_f32);
                    break 'rise;
                }
            }
            if curAnim(a0, 579) != 0 {
                if curAnimComplete(lw(a0 + 32)) != 0 && (lw(a0 + 16) & 0x10000000) == 0 {
                    setAnim(a0, 525, 0.0_f32, 1.0_f32);
                    break 'rise;
                }
            }
            if curAnim(a0, 577) != 0 {
                return;
            }
            if curAnim(a0, 579) != 0 {
                return;
            }
            if curAnim(a0, 524) != 0 || curAnim(a0, 525) != 0 {
                if curAnimComplete(lw(a0 + 32)) == 0 {
                    return;
                }
            }
            let flags = lw(s3 + 2708);
            'chosen: {
                if (flags & 8) == 0 {
                    if lw(s3 + 440) == 0 {
                        if curAnim(a0, 513) == 0 && curAnim(a0, 514) == 0 && curAnim(a0, 515) == 0 {
                            let roll = ps2float::mul(random_as_float(), 2.3283064e-10_f32);
                            if !(roll < lwc1(0x3ad868)) {
                                if roll < lwc1(0x3ad86c) {
                                    setAnim(a0, 514, 0.25_f32, 2.0_f32);
                                } else {
                                    setAnim(a0, 515, 0.25_f32, 2.0_f32);
                                }
                            } else {
                                setAnim(a0, 513, 0.25_f32, 2.0_f32);
                            }
                        } else {
                            if curAnimComplete(lw(a0 + 32)) == 0 {
                                break 'chosen;
                            }
                            let roll = ps2float::mul(random_as_float(), 2.3283064e-10_f32);
                            if !(roll < lwc1(0x3ad870)) {
                                if !(roll < lwc1(0x3ad874)) {
                                    setAnim(a0, 515, 0.0_f32, 2.0_f32);
                                } else {
                                    setAnim(a0, 514, 0.0_f32, 2.0_f32);
                                }
                            } else {
                                setAnim(a0, 513, 0.0_f32, 2.0_f32);
                            }
                        }
                    } else if curAnim(a0, 516) == 0 && curAnim(a0, 517) == 0 {
                        let roll = random_as_float();
                        if ps2float::mul(roll, 2.3283064e-10_f32) < 0.5_f32 {
                            setAnim(a0, 516, 0.25_f32, 2.0_f32);
                        } else {
                            setAnim(a0, 517, 0.25_f32, 2.0_f32);
                        }
                    } else {
                        if curAnimComplete(lw(a0 + 32)) == 0 {
                            break 'chosen;
                        }
                        let roll = random_as_float();
                        if !(ps2float::mul(roll, 2.3283064e-10_f32) < 0.5_f32) {
                            setAnim(a0, 517, 0.0_f32, 2.0_f32);
                        } else {
                            setAnim(a0, 516, 0.0_f32, 2.0_f32);
                        }
                    }
                } else {
                    let blend = 0.25_f32;
                    let turn = anglediff(lwc1(a0 + 80), lwc1(a0 + 76));
                    'walk: {
                        if !(45.0_f32 < turn) {
                            if !(turn < -45.0_f32) {
                                if curAnim(a0, 520) == 0 && curAnim(a0, 522) == 0 && curAnim(a0, 521) == 0 && curAnim(a0, 523) == 0 {
                                    start_plain_walk(a0, s3, blend);
                                }
                            } else {
                                'left: {
                                    if curAnim(a0, 520) != 0 {
                                        if curAnim(a0, 522) != 0 {
                                            if curAnimSlot1(a0, 520) != 0 {
                                                if lw(lw(a0 + 32) + 152) != 0 {
                                                    if curAnimSlot2(a0, 520) == 0 {
                                                        break 'left;
                                                    }
                                                }
                                            }
                                            if curAnimSlot1(a0, 522) == 0 {
                                                break 'walk;
                                            }
                                            if lw(lw(a0 + 32) + 152) == 0 {
                                                break 'walk;
                                            }
                                            if curAnimSlot2(a0, 522) != 0 {
                                                break 'walk;
                                            }
                                        }
                                    }
                                }
                                if lw(s3 + 440) == 0 {
                                    setAnim(a0, 520, 0.25_f32, lwc1(0x3ad864));
                                } else {
                                    setAnim(a0, 522, 0.25_f32, lwc1(0x3ad860));
                                }
                            }
                        } else {
                            'right: {
                                if curAnim(a0, 521) != 0 {
                                    if curAnim(a0, 523) != 0 {
                                        if curAnimSlot1(a0, 521) != 0 {
                                            if lw(lw(a0 + 32) + 152) != 0 {
                                                if curAnimSlot2(a0, 521) == 0 {
                                                    break 'right;
                                                }
                                            }
                                        }
                                        if curAnimSlot1(a0, 523) == 0 {
                                            break 'walk;
                                        }
                                        if lw(lw(a0 + 32) + 152) == 0 {
                                            break 'walk;
                                        }
                                        if curAnimSlot2(a0, 523) != 0 {
                                            break 'walk;
                                        }
                                    }
                                }
                            }
                            if lw(s3 + 440) == 0 {
                                setAnim(a0, 521, 0.25_f32, lwc1(0x3ad85c));
                            } else {
                                setAnim(a0, 523, 0.25_f32, lwc1(0x3ad858));
                            }
                        }
                    }
                    'slots: {
                        if curAnimSlot1(a0, 520) != 0 && lw(lw(a0 + 32) + 152) != 0 {
                            break 'slots;
                        }
                        if curAnimSlot1(a0, 522) != 0 && lw(lw(a0 + 32) + 152) != 0 {
                            break 'slots;
                        }
                        if curAnimSlot1(a0, 521) != 0 && lw(lw(a0 + 32) + 152) != 0 {
                            break 'slots;
                        }
                        if curAnimSlot1(a0, 523) == 0 {
                            break 'chosen;
                        }
                        if lw(lw(a0 + 32) + 152) == 0 {
                            break 'chosen;
                        }
                    }
                    if curAnimSlot2(a0, 520) == 0 && curAnimSlot2(a0, 522) == 0 && curAnimSlot2(a0, 521) == 0 && curAnimSlot2(a0, 523) == 0 {
                        let blend = 0.25_f32;
                        if !(lwc1(s3 + 2876) <= ps2float::mul(ps2float::mul(ps2float::mul(lwc1(lw(a0 + 32) + 140), blend), 6e+01_f32), lwc1(s3 + 2868))) {
                            break 'chosen;
                        }
                        if lw(s3 + 440) == 0 {
                            if curAnim(a0, 518) == 0 {
                                setAnim(a0, 518, blend, 2.0_f32);
                            }
                        } else if curAnim(a0, 519) == 0 {
                            setAnim(a0, 519, blend, 2.0_f32);
                        }
                    }
                }
            }
            extractAnimRoot(a0, sp, sp | 4, sp | 8, 2);
            let moved = ps2float::sub(lwc1(sp + 4), lwc1(sp));
            let mut size = fptodp(moved);
            if dpcmp(size, 0_i64) < 0 {
                size = dpsub(0_i64, size);
            }
            let distance = dptofp(size);
            let flags;
            'speed: {
                if !(distance == 0.0_f32) {
                    let walk = lwc1(s3 + 2876);
                    if !(walk == 0.0_f32) {
                        let mut walk_size = fptodp(walk);
                        if dpcmp(walk_size, 0_i64) < 0 {
                            walk_size = dpsub(0_i64, walk_size);
                        }
                        let step = fptodp(lwc1(sp + 8));
                        let product = dpmul(walk_size, step);
                        let per_second = fptodp(ps2float::mul(distance, 6e+01_f32));
                        let quotient = dpdiv(product, per_second);
                        let speed = dptofp(dpdiv(quotient, ld(0x3a9cf8)));
                        curAnimSpeed(lw(a0 + 32), speed);
                        flags = lw(s3 + 2708);
                        break 'speed;
                    }
                }
                curAnimSpeed(lw(a0 + 32), 1.0_f32);
                flags = lw(s3 + 2708);
            }
            sw(s3 + 2712, flags);
            return;
        }
        // the rise ended in a get-up animation: put the zombie on the ground
        let ground = lwc1(s3 + 0x1198);
        let anim = lw(a0 + 32);
        swc1(a0 + 52, ground);
        swc1(anim + 108, ground);
        let height = lwc1(a0 + 52);
        swc1(anim + 132, 0.0_f32);
        swc1(anim + 120, height);
        return;
    }
    if curAnimComplete(lw(a0 + 32)) != 0 {
        setAnim(a0, 518, 0.0_f32, 1.0_f32);
        swc1(s3 + 2928, 0.0_f32);
        swc1(s3 + 2920, 0.0_f32);
        swc1(s3 + 2924, 0.0_f32);
    }
}
