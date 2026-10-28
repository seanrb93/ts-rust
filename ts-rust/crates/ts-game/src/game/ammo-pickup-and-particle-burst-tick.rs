//! What the player holds after picking up ammo, and the tick of a burst of particles.

use crate::*;

/// After the player `player` picked up ammo for the gun `gun`: when the player's character has a weapon that is in the state 0 or 2 (and `flag` is 0), look through the weapon
/// combinations 1 to 32 that the player owns and use the gun in one of the hands, with an empty clip: the player switches (the current combination at +260) to the first
/// of those that is better (a combination is better when its first halfword, the rank, is larger than the current's) (@ 0x294e48).
pub fn gunPickedUpAmmo(player: i32, gun: i32, flag: i32) {
    let weapon = lw(lw(player + 3020) + 356);
    if weapon == 0 || flag != 0 {
        return;
    }
    let holder = lw(weapon + 24);
    if holder == 0 {
        return;
    }
    let state = lw(holder + 88);
    if state != 0 && state != 2 {
        return;
    }
    let combos = GUNCOMBOS;
    let guns = GUNS_TABLE;
    let slot = gun << 2;
    let mut combo = 1;
    let mut entry = combos + 8;
    let mut owned_at = player + 492;
    loop {
        let mut uses = 0;
        if own_get(player, combo) != 0 {
            let mut clip = 0;
            let left = lh(entry + 2);
            if left != -1 {
                if lw(left * 400 + guns + 16) == gun {
                    clip = lw(player + 272 + slot);
                    uses = 1;
                }
            }
            let right_gun = lh(entry + 4);
            if right_gun != -1 && lw(right_gun * 400 + guns + 16) == gun {
                clip = lw(player + 380 + slot);
                uses = 1;
            }
            if uses == 0 || clip != 0 {
                combo += 1;
            } else {
                if lh((lw(player + 260) << 3) + combos) >= lh(entry) {
                    sw(player + 260, combo);
                }
                combo += 1;
            }
        } else {
            combo += 1;
        }
        entry += 8;
        owned_at += 4;
        if combo >= combo_count() {
            break;
        }
    }
}

/// The tick of a burst of particles (the burst kind at +4 of the group, the count at +24, the positions from +928, their speeds from +2528, three floats each in turn): when the
/// group is running each position moves by its speed times the frame time, the speed in y falls by the kind's gravity, and a particle below the group's floor (+40) bounces
/// when the game is running: put on the floor, its y speed turned and scaled by the kind's bounce, its other speeds by the kind's friction (@ 0x29b298).
pub fn particleBurstTick(group: i32) {
    if (lw(group + 8) & 1) == 0 {
        return;
    }
    let mut i = 0;
    if lw(group + 24) > 0 {
        loop {
            let frame_time = lwc1(g::updateframesf);
            let at16 = i << 4;
            let px = group + 928 + at16;
            let py = group + 932 + at16;
            let pz = group + 936 + at16;
            let infos = g::particleinfos;
            let vx = group + 2528 + i * 12;
            let vy = group + 2532 + i * 12;
            let vz = group + 2536 + i * 12;
            swc1(px, ps2float::add(lwc1(px), ps2float::mul(lwc1(vx), frame_time)));
            swc1(py, ps2float::add(lwc1(py), ps2float::mul(lwc1(vy), frame_time)));
            swc1(pz, ps2float::add(lwc1(pz), ps2float::mul(lwc1(vz), frame_time)));
            swc1(vy, ps2float::add(lwc1(vy), ps2float::mul(lwc1(infos + lw(group + 4) * 104 + 52), frame_time)));
            let floor = lwc1(group + 40);
            let running = running_x();
            if lwc1(py) < floor {
                let friction = infos + 92;
                if running > 0 {
                    swc1(py, floor);
                    swc1(vy, ps2float::mul(-lwc1(vy), lwc1(lw(group + 4) * 104 + infos + 96)));
                    swc1(vx, ps2float::mul(lwc1(vx), lwc1(lw(group + 4) * 104 + friction)));
                    swc1(vz, ps2float::mul(lwc1(vz), lwc1(lw(group + 4) * 104 + friction)));
                }
            }
            i += 1;
            if i >= lw(group + 24) {
                break;
            }
        }
    }
}
