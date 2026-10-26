//! Function: working out the matrices of the bones of an animated character from its animations.

use crate::*;

/// Turn the character `owner` the way the root of the animation at `anim` has turned (@ the middle of calMatrices): while the animation is in its first half
/// (`half`), the angle it should have reached is 90 degrees over `half` of the time, and the difference with the angle kept at `last` is added to (or taken from) the
/// heading of the character at +84, which is kept between 0 and 360 degrees.
fn turn_with_root(a0: i32, anim: i32, last: i32, half: f32, add: bool) {
    let time = lwc1(a0 + anim);
    if !(time <= half) {
        return;
    }
    let from = lwc1(a0 + last);
    let to = ps2float::mul((time).pdiv(half), 9e+01_f32);
    let difference = anglediff(from, to);
    let owner = lw(a0 + 244);
    if add {
        let heading = ps2float::add(lwc1(owner + 84), difference);
        swc1(owner + 84, heading);
        if 3.6e+02_f32 < heading {
            swc1(owner + 84, ps2float::sub(heading, 3.6e+02_f32));
        }
    } else {
        let heading = ps2float::sub(lwc1(owner + 84), difference);
        swc1(owner + 84, heading);
        if heading < 0.0_f32 {
            swc1(owner + 84, ps2float::add(heading, 3.6e+02_f32));
        }
    }
    swc1(a0 + last, to);
}

/// Where in the animation `anim` the time `time` of the state lies between the two key frames `from` and `to` (0 up to 1), the key frame times being kept in frames of 60
/// to the second.
fn key_frame_fraction(anim: i32, time: f32, from: i32, to: i32) -> f32 {
    let times = lw(anim + 20);
    let a = (lw((from << 2) + times) as f32).pdiv(6e+01_f32);
    (ps2float::sub(time, a)).pdiv(ps2float::sub((lw((to << 2) + times) as f32).pdiv(6e+01_f32), a))
}

/// Calculate the matrices of the bones of the animated character `a0` (@ 0x212e08): move the time of its two animations (the second one blends in while it changes from
/// the first), find the two key frames of each, blend the position and the rotation of every bone between them, turn the heading with the root when it should, add the hit
/// reaction, make the global matrices and the fillets, tilt the head and the body of the characters that look about, apply the size of the cheats, and keep the movement
/// of the root.
pub fn calMatrices(a0: i32) {
    let frame = Frame::new(784);
    let sp = frame.sp();
    let owner = lw(a0 + 244);
    let f22_1 = if lw(owner + 8) != 0x1000 { lwc1(lw(owner + 352) + 2840) } else { lwc1(lw(owner + 352) + 12) };
    let s0_1 = a0 + 96;
    swc1(owner + 76, lwc1(owner + 84));
    let blend_length = lwc1(a0 + 224);
    let s2_1 = a0 + 160;
    if 0.0_f32 < blend_length {
        let blend_time = ps2float::add(lwc1(a0 + 232), (lwc1(g::updateframesf)).pdiv(6e+01_f32));
        let blend = (blend_time).pdiv(blend_length);
        swc1(a0 + 232, blend_time);
        swc1(a0 + 228, blend);
        if 1.0_f32 <= blend {
            let v1_1 = lw(a0 + 160);
            let a0_1 = lw(a0 + 220);
            let f0_2 = lwc1(a0 + 164);
            let f1_1 = lwc1(a0 + 180);
            let f2_2 = lwc1(a0 + 184);
            let f3_1 = lwc1(a0 + 188);
            let f4_2 = lwc1(a0 + 168);
            let f5_1 = lwc1(a0 + 172);
            let f6_1 = lwc1(a0 + 176);
            let f7_1 = lwc1(a0 + 192);
            let f8_1 = lwc1(a0 + 196);
            let f9_1 = lwc1(a0 + 200);
            let f10_1 = lwc1(a0 + 204);
            let f11_1 = lwc1(a0 + 212);
            let a2_2 = lw(a0 + 216);
            sw(a0 + 240, 1);
            sw(a0 + 96, v1_1);
            swc1(a0 + 100, f0_2);
            sw(a0 + 156, a0_1);
            sw(a0 + 220, -1);
            swc1(a0 + 116, f1_1);
            swc1(a0 + 120, f2_2);
            swc1(a0 + 124, f3_1);
            swc1(a0 + 104, f4_2);
            swc1(a0 + 108, f5_1);
            swc1(a0 + 112, f6_1);
            swc1(a0 + 128, f7_1);
            swc1(a0 + 132, f8_1);
            swc1(a0 + 136, f9_1);
            swc1(a0 + 140, f10_1);
            swc1(a0 + 148, f11_1);
            swc1(a0 + 224, 0.0_f32);
            sw(a0 + 160, 0);
            sw(a0 + 152, a2_2);
            swc1(a0 + 232, 0.0_f32);
            swc1(a0 + 228, 0.0_f32);
        }
    }
    let s7_1 = lw((lw(s0_1) << 2) + g::pAnimations);
    let f21_1 = lwc1(s7_1 + 16);
    let f0_3 = lwc1(g::updateframesf);
    'b3: {
        if !(0.0_f32 < f21_1) {
            sw(sp + 556, sp + 304);
        } else {
            let f1_2;
            'b5: {
                let f0_4 = ps2float::mul(f0_3, lwc1(s0_1 + 44));
                let f3_2 = lwc1(s0_1 + 4);
                sw(s0_1 + 56, 0);
                let v0_1 = lw(s7_1 + 12) & 1;
                let f3_3 = ps2float::add(f3_2, (f0_4).pdiv(6e+01_f32));
                swc1(s0_1 + 4, f3_3);
                if v0_1 == 0 {
                    if !(f21_1 < f3_3) {
                        sw(sp + 556, sp + 304);
                    } else {
                        swc1(s0_1 + 4, f21_1);
                        sw(s0_1 + 56, 1);
                        sw(sp + 556, sp + 304);
                    }
                } else {
                    if f21_1 < f3_3 {
                        sw(sp + 556, sp + 304);
                        loop {
                            let f0_5 = lwc1(s0_1 + 4);
                            sw(s0_1 + 56, 1);
                            let f0_6 = ps2float::sub(f0_5, f21_1);
                            swc1(s0_1 + 4, f0_6);
                            if !(f21_1 < f0_6) {
                                break;
                            }
                        }
                        f1_2 = lwc1(s0_1 + 4);
                        break 'b5;
                    }
                    sw(sp + 556, sp + 304);
                }
                f1_2 = lwc1(s0_1 + 4);
            }
            let v0_2;
            if !(f1_2 < 0.0_f32) {
                v0_2 = lw(s0_1 + 56);
            } else {
                let mut f1 = f1_2;
                let mut f0_7;
                loop {
                    f0_7 = ps2float::add(f1, f21_1);
                    f1 = f0_7;
                    if !(f0_7 < 0.0_f32) {
                        break;
                    }
                }
                swc1(s0_1 + 4, f0_7);
                sw(s0_1 + 56, 1);
                v0_2 = lw(s0_1 + 56);
            }
            if v0_2 == 0 {
                break 'b3;
            }
            if (lw(s7_1 + 12) & 1) == 0 {
                break 'b3;
            }
            extractAnimData(a0, s7_1, 0, lw(s7_1 + 4) - 1, sp + 240, f22_1);
            extractAnimData(a0, s7_1, 0, 0, sp + 272, f22_1);
            let f1_3 = ps2float::sub(ps2float::add(lwc1(sp + 276), lwc1(s0_1 + 24)), lwc1(sp + 244));
            let f2_3 = ps2float::sub(ps2float::add(lwc1(sp + 280), lwc1(s0_1 + 28)), lwc1(sp + 248));
            swc1(s0_1 + 20, ps2float::sub(ps2float::add(lwc1(sp + 272), lwc1(s0_1 + 20)), lwc1(sp + 240)));
            swc1(s0_1 + 24, f1_3);
            swc1(s0_1 + 28, f2_3);
        }
    }
    getKeyFrames(a0, s0_1, sp + 512, sp + 516);
    let mut f0_8 = lwc1(a0 + 224);
    if 0.0_f32 < f0_8 {
        let v0_3 = lw((lw(s2_1) << 2) + g::pAnimations);
        sw(sp + 528, v0_3);
        let f20_1 = lwc1(v0_3 + 16);
        let f0_9 = lwc1(g::updateframesf);
        if 0.0_f32 < f20_1 {
            let mut f1_4;
            'b14: {
                let f0_10 = ps2float::mul(f0_9, lwc1(s2_1 + 44));
                sw(s2_1 + 56, 0);
                let v0_4 = lw(lw(sp + 528) + 12) & 1;
                let f3_4 = ps2float::add(lwc1(s2_1 + 4), (f0_10).pdiv(6e+01_f32));
                swc1(s2_1 + 4, f3_4);
                if v0_4 == 0 {
                    if f20_1 < f3_4 {
                        swc1(s2_1 + 4, f21_1);
                        sw(s2_1 + 56, 1);
                    }
                } else if f20_1 < f3_4 {
                    loop {
                        let f0_11 = lwc1(s2_1 + 4);
                        sw(s2_1 + 56, 1);
                        let f0_12 = ps2float::sub(f0_11, f20_1);
                        swc1(s2_1 + 4, f0_12);
                        if !(f20_1 < f0_12) {
                            break;
                        }
                    }
                    f1_4 = lwc1(s2_1 + 4);
                    break 'b14;
                }
                f1_4 = lwc1(s2_1 + 4);
            }
            let v0_5;
            if !(f1_4 < 0.0_f32) {
                v0_5 = lw(s2_1 + 56);
            } else {
                let mut f0_13;
                loop {
                    f0_13 = ps2float::add(f1_4, f20_1);
                    f1_4 = f0_13;
                    if !(f0_13 < 0.0_f32) {
                        break;
                    }
                }
                swc1(s2_1 + 4, f0_13);
                sw(s2_1 + 56, 1);
                v0_5 = lw(s2_1 + 56);
            }
            let v1_2 = lw(sp + 528);
            if v0_5 != 0 {
                if (lw(v1_2 + 12) & 1) != 0 {
                    extractAnimData(a0, v1_2, 0, lw(v1_2 + 4) - 1, sp + 240, f22_1);
                    extractAnimData(a0, lw(sp + 528), 0, 0, sp + 272, f22_1);
                    let f4_3 = lwc1(sp + 272);
                    let f5_2 = lwc1(sp + 276);
                    let f3_5 = lwc1(sp + 280);
                    let f2_4 = ps2float::sub(ps2float::add(f5_2, lwc1(s2_1 + 24)), f5_2);
                    let f0_14 = ps2float::sub(ps2float::add(f3_5, lwc1(s2_1 + 28)), f3_5);
                    swc1(s2_1 + 20, ps2float::sub(ps2float::add(f4_3, lwc1(s2_1 + 20)), f4_3));
                    swc1(s2_1 + 24, f2_4);
                    swc1(s2_1 + 28, f0_14);
                }
            }
        }
        getKeyFrames(a0, s2_1, sp + 520, sp + 524);
        f0_8 = lwc1(a0 + 224);
    }
    let mut fp_1;
    'b21: {
        let a2_9 = lw(a0 + 244);
        let a0_8 = lw(g::lvnumplayers);
        if lw(a2_9 + 8) != 0x1000 {
            if lw(lw(a2_9 + 352)) < a0_8 {
                fp_1 = lw(s7_1 + 8);
                break 'b21;
            }
            fp_1 = 1;
            if (lw(a2_9 + 16) & 0xf000000) == 0 {
                break 'b21;
            }
        }
        fp_1 = lw(s7_1 + 8);
    }
    let mut a2_9 = lw(a0 + 244);
    'b23: {
        let mut a3_7 = lw(sp + 512);
        if !(f0_8 == 0.0_f32) {
            let v1_5 = lw(sp + 516);
            let f20_3;
            if a3_7 == v1_5 {
                f20_3 = 0.0_f32;
            } else {
                f20_3 = key_frame_fraction(s7_1, lwc1(s0_1 + 4), a3_7, v1_5);
            }
            let v0_8 = lw(sp + 520);
            let v1_6 = lw(sp + 524);
            let a1_11 = lw(sp + 528);
            let f21_2;
            if v0_8 == v1_6 {
                f21_2 = 0.0_f32;
            } else {
                f21_2 = key_frame_fraction(a1_11, lwc1(s2_1 + 4), v0_8, v1_6);
            }
            let mut s3_2 = 0;
            if fp_1 > 0 {
                sw(sp + 548, sp + 176);
                sw(sp + 552, sp + 208);
                sw(sp + 540, sp + 80);
                sw(sp + 532, sp + 48);
                sw(sp + 536, sp + 64);
                sw(sp + 544, sp + 96);
                loop {
                    let s0_4 = lw((s3_2 << 2) + lw(lw(a0 + 92)));
                    extractAnimData(a0, s7_1, s3_2, a3_7, sp + 112, f22_1);
                    let a3_9 = lw(sp + 516);
                    extractAnimData(a0, s7_1, s3_2, a3_9, sp + 144, f22_1);
                    let s0_5 = s0_4 << 6;
                    let a3_10 = lw(sp + 520);
                    extractAnimData(a0, lw(sp + 528), s3_2, a3_10, lw(sp + 548), f22_1);
                    let a3_11 = lw(sp + 524);
                    extractAnimData(a0, lw(sp + 528), s3_2, a3_11, lw(sp + 552), f22_1);
                    let f16_1 = lwc1(sp + 124);
                    let f15_1 = lwc1(sp + 128);
                    let f14_1 = lwc1(sp + 132);
                    let f12_12 = f20_3;
                    let f13_1 = lwc1(sp + 136);
                    let f0_16 = lwc1(sp + 156);
                    let f1_7 = lwc1(sp + 160);
                    let f2_9 = lwc1(sp + 164);
                    let f3_11 = lwc1(sp + 168);
                    let f4_7 = lwc1(sp + 188);
                    let f5_6 = lwc1(sp + 192);
                    let f6_3 = lwc1(sp + 196);
                    let f7_2 = lwc1(sp + 200);
                    let f8_2 = lwc1(sp + 220);
                    let f9_2 = lwc1(sp + 224);
                    let f10_2 = lwc1(sp + 228);
                    let f11_2 = lwc1(sp + 232);
                    let a2_17 = lw(sp + 540);
                    swc1(sp + 16, f16_1);
                    swc1(sp + 20, f15_1);
                    swc1(sp + 24, f14_1);
                    swc1(sp + 28, f13_1);
                    swc1(sp + 32, f0_16);
                    swc1(sp + 36, f1_7);
                    swc1(sp + 40, f2_9);
                    swc1(sp + 44, f3_11);
                    swc1(sp + 48, f4_7);
                    swc1(sp + 52, f5_6);
                    swc1(sp + 56, f6_3);
                    swc1(sp + 60, f7_2);
                    swc1(sp + 64, f8_2);
                    swc1(sp + 68, f9_2);
                    swc1(sp + 72, f10_2);
                    swc1(sp + 76, f11_2);
                    quaternionSlerp(&[
                        (sp + 16) as i64,
                        (sp + 32) as i64,
                        a2_17 as i64,
                        f12_12.to_bits() as i64,
                        f13_1.to_bits() as i64,
                        f14_1.to_bits() as i64,
                        f15_1.to_bits() as i64,
                        f16_1.to_bits() as i64,
                    ]);
                    let a0_21 = lw(sp + 532);
                    let a1_17 = lw(sp + 536);
                    let a2_18 = lw(sp + 544);
                    quaternionSlerp(&[a0_21 as i64, a1_17 as i64, a2_18 as i64, f21_2.to_bits() as i64]);
                    let f12_14 = lwc1(a0 + 228);
                    let a0_22 = lw(sp + 540);
                    let a1_18 = lw(sp + 544);
                    quaternionSlerp(&[a0_22 as i64, a1_18 as i64, sp as i64, f12_14.to_bits() as i64]);
                    quaternionToMatrix(lw(a0 + 4) + s0_5, sp);
                    let f3_12 = lwc1(sp + 112);
                    if s3_2 != 0 {
                        let f4_10 = lwc1(sp + 116);
                        let f5_8 = lwc1(sp + 120);
                        let f0_19 = ps2float::sub(lwc1(sp + 152), f5_8);
                        let f11_5 = ps2float::add(f4_10, ps2float::mul(ps2float::sub(lwc1(sp + 148), f4_10), f20_3));
                        swc1(s0_5 + lw(a0 + 4) + 48, ps2float::add(f3_12, ps2float::mul(ps2float::sub(lwc1(sp + 144), f3_12), f20_3)));
                        swc1(s0_5 + lw(a0 + 4) + 52, f11_5);
                        swc1(s0_5 + lw(a0 + 4) + 56, ps2float::add(f5_8, ps2float::mul(f0_19, f20_3)));
                    } else {
                        let f2_10 = lwc1(sp + 116);
                        if (lw(s7_1 + 12) & 2) == 0 {
                            let f3_14 = lwc1(sp + 116);
                            let f4_9 = lwc1(sp + 180);
                            let f1_9 = ps2float::sub(lwc1(sp + 212), f4_9);
                            let f0_18 = ps2float::mul(ps2float::sub(lwc1(sp + 148), f3_14), f20_3);
                            sw(s0_5 + lw(a0 + 4) + 48, 0);
                            let f11_4 = ps2float::add(f3_14, f0_18);
                            swc1(s0_5 + lw(a0 + 4) + 52, ps2float::add(f11_4, ps2float::mul(ps2float::sub(ps2float::add(f4_9, ps2float::mul(f1_9, f21_2)), f11_4), lwc1(a0 + 228))));
                            sw(s0_5 + lw(a0 + 4) + 56, 0);
                        } else {
                            let f9_3 = lwc1(sp + 180);
                            let f8_3 = lwc1(sp + 112);
                            let f6_4 = lwc1(sp + 176);
                            let f10_3 = lwc1(sp + 120);
                            let f11_3 = ps2float::add(f2_10, ps2float::mul(ps2float::sub(lwc1(sp + 148), f2_10), f20_3));
                            let f5_7 = ps2float::sub(lwc1(sp + 144), f8_3);
                            let f7_3 = lwc1(sp + 184);
                            let f0_17 = ps2float::add(f9_3, ps2float::mul(ps2float::sub(lwc1(sp + 212), f9_3), f21_2));
                            let f2_11 = lwc1(sp + 216);
                            let f4_8 = ps2float::sub(lwc1(sp + 208), f6_4);
                            let f1_8 = lwc1(a0 + 228);
                            let f3_13 = ps2float::sub(lwc1(sp + 152), f10_3);
                            let v0_9 = lw(a0 + 4);
                            swc1(a0 + 172, f0_17);
                            swc1(a0 + 112, ps2float::add(f10_3, ps2float::mul(f3_13, f20_3)));
                            swc1(a0 + 108, ps2float::add(f11_3, ps2float::mul(ps2float::sub(f0_17, f11_3), f1_8)));
                            swc1(a0 + 176, ps2float::add(f7_3, ps2float::mul(ps2float::sub(f2_11, f7_3), f21_2)));
                            swc1(a0 + 104, -ps2float::add(f8_3, ps2float::mul(f5_7, f20_3)));
                            swc1(a0 + 168, -ps2float::add(f6_4, ps2float::mul(f4_8, f21_2)));
                            sw(v0_9 + 48, 0);
                            swc1(lw(a0 + 4) + 52, lwc1(a0 + 108));
                            sw(lw(a0 + 4) + 56, 0);
                        }
                    }
                    s3_2 = s3_2 + 1;
                    a3_7 = lw(sp + 512);
                    if s3_2 >= fp_1 {
                        break;
                    }
                }
                a2_9 = lw(a0 + 244);
            }
        } else {
            let v1_3 = lw(sp + 516);
            let f20_2;
            if a3_7 == v1_3 {
                f20_2 = 0.0_f32;
            } else {
                f20_2 = key_frame_fraction(s7_1, lwc1(s0_1 + 4), a3_7, v1_3);
            }
            let mut s3_1 = 0;
            if fp_1 <= 0 {
                break 'b23;
            }
            loop {
                let s0_2 = lw((s3_1 << 2) + lw(lw(a0 + 92)));
                extractAnimData(a0, s7_1, s3_1, a3_7, sp + 112, f22_1);
                let a3_8 = lw(sp + 516);
                extractAnimData(a0, s7_1, s3_1, a3_8, sp + 144, f22_1);
                let s0_3 = s0_2 << 6;
                let f6_2 = lwc1(sp + 128);
                let f5_3 = lwc1(sp + 132);
                let f4_4 = lwc1(sp + 136);
                let f0_15 = lwc1(sp + 156);
                let f1_5 = lwc1(sp + 160);
                let f2_6 = lwc1(sp + 164);
                let f3_7 = lwc1(sp + 168);
                swc1(sp + 16, lwc1(sp + 124));
                swc1(sp + 20, f6_2);
                swc1(sp + 24, f5_3);
                swc1(sp + 28, f4_4);
                swc1(sp + 32, f0_15);
                swc1(sp + 36, f1_5);
                swc1(sp + 40, f2_6);
                swc1(sp + 44, f3_7);
                quaternionSlerp(&[(sp + 16) as i64, (sp + 32) as i64, sp as i64, f20_2.to_bits() as i64]);
                quaternionToMatrix(lw(a0 + 4) + s0_3, sp);
                if s3_1 != 0 {
                    let v0_7 = lw(a0 + 4);
                    swc1(s0_3 + v0_7 + 48, lwc1(sp + 112));
                    swc1(s0_3 + lw(a0 + 4) + 52, lwc1(sp + 116));
                    swc1(s0_3 + lw(a0 + 4) + 56, lwc1(sp + 120));
                } else {
                    let f3_8 = lwc1(sp + 112);
                    if (lw(s7_1 + 12) & 2) == 0 {
                        sw(lw(a0 + 4) + 48, 0);
                        let f1_6 = lwc1(sp + 116);
                        swc1(lw(a0 + 4) + 52, ps2float::add(f1_6, ps2float::mul(ps2float::sub(lwc1(sp + 148), f1_6), f20_2)));
                    } else {
                        let f4_5 = lwc1(sp + 116);
                        let f5_4 = lwc1(sp + 120);
                        let v0_6 = lw(a0 + 4);
                        let f4_6 = ps2float::add(f4_5, ps2float::mul(ps2float::sub(lwc1(sp + 148), f4_5), f20_2));
                        let f5_5 = ps2float::add(f5_4, ps2float::mul(ps2float::sub(lwc1(sp + 152), f5_4), f20_2));
                        swc1(a0 + 104, ps2float::add(f3_8, ps2float::mul(ps2float::sub(lwc1(sp + 144), f3_8), f20_2)));
                        swc1(a0 + 108, f4_6);
                        swc1(a0 + 112, f5_5);
                        sw(v0_6 + 48, 0);
                        swc1(lw(a0 + 4) + 52, lwc1(a0 + 108));
                    }
                    sw(lw(a0 + 4) + 56, 0);
                }
                s3_1 = s3_1 + 1;
                a3_7 = lw(sp + 512);
                if s3_1 >= fp_1 {
                    break;
                }
            }
            a2_9 = lw(a0 + 244);
        }
    }
    let v1_4 = lw(a2_9 + 8);
    let mut f21_3 = 0.5_f32;
    let v0_10;
    if v1_4 != 0x1000 {
        let v0_11 = lw(a2_9 + 352);
        v0_10 = lw(a0 + 156);
        if lw(v0_11 + 8) == 1 {
            f21_3 = lwc1(0x3ac824);
        }
    } else {
        v0_10 = lw(a0 + 156);
    }
    if v0_10 == 0 {
        turn_with_root(a0, 100, 148, f21_3, true);
    }
    if lw(a0 + 220) == 0 {
        turn_with_root(a0, 164, 212, f21_3, true);
    }
    if lw(a0 + 156) == 1 {
        turn_with_root(a0, 100, 148, f21_3, false);
    }
    if lw(a0 + 220) == 1 {
        turn_with_root(a0, 164, 212, f21_3, false);
    }
    let f1_16 = lwc1(a2_9 + 200);
    let a1_22;
    if !(0.0_f32 < f1_16) {
        a1_22 = lw(a0 + 4);
    } else {
        animHitReactUpdate(a2_9);
        let mut a2_20 = lw(a0 + 244);
        let v0_13 = lw(a2_20 + 336);
        if v0_13 >= 0 {
            let a0_25 = lw(a0 + 4) + (v0_13 << 6);
            sceVu0MulMatrix(a0_25, a0_25, a2_20 + 208);
            a2_20 = lw(a0 + 244);
        }
        let v0_14 = lw(a2_20 + 340);
        if v0_14 >= 0 {
            let a0_26 = lw(a0 + 4) + (v0_14 << 6);
            sceVu0MulMatrix(a0_26, a0_26, a2_20 + 272);
        }
        a1_22 = lw(a0 + 4);
    }
    calGlobalMatrices(a0, a1_22, 0, -1);
    let mut s4_1 = 0;
    calDoubleFillets(a0, 0, 0.5_f32);
    let a2_24 = lw(a0 + 244);
    let a3_13 = lw(lw(a0 + 92) + 12);
    let f21_4;
    let f26_1;
    if lw(a2_24 + 8) != 0x1000 {
        s4_1 = lw(a2_24 + 352);
        f21_4 = lwc1(s4_1 + 2968);
        f26_1 = lwc1(s4_1 + 2972);
    } else {
        let v0_15 = lw(a2_24 + 352);
        f26_1 = lwc1(v0_15 + 8);
        f21_4 = lwc1(v0_15 + 4);
    }
    let a2_27;
    'b48: {
        'b49: {
            let v1_9 = lw(a2_24 + 12);
            let v1_11;
            if v1_9 < 201 {
                v1_11 = lw(a0 + 4);
            } else {
                if v1_9 < lw(g::lvnumplayers) + 201 {
                    let f23_1 = lwc1(0x3ac828);
                    let v1_10 = lw(a0);
                    let mut s3_3 = 0;
                    let f22_2 = 1.8e+02_f32;
                    let s5_1 = lw(v1_10);
                    let f13_6 = lwc1(a2_24 + 80);
                    let f12_20 = lwc1(a2_24 + 76);
                    let f21_5 = -(ps2float::mul(f21_4, f23_1).pdiv(f22_2));
                    let f0_26 = anglediff(f12_20, f13_6);
                    let f13_7 = lwc1(lw(a0 + 244) + 80);
                    let f0_27 = anglediff(f26_1, f13_7);
                    let a0_29 = lw(sp + 556);
                    let f13_8 = ps2float::sub(ps2float::mul(f0_26, f23_1).pdiv(f22_2), ps2float::mul(f0_27, f23_1).pdiv(f22_2));
                    matrixRotZXY(a0_29, f21_5, f13_8, 0.0_f32);
                    if s5_1 <= 0 {
                        break 'b49;
                    }
                    let f20_8 = lwc1(0x3ac82c);
                    let mut v0_16 = lw(a0 + 92);
                    loop {
                        'b53: {
                            if ((lw((s3_3 << 2) + lw(v0_16 + 32)) - 7) as u32) < (4 as u32) {
                                let a0_30 = lb(s3_3 * 80 + (v1_10 - s5_1 * 80) + 1);
                                if a0_30 < 0 {
                                    s3_3 = s3_3 + 1;
                                    break 'b53;
                                }
                                if a0_30 >= lw(lw(a0) + 4) {
                                    s3_3 = s3_3 + 1;
                                    break 'b53;
                                }
                                let s0_6 = a0_30 << 6;
                                let v0_17 = s0_6 + lw(a0 + 4);
                                swc1(v0_17 + 52, ps2float::sub(lwc1(v0_17 + 52), f20_8));
                                if s4_1 != 0 {
                                    if lh(s4_1 + 4) == 24 {
                                        let v0_18 = s0_6 + lw(a0 + 4);
                                        swc1(v0_18 + 52, ps2float::sub(lwc1(v0_18 + 52), lwc1(g::heightoffset_rf_114)));
                                    }
                                }
                                let a2_25 = lw(a0 + 4);
                                let a1_24 = lw(sp + 556);
                                let a2_26 = a2_25 + s0_6;
                                sceVu0MulMatrix(sp + 368, a1_24, a2_26);
                                let f1_17 = ps2float::add(lwc1(sp + 420), f20_8);
                                swc1(sp + 420, f1_17);
                                if s4_1 != 0 {
                                    if lh(s4_1 + 4) == 24 {
                                        swc1(sp + 420, ps2float::add(f1_17, lwc1(g::heightoffset_rf_114)));
                                    }
                                }
                                let a0_32 = lw(a0 + 4);
                                sceVu0CopyMatrix(a0_32 + s0_6, sp + 368);
                            }
                            s3_3 = s3_3 + 1;
                        }
                        if s3_3 >= s5_1 {
                            break;
                        }
                        v0_16 = lw(a0 + 92);
                    }
                    a2_27 = lw(g::allowedcheats);
                    break 'b48;
                }
                v1_11 = lw(a0 + 4);
            }
            let s2_2 = a3_13 << 6;
            let v1_12 = s2_2 + v1_11;
            let f23_2 = lwc1(0x3ac830);
            let f24_1 = 1.8e+02_f32;
            swc1(sp + 496, lwc1(v1_12 + 48));
            let f22_3 = ps2float::mul(f21_4, f23_2).pdiv(f24_1);
            swc1(sp + 500, lwc1(v1_12 + 52));
            swc1(sp + 504, lwc1(v1_12 + 56));
            swc1(v1_12 + 48, 0.0_f32);
            swc1(s2_2 + lw(a0 + 4) + 52, 0.0_f32);
            swc1(s2_2 + lw(a0 + 4) + 56, 0.0_f32);
            let v0_19 = lw(a0 + 244);
            let f13_9 = lwc1(v0_19 + 80);
            let f12_23 = lwc1(v0_19 + 76);
            let f0_28 = anglediff(f12_23, f13_9);
            let f13_10 = lwc1(lw(a0 + 244) + 80);
            let f0_29 = anglediff(f26_1, f13_10);
            let a0_34 = lw(sp + 556);
            let f12_25 = ps2float::mul(f22_3, -0.5_f32);
            let f13_11 = ps2float::mul(ps2float::sub(ps2float::mul(f0_28, f23_2).pdiv(f24_1), ps2float::mul(f0_29, f23_2).pdiv(f24_1)), 0.5_f32);
            matrixRotZXY(a0_34, f12_25, f13_11, 0.0_f32);
            let a1_26 = lw(sp + 556);
            let a2_28 = lw(a0 + 4) + s2_2;
            sceVu0MulMatrix(sp + 368, a1_26, a2_28);
            let f0_30 = lwc1(sp + 500);
            let a0_36 = lw(a0 + 4) + s2_2;
            let f2_14 = lwc1(sp + 504);
            swc1(sp + 416, lwc1(sp + 496));
            swc1(sp + 420, f0_30);
            swc1(sp + 424, f2_14);
            sceVu0CopyMatrix(a0_36, sp + 368);
            let v0_20 = lw(a0 + 244);
            let f13_12 = lwc1(v0_20 + 80);
            let f12_26 = lwc1(v0_20 + 76);
            let f0_31 = anglediff(f12_26, f13_12);
            let f13_13 = lwc1(lw(a0 + 244) + 80);
            let f0_32 = anglediff(f26_1, f13_13);
            let a0_37 = lw(sp + 556);
            let f12_28 = -f22_3;
            let f13_14 = ps2float::sub(ps2float::mul(f0_31, f23_2).pdiv(f24_1), ps2float::mul(f0_32, f23_2).pdiv(f24_1));
            matrixRotZXY(a0_37, f12_28, f13_14, 0.0_f32);
            let v1_13 = lw(a0 + 92);
            let mut s3_4 = lw(v1_13 + 16);
            if lw(v1_13 + 20) < s3_4 {
                break 'b49;
            }
            let mut v1_14 = lw(a0 + 4);
            loop {
                let s0_7 = s3_4 << 6;
                let a1_28 = lw(sp + 556);
                let v0_21 = s2_2 + v1_14;
                s3_4 = s3_4 + 1;
                let v1_15 = s0_7 + v1_14;
                swc1(v1_15 + 48, ps2float::sub(lwc1(v1_15 + 48), lwc1(v0_21 + 48)));
                let v0_22 = lw(a0 + 4);
                let v0_23 = s0_7 + v0_22;
                swc1(v0_23 + 52, ps2float::sub(lwc1(v0_23 + 52), lwc1(s2_2 + v0_22 + 52)));
                let v1_16 = lw(a0 + 4);
                let v1_17 = s0_7 + v1_16;
                swc1(v1_17 + 56, ps2float::sub(lwc1(v1_17 + 56), lwc1(s2_2 + v1_16 + 56)));
                let a2_29 = lw(a0 + 4) + s0_7;
                sceVu0MulMatrix(sp + 368, a1_28, a2_29);
                let a0_39 = lw(a0 + 4);
                let v0_24 = s2_2 + a0_39;
                let f2_15 = lwc1(sp + 420);
                let a0_40 = a0_39 + s0_7;
                let f3_15 = lwc1(sp + 424);
                swc1(sp + 416, ps2float::add(lwc1(sp + 416), lwc1(v0_24 + 48)));
                swc1(sp + 420, ps2float::add(f2_15, lwc1(v0_24 + 52)));
                swc1(sp + 424, ps2float::add(f3_15, lwc1(v0_24 + 56)));
                sceVu0CopyMatrix(a0_40, sp + 368);
                if lw(lw(a0 + 92) + 20) < s3_4 {
                    break 'b49;
                }
                v1_14 = lw(a0 + 4);
            }
        }
        a2_27 = lw(g::allowedcheats);
    }
    let mut a2_27 = a2_27;
    let mut a1_30 = lw(g::unlockedcheats);
    let mut a0_41 = lw(g::selectedcheats);
    if (a2_27 & 1 & (a1_30 & a0_41)) != 0 {
        matrixScaleSet(sp + 432, 2.0_f32, 2.0_f32, 2.0_f32);
        let a0_43 = lw(a0 + 4) + (lw(lw(a0 + 92) + 20) << 6);
        sceVu0MulMatrix(a0_43, a0_43, sp + 432);
        a2_27 = lw(g::allowedcheats);
        a1_30 = lw(g::unlockedcheats);
        a0_41 = lw(g::selectedcheats);
    }
    let mut v1_18 = a1_30 & a0_41;
    if (a2_27 & 2 & v1_18) != 0 {
        matrixScaleSet(sp + 432, 0.5_f32, 0.5_f32, 0.5_f32);
        let a0_45 = lw(a0 + 4) + (lw(lw(a0 + 92) + 20) << 6);
        sceVu0MulMatrix(a0_45, a0_45, sp + 432);
        a2_27 = lw(g::allowedcheats);
        v1_18 = lw(g::unlockedcheats) & lw(g::selectedcheats);
    }
    if (a2_27 & 4 & v1_18) != 0 {
        matrixScaleSet(sp + 432, 2.0_f32, 2.0_f32, 2.0_f32);
        let a0_47 = lw(a0 + 4) + (lw(lw(a0 + 92) + 24) << 6);
        sceVu0MulMatrix(a0_47, a0_47, sp + 432);
        let a0_48 = lw(a0 + 4) + (lw(lw(a0 + 92) + 28) << 6);
        sceVu0MulMatrix(a0_48, a0_48, sp + 432);
    }
    let v0_26;
    'b63: {
        let v0_25 = lw(g::allowedcheats);
        if (lw(lw(a0 + 244) + 16) & 0x10000000) == 0 {
            if (v0_25 & 0x2000 & (lw(g::unlockedcheats) & lw(g::selectedcheats))) == 0 {
                v0_26 = lw(s7_1 + 12);
                break 'b63;
            }
        }
        let f12_32 = lwc1(0x3ac834);
        matrixScaleSet(sp + 432, f12_32, f12_32, f12_32);
        let a0_50 = lw(a0 + 4) + (lw(lw(a0 + 92) + 20) << 6);
        sceVu0MulMatrix(a0_50, a0_50, sp + 432);
        v0_26 = lw(s7_1 + 12);
    }
    if (v0_26 & 2) == 0 {
        sw(a0 + 200, 0);
        sw(a0 + 128, 0);
        sw(a0 + 132, 0);
        sw(a0 + 136, 0);
        sw(a0 + 192, 0);
        sw(a0 + 196, 0);
    } else {
        let f8_4 = lwc1(a0 + 104);
        let f14_9 = ps2float::sub(f8_4, lwc1(a0 + 116));
        let f7_4 = lwc1(a0 + 112);
        let f6_5 = lwc1(a0 + 168);
        let f12_33 = ps2float::sub(f7_4, lwc1(a0 + 124));
        let f9_4 = lwc1(a0 + 108);
        let f11_6 = ps2float::sub(f6_5, lwc1(a0 + 180));
        let f5_9 = lwc1(a0 + 172);
        let f4_11 = lwc1(a0 + 176);
        let f1_18 = ps2float::sub(f5_9, lwc1(a0 + 184));
        let f3_16 = lwc1(a0 + 224);
        let f10_4 = ps2float::sub(f4_11, lwc1(a0 + 188));
        swc1(a0 + 132, ps2float::sub(f9_4, lwc1(a0 + 120)));
        swc1(a0 + 116, f8_4);
        swc1(a0 + 120, f9_4);
        swc1(a0 + 124, f7_4);
        swc1(a0 + 196, f1_18);
        swc1(a0 + 180, f6_5);
        swc1(a0 + 184, f5_9);
        swc1(a0 + 188, f4_11);
        swc1(a0 + 128, f14_9);
        swc1(a0 + 136, f12_33);
        swc1(a0 + 192, f11_6);
        swc1(a0 + 200, f10_4);
        if 0.0_f32 < f3_16 {
            let f2_16 = lwc1(a0 + 228);
            swc1(a0 + 132, 0.0_f32);
            let f0_33 = ps2float::sub(1.0_f32, f2_16);
            swc1(a0 + 136, ps2float::add(ps2float::mul(f0_33, f12_33), ps2float::mul(f2_16, f10_4)));
            swc1(a0 + 128, ps2float::add(ps2float::mul(f0_33, f14_9), ps2float::mul(f2_16, f11_6)));
        }
    }
}
