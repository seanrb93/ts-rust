//! Function: the update of a gun a player holds (the state of the gun: taking it out, holding it, firing, reloading, putting it away, and the animation of the gun on the screen).

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    ps2float::mul(random_as_float(), 2.3283064e-10_f32)
}

/// The frames that passed as seconds.
fn seconds() -> f32 {
    (lwc1(g::updateframesf)).pdiv(6e+01_f32)
}

/// Whether a button of the player is held (`held`) or newly pressed (`new`) or released this frame, among the bits at `bits_at` of the controls of the player.
fn button_held(bits_at: i32) -> bool {
    let player = lw(g::currentplayer);
    (joyBut(lw(player)) & lw(lw(g::currentplayer) + bits_at)) != 0
}

/// Half of a number, rounding towards zero.
fn half(value: i32) -> i32 {
    (value + ((value as u32) >> 31) as i32) >> 1
}

/// What the player can do with the gun this frame: the button they hold or press says whether a shot of the first or the second kind is made, and whether the button is
/// held on (the second value). `None` when the gun may not be fired yet (it needs more time, or has too little ammunition).
fn fire_choice(a0: i32, a1: i32, s6_3: i32, s7_3: i32, s0_5: i32, fp: i32) -> Option<(i32, i32)> {
    let mut player = lw(g::currentplayer);
    let new_a = joyNewBut(lw(player));
    player = lw(g::currentplayer);
    let delay;
    let ammo_at;
    let alt;
    let held;
    if (new_a & lw(player + 28)) == 0 {
        let held_a = joyBut(lw(player));
        player = lw(g::currentplayer);
        if (held_a & lw(player + 28)) == 0 {
            let new_b = joyNewBut(lw(player));
            player = lw(g::currentplayer);
            if (new_b & lw(player + 32)) != 0 {
                delay = lw(s6_3 + 196);
                ammo_at = 60;
                alt = 1;
                held = 0;
            } else {
                let held_b = joyBut(lw(player));
                player = lw(g::currentplayer);
                if (held_b & lw(player + 32)) == 0 {
                    sw(a0 + 56, 0);
                    return None;
                }
                delay = lw(s6_3 + 200);
                ammo_at = 64;
                alt = 1;
                held = 1;
            }
        } else {
            delay = lw(s6_3 + 64);
            ammo_at = 64;
            alt = 0;
            held = 1;
        }
    } else {
        delay = lw(s6_3 + 60);
        ammo_at = 60;
        alt = 0;
        held = 0;
    }
    if lw(a0 + 148) < delay {
        return None;
    }
    if s7_3 != 0 {
        if s0_5 < half(lw(s6_3 + ammo_at)) {
            return None;
        }
    }
    if a1 != fp {
        return None;
    }
    Some((alt, held))
}

/// The remote hand: while the player controls a turret (mode 6 or 7) the gun slot makes a character (the arm) that follows the player, swipes at what the hand reaches when
/// the animation plays, and sets the stats of the swipe; gives the prop of the gun slot after.
fn remote_hand(a0: i32, sp: i32, s4_1: i32, anim: i32, s0_1: i32) -> i32 {
    let mut s1_11 = lw(g::currentplayer);
    let s5_1 = lw(s1_11 + 384);
    let mut s3_2 = s0_1;
    let pi;
    let f1_11;
    if s0_1 != 0 {
        f1_11 = lwc1(s5_1 + 76);
    } else {
        let kind = lh(s4_1 + 4);
        let chr = chrNew(kind, kind, 0, lw(s1_11) + 104, -1, 0, lwc1(s5_1 + 48), lwc1(s5_1 + 52), lwc1(s5_1 + 56), lwc1(s5_1 + 76));
        s3_2 = chr;
        sw(a0 + 12, chr);
        propRemovePropFromCollisionList(chr);
        let player = lw(g::currentplayer);
        sw(chr + 8, 16);
        let inst = lw(chr + 32);
        let flags = lw(chr + 16);
        let inst_flags = lhu(inst + 292);
        sw(chr + 12, lw(player) + 201);
        let number = lw(player);
        sh(inst + 292, inst_flags & 0x3fff);
        sw(chr + 16, flags | 0x40000000 | 0x1000000 << (number & 31));
        obInstSetAnim(inst, 3, 1.0_f32);
        s1_11 = lw(g::currentplayer);
        f1_11 = lwc1(s5_1 + 76);
    }
    let mind = lw(s3_2 + 352);
    swc1(s3_2 + 76, f1_11);
    swc1(s3_2 + 80, lwc1(s5_1 + 80));
    swc1(s3_2 + 84, lwc1(s5_1 + 84));
    swc1(mind + 2968, lwc1(s1_11 + 284));
    swc1(mind + 2972, lwc1(s1_11 + 276));
    'b28: {
        let offset;
        if lh(s4_1 + 4) != 24 {
            let model = lw(lw(lw(lw(s1_11 + 384) + 32) + 88) + 12);
            if model != 0 {
                if model != 1 {
                    break 'b28;
                }
                offset = g::offsetvecjoy_f_190;
            } else {
                offset = g::offsetvecjoy_m_189;
            }
        } else {
            offset = g::offsetvecjoy_rf_191;
        }
        swc1(sp + 64, lwc1(offset));
        swc1(sp + 68, lwc1(offset + 4));
        swc1(sp + 72, lwc1(offset + 8));
    }
    pi = lwc1(0x3ad3bc);
    matrixRotY(sp, ps2float::mul(lwc1(s5_1 + 80), pi).pdiv(1.8e+02_f32));
    matrixVecMulAligned(&[sp as i64, (sp + 64) as i64]);
    let player_data = lw(s5_1 + 356);
    swc1(s3_2 + 48, ps2float::add(lwc1(player_data + 140), lwc1(sp + 64)));
    let f1_13 = lwc1(player_data + 144);
    swc1(s3_2 + 52, ps2float::add(ps2float::sub(ps2float::add(f1_13, ps2float::sub(lwc1(player_data + 156), f1_13)), lwc1(player_data + 104)), lwc1(sp + 68)));
    swc1(s3_2 + 56, ps2float::add(lwc1(player_data + 148), lwc1(sp + 72)));
    if curAnim(s3_2, anim) == 0 {
        if (lw(s4_1 + 2708) & 32) != 0 {
            return lw(a0 + 12);
        }
        let mut player = lw(g::currentplayer);
        let held = joyBut(lw(player));
        let mut s1_13 = lw(g::currentplayer);
        if (held & lw(s1_13 + 28)) == 0 {
            let held_b = joyBut(lw(s1_13));
            s1_13 = lw(g::currentplayer);
            if (held_b & lw(s1_13 + 32)) == 0 {
                return lw(a0 + 12);
            }
        }
        player = s1_13;
        let _ = player;
        let inst = lw(lw(a0 + 12) + 32);
        let flags = lhu(inst + 292);
        sw((lw(s1_13) << 2) + 0x1fe99b0, 0);
        sw(a0 + 156, 1);
        sh(inst + 292, flags | 0x8000);
        obInstSetAnim(inst, anim, lwc1(0x3ad3cc));
        if lw(g::updateframesi) <= 0 {
            sw(mind + 2928, 0);
        } else {
            let at = s5_1 + 48;
            if !(0.5_f32 < random_unit()) {
                soundStart(172, at);
            } else {
                soundStart(171, at);
            }
            sw(mind + 2928, 0);
        }
        let prop = lw(a0 + 12);
        sw(mind + 2920, 0);
        sw(mind + 2924, 0);
        return prop;
    }
    if curAnimComplete(lw(s3_2 + 32)) == 0 {
        if lw(a0 + 156) == 0 {
            return lw(a0 + 12);
        }
        let mut hit = 0;
        'b33: {
            matrixTransRotY(sp + 80, lwc1(s3_2 + 48), lwc1(s3_2 + 52), lwc1(s3_2 + 56), ps2float::mul(lwc1(s3_2 + 76), pi).pdiv(1.8e+02_f32));
            let inst = lw(s3_2 + 32);
            sceVu0MulMatrix(sp + 144, sp + 80, lw(inst + 4) + (lw(lw(inst + 92) + 20) << 6));
            let inst = lw(s3_2 + 32);
            let f0_26 = lwc1(sp + 196);
            let f1_14 = lwc1(sp + 200);
            swc1(sp + 208, lwc1(sp + 192));
            swc1(sp + 212, f0_26);
            swc1(sp + 216, f1_14);
            sceVu0MulMatrix(sp + 144, sp + 80, lw(inst + 4) + (lw(lw(inst + 92) + 28) << 6));
            let f4_5 = lwc1(sp + 208);
            let f5_4 = lwc1(sp + 212);
            let f6_3 = lwc1(sp + 216);
            let f5_5 = ps2float::add(f5_4, ps2float::mul(ps2float::sub(lwc1(sp + 196), f5_4), 2.5_f32));
            let f6_4 = ps2float::add(f6_3, ps2float::mul(ps2float::sub(lwc1(sp + 200), f6_3), 2.5_f32));
            swc1(sp + 224, ps2float::add(f4_5, ps2float::mul(ps2float::sub(lwc1(sp + 192), f4_5), 2.5_f32)));
            swc1(sp + 228, f5_5);
            swc1(sp + 232, f6_4);
            if lwc1(mind + 2920) == 0.0_f32 && lwc1(mind + 2924) == 0.0_f32 && lwc1(mind + 2928) == 0.0_f32 {
                break 'b33;
            }
            let inst = lw(s3_2 + 32);
            let time = lwc1(inst + 100);
            let length = lwc1(lw((lw(inst + 96) << 2) + g::pAnimations) + 16);
            if ps2float::mul(length, lwc1(0x3ad3c0)) < time {
                if time < ps2float::mul(length, 0.5_f32) {
                    let found = propSwipeTest(s5_1, sp + 208, mind + 2920, sp + 224, sp + 240, 0);
                    hit = if (found ^ lw((lw(lw(g::currentplayer)) << 2) + 0x1fe99b0)) != 0 { found } else { 0 };
                }
            }
        }
        let tip;
        'b35: {
            let frames = lw(g::updateframesi);
            if hit != 0 {
                if frames > 0 {
                    bulletExplodeMines(sp + 240, 0.25_f32);
                    let dx = ps2float::sub(lwc1(sp + 224), lwc1(sp + 208));
                    let base_z = lwc1(sp + 216);
                    let dy = ps2float::sub(lwc1(sp + 228), lwc1(sp + 212));
                    let tip_z = lwc1(sp + 232);
                    swc1(sp + 256, dx);
                    let dz = ps2float::sub(tip_z, base_z);
                    let number = lw(lw(g::currentplayer));
                    swc1(sp + 260, dy);
                    sw((number << 2) + 0x1fe99b0, hit);
                    swc1(sp + 264, dz);
                    let inverse = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt());
                    let kind = lw(hit + 8);
                    let nx = ps2float::mul(lwc1(sp + 256), inverse);
                    let ny = ps2float::mul(lwc1(sp + 260), inverse);
                    swc1(sp + 264, ps2float::mul(lwc1(sp + 264), inverse));
                    swc1(sp + 256, nx);
                    swc1(sp + 260, ny);
                    if kind == 8 {
                        'b37: {
                            'b38: {
                                let parts = lw(lw(lw(hit + 32)));
                                let last = chrGetLastHitPart();
                                if lw((last << 2) + lw(lw(lw(hit + 32) + 92) + 32)) != 0 {
                                    let last = chrGetLastHitPart();
                                    let state = lw(hit + 32);
                                    let mut n = 0;
                                    if lw((last << 2) + lw(lw(state + 92) + 32)) != 11 {
                                        loop {
                                            if n >= parts {
                                                break 'b38;
                                            }
                                            if lw((n << 2) + lw(lw(state + 92) + 32)) == 0 {
                                                break;
                                            }
                                            n += 1;
                                        }
                                        chrSetLastHitPart(n);
                                        break 'b37;
                                    }
                                }
                            }
                        }
                        let damage;
                        'b41: {
                            'b42: {
                                let diff = anglediff(lwc1(s5_1 + 80), lwc1(hit + 80));
                                if !(0.0_f32 <= diff) {
                                    let diff2 = anglediff(lwc1(s5_1 + 80), lwc1(hit + 80));
                                    if !(-diff2 < 6e+01_f32) {
                                        break 'b42;
                                    }
                                } else {
                                    let diff2 = anglediff(lwc1(s5_1 + 80), lwc1(hit + 80));
                                    if !(diff2 < 6e+01_f32) {
                                        break 'b42;
                                    }
                                }
                                damage = lwc1(0x3ad3c4);
                                break 'b41;
                            }
                            damage = 0.5_f32;
                        }
                        propDamage(hit, s5_1, sp + 240, 1, sp + 224, sp + 256, 0, 0, damage);
                        tip = lwc1(sp + 224);
                        break 'b35;
                    }
                    propDamage(hit, s5_1, sp + 240, 1, sp + 224, sp + 256, 0, 0, lwc1(0x3ad3c8));
                }
            }
            tip = lwc1(sp + 224);
        }
        let prop = lw(a0 + 12);
        swc1(mind + 2920, tip);
        swc1(mind + 2924, lwc1(sp + 228));
        swc1(mind + 2928, lwc1(sp + 232));
        return prop;
    }
    let inst = lw(s3_2 + 32);
    sh(inst + 292, lhu(inst + 292) & 0x7fff);
    obInstSetAnim(inst, 3, 1.0_f32);
    lw(a0 + 12)
}

/// Fire the remote gun with the buttons of the player (while they control a turret): the first button fires the main shot, the second the other one.
fn remote_fire(a0: i32, s4_1: i32) -> Option<i32> {
    let player = lw(g::currentplayer);
    let s5_1 = lw(player + 384);
    let remote = lw(lw(s5_1 + 352) + 0x11b4);
    let chain = lw(lw(remote + 352) + 100);
    let first = joyBut(lw(player));
    let now = lw(g::currentplayer);
    let (alt, id);
    if (first & lw(now + 28)) == 0 {
        let second = joyBut(lw(now));
        let again = lw(g::currentplayer);
        if (second & lw(again + 32)) == 0 {
            return None;
        }
        alt = 1;
        id = lw(again);
    } else {
        alt = 0;
        id = lw(now);
    }
    let _ = s4_1;
    gunRemoteFire(remote, chain, alt, 0, 4, id);
    Some(lw(a0 + 12))
}

/// Switch to the gun the player wants (state 3 to 4) or begin to put the held one away: the end of a gun that stays.
fn hold_gun(a0: i32, a1: i32, s3_1: i32, s4_1: i32, s7_1: i32) -> Option<i32> {
    let s6_3 = lw(s3_1) * 400 + g::guns;
    gunUpdateParts(a0, s3_1);
    if lw(s3_1) == s7_1 {
        let player = lw(g::currentplayer);
        if lw(player + 296) == 4 {
            return Some(lw(a0 + 12));
        }
        let wanted = s7_1 * 400;
        if lw(g::updateframesi) <= 0 {
            return Some(lw(a0 + 12));
        }
        let mut s7_3 = 0;
        let ammo_kind = lw(wanted + (g::guns + 16));
        if lh((lw(s4_1 + 264) << 3) + (GUNCOMBOS + 2)) != -1 {
            s7_3 = ((lw(s4_1 + 484) as u32) < (1 as u32)) as i32;
        }
        let left_ammo = lw(player + 748);
        let mut right_ammo = -1;
        if s7_3 != 0 {
            right_ammo = lw(player + 556);
        }
        let mut fp = 0;
        let s0_5 = if a1 != 0 {
            if s7_3 == 0 { right_ammo } else { left_ammo }
        } else {
            right_ammo
        };
        if s7_3 != 0 {
            let mode = lw(player + 408);
            let slot = ammo_kind << 2;
            if mode == 1 {
                let less = (left_ammo < right_ammo) as i32;
                if lw(s4_1 + slot + 272) > 0 {
                    if less != 0 {
                        fp = 1;
                    } else if lw(player + 600) != mode {
                        fp = 1;
                    } else if lw(s4_1 + slot + 380) == 0 {
                        fp = 1;
                    }
                }
            }
        }
        let reload = joyNewBut(lw(player));
        let player_now = lw(g::currentplayer);
        if (reload & lw(player_now + 40)) != 0 {
            gunReload(lw(lw(player_now + 384) + 352), s3_1, a0, 1, a1);
        }
        if let Some((alt, held)) = fire_choice(a0, a1, s6_3, s7_3, s0_5, fp) {
            gunPlayerFire(s3_1, a0, alt, held, a1);
        }
        if button_held(28) {
            return Some(lw(a0 + 12));
        }
        if button_held(32) {
            return Some(lw(a0 + 12));
        }
        if lw(s3_1) == -1 {
            return Some(lw(a0 + 12));
        }
        if lw(s3_1 + (lw(s6_3 + 16) << 2) + 4) != 0 {
            return Some(lw(a0 + 12));
        }
        let player_last = lw(g::currentplayer);
        gunReload(lw(lw(player_last + 384) + 352), s3_1, a0, 0, a1);
        return Some(lw(a0 + 12));
    }
    let player = lw(g::currentplayer);
    sw(a0, 2);
    sw(a0 + 4, 0);
    let prop = lw(a0 + 12);
    if ((lw(player + 296) - 6) as u32) < (2 as u32) {
        return Some(prop);
    }
    swc1(player + 268, lwc1(s6_3 + 44));
    swc1(player + 272, lwc1(s6_3 + 48));
    Some(prop)
}

/// The sway of the gun on the screen after the update of the state (the rumble of a long fuse): set the vibration of the controller.
fn fuse_rumble(a0: i32, player: i32) -> Option<i32> {
    let left = joyGetShockLeft(lw(player));
    let right = joyGetShockRight(lw(lw(g::currentplayer)));
    let right = if (lw(g::gameframe) & 3) != 0 { right } else { right + 128 };
    joySetShock(lw(player), left, right);
    Some(lw(a0 + 12))
}

/// Update the gun slot `a0` of the player for the hand `a1` (0 right, 1 left) (@ 0x2922d8): the state at +0 and the time at +4 say what it does (0 not held, 1 held, 2 and 3
/// being taken out or put away, 5 to 7 reloading, 8 / 19 / 18 pumping or cocking, 9 to 15 the shotgun reload, 16 and 17 a grenade being primed) and move the model of the gun on the
/// screen by it.
pub fn gunUpdate(a0: i32, a1: i32) {
    let frame = Frame::new(464);
    let sp = frame.sp();
    let s4_1 = lw(lw(lw(g::currentplayer) + 384) + 352);
    let s3_1 = if a1 != 0 { s4_1 + 268 } else { s4_1 + 376 };
    let combo = (lw(s4_1 + 260) << 3) + GUNCOMBOS;
    let mut s7_1 = if a1 == 0 { lh(combo + 4) } else { lh(combo + 2) };
    let able = if a1 == 0 { gunRightHandAble(s4_1) } else { gunLeftHandAble(s4_1) };
    if able == 0 {
        s7_1 = -1;
    }
    let frames = lw(g::updateframesi);
    let s0_1 = lw(a0 + 12);
    sw(a0 + 148, lw(a0 + 148) + frames);
    sw(s3_1 + 68, lw(s3_1 + 68) + frames);
    if s0_1 != 0 {
        let inst = lw(s0_1 + 32);
        sh(inst + 292, lhu(inst + 292) & 0xfffe);
    }
    let held_prop = lw(s3_1 + 64);
    if held_prop != 0 {
        let inst = lw(held_prop + 32);
        sh(inst + 292, lhu(inst + 292) & 0xfffc);
    }
    let mut state = lw(a0);
    let mut prop = s0_1;
    if state != 0 {
        let now = lw(g::gameframe);
        if prop != 0 {
            let scale = lwc1(0x3ad3b0);
            let step = lwc1(0x3ad3b4);
            let mut gun = lw(s3_1);
            let info = gun * 400 + g::guns;
            let wave = lwc1(((ps2float::add(ps2float::mul(ps2float::mul(now as f32, scale).pdiv(1.8e+02_f32), step), 0.5_f32) as i32 & 2047) << 2) + g::g_SinTable);
            let before = lwc1(((ps2float::add(ps2float::mul(ps2float::mul((now - frames) as f32, scale).pdiv(1.8e+02_f32), step), 0.5_f32) as i32 & 2047) << 2) + g::g_SinTable);
            let side = if a1 != 0 { -1.0_f32 } else { 1.0_f32 };
            if 0.5_f32 <= wave {
                if before < 0.5_f32 {
                    let r = random_unit();
                    prop = lw(a0 + 12);
                    gun = lw(s3_1);
                    swc1(a0 + 64, ps2float::mul(r, 5.0_f32));
                }
            }
            let player = lw(g::currentplayer);
            let height = lwc1(lw(g::currentplayer) + 396);
            let sway = ps2float::mul(side, ps2float::mul(ps2float::sub(wave, 0.5_f32), lwc1(a0 + 64)));
            let across = if a1 == 0 { lwc1(info + 32) } else { -lwc1(info + 32) };
            swc1(prop + 48, ps2float::add(across, height));
            swc1(prop + 52, ps2float::add(lwc1(info + 36), lwc1(player + 400)));
            swc1(prop + 56, ps2float::add(lwc1(info + 40), lwc1(player + 404)));
            let (f4_2, f2_4);
            if ((gun - 14) as u32) >= (5 as u32) && gun != 25 {
                f2_4 = lwc1(player + 260);
                f4_2 = 1.0_f32;
            } else {
                f4_2 = lwc1(0x3ad3b8);
                f2_4 = lwc1(player + 260);
            }
            let f21_2 = (f2_4).pdiv(6e+01_f32);
            let f20_2 = ps2float::mul(f4_2, 3e+01_f32);
            swc1(prop + 72, ps2float::add(lwc1(a0 + 60), ps2float::mul(ps2float::mul(f20_2, lwc1(player + 816)), f21_2)));
            let f22_1 = lwc1(player + 812);
            let f0_3 = if ilinkGetNumLocalPlayers() != 2 { ps2float::mul(f20_2, f22_1) } else { ps2float::mul(ps2float::add(f20_2, f20_2), f22_1) };
            swc1(prop + 76, ps2float::sub(1.8e+02_f32, ps2float::mul(f0_3, f21_2)));
            prop = lw(a0 + 12);
            state = lw(a0);
            swc1(prop + 88, ps2float::add(sway, -2.0_f32));
        }
    }
    let mut result = Some(prop);
    if (state as u32) < (20 as u32) {
        result = match gun_state(a0, a1, sp, s3_1, s4_1, s7_1, state, prop) {
            Step::Prop(p) => Some(p),
            Step::Skip => None,
        };
    }
    if let Some(prop_now) = result {
        if prop_now != 0 {
            let view = lw(g::viewguns);
            if view == 0 {
                sw(prop_now + 16, lw(prop_now + 16) & -2);
            } else {
                sw(prop_now + 16, lw(prop_now + 16) | 1);
            }
        }
    }
}

/// What a state of the gun gives back: the gun prop to finish the update with, or that the update is over.
enum Step {
    Prop(i32),
    Skip,
}

/// Put the gun prop away, give the ammunition back and take the next gun in the combo (state 3).
fn finish_putting_away(a0: i32, s3_1: i32, s4_1: i32, s7_1: i32) -> Step {
    let first = lw(a0 + 140);
    let mut to = s3_1 + 8;
    let mut limit = g::ammoinfo + 8;
    let mut at = s4_1 + 624;
    let mut n = 13;
    loop {
        let total = lw(at) + lw(to);
        sw(at, total);
        let max = lw(limit);
        if max < total {
            sw(at, max);
        }
        sw(to, 0);
        limit += 8;
        to += 4;
        n -= 1;
        at += 4;
        if n < 0 {
            break;
        }
    }
    if first != 0 {
        propDelete(first);
        sw(a0 + 140, 0);
    }
    let second = lw(a0 + 144);
    if second != 0 {
        propDelete(second);
        sw(a0 + 144, 0);
    }
    propDelete(lw(a0 + 12));
    sw(a0 + 12, 0);
    let held = lw(s3_1 + 64);
    if held != 0 {
        propDelete(held);
        sw(s3_1 + 64, 0);
    }
    sw(s3_1, s7_1);
    sw(s4_1 + 264, lw(s4_1 + 260));
    if s7_1 >= 0 {
        return change_state(a0, 4);
    }
    let prop = lw(a0 + 12);
    sw(a0, 0);
    Step::Prop(prop)
}

/// Go to the state `new_state` with the time 0.
fn change_state(a0: i32, new_state: i32) -> Step {
    let prop = lw(a0 + 12);
    sw(a0, new_state);
    sw(a0 + 4, 0);
    Step::Prop(prop)
}

/// The end of the states that move the gun by the offsets `lift` and `slide` before the update of its parts.
fn move_and_update(a0: i32, s3_1: i32, prop: i32, lift: f32, slide: f32) -> Step {
    swc1(prop + 52, ps2float::add(lwc1(prop + 52), lift));
    swc1(prop + 72, ps2float::add(lwc1(prop + 72), slide));
    gunUpdateParts(a0, s3_1);
    Step::Prop(lw(a0 + 12))
}

/// The state 8 and 19 end (reloaded when the time `length` has gone by): go back to state 1 with the speeds of the gun.
fn end_of_pump(a0: i32, info: i32, done: bool) -> Step {
    if !done {
        return Step::Prop(lw(a0 + 12));
    }
    sw(a0 + 4, 0);
    let player = lw(g::currentplayer);
    let first = lwc1(info + 44);
    sw(a0, 1);
    swc1(player + 268, first);
    let prop = lw(a0 + 12);
    swc1(player + 272, lwc1(info + 48));
    Step::Prop(prop)
}

/// Play the sound `sound` for the gun and finish.
fn sound_and_end(a0: i32, sound: i32) -> Step {
    soundStart(sound, 0);
    Step::Prop(lw(a0 + 12))
}

/// The state machine of a gun: one state per call (see `gunUpdate`).
fn gun_state(a0: i32, a1: i32, sp: i32, s3_1: i32, s4_1: i32, s7_1: i32, state: i32, prop: i32) -> Step {
    let seconds_now = seconds();
    match state {
        0 => {
            let player = lw(g::currentplayer);
            if s7_1 >= 0 {
                let mode = lw(player + 296);
                if mode != 6 && mode != 7 {
                    sw(s3_1, s7_1);
                    sw(a0 + 4, 0);
                    sw(s4_1 + 264, lw(s4_1 + 260));
                    sw(a0, 4);
                    if prop == 0 {
                        return Step::Skip;
                    }
                    propDelete(prop);
                    sw(a0 + 12, 0);
                    return Step::Prop(0);
                }
            }
            let s1_11 = lw(g::currentplayer);
            if a1 != 0 {
                return Step::Prop(prop);
            }
            let s5_1 = lw(s1_11 + 384);
            let mode = lw(s1_11 + 296);
            let anim = if (lw(lw(lw(s5_1 + 32) + 88) + 12) ^ 1) == 0 { 503 } else { 465 };
            let mut remote = false;
            if ((mode - 6) as u32) < (2 as u32) {
                if mode == 7 {
                    remote = if prop != 0 { curAnim(prop, anim) == 0 } else { true };
                }
            } else {
                swc1(s1_11 + 272, 6e+01_f32);
                swc1(s1_11 + 268, 6e+01_f32);
            }
            if !remote {
                return Step::Prop(remote_hand(a0, sp, s4_1, anim, prop));
            }
            match remote_fire(a0, s4_1) {
                Some(p) => Step::Prop(p),
                None => Step::Prop(lw(a0 + 12)),
            }
        }
        1 => match hold_gun(a0, a1, s3_1, s4_1, s7_1) {
            Some(p) => Step::Prop(p),
            None => Step::Prop(lw(a0 + 12)),
        },
        2 => {
            let length = lwc1(0x3ad3d0);
            let fraction = (lwc1(a0 + 4)).pdiv(length);
            swc1(prop + 52, ps2float::sub(lwc1(prop + 52), ps2float::mul(fraction, lwc1(0x3ad3d4))));
            gunUpdateParts(a0, s3_1);
            let prop_now = lw(a0 + 12);
            let step = seconds_now;
            swc1(prop_now + 72, ps2float::add(lwc1(prop_now + 72), ps2float::mul(fraction, 6e+01_f32)));
            let time = ps2float::add(lwc1(a0 + 4), step);
            swc1(a0 + 4, time);
            if !(length <= time) {
                return Step::Prop(prop_now);
            }
            sw(a0 + 4, 0);
            sw(a0, 3);
            Step::Prop(prop_now)
        }
        3 => finish_putting_away(a0, s3_1, s4_1, s7_1),
        4 => Step::Prop(prop),
        5 => {
            let mut prop_now = prop;
            let time = ps2float::sub(lwc1(a0 + 4), seconds_now);
            swc1(a0 + 4, time);
            if time <= 0.0_f32 {
                let mut left_able = 0;
                let slot = lw(lw(s3_1) * 400 + (g::guns + 16)) << 2;
                let ammo = lw(s4_1 + slot + 620) + lw(s3_1 + slot + 4);
                if lh((lw(s4_1 + 260) << 3) + (GUNCOMBOS + 2)) != -1 {
                    left_able = (gunLeftHandAble(s4_1) != 0) as i32;
                }
                let mut left_empty = 0;
                if left_able != 0 {
                    let at = s4_1 + slot;
                    let count = if a1 == 0 { lw(at + 272) } else { lw(at + 380) };
                    left_empty = (count < 1) as i32;
                }
                'b62: {
                    'b66: {
                        if ammo <= 0 {
                            if gunShouldKeepWithNoAmmo(s4_1, lw(s4_1 + 260)) == 0 {
                                if left_able != 0 {
                                    if left_empty == 0 {
                                        break 'b66;
                                    }
                                }
                                sw(a0 + 4, 0);
                                sw(a0, 3);
                                changeToComboWithAmmo(s4_1);
                                prop_now = lw(a0 + 12);
                                break 'b62;
                            }
                        }
                    }
                    sw(a0 + 4, 0);
                    sw(a0, 6);
                    prop_now = lw(a0 + 12);
                }
            }
            move_and_update(a0, s3_1, prop_now, lwc1(0x3ad424), 6e+01_f32)
        }
        6 => {
            let length = lwc1(0x3ad3e4);
            let fraction = (ps2float::sub(length, lwc1(a0 + 4))).pdiv(length);
            let info = lw(s3_1) * 400 + g::guns;
            let slide = ps2float::add(lwc1(prop + 72), ps2float::mul(fraction, 6e+01_f32));
            swc1(prop + 52, ps2float::sub(lwc1(prop + 52), ps2float::mul(fraction, lwc1(0x3ad3e8))));
            swc1(prop + 72, slide);
            gunUpdateParts(a0, s3_1);
            let time = ps2float::add(lwc1(a0 + 4), seconds_now);
            swc1(a0 + 4, time);
            if !(length <= time) {
                return Step::Prop(lw(a0 + 12));
            }
            sw(a0 + 4, 0);
            let player = lw(g::currentplayer);
            let first = lwc1(info + 44);
            sw(a0, 1);
            swc1(player + 268, first);
            let flag = lw(player + 20) & 2;
            swc1(player + 272, lwc1(info + 48));
            if flag != 0 {
                swc1(player + 264, first);
            }
            let gun = lw(s3_1);
            if gun != 25 {
                if gun == 18 {
                    return Step::Prop(lw(a0 + 12));
                }
                return sound_and_end(a0, 137);
            }
            sound_and_end(a0, 135)
        }
        7 => {
            let mut length = lwc1(0x3ad3d8);
            let time_now = lwc1(a0 + 4);
            if lw(s3_1) == 8 {
                length = lwc1(0x3ad3dc);
            }
            let fraction = (time_now).pdiv(length);
            let slide = ps2float::add(lwc1(prop + 72), ps2float::mul(fraction, 6e+01_f32));
            swc1(prop + 52, ps2float::sub(lwc1(prop + 52), ps2float::mul(fraction, lwc1(0x3ad3e0))));
            swc1(prop + 72, slide);
            gunUpdateParts(a0, s3_1);
            let time = ps2float::add(lwc1(a0 + 4), seconds_now);
            swc1(a0 + 4, time);
            if !(length <= time) {
                return Step::Prop(lw(a0 + 12));
            }
            if lw(s3_1) == 23 {
                let mut n = 0;
                let cartridge = if lw(a0 + 52) == 0 { g::guns + 9252 } else { g::guns + 9388 };
                if lw(a0 + 184) > 0 {
                    let mut at = lw(a0 + 180);
                    loop {
                        n += 1;
                        addCartridge(at, 0, lw(cartridge + 4), s3_1, a0, g::guns + 9200, s4_1);
                        if n >= lw(a0 + 184) {
                            break;
                        }
                        at = lw(a0 + 180);
                    }
                }
            }
            resetPlayerGunReload(a0, s3_1);
            change_state(a0, 8)
        }
        8 | 19 => {
            let gun = lw(s3_1);
            let info = gun * 400 + g::guns;
            let (length, kick, up) = if state == 8 {
                (if gun == 8 { lwc1(0x3ad3f0) } else { lwc1(0x3ad3ec) }, lwc1(0x3ad3f4), 6e+01_f32)
            } else {
                (lwc1(0x3ad440), lwc1(0x3ad444), 9e+01_f32)
            };
            let fraction = (ps2float::sub(length, lwc1(a0 + 4))).pdiv(length);
            let slide = ps2float::add(lwc1(prop + 72), ps2float::mul(fraction, up));
            swc1(prop + 52, ps2float::sub(lwc1(prop + 52), ps2float::mul(fraction, kick)));
            swc1(prop + 72, slide);
            gunUpdateParts(a0, s3_1);
            let time = ps2float::add(lwc1(a0 + 4), seconds_now);
            let done = length <= time;
            swc1(a0 + 4, time);
            end_of_pump(a0, info, done)
        }
        9 => {
            let length = lwc1(0x3ad3f8);
            let fraction = (lwc1(a0 + 4)).pdiv(length);
            swc1(a0 + 136, ps2float::mul(fraction, 45.0_f32));
            swc1(prop + 72, ps2float::add(lwc1(prop + 72), ps2float::mul(fraction, -2e+01_f32)));
            gunUpdateParts(a0, s3_1);
            updateShotgunReload(a0);
            let time = ps2float::add(lwc1(a0 + 4), seconds_now);
            swc1(a0 + 4, time);
            if !(length <= time) {
                return Step::Prop(lw(a0 + 12));
            }
            resetPlayerGunReload(a0, s3_1);
            let rest = ps2float::sub(lwc1(a0 + 4), length);
            sw(a0, 10);
            let prop_now = lw(a0 + 12);
            swc1(a0 + 4, ps2float::add(rest, 0.0_f32));
            Step::Prop(prop_now)
        }
        10 => {
            let length = lwc1(0x3ad3fc);
            let fraction = (lwc1(a0 + 4)).pdiv(length);
            let side = if a1 != 0 { -1.0_f32 } else { 1.0_f32 };
            let roll = ps2float::add(lwc1(prop + 88), ps2float::mul(side, ps2float::mul(fraction, 35.0_f32)));
            let drop = ps2float::sub(lwc1(prop + 52), ps2float::mul(fraction, 0.0_f32));
            swc1(prop + 72, ps2float::add(lwc1(prop + 72), ps2float::sub(-9e+01_f32, ps2float::mul(ps2float::sub(1.0_f32, fraction), -7e+01_f32))));
            swc1(prop + 88, roll);
            swc1(prop + 52, drop);
            gunUpdateParts(a0, s3_1);
            updateShotgunReload(a0);
            let time = ps2float::add(lwc1(a0 + 4), seconds_now);
            swc1(a0 + 4, time);
            if !(length <= time) {
                return Step::Prop(lw(a0 + 12));
            }
            resetPlayerGunReload(a0, s3_1);
            let first = lw(a0 + 140);
            let hand = lwc1(0x3ad400);
            let shell = lwc1(0x3ad404);
            let second = lw(a0 + 144);
            let rest = ps2float::add(ps2float::sub(lwc1(a0 + 4), length), 0.0_f32);
            sw(a0, 11);
            let prop_now = lw(a0 + 12);
            swc1(a0 + 4, rest);
            swc1(first + 68, shell);
            swc1(first + 60, 0.0_f32);
            swc1(first + 64, hand);
            swc1(second + 68, shell);
            swc1(second + 60, 0.0_f32);
            swc1(second + 64, hand);
            Step::Prop(prop_now)
        }
        11 => {
            let side = if a1 != 0 { -1.0_f32 } else { 1.0_f32 };
            let time_now = lwc1(a0 + 4);
            let length = lwc1(0x3ad408);
            let fraction = (time_now).pdiv(length);
            let slide = ps2float::add(lwc1(prop + 72), ps2float::add(ps2float::mul(ps2float::mul(fraction, 1.4e+02_f32), fraction), -9e+01_f32));
            let drop = ps2float::sub(lwc1(prop + 52), ps2float::add(ps2float::mul(ps2float::mul(ps2float::mul(fraction, lwc1(0x3ad40c)), fraction), fraction), 0.0_f32));
            swc1(prop + 88, ps2float::add(lwc1(prop + 88), ps2float::mul(side, ps2float::mul(ps2float::sub(1.0_f32, fraction), 35.0_f32))));
            swc1(prop + 72, slide);
            swc1(prop + 52, drop);
            gunUpdateParts(a0, s3_1);
            updateShotgunReload(a0);
            let time = ps2float::add(lwc1(a0 + 4), seconds_now);
            swc1(a0 + 4, time);
            if !(length <= time) {
                return Step::Prop(lw(a0 + 12));
            }
            let first = lw(a0 + 140);
            sw(a0, 14);
            swc1(a0 + 4, ps2float::add(ps2float::sub(time, length), 0.0_f32));
            drop_shells(a0, first)
        }
        12 => {
            let length = lwc1(0x3ad418);
            let fraction = (lwc1(a0 + 4)).pdiv(length);
            let drop = ps2float::sub(lwc1(prop + 52), ps2float::mul(ps2float::sub(1.0_f32, fraction), lwc1(0x3ad41c)));
            swc1(prop + 72, ps2float::add(lwc1(prop + 72), ps2float::sub(5e+01_f32, ps2float::mul(fraction, 7e+01_f32))));
            swc1(prop + 52, drop);
            gunUpdateParts(a0, s3_1);
            updateShotgunReload(a0);
            let time = ps2float::add(lwc1(a0 + 4), seconds_now);
            swc1(a0 + 4, time);
            if !(length <= time) {
                return Step::Prop(lw(a0 + 12));
            }
            sw(a0 + 4, 0);
            sw(a0, 13);
            soundDelayStartEx(197, 0, 3, 0);
            Step::Prop(lw(a0 + 12))
        }
        13 => {
            let length = lwc1(0x3ad420);
            let left = ps2float::sub(1.0_f32, (lwc1(a0 + 4)).pdiv(length));
            swc1(a0 + 136, ps2float::mul(ps2float::mul(ps2float::mul(left, 45.0_f32), left), left));
            swc1(prop + 72, ps2float::add(lwc1(prop + 72), ps2float::mul(left, -2e+01_f32)));
            gunUpdateParts(a0, s3_1);
            updateShotgunReload(a0);
            let time = ps2float::add(lwc1(a0 + 4), seconds_now);
            swc1(a0 + 4, time);
            if !(length <= time) {
                return Step::Prop(lw(a0 + 12));
            }
            sw(a0 + 4, 0);
            let first = lw(a0 + 140);
            sw(a0, 1);
            drop_shells(a0, first)
        }
        14 => {
            let slide = ps2float::add(lwc1(prop + 72), 5e+01_f32);
            let step = seconds_now;
            swc1(prop + 52, ps2float::add(lwc1(prop + 52), lwc1(0x3ad410)));
            swc1(prop + 72, slide);
            swc1(a0 + 4, ps2float::add(lwc1(a0 + 4), step));
            gunUpdateParts(a0, s3_1);
            if lwc1(0x3ad414) <= lwc1(a0 + 4) {
                sw(a0 + 4, 0);
                sw(a0, 12);
                let zero = lwc1(a0 + 4);
                let first = propNew(222, zero, zero, zero, zero);
                let player = lw(g::currentplayer);
                sw(first + 140, 2);
                sw(first + 136, 2);
                let number = lw(player);
                let flags = lw(first + 16) | 0x40000000;
                sw(a0 + 140, first);
                sw(first + 16, flags);
                sw(first + 12, number + 201);
                let second = propNew(222, zero, zero, zero, zero);
                let player = lw(g::currentplayer);
                sw(second + 136, 2);
                sw(second + 140, 2);
                let number = lw(player);
                let flags = lw(second + 16) | 0x40000000;
                sw(a0 + 144, second);
                sw(second + 16, flags);
                sw(second + 12, number + 201);
            }
            updateShotgunReload(a0);
            Step::Prop(lw(a0 + 12))
        }
        15 => {
            let length = lwc1(0x3ad428);
            let fraction = (lwc1(a0 + 4)).pdiv(length);
            let roll = ps2float::add(lwc1(prop + 88), ps2float::mul(fraction, -5e+01_f32));
            swc1(prop + 52, ps2float::sub(lwc1(prop + 52), ps2float::mul(fraction, lwc1(0x3ad42c))));
            swc1(prop + 88, roll);
            gunUpdateParts(a0, s3_1);
            let time = ps2float::add(lwc1(a0 + 4), seconds_now);
            swc1(a0 + 4, time);
            if !(length <= time) {
                return Step::Prop(lw(a0 + 12));
            }
            sw(a0 + 4, 0);
            sw(a0, 16);
            sound_and_end(a0, 186)
        }
        16 => {
            let length = lwc1(0x3ad430);
            let left = ps2float::sub(1.0_f32, (lwc1(a0 + 4)).pdiv(length));
            let roll = ps2float::add(lwc1(prop + 88), ps2float::mul(left, -5e+01_f32));
            swc1(prop + 52, ps2float::sub(lwc1(prop + 52), ps2float::mul(left, lwc1(0x3ad434))));
            swc1(prop + 88, roll);
            gunUpdateParts(a0, s3_1);
            updateFuse(a0, 0.0_f32);
            let time = ps2float::add(lwc1(a0 + 4), seconds_now);
            swc1(a0 + 4, time);
            if length <= time {
                swc1(a0 + 4, 0.0_f32);
                sw(a0, 17);
                soundStart(178, 0);
            }
            let player = lw(g::currentplayer);
            if (lhu(lw(player + 24) + 2) & 80) != 80 {
                return Step::Prop(lw(a0 + 12));
            }
            rumble(a0, player)
        }
        17 => {
            let length = lwc1(0x3ad438);
            let fraction = (lwc1(a0 + 4)).pdiv(length);
            gunUpdateParts(a0, s3_1);
            updateFuse(a0, fraction);
            let time = ps2float::add(lwc1(a0 + 4), seconds_now);
            swc1(a0 + 4, time);
            let mut fire = true;
            if !(length <= time) {
                let mut player = lw(g::currentplayer);
                let held = joyBut(lw(lw(g::currentplayer)));
                player = lw(g::currentplayer);
                let mut any = (held & lw(player + 28)) != 0;
                if !any {
                    let held_b = joyBut(lw(player));
                    player = lw(g::currentplayer);
                    any = (held_b & lw(player + 32)) != 0;
                }
                if any {
                    let released = joyReleaseBut(lw(player));
                    let now = lw(g::currentplayer);
                    if (released & lw(now + 28)) == 0 {
                        let released_b = joyReleaseBut(lw(now));
                        let again = lw(g::currentplayer);
                        if (released_b & lw(again + 32)) == 0 {
                            fire = false;
                        }
                    }
                }
            }
            if fire {
                playerFireShots(a0);
            }
            let player = lw(g::currentplayer);
            let screen = lw(player + 24);
            if (lhu(screen + 2) & 80) != 80 {
                return Step::Prop(lw(a0 + 12));
            }
            rumble(a0, player)
        }
        _ => {
            // state 18
            let time = ps2float::sub(lwc1(a0 + 4), seconds_now);
            swc1(a0 + 4, time);
            let mut prop_now = prop;
            if time <= 0.0_f32 {
                if lw(s4_1 + (lw(lw(s3_1) * 400 + (g::guns + 16)) << 2) + 620) > 0 {
                    swc1(a0 + 4, 0.0_f32);
                    sw(a0, 19);
                } else {
                    swc1(a0 + 4, 0.0_f32);
                    sw(a0, 3);
                    changeToComboWithAmmo(s4_1);
                    prop_now = lw(a0 + 12);
                }
            }
            move_and_update(a0, s3_1, prop_now, lwc1(0x3ad43c), 9e+01_f32)
        }
    }
}

/// Drop the shells (the two props at +140 and +144) and finish.
fn drop_shells(a0: i32, first: i32) -> Step {
    propDelete(first);
    sw(a0 + 140, 0);
    let second = lw(a0 + 144);
    propDelete(second);
    let prop = lw(a0 + 12);
    sw(a0 + 144, 0);
    Step::Prop(prop)
}

/// Shake the controller (stronger on some frames) while a grenade is primed: the part of the update after the states.
fn rumble(a0: i32, player: i32) -> Step {
    match fuse_rumble(a0, player) {
        Some(p) => Step::Prop(p),
        None => Step::Skip,
    }
}
