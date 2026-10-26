//! What a hit does to the swinging, hanging and flying props, deleting a prop, and what an enemy knows of its surroundings.

use crate::*;

/// Copy 12 bytes (a vector of three floats) from the game's constant at `from` to `to`.
fn copy_vector(from: i32, to: i32) {
    let tail = lw(from + 8);
    sd(to, ld(from));
    sw(to + 8, tail);
}

/// The boat sign is swung by a hit: its swing at +20 of its state changes by the hit's direction (`hit_direction`) against the way of the sign, the force `force` and 18 (@ 0x2797a8).
pub fn boatsignDamage(prop: i32, _a1: i32, _a2: i32, _a3: i32, _a4: i32, hit_direction: i32, force: f32) {
    let frame = Frame::new(128);
    let sp = frame.sp();
    copy_vector(0x3a85a8, sp);
    memset(sp + 16, 0, 12);
    let state = lw(prop + 352);
    let multiplier = getSwingDirectionMultiplier(hit_direction, sp, sp + 16);
    swc1(state + 20, lwc1(state + 20) + multiplier * force * 18.0_f32);
}

/// The banner: like the boat sign about its own way, with the sign of the swing turned round when the banner faces away (its direction at +76 180 or more) (@ 0x279908).
pub fn bannerDamage(prop: i32, _a1: i32, _a2: i32, _a3: i32, _a4: i32, hit_direction: i32, force: f32) {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let state = lw(prop + 352);
    copy_vector(0x3a85b8, sp);
    copy_vector(0x3a85c8, sp + 16);
    let multiplier = getSwingDirectionMultiplier(hit_direction, sp, sp + 16);
    let swing = lwc1(state + 20);
    let change;
    let scale;
    if !(180.0_f32 <= lwc1(prop + 76)) {
        change = multiplier * force;
        scale = 18.0_f32;
    } else {
        scale = 18.0_f32;
        change = -multiplier * force;
    }
    swc1(state + 20, swing + change * scale);
}

/// The chandelier: swung in two directions, +20 and +28 of its state, by 10 times the force (@ 0x279ae8).
pub fn chandelierDamage(prop: i32, _a1: i32, _a2: i32, _a3: i32, _a4: i32, hit_direction: i32, force: f32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    copy_vector(0x3a85a8, sp);
    copy_vector(0x3a85d8, sp + 16);
    memset(sp + 32, 0, 12);
    let scale = 10.0_f32;
    let state = lw(prop + 352);
    let first = getSwingDirectionMultiplier(hit_direction, sp, sp + 32);
    swc1(state + 20, lwc1(state + 20) + first * force * scale);
    let second = getSwingDirectionMultiplier(hit_direction, sp + 16, sp + 32);
    swc1(state + 28, lwc1(state + 28) + second * force * scale);
}

/// The lantern: the same as the chandelier with 18 (@ 0x2799e0).
pub fn lanternDamage(prop: i32, _a1: i32, _a2: i32, _a3: i32, _a4: i32, hit_direction: i32, force: f32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    copy_vector(0x3a85a8, sp);
    copy_vector(0x3a85d8, sp + 16);
    memset(sp + 32, 0, 12);
    let scale = 18.0_f32;
    let state = lw(prop + 352);
    let first = getSwingDirectionMultiplier(hit_direction, sp, sp + 32);
    swc1(state + 20, lwc1(state + 20) + first * force * scale);
    let second = getSwingDirectionMultiplier(hit_direction, sp + 16, sp + 32);
    swc1(state + 28, lwc1(state + 28) + second * force * scale);
}

/// How a hit turns a vehicle: the vehicle's side vector (12 bytes at 0x3a8160) is turned by its three angles (at +52 of its state, by the game's constant `to_radians_at`), and the
/// angle (0 to 360) between that and the hit's direction gives -1 to 1 as for a swing; the turn (+96 of the state) changes by that, the force and `scale` (@ copterDamage, taxiDamage).
fn vehicle_hit(prop: i32, hit_direction: i32, force: f32, to_radians_at: i32, scale: f32) {
    let frame = Frame::new(192);
    let sp = frame.sp();
    copy_vector(0x3a8160, sp + 80);
    let k = lwc1(to_radians_at);
    let state = lw(prop + 352);
    let degrees = 180.0_f32;
    let mut result = 0.0_f32;
    matrixRotZXY(
        sp,
        (lwc1(state + 52) * k).pdiv(degrees),
        (lwc1(state + 56) * k).pdiv(degrees),
        (lwc1(state + 60) * k).pdiv(degrees),
    );
    matrixVecMulAligned(&[sp as i64, (sp + 80) as i64]);
    let x = lwc1(hit_direction);
    let y = lwc1(hit_direction + 4);
    let z = lwc1(hit_direction + 8);
    fptodp(x * x + y * y + z * z);
    let x = lwc1(hit_direction);
    let y = lwc1(hit_direction + 4);
    let z = lwc1(hit_direction + 8);
    let wide = fptodp(x * x + y * y + z * z);
    let root = sqrt(wide);
    let length = dptofp(root);
    let nx = (lwc1(hit_direction)).pdiv(length);
    let ny = (lwc1(hit_direction + 4)).pdiv(length);
    let nz = (lwc1(hit_direction + 8)).pdiv(length);
    let sz = lwc1(sp + 88);
    let sy = lwc1(sp + 84);
    let first = lwc1(sp + 80) * nx;
    swc1(sp + 64, nx);
    swc1(sp + 68, ny);
    swc1(sp + 72, nz);
    let dot = first + sy * ny + sz * nz;
    let mut angle = (acosf(dot) * degrees).pdiv(k);
    if angle < result {
        angle += 360.0_f32;
    }
    let ninety = 90.0_f32;
    if angle < ninety {
        result = -(((ninety - angle)).pdiv(ninety));
    }
    if ninety <= angle && angle < degrees {
        result = ((angle - ninety)).pdiv(ninety);
    }
    let change;
    if degrees <= angle && angle < 270.0_f32 {
        result = ((ninety - (angle - degrees))).pdiv(ninety);
    }
    if !(270.0_f32 <= angle) {
        change = force * result;
    } else {
        change = force * -(((angle - 270.0_f32)).pdiv(90.0_f32));
    }
    swc1(state + 96, lwc1(state + 96) + change * scale);
}

/// The helicopter's turn from a hit (@ 0x277860).
pub fn copterDamage(prop: i32, _a1: i32, _a2: i32, _a3: i32, _a4: i32, hit_direction: i32, force: f32) {
    vehicle_hit(prop, hit_direction, force, 0x3ace5c, 20.0_f32);
}

/// The taxi's turn from a hit (@ 0x2775c0).
pub fn taxiDamage(prop: i32, _a1: i32, _a2: i32, _a3: i32, _a4: i32, hit_direction: i32, force: f32) {
    vehicle_hit(prop, hit_direction, force, 0x3ace58, 13.0_f32);
}

/// Delete a prop: its model instance is deleted, it is taken off what it was attached to, its number is -1 and flag 0x80000000 set, a prop of kind 2048 gives back its head
/// record, the count of deleted props goes up, and it leaves the collision list (@ 0x269340).
pub fn propDelete(prop: i32) {
    let model = lw(prop + 32);
    if model != 0 {
        obInstDelete(model);
        sw(prop + 32, 0);
    }
    if lw(prop + 188) != 0 {
        propUnattach(prop);
    }
    let flags = lw(prop + 16);
    let kind = lw(prop + 8);
    sw(prop, -1);
    sw(prop + 16, flags | -0x80000000);
    if kind == 2048 {
        freehead(lw(prop + 352));
    }
    sw(g::num_deleted_props, lw(g::num_deleted_props) + 1);
    propRemovePropFromCollisionList(prop);
}

/// Note an explosion near the enemy: every explosion record (16 bytes at `aiexplosiondata`) is set to the enemy's own position and the nearest by a distance that starts
/// negative is looked for (it is never found); when one is found and within 4 (16 squared) the enemy is told to check whether it is in front (@ 0x2cd718).
pub fn enemyExplosionNearby(chr: i32) {
    let count = lw(0x3b01ac);
    let mut nearest = -1.0_f32;
    let mut index = -1;
    let mut i = 0;
    if count > 0 {
        loop {
            let x = lwc1(chr + 48);
            let entry = g::aiexplosiondata + (i << 4);
            swc1(entry, x);
            let y = lwc1(chr + 52);
            swc1(entry + 4, y);
            let z = lwc1(chr + 56);
            let distance = x * x + y * y + z * z;
            swc1(entry + 8, z);
            if distance < nearest {
                nearest = distance;
                index = i;
            }
            i += 1;
            if i >= count {
                break;
            }
        }
    }
    if index != -1 && nearest < 16.0_f32 {
        checkInFront(chr, (index << 4) + g::aiexplosiondata, 0.0_f32);
    }
}

/// Whether the enemy's target (a character of a player whose weapon is low in ammo, the target's record +748 or +444 under 10) is aiming at the enemy from more than 5 away (@ 0x2cc3a0).
pub fn enemyFiredAt(chr: i32) -> i32 {
    let target = lw(lw(chr + 352) + 2788);
    if target == 0 {
        return 0;
    }
    let target_state = lw(target + 352);
    if target_state == 0 {
        return 0;
    }
    let dx = lwc1(target + 48) - lwc1(chr + 48);
    let dy = lwc1(target + 52) - lwc1(chr + 52);
    let number = lw(target_state);
    let dz = lwc1(target + 56) - lwc1(chr + 56);
    let ammo;
    if number >= lw(g::lvnumplayers) {
        let record = number * 0x1210 + lw(g::chrinfo);
        if lw(record + 440) == 0 {
            return 0;
        }
        ammo = lw(record + 444);
    } else {
        let player = number * 1820 + lw(g::players);
        if lw(player + 612) == 0 {
            return 0;
        }
        ammo = lw(player + 748);
    }
    if ammo < 10 {
        if !(25.0_f32 < dx * dx + dy * dy + dz * dz) {
            return 0;
        }
        let aiming = checkAimingFront(chr, target, lwc1(0x3ada78));
        return (aiming != 0) as i32;
    }
    0
}

/// How the enemy knows of its target: 0 not at all (or with the invisibility cheat), 1 when it is in front of it (within the enemy's field) and it has already been seen, 2 when a
/// bullet sounded near it (a player's character with a sound radius under the distance), 3 when the target is within 2 (@ 0x2c66a0).
pub fn awareOfTarget(chr: i32) -> i32 {
    let state = lw(chr + 352);
    if lw(g::invisible) != 0 {
        return 0;
    }
    let in_front = checkInFront(chr, lw(state + 2788) + 48, lwc1(state + 2700));
    if in_front != 0 && lw(state + 2772) != 0 {
        return 1;
    }
    let flags = lw(state + 2704);
    let players = lw(g::lvnumplayers);
    if (flags & 32) != 0 && players > 0 {
        let z = lwc1(chr + 56);
        let chars = lw(g::chrinfo);
        let x = lwc1(chr + 48);
        let y = lwc1(chr + 52);
        let mut i = 0;
        loop {
            let record = i * 0x1210 + chars;
            let dy = lwc1(record + 2936) - y;
            let dx = lwc1(record + 2932) - x;
            let dz = lwc1(record + 2940) - z;
            i += 1;
            if (dx * dx + dy * dy + dz * dz) < lwc1((lw(record + 2944) << 2) + g::bulletSoundRadius) {
                return 2;
            }
            if i >= players {
                break;
            }
        }
    }
    let target = lw(state + 2788);
    let dy = lwc1(target + 52) - lwc1(chr + 52);
    let dx = lwc1(target + 48) - lwc1(chr + 48);
    let dz = lwc1(target + 56) - lwc1(chr + 56);
    if (dx * dx + dy * dy + dz * dz) < 4.0_f32 { 3 } else { 0 }
}
