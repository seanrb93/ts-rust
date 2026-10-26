//! Functions: the tick of a car prop (the cars and the animated machines of the levels) and the tick of the camera of the current player.

use crate::*;

/// Move one moving part of a car model: the matrix at `dest` is copied from `source`, then moved by `pivot`, turned about the X axis by `angle_x` (and, with `angle_y`, about the
/// Y axis first), and moved back (`sp` is the scratch matrix).
fn car_part(sp: i32, dest: i32, source: i32, pivot: i32, rotation: i32, step: f32, angle_y: Option<f32>) {
    swc1(rotation, ps2float::add(lwc1(rotation), step));
    sceVu0CopyMatrix(dest, source);
    matrixTrans(sp, lwc1(pivot), lwc1(pivot + 4), lwc1(pivot + 8));
    sceVu0MulMatrix(dest, dest, sp);
    if let Some(angle) = angle_y {
        matrixRotY(sp, angle);
        sceVu0MulMatrix(dest, dest, sp);
    }
    matrixRotX(sp, lwc1(rotation));
    sceVu0MulMatrix(dest, dest, sp);
    matrixTrans(sp, -lwc1(pivot), -lwc1(pivot + 4), -lwc1(pivot + 8));
    sceVu0MulMatrix(dest, dest, sp);
}

/// The tick of the car prop `a0` (@ 0x276678): it moves along its path (by the speed in the data at +68, relative or absolute) and the matrices of its model are made by the
/// kind of the prop (310 the arm of a machine, 311 a turning wheel, 312 and 313 cars with lights); then the nearest pad on its path is kept in +100.
pub fn carPropTick(a0: i32) {
    let frame = Frame::new(528);
    let sp = frame.sp();
    let data = lw(a0 + 352);
    let mut speed = lwc1(data + 68);
    sw(data + 84, lw(data + 84) + lw(g::updateframesi));
    if speed == 0.0_f32 {
        speed = 1.0_f32;
    }
    let absolute = lw(data);
    let step = ps2float::mul(speed, lwc1(g::updateframesf));
    swc1(a0 + 52, ps2float::sub(lwc1(a0 + 52), lwc1(data + 80)));
    let height;
    if absolute != 0 {
        pathPosUpdateRel(a0, ps2float::mul(ps2float::mul(step, lwc1(0x3ace0c)), lwc1(0x3ace10)));
        height = lwc1(a0 + 52);
    } else {
        let value = dpmul(fptodp(step), ld(0x3a8150));
        let value = dpmul(value, ld(0x3a8158));
        pathPosUpdateAbs(a0, dptofp(value));
        height = lwc1(a0 + 52);
    }
    let kind = lw(a0 + 4);
    swc1(a0 + 52, ps2float::add(height, lwc1(data + 80)));
    let spin = ps2float::mul(lwc1(data + 96), lwc1(0x3ace14));
    swc1(data + 96, spin);
    let model_data = || lw(a0 + 32);
    let matrices = || lw(model_data() + 4);
    'b4: {
        if kind == 311 {
            let angle = lwc1(data + 60);
            sw(sp + 288, a0 + 48);
            let to_radians = lwc1(0x3ace18);
            let x = ps2float::mul(lwc1(data + 52), to_radians).pdiv(1.8e+02_f32);
            let z = ps2float::mul(ps2float::add(angle, spin), to_radians).pdiv(1.8e+02_f32);
            let y = ps2float::mul(lwc1(data + 56), to_radians).pdiv(1.8e+02_f32);
            matrixRotZXY(matrices(), x, y, z);
        } else if kind >= 312 {
            if kind == 312 {
                let frames = lw(g::gameframe) as f32;
                let scale = lwc1(0x3ace34);
                let phase = ps2float::add(ps2float::mul(ps2float::mul(frames, lwc1(0x3ace30)), scale), 0.5_f32);
                swc1(data + 104, ps2float::add(lwc1(data + 104), ps2float::mul(lwc1(g::updateframesf), 31.0_f32)));
                let mut flags = 0;
                if 0.5_f32 < lwc1(((phase as i32 & 2047) << 2) + g::g_SinTable) {
                    flags = 2;
                }
                let mut second = 0;
                if 0.5_f32 < lwc1(((ps2float::add(ps2float::mul(ps2float::mul(frames, lwc1(0x3ace38)), scale), 0.5_f32) as i32 & 2047) << 2) + g::g_SinTable) {
                    second = 4;
                }
                let mut third = 0;
                if 0.5_f32 < lwc1(((ps2float::add(ps2float::mul(ps2float::mul(frames, lwc1(0x3ace3c)), scale), 0.5_f32) as i32 & 2047) << 2) + g::g_SinTable) {
                    third = 8;
                }
                let model = lw(a0 + 32);
                let header = lw(model);
                sh(model + 292, flags + second + third);
                let mut n = 0;
                sw(sp + 288, a0 + 48);
                let sound;
                if lw(header + 4) > 0 {
                    let mut model = model;
                    loop {
                        let f0;
                        'b12: {
                            if n != 6 {
                                let table = lw(model + 4);
                                if n != 7 {
                                    sceVu0UnitMatrix(table + (n << 6));
                                    f0 = lwc1(data + 96);
                                    break 'b12;
                                }
                                let skeleton_header = lw(model);
                                let pivot = lw(skeleton_header - lw(skeleton_header) * 80 + 72);
                                let at = pivot + 12;
                                matrixTrans(table + 448, lwc1(pivot + 12), lwc1(at + 4), lwc1(at + 8));
                                matrixRotZXY(sp, ps2float::mul(lwc1(data + 104), lwc1(0x3ace44)).pdiv(1.8e+02_f32), 0.0_f32, 0.0_f32);
                                let part = lw(lw(a0 + 32) + 4) + 448;
                                sceVu0MulMatrix(part, part, sp);
                                matrixTrans(sp, -lwc1(pivot + 12), -lwc1(at + 4), -lwc1(at + 8));
                                let part = lw(lw(a0 + 32) + 4) + 448;
                                sceVu0MulMatrix(part, part, sp);
                            } else {
                                let skeleton_header = lw(model);
                                let part = lw(model + 4) + 384;
                                let pivot = lw(skeleton_header - lw(skeleton_header) * 80 + 72);
                                matrixTrans(part, lwc1(pivot), lwc1(pivot + 4), lwc1(pivot + 8));
                                matrixRotZXY(sp, 0.0_f32, ps2float::mul(lwc1(data + 104), lwc1(0x3ace40)).pdiv(1.8e+02_f32), 0.0_f32);
                                let part = lw(lw(a0 + 32) + 4) + 384;
                                sceVu0MulMatrix(part, part, sp);
                                matrixTrans(sp, -lwc1(pivot), -lwc1(pivot + 4), -lwc1(pivot + 8));
                                let part = lw(lw(a0 + 32) + 4) + 384;
                                sceVu0MulMatrix(part, part, sp);
                            }
                            f0 = lwc1(data + 96);
                        }
                        let offset = if n == 6 { 384 } else if n == 7 { 448 } else { n << 6 };
                        let to_radians = lwc1(0x3ace48);
                        let x = ps2float::mul(lwc1(data + 52), to_radians).pdiv(1.8e+02_f32);
                        let z = ps2float::mul(ps2float::add(lwc1(data + 60), f0), to_radians).pdiv(1.8e+02_f32);
                        let y = ps2float::mul(lwc1(data + 56), to_radians).pdiv(1.8e+02_f32);
                        n += 1;
                        matrixRotZXY(sp, x, y, z);
                        sceVu0CopyMatrix(sp + 224, lw(lw(a0 + 32) + 4) + offset);
                        sceVu0MulMatrix(lw(lw(a0 + 32) + 4) + offset, sp, sp + 224);
                        model = lw(a0 + 32);
                        if n >= lw(lw(model) + 4) {
                            break;
                        }
                    }
                    sound = lw(data + 108);
                } else {
                    sound = lw(data + 108);
                }
                soundSetLoopPos(sound, lw(sp + 288));
                break 'b4;
            }
            if kind == 313 {
                let frames = lw(g::gameframe) as f32;
                let scale = lwc1(0x3ace20);
                let mut flags = 0;
                if 0.5_f32 < lwc1(((ps2float::add(ps2float::mul(ps2float::mul(frames, lwc1(0x3ace1c)), scale), 0.5_f32) as i32 & 2047) << 2) + g::g_SinTable) {
                    flags = 2;
                }
                let mut second = 0;
                if 0.5_f32 < lwc1(((ps2float::add(ps2float::mul(ps2float::mul(frames, lwc1(0x3ace24)), scale), 0.5_f32) as i32 & 2047) << 2) + g::g_SinTable) {
                    second = 4;
                }
                let mut third = 0;
                if 0.5_f32 < lwc1(((ps2float::add(ps2float::mul(ps2float::mul(frames, lwc1(0x3ace28)), scale), 0.5_f32) as i32 & 2047) << 2) + g::g_SinTable) {
                    third = 8;
                }
                let model = lw(a0 + 32);
                let header = lw(model);
                sh(model + 292, flags + second + third);
                let mut n = 0;
                if lw(header + 4) > 0 {
                    sw(sp + 288, a0 + 48);
                    let mut table = lw(model + 4);
                    loop {
                        let offset = n << 6;
                        n += 1;
                        sceVu0UnitMatrix(table + offset);
                        let to_radians = lwc1(0x3ace2c);
                        let x = ps2float::mul(lwc1(data + 52), to_radians).pdiv(1.8e+02_f32);
                        let z = ps2float::mul(ps2float::add(lwc1(data + 60), lwc1(data + 96)), to_radians).pdiv(1.8e+02_f32);
                        let y = ps2float::mul(lwc1(data + 56), to_radians).pdiv(1.8e+02_f32);
                        matrixRotZXY(sp, x, y, z);
                        sceVu0CopyMatrix(sp + 160, lw(lw(a0 + 32) + 4) + offset);
                        sceVu0MulMatrix(lw(lw(a0 + 32) + 4) + offset, sp, sp + 160);
                        let model = lw(a0 + 32);
                        if n >= lw(lw(model) + 4) {
                            break;
                        }
                        table = lw(model + 4);
                    }
                    break 'b4;
                }
            }
            sw(sp + 288, a0 + 48);
        } else {
            if kind == 310 {
                let arm = lw(data + 16);
                let target = lw(data + 20);
                let arm2 = arm + 12;
                let model = lw(a0 + 32);
                let target2 = target + 12;
                swc1(target, lwc1(arm));
                let header = lw(model);
                let count = lw(header);
                swc1(target + 4, lwc1(arm + 4));
                swc1(target + 8, lwc1(arm + 8));
                let skeleton = header - count * 80;
                let pivot_a = lw(skeleton + 152);
                swc1(target + 12, lwc1(arm + 12));
                let pivot_b = lw(skeleton + 552);
                swc1(target2 + 4, lwc1(arm2 + 4));
                swc1(target2 + 8, lwc1(arm2 + 8));
                let y = lwc1(a0 + 52);
                let dx = ps2float::sub(lwc1(a0 + 48), lwc1(arm));
                let z = lwc1(a0 + 56);
                swc1(sp + 144, dx);
                let dy = ps2float::sub(y, lwc1(arm + 4));
                swc1(sp + 148, dy);
                let dz = ps2float::sub(z, lwc1(arm + 8));
                swc1(sp + 152, dz);
                let ay = lwc1(arm + 4);
                let az = lwc1(arm + 8);
                let length = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt();
                let k = (ps2float::add(length, lwc1(pivot_a + 8))).pdiv(length);
                swc1(arm, ps2float::add(lwc1(arm), ps2float::mul(k, lwc1(sp + 144))));
                let k = (ps2float::add(length, lwc1(pivot_a + 8))).pdiv(length);
                swc1(arm + 4, ps2float::add(ay, ps2float::mul(k, lwc1(sp + 148))));
                let k = (ps2float::add(length, lwc1(pivot_a + 8))).pdiv(length);
                swc1(arm + 8, ps2float::add(az, ps2float::mul(k, lwc1(sp + 152))));
                sw(a0 + 76, 0);
                let bearing_a = atan2f(lwc1(sp + 144), lwc1(sp + 152));
                let dx = ps2float::sub(lwc1(arm), lwc1(arm + 12));
                swc1(sp + 144, dx);
                let dy = ps2float::sub(lwc1(arm + 4), lwc1(arm2 + 4));
                swc1(sp + 148, dy);
                let dz = ps2float::sub(lwc1(arm + 8), lwc1(arm2 + 8));
                swc1(sp + 152, dz);
                let reach = lwc1(pivot_b + 8);
                let sx = lwc1(sp + 144);
                let reach = ps2float::sub(reach, lwc1(pivot_a + 8));
                sw(sp + 288, a0 + 48);
                let full = lwc1(0x3ace4c);
                let length = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt();
                swc1(arm + 12, ps2float::add(lwc1(arm + 12), ps2float::mul((ps2float::add(length, reach)).pdiv(length), sx)));
                let az2 = lwc1(arm2 + 8);
                let length = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt();
                swc1(arm2 + 4, ps2float::add(lwc1(arm2 + 4), ps2float::mul((ps2float::add(length, ps2float::sub(lwc1(pivot_b + 8), lwc1(pivot_a + 8)))).pdiv(length), lwc1(sp + 148))));
                let length = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt();
                swc1(arm2 + 8, ps2float::add(az2, ps2float::mul((ps2float::add(length, ps2float::sub(lwc1(pivot_b + 8), lwc1(pivot_a + 8)))).pdiv(length), lwc1(sp + 152))));
                let bearing_b = atan2f(lwc1(sp + 144), lwc1(sp + 152));
                matrixTrans(matrices(), 0.0_f32, 0.0_f32, 0.0_f32);
                matrixRotY(matrices() + 64, bearing_a);
                let speed_step = ps2float::mul(step, lwc1(0x3ace50));
                let degrees = ps2float::mul(bearing_a, 1.8e+02_f32).pdiv(full);
                let heading = lwc1(data + 56);
                let base = matrices();
                let pivot = lw(skeleton + 232);
                swc1(g::rot_123, ps2float::add(lwc1(g::rot_123), speed_step));
                sceVu0CopyMatrix(base + 128, base + 64);
                matrixTrans(sp, lwc1(pivot), lwc1(pivot + 4), lwc1(pivot + 8));
                sceVu0MulMatrix(matrices() + 128, matrices() + 128, sp);
                matrixRotY(sp, ps2float::mul(ps2float::sub(heading, degrees), full).pdiv(1.8e+02_f32));
                sceVu0MulMatrix(matrices() + 128, matrices() + 128, sp);
                matrixRotX(sp, lwc1(g::rot_123));
                sceVu0MulMatrix(matrices() + 128, matrices() + 128, sp);
                matrixTrans(sp, -lwc1(pivot), -lwc1(pivot + 4), -lwc1(pivot + 8));
                sceVu0MulMatrix(matrices() + 128, matrices() + 128, sp);
                let base = matrices();
                let pivot = lw(skeleton + 312);
                let heading2 = ps2float::sub(lwc1(data + 56), degrees);
                car_part(sp, base + 192, base + 64, pivot, g::rot_124, speed_step, Some(ps2float::mul(heading2, full).pdiv(1.8e+02_f32)));
                let base = matrices();
                let pivot = lw(skeleton + 392);
                car_part(sp, base + 256, base + 64, pivot, g::rot_125, speed_step, None);
                let base = matrices();
                let pivot = lw(skeleton + 472);
                car_part(sp, base + 320, base + 64, pivot, g::rot_126, speed_step, None);
                matrixTrans(matrices() + 384, ps2float::sub(lwc1(arm), lwc1(a0 + 48)), ps2float::sub(lwc1(arm + 4), lwc1(a0 + 52)), ps2float::sub(lwc1(arm + 8), lwc1(a0 + 56)));
                matrixRotY(sp, bearing_b);
                sceVu0MulMatrix(matrices() + 384, matrices() + 384, sp);
                matrixTrans(sp, 0.0_f32, 0.0_f32, lwc1(0x3ace54));
                sceVu0MulMatrix(matrices() + 384, matrices() + 384, sp);
                let base = matrices();
                let pivot = lw(skeleton + 632);
                car_part(sp, base + 448, base + 384, pivot, g::rot_127, speed_step, None);
                let base = matrices();
                let pivot = lw(skeleton + 712);
                car_part(sp, base + 512, base + 384, pivot, g::rot_128, speed_step, None);
                break 'b4;
            }
            sw(sp + 288, a0 + 48);
        }
    }
    let pad = lw(data + 100);
    let found = findpadslinked2pad(pad, sp + 68, 19);
    let position = lw(sp + 288);
    sw(sp + 64, lw(data + 100));
    let nearest = closestpadinlist(position, found + 1, sp + 64);
    sw(data + 100, nearest);
}

/// The tick of the camera of the current player (@ 0x27e320): the matrices of the player are set up, the perspective is made, and the position and direction of the camera are
/// found by the camera mode (0 to 9: the player's eyes, a fixed viewing prop, following a prop, flying around, ...); the view and its inverse are made.
pub fn camTick() {
    let frame = Frame::new(256);
    let sp = frame.sp();
    let player = lw(g::currentplayer);
    let source = 0x3a8770;
    let word = lw(source + 8);
    sd(sp + 16, ld(source));
    sw(sp + 24, word);
    memset(sp + 32, 0, 12);
    let current = lw(g::currentplayer);
    let flip = lw(g::mnum);
    let matrix_offset = flip << 6;
    let player_offset = lw(current) << 7;
    sw(current + 1772, player_offset + (matrix_offset + g::imvm));
    sw(current + 1760, player_offset + (matrix_offset + g::pm));
    sw(current + 1764, player_offset + (matrix_offset + g::mvm));
    sw(current + 1768, player_offset + (matrix_offset + g::mvpm));
    sw(g::mnum, 1 - flip);
    if debugmenuGetJoyMode() == 2 {
        viewTick();
    }
    'b2: {
        'b3: {
            'b4: {
                let now = lw(g::currentplayer);
                let window = g::window;
                let ratio = lwc1(0x3acfc8);
                let to_radians = lwc1(0x3acfcc);
                let full = 1.8e+02_f32;
                let near = lwc1(g::near_105);
                let aspect = ps2float::mul(lw(window + 12) as f32, ratio).pdiv(lw(window + 32) as f32);
                let fov = ps2float::mul(lwc1(now + 260), to_radians).pdiv(full);
                let far = lwc1(g::far_106);
                matrixPerspective(lw(now + 1760), aspect, fov, near, far);
                let half = tanf(ps2float::mul(ps2float::mul(lwc1(lw(g::currentplayer) + 260), 0.5_f32), to_radians).pdiv(full));
                let height = lw(window + 32) as f32;
                let now = lw(g::currentplayer);
                let width = lw(window + 12) as f32;
                swc1(now + 252, half);
                swc1(now + 256, ps2float::mul(ps2float::mul(half, ratio), ((width).pdiv(height))));
                let mode = camGetMode();
                if (mode as u32) < (10 as u32) {
                    let mut eye = 0;
                    let mut prop_now = 0;
                    'b5: {
                        'b6: {
                            let fall = 'match_end: {
                                match mode {
                                    0 => break 'b4,
                                    1 => {
                                        sw(g::cammode, 2);
                                        swc1(0x3b0130, 1.5_f32);
                                        swc1(0x3b0128, 1.5_f32);
                                        break 'b5;
                                    }
                                    2 => break 'b5,
                                    3 => break 'b3,
                                    4 => {
                                        let viewed = lw(player + 904);
                                        swc1(0x3b0128, 2.5_f32);
                                        sw(g::cammode, 5);
                                        sw(g::viewprop_mode, 0);
                                        swc1(0x3b0120, 0.0_f32);
                                        swc1(0x3b0124, 0.0_f32);
                                        swc1(0x3b0130, 0.0_f32);
                                        swc1(0x3b012c, 0.0_f32);
                                        if viewed != 0 {
                                            if lw(viewed + 8) == 8 {
                                                let zero = fptodp(0.0_f32);
                                                let size = fptodp(lwc1(player + 116));
                                                let half = dpmul(size, 0x3fe0000000000000_i64);
                                                let sum = dpadd(zero, half);
                                                swc1(0x3b0130, dptofp(sum));
                                            }
                                        }
                                    }
                                    5 => {}
                                    6 => {
                                        let target = lw(player + 904);
                                        let mut now = lw(g::currentplayer);
                                        if target == 0 {
                                            break 'b2;
                                        }
                                        let back = 2.5_f32;
                                        let dy = ps2float::mul(lwc1(player + 192), back);
                                        let room = lw(player + 896);
                                        let dz = ps2float::mul(lwc1(player + 196), back);
                                        swc1(player + 884, ps2float::sub(lwc1(target + 48), ps2float::mul(lwc1(player + 188), back)));
                                        swc1(player + 888, ps2float::sub(lwc1(target + 52), dy));
                                        swc1(player + 892, ps2float::sub(lwc1(target + 56), dz));
                                        if room != 0 {
                                            sw(player + 900, room);
                                        }
                                        let at = player + 884;
                                        if lw(target + 8) != 8 {
                                            let found = moveFindRoom(at, 1.0_f32, lwc1(player + 132));
                                            let looked = lw(player + 904);
                                            sw(player + 896, found);
                                            matrixLookAt(lw(player + 1764), at, looked + 48, sp + 16);
                                            now = lw(g::currentplayer);
                                            let _ = now;
                                            break 'b2;
                                        }
                                        let height = fptodp(lwc1(player + 888));
                                        let lift = dpmul(fptodp(lwc1(player + 116)), ld(0x3a87d8));
                                        swc1(player + 888, dptofp(dpadd(height, lift)));
                                        swc1(sp + 32, lwc1(target + 48));
                                        let height = fptodp(lwc1(target + 52));
                                        let lift = dpmul(fptodp(lwc1(player + 116)), 0x3fe0000000000000_i64);
                                        swc1(sp + 36, dptofp(dpadd(height, lift)));
                                        swc1(sp + 40, lwc1(lw(player + 904) + 56));
                                        let found = moveFindRoom(at, 1.0_f32, lwc1(player + 132));
                                        sw(player + 896, found);
                                        matrixLookAt(lw(player + 1764), at, sp + 32, sp + 16);
                                        let current = lw(g::currentplayer);
                                        sceVu0InversMatrix(lw(current + 1772), lw(current + 1764));
                                        break 'b2;
                                    }
                                    7 => {
                                        let now = lw(g::currentplayer);
                                        let old_room = lw(player + 896);
                                        let x = lwc1(player + 152);
                                        let y = lwc1(player + 156);
                                        let z = lwc1(player + 160);
                                        let view = lw(now + 1764);
                                        sw(player + 896, lw(player + 796));
                                        swc1(player + 884, x);
                                        swc1(player + 888, y);
                                        swc1(player + 892, z);
                                        sw(player + 900, old_room);
                                        matrixLookAt(view, now + 152, now + 176, now + 164);
                                        break 'b2;
                                    }
                                    8 => {
                                        let room = lw(player + 796);
                                        let view = lw(player + 1764);
                                        sw(player + 900, lw(player + 896));
                                        sw(player + 896, room);
                                        matrixLookAt(view, g::flexcam, g::flexcam + 12, player + 164);
                                        break 'b2;
                                    }
                                    _ => {
                                        // 9: flying around the first player's character
                                        let chr = lw(player + 384);
                                        let angle = lwc1(0x3b0120);
                                        if chr == 0 {
                                            break 'b3;
                                        }
                                        swc1(0x3b0128, 0.5_f32);
                                        swc1(0x3b0124, 3e+01_f32);
                                        let _ = angle;
                                        let angle = ps2float::add(angle, lwc1(0x3acfd4));
                                        swc1(0x3b0120, angle);
                                        let size = fptodp(lwc1(player + 116));
                                        let half = dpmul(size, 0x3fe0000000000000_i64);
                                        swc1(0x3b0130, dptofp(half));
                                        let mut angle = angle;
                                        let at = player + 884;
                                        let _ = at;
                                        if 3.6e+02_f32 < angle {
                                            loop {
                                                angle = dptofp(dpsub(fptodp(angle), ld(0x3a87c8)));
                                                swc1(0x3b0120, angle);
                                                if !(3.6e+02_f32 < angle) {
                                                    break;
                                                }
                                            }
                                        }
                                        if angle < 3.6e+02_f32 {
                                            loop {
                                                angle = dptofp(dpadd(fptodp(angle), ld(0x3a87d0)));
                                                swc1(0x3b0120, angle);
                                                if !(angle < 3.6e+02_f32) {
                                                    break;
                                                }
                                            }
                                        }
                                        let to_radians = lwc1(0x3acfd8);
                                        let full = 1.8e+02_f32;
                                        let cos_a = cosf(ps2float::mul(angle, to_radians).pdiv(full));
                                        let sin_a = sinf(ps2float::mul(lwc1(0x3b0120), to_radians).pdiv(full));
                                        let cos_b = cosf(ps2float::mul(lwc1(0x3b0124), to_radians).pdiv(full));
                                        let sin_b = sinf(ps2float::mul(lwc1(0x3b0124), to_radians).pdiv(full));
                                        prop_now = lw(player + 896);
                                        let dx = cos_b * sin_a;
                                        swc1(sp + 52, sin_b);
                                        let lift = lwc1(0x3b0130);
                                        swc1(sp + 56, cos_b * cos_a);
                                        swc1(sp + 48, dx);
                                        let distance = lwc1(0x3b0128);
                                        let x = lwc1(chr + 48);
                                        swc1(sp + 32, x);
                                        swc1(sp + 36, ps2float::add(lwc1(chr + 52), lift));
                                        swc1(sp + 40, lwc1(chr + 56));
                                        swc1(player + 884, ps2float::add(x, ps2float::mul(distance, dx)));
                                        swc1(player + 888, ps2float::add(lwc1(sp + 36), ps2float::mul(distance, lwc1(sp + 52))));
                                        swc1(player + 892, ps2float::add(lwc1(sp + 40), ps2float::mul(distance, lwc1(sp + 56))));
                                        eye = player + 884;
                                        break 'match_end true;
                                    }
                                }
                                false
                            };
                            if !fall {
                                // modes 4 and 5: the viewing prop is chosen and moved by the pad of the second controller
                                let mut viewed = 0;
                                if (joyNewButLOCAL(1) & 128) == 0 {
                                    viewed = lw(player + 904);
                                } else {
                                    let next = propGetNext(lw(player + 904), 8);
                                    sw(player + 904, next);
                                    viewed = lw(player + 904);
                                }
                                let mode_now = lw(g::viewprop_mode);
                                if (joyNewButLOCAL(1) & 32) != 0 {
                                    let next = mode_now + 1;
                                    sw(g::viewprop_mode, next);
                                    if next >= 3 {
                                        sw(g::viewprop_mode, 0);
                                    }
                                }
                                let angle_x;
                                'b15: {
                                    let slow = 2e+01_f32;
                                    let zoom = joyGetAxisLOCAL(1, 3);
                                    swc1(0x3b0128, ps2float::sub(lwc1(0x3b0128), (zoom).pdiv(slow)));
                                    eye = player + 884;
                                    if (joyButLOCAL(1) & 64) == 0 {
                                        let turn = joyGetAxisLOCAL(1, 0);
                                        let mut yaw = ps2float::add(lwc1(0x3b0120), ps2float::mul(turn, 1.8e+02_f32).pdiv(6e+01_f32));
                                        swc1(0x3b0120, yaw);
                                        if 3.6e+02_f32 < yaw {
                                            loop {
                                                yaw = dptofp(dpsub(fptodp(lwc1(0x3b0120)), ld(0x3a87a8)));
                                                swc1(0x3b0120, yaw);
                                                if !(3.6e+02_f32 < yaw) {
                                                    break;
                                                }
                                            }
                                        }
                                        if yaw < 3.6e+02_f32 {
                                            loop {
                                                yaw = dptofp(dpadd(fptodp(yaw), ld(0x3a87b0)));
                                                swc1(0x3b0120, yaw);
                                                if !(yaw < 3.6e+02_f32) {
                                                    break;
                                                }
                                            }
                                        }
                                        let pitch_turn = joyGetAxisLOCAL(1, 1);
                                        let mut pitch = ps2float::add(lwc1(0x3b0124), ps2float::mul(pitch_turn, 1.8e+02_f32).pdiv(6e+01_f32));
                                        swc1(0x3b0124, pitch);
                                        angle_x = lwc1(0x3b0120);
                                        if 3.6e+02_f32 < pitch {
                                            loop {
                                                pitch = dptofp(dpsub(fptodp(lwc1(0x3b0124)), ld(0x3a87b8)));
                                                swc1(0x3b0124, pitch);
                                                if !(3.6e+02_f32 < pitch) {
                                                    break;
                                                }
                                            }
                                        }
                                        if pitch < 3.6e+02_f32 {
                                            loop {
                                                pitch = dptofp(dpadd(fptodp(pitch), ld(0x3a87c0)));
                                                swc1(0x3b0124, pitch);
                                                if !(pitch < 3.6e+02_f32) {
                                                    break 'b15;
                                                }
                                            }
                                        }
                                    } else {
                                        let up = joyGetAxisLOCAL(1, 1);
                                        swc1(0x3b0130, ps2float::add(lwc1(0x3b0130), (up).pdiv(slow)));
                                        let side = joyGetAxisLOCAL(1, 0);
                                        angle_x = lwc1(0x3b0120);
                                        swc1(0x3b012c, ps2float::add(lwc1(0x3b012c), (side).pdiv(slow)));
                                    }
                                }
                                let to_radians = lwc1(0x3acfd0);
                                let full = 1.8e+02_f32;
                                let cos_a = cosf(ps2float::mul(angle_x, to_radians).pdiv(full));
                                let sin_a = sinf(ps2float::mul(lwc1(0x3b0120), to_radians).pdiv(full));
                                let cos_b = cosf(ps2float::mul(lwc1(0x3b0124), to_radians).pdiv(full));
                                let sin_b = sinf(ps2float::mul(lwc1(0x3b0124), to_radians).pdiv(full));
                                let z_dir = cos_b * cos_a;
                                let mode_view = lw(g::viewprop_mode);
                                let x_dir = cos_b * sin_a;
                                swc1(sp + 52, sin_b);
                                swc1(sp + 56, z_dir);
                                swc1(sp + 48, x_dir);
                                if mode_view >= 2 {
                                    let saved = 0x1fe9940;
                                    let z = lwc1(saved + 8);
                                    let y = lwc1(saved + 4);
                                    swc1(sp + 32, lwc1(0x1fe9940));
                                    swc1(sp + 36, y);
                                    swc1(sp + 40, z);
                                } else {
                                    let x = lwc1(viewed + 48);
                                    let saved = 0x1fe9940;
                                    let lift = lwc1(0x3b0130);
                                    swc1(sp + 32, x);
                                    let side = lwc1(0x3b012c);
                                    let y = ps2float::add(lwc1(viewed + 52), lift);
                                    let nx = ps2float::add(x, ps2float::mul(side, z_dir));
                                    swc1(sp + 36, y);
                                    let z = lwc1(viewed + 56);
                                    swc1(sp + 32, nx);
                                    let nz = ps2float::sub(z, ps2float::mul(side, x_dir));
                                    swc1(saved + 4, y);
                                    swc1(sp + 40, nz);
                                    swc1(saved + 8, nz);
                                    swc1(0x1fe9940, nx);
                                    if mode_view == 0 {
                                        let distance = lwc1(0x3b0128);
                                        let place = 0x1fe9930;
                                        swc1(player + 884, ps2float::add(nx, ps2float::mul(distance, x_dir)));
                                        swc1(player + 888, ps2float::add(lwc1(sp + 36), ps2float::mul(distance, lwc1(sp + 52))));
                                        swc1(player + 892, ps2float::add(lwc1(sp + 40), ps2float::mul(distance, lwc1(sp + 56))));
                                        let y = lwc1(sp + 36);
                                        let z = lwc1(sp + 40);
                                        swc1(0x1fe9930, lwc1(sp + 32));
                                        swc1(place + 8, z);
                                        swc1(place + 4, y);
                                    }
                                }
                                prop_now = lw(player + 896);
                            }
                        }
                        if prop_now != 0 {
                            sw(player + 900, prop_now);
                        }
                        let found = moveFindRoom(eye, 1.0_f32, lwc1(player + 132));
                        sw(player + 896, found);
                        matrixLookAt(lw(player + 1764), eye, sp + 32, sp + 16);
                        break 'b2;
                    }
                    // modes 1 and 2: behind the player's eyes, pushed in by the walls
                    let distance = lwc1(0x3b0128);
                    let room = lw(player + 896);
                    let back_x = -ps2float::mul(distance, lwc1(player + 188));
                    swc1(sp + 48, back_x);
                    swc1(sp + 52, -ps2float::mul(distance, lwc1(player + 192)));
                    swc1(sp + 56, -ps2float::mul(distance, lwc1(player + 196)));
                    let y = lwc1(player + 156);
                    let z = lwc1(player + 160);
                    swc1(player + 884, ps2float::add(lwc1(player + 152), back_x));
                    swc1(player + 888, ps2float::add(y, lwc1(sp + 52)));
                    swc1(player + 892, ps2float::add(z, lwc1(sp + 56)));
                    if room != 0 {
                        sw(player + 900, room);
                    }
                    let one = 1.0_f32;
                    let at = player + 884;
                    let height = lwc1(player + 132);
                    let found = moveFindRoom(at, one, height);
                    let chr = lw(player + 384);
                    sw(sp + 96, found);
                    let flags = lw(chr + 16);
                    sw(chr + 16, flags & -33);
                    sw(sp, 1);
                    sw(sp + 8, 0);
                    let hit = bgLineTest(found, player + 152, sp + 48, 0, sp + 64, sp + 80, sp + 96, 0);
                    let hit_x = lwc1(sp + 64);
                    if hit != 0 {
                        swc1(player + 884, hit_x);
                        swc1(player + 888, lwc1(sp + 68));
                        swc1(player + 892, lwc1(sp + 72));
                        let found = moveFindRoom(at, one, lwc1(player + 132));
                        sw(sp + 96, found);
                    }
                    let chr = lw(player + 384);
                    sw(player + 896, lw(sp + 96));
                    sw(chr + 16, flags);
                    matrixLookAt(lw(player + 1764), at, player + 176, player + 164);
                    break 'b2;
                }
                // mode 0 and a bad mode: the camera of the player's eyes
            }
            let now = lw(g::currentplayer);
            let old_room = lw(player + 896);
            let x = lwc1(player + 152);
            let y = lwc1(player + 156);
            let z = lwc1(player + 160);
            let view = lw(now + 1764);
            sw(player + 896, lw(player + 796));
            swc1(player + 884, x);
            swc1(player + 888, y);
            swc1(player + 892, z);
            sw(player + 900, old_room);
            matrixLookAt(view, now + 152, now + 176, now + 164);
        }
    }
    let now = lw(g::currentplayer);
    sceVu0MulMatrix(lw(now + 1768), lw(now + 1760), lw(now + 1764));
    let now = lw(g::currentplayer);
    sceVu0InversMatrix(lw(now + 1772), lw(now + 1764));
}
