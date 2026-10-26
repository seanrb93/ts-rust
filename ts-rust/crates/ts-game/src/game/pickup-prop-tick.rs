//! Functions: the tick of a pickup prop (weapons, ammo, health, armour and bags).

use crate::*;

/// Reset the character prop `holder` that was carrying a bag: its animation is stopped and all the matrices of its model are cleared.
fn drop_bag_reset(holder: i32) {
    let flags = lw(holder + 16) & -129;
    sw(holder + 72, 0);
    let data = lw(holder + 32);
    sw(holder + 188, 0);
    sw(holder + 196, 0);
    sw(holder + 76, 0);
    sw(holder + 88, 0);
    let speed = lwc1(holder + 72);
    sw(holder + 192, -1);
    sw(holder + 140, 2);
    sw(holder + 16, flags & -0x40000001);
    sh(data + 292, 0);
    obInstSetAnim(data, 0, speed);
    let data = lw(holder + 32);
    if lw(lw(data) + 4) > 0 {
        let mut n = 0;
        let mut table = lw(data + 4);
        loop {
            sceVu0UnitMatrix(table + (n << 6));
            n += 1;
            let data = lw(holder + 32);
            if n >= lw(lw(data) + 4) {
                break;
            }
            table = lw(data + 4);
        }
    }
}

/// The tick of the pickup prop `a0` (@ 0x26a330): a pickup that times out fades and is deleted; it turns or bobs; and then every character is tested for touching it and, when one
/// does, the pickup does its effect by its kind (`lw(data)`: 1 a bag, 2 a weapon, 4 armour, 8 health, 32 ammo).
pub fn pickupPropTick(a0: i32) {
    let data = lw(a0 + 352);
    let owner_kind;
    if lw(a0 + 488) == 0 {
        owner_kind = lw(data + 92);
    } else {
        let left = lw(a0 + 492) - lw(g::updateframesi);
        let left_f = left as f32;
        sw(a0 + 492, left);
        'b2: {
            if left_f <= 9e+01_f32 {
                obInstSetOverrideStates(lw(a0 + 32), 3, -1, -1, ((left_f * 127.0_f32).pdiv(9e+01_f32)) as i32 & 255);
                if lw(data) != 2 {
                    break 'b2;
                }
                let pad = lw(data + 64) * 28 + lw(g::levelpads);
                sw(pad + 4, lw(pad + 4) & -0x10001);
            }
        }
        if lw(a0 + 492) <= 0 {
            pickupPropDelete(a0);
            return;
        }
        owner_kind = lw(data + 92);
    }
    'b4: {
        'b5: {
            if owner_kind == -1 {
                let mode_kind;
                if lw(g::curgameparms + 72) != 7 {
                    mode_kind = lw(a0 + 4);
                } else {
                    if lw(data) != 1 {
                        break 'b4;
                    }
                    mode_kind = lw(a0 + 4);
                }
                let frames = lwc1(g::updateframesf);
                if mode_kind != 325 {
                    let turn = lwc1(a0 + 76) + (frames + frames);
                    swc1(a0 + 76, turn);
                    if 3.6e+02_f32 <= turn {
                        swc1(a0 + 76, turn - 3.6e+02_f32);
                    }
                } else {
                    lobsterPropTick(a0);
                }
                let size = lw(data + 96);
                'b9: {
                    if size != -1 {
                        let flags;
                        if lw(data) != 2 {
                            flags = lw(a0 + 16);
                        } else if ((lw(a0 + 4) - 342) as u32) < (11 as u32) {
                            flags = lw(a0 + 16);
                        } else {
                            let time = lw(g::gameframe) as f32 * lwc1(0x3acc00);
                            let a = fptodp(time);
                            let b = fptodp(lwc1(data + 104));
                            let c = dpmul(b, ld(0x3a7dd8));
                            let d = dpadd(a, c);
                            let e = dpmul(d, ld(0x3a7de0));
                            let f = dpadd(e, 0x3fe0000000000000_i64);
                            let index = dptoli(f);
                            swc1(a0 + 52, lwc1(data + 28) + lwc1((((index as i32) & 2047) << 2) + g::g_SinTable) * lwc1(0x3acc04));
                            flags = lw(a0 + 16);
                        }
                        if (flags & 1) == 0 {
                            break 'b5;
                        }
                        if lwc1(data + 100) == 1.0_f32 {
                            break 'b4;
                        }
                        let model = lw(a0 + 32);
                        let mut n = 0;
                        if lw(lw(model) + 4) <= 0 {
                            break 'b4;
                        }
                        let mut table = lw(model + 4);
                        loop {
                            let scale = lwc1(data + 100);
                            let matrix = table + (n << 6);
                            n += 1;
                            matrixScaleSet(matrix, scale, scale, scale);
                            let model = lw(a0 + 32);
                            if n >= lw(lw(model) + 4) {
                                break 'b9;
                            }
                            table = lw(model + 4);
                        }
                    }
                }
            }
        }
    }
    'b12: {
        let mut players = lw(g::lvnumplayers);
        let mut chars = lw(g::numactivechars);
        let mut index = 0;
        if (players + chars) <= 0 {
            break 'b12;
        }
        let mut chr = 0;
        let mut prop = 0;
        'bag: {
            'b18: {
                'b17: {
                    'b16: {
                        'b15: {
                            'l13: loop {
                                'b19: {
                                    chr = lw(g::chrinfo) + index * 0x1210;
                                    let state = lw(chr + 2716);
                                    prop = lw(chr + 3020);
                                    if (state & 16) == 0 && state != 0 {
                                        if (ld(prop + 16) & 0x80000001_i64) != 1_i64 {
                                            index += 1;
                                            break 'b19;
                                        }
                                        if ((lw(chr + 8) - 2) as u32) < (2 as u32) {
                                            index += 1;
                                            break 'b19;
                                        }
                                        let ground;
                                        if (lw(prop + 16) & 0xf000000) != 0 {
                                            ground = lwc1(prop + 52);
                                        } else if index < players {
                                            ground = lwc1(prop + 52);
                                        } else {
                                            ground = lwc1(prop + 52) - lwc1(0x3acc08);
                                        }
                                        let dx = lwc1(a0 + 48) - lwc1(prop + 48);
                                        let eye = lwc1(lw(a0 + 352) + 20);
                                        let dz = lwc1(a0 + 56) - lwc1(prop + 56);
                                        let base = if lw(data + 96) != -1 { lwc1(data + 28) } else { lwc1(a0 + 52) };
                                        let mut top = base + lwc1(a0 + 512) - eye;
                                        if lw(data + 92) == -1 {
                                            let extra = pickupGetExtraHeight(lw(a0 + 4), lw(data));
                                            top = top - extra;
                                        }
                                        let reach = lwc1(a0 + 524) + lwc1(0x3acc0c);
                                        if !((top - reach) <= ground) {
                                            index += 1;
                                        } else if !(ground <= top + reach) {
                                            index += 1;
                                        } else if !((dx * dx + dz * dz).abs().psqrt() < 1.0_f32) {
                                            index += 1;
                                        } else {
                                            let mut allowed = 1;
                                            if lw(chr + 2708) == 32 {
                                                index += 1;
                                            } else {
                                                let mut drop_item = 0;
                                                'b25: {
                                                    let case = lw(data) - 1;
                                                    if (case as u32) < (16 as u32) {
                                                        match case {
                                                            0 => {
                                                                allowed = 1;
                                                                drop_item = 0;
                                                            }
                                                            3 => {
                                                                if 1.0_f32 <= lwc1(prop + 520) {
                                                                    allowed = 0;
                                                                }
                                                                if lw(g::curgameparms + 72) != 7 {
                                                                    break 'b25;
                                                                }
                                                                allowed = if lw(chr) >= lw(g::lvnumplayers) { 0 } else { allowed };
                                                                break 'b25;
                                                            }
                                                            7 => {
                                                                if !(1.0_f32 <= lwc1(chr + 12)) {
                                                                    break 'b25;
                                                                }
                                                                allowed = 0;
                                                            }
                                                            15 => {
                                                                allowed = 0;
                                                                drop_item = 1;
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                                'b28: {
                                                    'b29: {
                                                        if index >= lw(g::lvnumplayers) {
                                                            if ((lw(g::curgameparms + 72) - 6) as u32) < (2 as u32) {
                                                                allowed = 0;
                                                                break 'b29;
                                                            }
                                                        }
                                                        if lw(g::curgameparms + 72) != 4 {
                                                            break 'b28;
                                                        }
                                                        if prop != lw(g::escort) {
                                                            break 'b28;
                                                        }
                                                        allowed = 0;
                                                    }
                                                    drop_item = 0;
                                                }
                                                let drop_now = if (lw(g::curgameparms + 72) ^ 1) == 0 { 0 } else { drop_item };
                                                if allowed != 0 {
                                                    let pad = lw(data + 64);
                                                    let mut pickup_kind = lw(data);
                                                    if pad >= 0 {
                                                        let record = pad * 28 + lw(g::levelpads);
                                                        sw(record + 4, lw(record + 4) & -0x10001);
                                                        sw(data + 64, -1);
                                                        pickup_kind = lw(data);
                                                    }
                                                    if pickup_kind != 1 && lw(data + 92) == -1 && lw(a0 + 488) == 0 {
                                                        sw(a0 + 16, lw(a0 + 16) & -2 & -3);
                                                        if lw(g::curgameparms + 72) != 7 {
                                                            timerNew(0, a0, 0x273ac0, 15.0_f32);
                                                        }
                                                    }
                                                    let case = lw(data) - 1;
                                                    if (case as u32) < (32 as u32) {
                                                        match case {
                                                            0 => break 'b18,
                                                            1 => break 'b17,
                                                            3 => break 'b16,
                                                            7 => break 'b15,
                                                            31 => break 'l13,
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                                if drop_now == 0 {
                                                    index += 1;
                                                } else {
                                                    let carried = lw(chr + 484);
                                                    if carried == 0 {
                                                        index += 1;
                                                    } else if lw(chr + 24) != lw(data + 16) {
                                                        index += 1;
                                                    } else {
                                                        let carried_data = lw(carried + 352);
                                                        propDropBagOnDrop(chr, a0, carried);
                                                        index += 1;
                                                        drop_bag_reset(carried);
                                                        sw(chr + 484, 0);
                                                        sw(carried_data + 68, 1);
                                                        updateTeamFlags(prop, carried, 0);
                                                    }
                                                }
                                            }
                                        }
                                        chars = lw(g::numactivechars);
                                        break 'b19;
                                    }
                                    index += 1;
                                }
                                players = lw(g::lvnumplayers);
                                if index >= players + chars {
                                    break 'b12;
                                }
                            }
                            // ammo
                            let gun = lw(data + 16);
                            let amount = lw(data + 12);
                            let before = lw(chr + (gun << 2) + 620);
                            gunAddAmmo(chr, gun, amount);
                            if lw(prop + 356) != 0 {
                                soundStart(183, prop + 48);
                            }
                            StatsAdd(35, lw(chr), 1.0_f32);
                            gunPickedUpAmmo(chr, lw(data + 16), before);
                            break 'b12;
                        }
                        // health
                        let health = lwc1(chr + 12) + lwc1(data + 8);
                        swc1(chr + 12, health);
                        if 1.0_f32 < health {
                            swc1(chr + 12, 1.0_f32);
                        }
                        if lw(prop + 356) != 0 {
                            soundStart(185, prop + 48);
                        }
                        StatsAdd(34, lw(chr), 1.0_f32);
                        break 'b12;
                    }
                    // armour
                    let armour = lwc1(prop + 520) + lwc1(data + 8);
                    swc1(prop + 520, armour);
                    if 1.0_f32 < armour {
                        swc1(prop + 520, 1.0_f32);
                    }
                    if lw(prop + 356) != 0 {
                        soundStart(184, prop + 48);
                    }
                    StatsAdd(33, lw(chr), 1.0_f32);
                    break 'b12;
                }
                // weapon
                let combo = lw(data + 12);
                if ((combo - 1) as u32) >= (32 as u32) {
                    break 'b12;
                }
                if lvIsValidGunCombo(combo) == 0 {
                    break 'b12;
                }
                let remains;
                'b44: {
                    let combo = lw(data + 12);
                    if ((combo - 1) as u32) >= (32 as u32) {
                        remains = lw(data + 92);
                    } else {
                        if lvIsValidGunCombo(combo) != 0 {
                            gunAddAmmo(chr, lw(data + 44), lw(data + 48));
                            let player_kind = lw(chr + 8);
                            let proceed = if player_kind == 1 {
                                (lw(lw(data + 12) * 400 + (g::guns + 28)) & 2) != 0
                            } else {
                                player_kind == 0
                            };
                            if !proceed {
                                remains = lw(data + 92);
                                break 'b44;
                            }
                            let at = chr + 488 + (lw(data + 12) << 2);
                            let had = lw(at);
                            sw(at, 1);
                            let switch = gunShouldSwitchToOnPickup(chr, lw(data + 12));
                            let human = lw(prop + 356);
                            if switch != 0 {
                                sw(chr + 260, lw(data + 12));
                            }
                            let combo_now;
                            'b49: {
                                if human == 0 {
                                    combo_now = lw(chr + 260);
                                    break 'b49;
                                }
                                let current;
                                if had != 0 {
                                    current = lw(chr + 260);
                                } else {
                                    let strings = lw((lw(g::cur_language) << 2) + g::languages);
                                    let name_at = (lh((lw(data + 12) << 3) + (g::guncombos + 6)) << 2) + strings;
                                    let text = vafmt(&[0x3af920, lw(strings + 0x11e0) as i64, lw(name_at) as i64, name_at as i64]);
                                    playerMessage(lw(prop + 356), text);
                                    current = lw(chr + 260);
                                }
                                let index_sound = current - 7;
                                if (index_sound as u32) < (26 as u32) {
                                    match index_sound {
                                        0 | 1 | 2 | 3 | 10 => {
                                            soundStart(134, prop + 48);
                                            combo_now = lw(chr + 260);
                                            break 'b49;
                                        }
                                        11 | 12 | 13 => {
                                            soundStart(183, prop + 48);
                                            combo_now = lw(chr + 260);
                                            break 'b49;
                                        }
                                        25 => {
                                            soundStart(135, prop + 48);
                                            combo_now = lw(chr + 260);
                                            break 'b49;
                                        }
                                        _ => {}
                                    }
                                }
                                soundStart(133, prop + 48);
                                combo_now = lw(chr + 260);
                            }
                            StatsAddGun(4, lw(chr), combo_now, 1.0_f32);
                        }
                        remains = lw(data + 92);
                    }
                }
                if remains == -1 {
                    break 'b12;
                }
                pickupPropDelete(a0);
                break 'b12;
            }
            break 'bag;
        }
        // bag
        if propChrCanPickupBag(chr, a0) == 0 {
            if lw(g::curgameparms + 72) == 2 {
                if lw(data + 68) != 4 {
                    if lw(chr + 24) == lw(data + 16) {
                        if (lw(a0 + 16) & 64) != 0 {
                            soundStart(232, a0 + 48);
                            let word = lw(data + 40);
                            swc1(a0 + 48, lwc1(data + 24));
                            sw(data + 36, word);
                            let flags = lw(a0 + 16);
                            swc1(a0 + 52, lwc1(data + 28));
                            let z = lwc1(data + 32);
                            sw(a0 + 16, flags & -65 | 1);
                            swc1(a0 + 56, z);
                            sw(data + 68, 1);
                            swc1(a0 + 112, lwc1(data + 24));
                            swc1(a0 + 116, lwc1(data + 28));
                            let z = lwc1(data + 32);
                            sw(a0 + 72, 0);
                            sw(a0 + 76, 0);
                            sw(a0 + 88, 0);
                            swc1(a0 + 120, z);
                            bagSecuredMessage(chr);
                            propMakeVisible(a0);
                        }
                    }
                }
            }
        } else {
            let mut flags = lw(a0 + 16);
            'b52: {
                if lw(g::curgameparms + 72) == 2 {
                    if (flags & 64) != 0 {
                        break 'b52;
                    }
                    let count = lw(g::lvnumplayers);
                    let mut n = 0;
                    let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 0x11d8);
                    if count > 0 {
                        let mut player_offset = 0;
                        let mut chr_offset = 0;
                        loop {
                            if lw(chr_offset + lw(g::chrinfo) + 24) != lw(data + 16) {
                                n += 1;
                            } else {
                                playerMessage(lw(g::players) + player_offset, text);
                                n += 1;
                            }
                            player_offset += 1820;
                            chr_offset += 0x1210;
                            if n >= lw(g::lvnumplayers) {
                                break;
                            }
                        }
                        flags = lw(a0 + 16);
                    }
                }
            }
            let held = flags | 64;
            let carried;
            'b57: {
                'b58: {
                    sw(a0 + 16, held);
                    sw(data + 68, 4);
                    let model = lw(prop + 32);
                    let skeleton = lw(model + 92);
                    let cleared = lw(a0 + 16) & -3;
                    sw(chr + 484, a0);
                    sw(a0 + 16, cleared);
                    propSetAttached(a0, model, lw(skeleton + 24), 2);
                    if lw(g::curgameparms + 72) != 7 {
                        soundStart(232, prop + 48);
                    } else {
                        let level = lw(g::lvlevelnum) - 2;
                        if (level as u32) < (26 as u32) {
                            let sound = match level {
                                0 => 226,
                                3 => 227,
                                6 => 229,
                                7 => 228,
                                12 => 225,
                                19 => 224,
                                21 => 223,
                                22 => 230,
                                25 => 231,
                                _ => break 'b58,
                            };
                            soundStart(sound, prop + 48);
                            carried = lw(chr + 484);
                            break 'b57;
                        }
                    }
                }
                carried = lw(chr + 484);
            }
            updateTeamFlags(prop, carried, 1);
            let owner = lw(chr + 3020);
            let human = lw(prop + 356);
            swc1(a0 + 564, lwc1(owner + 48));
            swc1(a0 + 568, lwc1(owner + 52));
            swc1(a0 + 572, lwc1(owner + 56));
            if human != 0 {
                let mode = lw(g::curgameparms + 72);
                if mode == 2 || mode == 5 {
                    let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 0x11dc);
                    hudMessage(lw(human), text);
                }
            }
            StatsAdd(37, lw(chr), 1.0_f32);
        }
    }
}
