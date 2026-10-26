//! Making the matrices of the animated props every tick.

use crate::*;

/// Make the matrices of every prop whose model is animated and has its own matrices: for the props of kind other than 8 that are not the stage prop, that have bones and are not
/// attached to another prop (+188), the model matrix is made by the kind of the prop's turn (+140: 0 none, 1 and 3 about y, 2 in all three axes; kind 4 is left out) and, for each
/// of the drawing flags of the prop (0x1000000, 0x2000000, 0x4000000 and 0x8000000) the list of matrices of that kind (the model's +8, +12, +16 and +20) is made as the view matrix
/// (the player's, when +136 is 1, else the model matrix alone) times each bone's. The stage prop and the props of kind 8 are turned about y alone and use the first player's
/// matrices (@ 0x273bb0).
pub fn animMtxTick() {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let mut count = lw(g::num_props);
    let mut i = 0;
    if count <= 0 {
        return;
    }
    loop {
        'prop: {
            let prop = lw(g::levelprops) + i * 592;
            let mut flags = lw(prop + 16);
            if flags < 0 {
                i += 1;
                break 'prop;
            }
            'step: {
                let model = lw(prop + 32);
                if lw(model + 4) != 0 {
                    'ordinary: {
                        if lw(prop + 8) != 8 {
                            if prop != lw(g::stageprop) {
                                let skeleton = lw(model);
                                if lw(skeleton + 4) <= 0 {
                                    i += 1;
                                    break 'prop;
                                }
                                if lw(prop + 188) != 0 {
                                    i += 1;
                                    break 'prop;
                                }
                                let turn = lw(prop + 140);
                                if turn == 4 {
                                    break 'step;
                                }
                                if (turn as u32) < 5 {
                                    match turn {
                                        0 => {
                                            sceVu0UnitMatrix(sp);
                                            flags = lw(prop + 16);
                                        }
                                        1 => {
                                            let x = lwc1(prop + 48);
                                            let y = lwc1(prop + 52);
                                            let angle = ps2float::mul(lwc1(prop + 76), lwc1(0x3acd90)).pdiv(1.8e+02_f32);
                                            let z = lwc1(prop + 56);
                                            matrixTransRotY(sp, x, y, z, angle);
                                            flags = lw(prop + 16);
                                        }
                                        2 => {
                                            let radians = lwc1(0x3acd94);
                                            let half_turn = 1.8e+02_f32;
                                            let x = lwc1(prop + 48);
                                            let a = ps2float::mul(lwc1(prop + 72), radians).pdiv(half_turn);
                                            let y = lwc1(prop + 52);
                                            let b = ps2float::mul(lwc1(prop + 76), radians).pdiv(half_turn);
                                            let c = ps2float::mul(lwc1(prop + 88), radians).pdiv(half_turn);
                                            let z = lwc1(prop + 56);
                                            matrixTransRotZXY(sp, x, y, z, a, b, c);
                                            flags = lw(prop + 16);
                                        }
                                        3 => {
                                            let x = lwc1(prop + 48);
                                            let y = lwc1(prop + 52);
                                            let angle = ps2float::mul(lwc1(prop + 76), lwc1(0x3acd98)).pdiv(1.8e+02_f32);
                                            let z = lwc1(prop + 56);
                                            matrixTransRotY(sp, x, y, z, angle);
                                            flags = lw(prop + 16);
                                        }
                                        _ => {
                                            let x = lwc1(prop + 48);
                                            let y = lwc1(prop + 52);
                                            let angle = ps2float::mul(lwc1(prop + 76), lwc1(0x3acd9c)).pdiv(1.8e+02_f32);
                                            let z = lwc1(prop + 56);
                                            matrixTransRotY(sp, x, y, z, angle);
                                            matrixroll(lw(lw(prop + 32) + 4), prop + 100, lwc1(prop + 92));
                                            sceVu0MulMatrix(sp, sp, lw(lw(prop + 32) + 4));
                                            flags = lw(prop + 16);
                                        }
                                    }
                                }
                                // first list (+8)
                                'first: {
                                    if (flags & 0x1000000) != 0 {
                                        let skeleton;
                                        if lw(prop + 136) != 1 {
                                            sceVu0CopyMatrix(sp + 64, sp);
                                            skeleton = lw(model);
                                        } else {
                                            sceVu0MulMatrix(sp + 64, lw(lw(g::players) + 1764), sp);
                                            skeleton = lw(model);
                                        }
                                        let mut bone = 0;
                                        if lw(skeleton + 4) > 0 {
                                            i += 1;
                                            let mut list = lw(model + 8);
                                            loop {
                                                let offset = bone << 6;
                                                let out = list + offset;
                                                bone += 1;
                                                sceVu0MulMatrix(out, sp + 64, lw(model + 4) + offset);
                                                if bone >= lw(lw(model) + 4) {
                                                    break;
                                                }
                                                list = lw(model + 8);
                                            }
                                            flags = lw(prop + 16);
                                            break 'first;
                                        }
                                        flags = lw(prop + 16);
                                    }
                                    i += 1;
                                }
                                if (flags & 0x2000000) != 0 {
                                    let skeleton;
                                    if lw(prop + 136) != 1 {
                                        sceVu0CopyMatrix(sp + 64, sp);
                                        skeleton = lw(model);
                                    } else {
                                        sceVu0MulMatrix(sp + 64, lw(lw(g::players) + 3584), sp);
                                        skeleton = lw(model);
                                    }
                                    let mut bone = 0;
                                    if lw(skeleton + 4) > 0 {
                                        let mut list = lw(model + 12);
                                        loop {
                                            let offset = bone << 6;
                                            let out = list + offset;
                                            bone += 1;
                                            sceVu0MulMatrix(out, sp + 64, lw(model + 4) + offset);
                                            if bone >= lw(lw(model) + 4) {
                                                break;
                                            }
                                            list = lw(model + 12);
                                        }
                                    }
                                    flags = lw(prop + 16);
                                }
                                if (flags & 0x4000000) != 0 {
                                    let skeleton;
                                    if lw(prop + 136) != 1 {
                                        sceVu0CopyMatrix(sp + 64, sp);
                                        skeleton = lw(model);
                                    } else {
                                        sceVu0MulMatrix(sp + 64, lw(lw(g::players) + 0x151c), sp);
                                        skeleton = lw(model);
                                    }
                                    let mut bone = 0;
                                    if lw(skeleton + 4) > 0 {
                                        let mut list = lw(model + 16);
                                        loop {
                                            let offset = bone << 6;
                                            let out = list + offset;
                                            bone += 1;
                                            sceVu0MulMatrix(out, sp + 64, lw(model + 4) + offset);
                                            if bone >= lw(lw(model) + 4) {
                                                break;
                                            }
                                            list = lw(model + 16);
                                        }
                                    }
                                    flags = lw(prop + 16);
                                }
                                if (flags & 0x8000000) == 0 {
                                    break 'ordinary;
                                }
                                let skeleton;
                                if lw(prop + 136) != 1 {
                                    sceVu0CopyMatrix(sp + 64, sp);
                                    skeleton = lw(model);
                                } else {
                                    sceVu0MulMatrix(sp + 64, lw(lw(g::players) + 0x1c38), sp);
                                    skeleton = lw(model);
                                }
                                let mut bone = 0;
                                if lw(skeleton + 4) <= 0 {
                                    break 'ordinary;
                                }
                                let mut list = lw(model + 20);
                                loop {
                                    let offset = bone << 6;
                                    let out = list + offset;
                                    bone += 1;
                                    sceVu0MulMatrix(out, sp + 64, lw(model + 4) + offset);
                                    if bone >= lw(lw(model) + 4) {
                                        break;
                                    }
                                    list = lw(model + 20);
                                }
                                count = lw(g::num_props);
                                break 'prop;
                            }
                        }
                        // the stage prop and the props of kind 8
                        'special: {
                            let x = lwc1(prop + 48);
                            let y = lwc1(prop + 52);
                            let angle = ps2float::mul(lwc1(prop + 76), lwc1(0x3acd8c)).pdiv(1.8e+02_f32);
                            let z = lwc1(prop + 56);
                            matrixTransRotY(sp, x, y, z, angle);
                            flags = lw(prop + 16);
                            let players = lw(g::players);
                            'first_special: {
                                if (flags & 0x1000000) != 0 {
                                    sceVu0MulMatrix(sp + 64, lw(players + 1764), sp);
                                    let mut bone = 0;
                                    if lw(lw(model) + 4) > 0 {
                                        i += 1;
                                        let mut list = lw(model + 8);
                                        loop {
                                            let offset = bone << 6;
                                            let out = list + offset;
                                            bone += 1;
                                            sceVu0MulMatrix(out, sp + 64, lw(model + 4) + offset);
                                            if bone >= lw(lw(model) + 4) {
                                                break;
                                            }
                                            list = lw(model + 8);
                                        }
                                        flags = lw(prop + 16);
                                        break 'first_special;
                                    }
                                    flags = lw(prop + 16);
                                }
                                i += 1;
                            }
                            let players = lw(g::players);
                            if (flags & 0x2000000) != 0 {
                                sceVu0MulMatrix(sp + 64, lw(players + 3584), sp);
                                let mut bone = 0;
                                if lw(lw(model) + 4) > 0 {
                                    let mut list = lw(model + 12);
                                    loop {
                                        let offset = bone << 6;
                                        let out = list + offset;
                                        bone += 1;
                                        sceVu0MulMatrix(out, sp + 64, lw(model + 4) + offset);
                                        if bone >= lw(lw(model) + 4) {
                                            break;
                                        }
                                        list = lw(model + 12);
                                    }
                                }
                                flags = lw(prop + 16);
                            }
                            let players = lw(g::players);
                            if (flags & 0x4000000) != 0 {
                                sceVu0MulMatrix(sp + 64, lw(players + 0x151c), sp);
                                let mut bone = 0;
                                if lw(lw(model) + 4) > 0 {
                                    let mut list = lw(model + 16);
                                    loop {
                                        let offset = bone << 6;
                                        let out = list + offset;
                                        bone += 1;
                                        sceVu0MulMatrix(out, sp + 64, lw(model + 4) + offset);
                                        if bone >= lw(lw(model) + 4) {
                                            break;
                                        }
                                        list = lw(model + 16);
                                    }
                                }
                                flags = lw(prop + 16);
                            }
                            let players = lw(g::players);
                            if (flags & 0x8000000) != 0 {
                                sceVu0MulMatrix(sp + 64, lw(players + 0x1c38), sp);
                                let mut bone = 0;
                                count = lw(g::num_props);
                                if lw(lw(model) + 4) <= 0 {
                                    break 'prop;
                                }
                                let mut list = lw(model + 20);
                                loop {
                                    let offset = bone << 6;
                                    let out = list + offset;
                                    bone += 1;
                                    sceVu0MulMatrix(out, sp + 64, lw(model + 4) + offset);
                                    if bone >= lw(lw(model) + 4) {
                                        break;
                                    }
                                    list = lw(model + 20);
                                }
                                count = lw(g::num_props);
                                break 'prop;
                            }
                            count = lw(g::num_props);
                            break 'prop;
                        }
                    }
                    count = lw(g::num_props);
                    break 'prop;
                }
            }
            i += 1;
        }
        if i >= count {
            break;
        }
    }
}
