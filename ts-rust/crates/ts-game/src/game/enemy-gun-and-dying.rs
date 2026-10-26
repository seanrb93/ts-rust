//! Functions: an enemy's gun firing, loading a tile of the map maker, the dying of a character, drawing the particles, the string edit's tick, the pad action page, the arcade
//! options page and the view of a dead player.

use super::particle_spawners::random_as_float;
use crate::*;

/// Copy 32 bytes the way the compiled code does: three doublewords are read first, the first one is read and written, then the other three are written.
fn copy_menu_item(dst: i32, src: i32) {
    let second = ld(src + 8);
    let third = ld(src + 16);
    let fourth = ld(src + 24);
    sd(dst, ld(src));
    sd(dst + 8, second);
    sd(dst + 16, third);
    sd(dst + 24, fourth);
}

/// Update the gun of an enemy (@ 0x2bfd78): the muzzle flashes are turned off and the guns updated; the enemy detonates mines when it can; while it is in the aim
/// animation it fires the gun in short bursts, a burst length picked from the firing range table and the gun's fire rate, each shot made with a random choice of the
/// two fire modes.
pub fn enemyGunUpdate(a0: i32) {
    let state = lw(a0 + 352);
    if lw(state + 2716) == 0 {
        return;
    }
    if lw(state + 8) == 2 {
        return;
    }
    gunChrMuzzleOff(a0, 0);
    gunChrMuzzleOff(a0, 1);
    gunChrUpdate(a0, 0);
    gunChrUpdate(a0, 1);
    if gunCanDetonateMines(state) != 0 {
        enemyDetonateMines(a0);
    }
    let target = lw(state + 2788);
    'done: {
        let timer;
        if target == 0 {
            timer = lw(state + 444);
        } else {
            if lw(target + 8) == 64 {
                let flags = lw(state + 2704);
                sw(state + 2704, flags & -7);
                return;
            }
            timer = lw(state + 444);
        }
        let frames = lw(g::updateframesi);
        let second_timer = lw(state + 332);
        sw(state + 444, timer + frames);
        if second_timer != 0 {
            sw(state + 336, lw(state + 336) + frames);
        }
        let burst;
        'burst: {
            let flags = lw(state + 2704);
            if (flags & 4) != 0 {
                let offset = lw(state + 376) * 12;
                sw(state + 2704, flags & -5);
                let length = lw(offset + (g::firerange + 8));
                let at = lw(state) + g::curgameparms;
                sw(state + 3004, length);
                if lbu(at + 49) != 0 {
                    burst = lw(state + 2772);
                    break 'burst;
                }
                sw(state + 3004, (length + ((length as u32) >> 31) as i32) >> 1);
            }
            burst = lw(state + 2772);
        }
        'shooting: {
            if burst == 0 {
                if (lw(state + 2716) & 0x68c000) == 0 {
                    break 'shooting;
                }
            }
            let model;
            if lw(g::curgameparms + 72) == 7 {
                model = lw(a0 + 32);
            } else if (lw(state + 2704) & 2) == 0 {
                model = lw(a0 + 32);
            } else if lw(state + 440) == 0 {
                model = lw(a0 + 32);
            } else if target == 0 {
                model = lw(a0 + 32);
            } else if (lw(a0 + 16) & 0xf000000) != 0 {
                model = lw(a0 + 32);
            } else {
                if (lw(target + 16) & 0xf000000) == 0 {
                    if lw(state + 444) < lw(lw(state + 376) * 400 + (g::guns + 60)) {
                        return;
                    }
                    let random = random_as_float();
                    if !(lwc1(0x3ad890) < ps2float::mul(random, 2.3283064e-10_f32)) {
                        gunChrFire(a0, state + 376, 1);
                        return;
                    }
                    gunChrFire(a0, state + 376, 0);
                    return;
                }
                model = lw(a0 + 32);
            }
            if (lw(lw((lw(model + 96) << 2) + g::pAnimations) + 12) & 64) == 0 {
                return;
            }
            if (lw(state + 2704) & 2) == 0 {
                return;
            }
            if (lw(state + 2708) & 16) == 0 {
                return;
            }
            if !(lwc1(model + 224) == 0.0_f32) {
                return;
            }
            let mut angle = fptodp(lwc1(state + 2780));
            if dpcmp(angle, 0_i64) < 0 {
                angle = dpsub(0_i64, angle);
            }
            let kind;
            if dpcmp(angle, 0x4024000000000000_i64) < 0 {
                kind = lw(state + 8);
            } else {
                let mut angle = fptodp(lwc1(state + 2780));
                if dpcmp(angle, 0_i64) < 0 {
                    angle = dpsub(0_i64, angle);
                }
                if dpcmp(angle, ld(0x3a9d08)) >= 0 {
                    return;
                }
                if curAnim(a0, 478) != 0 {
                    kind = lw(state + 8);
                } else {
                    if curAnim(a0, 477) == 0 {
                        return;
                    }
                    kind = lw(state + 8);
                }
            }
            let second;
            'second: {
                'fired: {
                    let ready = lw(state + 440);
                    if kind == 1 {
                        if ready == 0 {
                            break 'fired;
                        }
                        if lw(state + 444) >= lw(lw(state + 376) * 400 + (g::guns + 60)) {
                            let random = random_as_float();
                            if !(lwc1(0x3ad894) < ps2float::mul(random, 2.3283064e-10_f32)) {
                                gunChrFire(a0, state + 376, 1);
                            } else {
                                gunChrFire(a0, state + 376, 0);
                            }
                            let left = lw(state + 3004) - 1;
                            sw(state + 3004, left);
                            if left > 0 {
                                return;
                            }
                            break 'shooting;
                        }
                    }
                    if ready != 0 {
                        if lw(state + 444) < lw(lw(state + 376) * 400 + (g::guns + 60)) {
                            second = lw(state + 332);
                            break 'second;
                        }
                        let random = random_as_float();
                        if !(lwc1(0x3ad898) < ps2float::mul(random, 2.3283064e-10_f32)) {
                            gunChrFire(a0, state + 376, 1);
                        } else {
                            gunChrFire(a0, state + 376, 0);
                        }
                        let left = lw(state + 3004);
                        sw(state + 3004, left - 1);
                        if curAnim(a0, 449) == 0 {
                            if lw(state + 3004) > 0 {
                                second = lw(state + 332);
                                break 'second;
                            }
                            let flags = lw(state + 2704);
                            sw(state + 3004, 0);
                            sw(state + 2704, flags & -3);
                        } else {
                            sw(state + 3004, 1);
                        }
                    }
                }
                second = lw(state + 332);
            }
            if second == 0 {
                return;
            }
            if (lw(lw((lw(lw(a0 + 32) + 96) << 2) + g::pAnimations) + 12) & 128) == 0 {
                return;
            }
            if lw(state + 336) < lw(lw(state + 268) * 400 + (g::guns + 60)) {
                return;
            }
            let random = random_as_float();
            if !(lwc1(0x3ad89c) < ps2float::mul(random, 2.3283064e-10_f32)) {
                gunChrFire(a0, state + 268, 1);
                return;
            }
            gunChrFire(a0, state + 268, 0);
            return;
        }
        let flags = lw(state + 2704);
        sw(state + 3004, 0);
        sw(state + 2704, flags & -3);
        break 'done;
    }
}

/// Load a tile of the map maker into the tile slot `a0` from the tile description `a1` (@ 0x2546a0): the tile's object and pad file are fetched from the preload (the object is
/// copied when more than one tile uses it), the volumes, the objects with their floors, walls and glass, the portals and the textures are relocated and transformed by the
/// tile's rotation and position, and the tile's pads are added.
pub fn bgTileLoad(a0: i32, a1: i32) {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let object = preloadGetOb((lw(a1) << 6) + g::tilefilenames);
    let pad_file = preloadGetFile((lw(a1) << 6) + g::tilepadfilenames);
    let mut tile = lw(a1);
    let offset = tile << 2;
    sw(offset + g::tilepaddata, lw(pad_file + 16));
    let size = lw(object + 20);
    let data;
    if lw(offset + 0x1fc49b8) < 2 {
        data = lw(object + 16);
    } else {
        let rounded = (size + 15) & -16;
        data = memAlloc(rounded, 1);
        memcpy(data, lw(object + 16), rounded);
        tile = lw(a1);
    }
    let users = (tile << 2) + 0x1fc49b8;
    sw(users, lw(users) - 1);
    sceVu0CopyMatrix(sp, (lw(a1 + 16) << 6) + g::tilerotmtx);
    matrixSetTrans(sp, lwc1(a1 + 4), lwc1(a1 + 8), lwc1(a1 + 12));
    let rooms = lw(data + 4) + data;
    let volumes = lw(data + 12);
    let portals = lw(data + 8) + data;
    sw(a0 + 8, lw(data) + data);
    sw(a0, rooms);
    sw(a0 + 4, portals);
    if volumes == 0 {
        sw(a0 + 28, 0);
        sw(a0 + 12, 0);
    } else {
        let at = volumes + data;
        let count = lw(at);
        sw(a0 + 12, at + 4);
        sw(a0 + 28, count);
        if count > 0 {
            let mut i = 0;
            let mut offset = 0;
            loop {
                i += 1;
                bgVolumeTransform(lw(a0 + 12) + offset, a1, sp);
                offset += 44;
                if i >= lw(a0 + 28) {
                    break;
                }
            }
        }
    }
    sw(a0 + 16, 0);
    let mut table = lw(a0);
    let mut number = 1;
    let first = lw(table + 44);
    sw(a0 + 44, lw(a1 + 28));
    sw(a0 + 48, 11 - lw(a1 + 32));
    sw(a0 + 56, lw(a1 + 24));
    if first != 0 {
        let mut offset = number * 44;
        loop {
            let entry = offset + table;
            let relocated = lw(entry) + data;
            sw(entry, relocated);
            obRelocate(relocated, data);
            obTransform(lw(offset + lw(a0)), sp);
            obCalcBounds(lw(offset + lw(a0)), 0);
            let colour = lw(a1 + 24);
            if colour != 0 {
                obColour(lw(offset + lw(a0)), colour);
            }
            table = lw(a0);
            let room = offset + table;
            let word = lw(room + 4);
            sw(lw(room) + 12, lw(a0 + 8));
            if word != 0 {
                sw(room + 4, word + data);
            }
            let floors = lw(room + 8);
            let floors_at = floors + data;
            let next = number + 1;
            if floors != 0 {
                let mut i = 0;
                let head = lw(floors_at);
                sw(room + 8, floors_at);
                if head != 0 {
                    let mut at = floors_at;
                    loop {
                        i += 1;
                        let floor = lw(at) + data;
                        sw(at, floor);
                        bgFloorTransform(floor, sp);
                        at = (i << 2) + floors_at;
                        if lw((i << 2) + floors_at) == 0 {
                            break;
                        }
                    }
                    table = lw(a0);
                }
            }
            let room = number * 44 + table;
            let walls = lw(room + 12);
            if walls != 0 {
                let walls_at = walls + data;
                let mut i = 0;
                let head = lw(walls_at);
                sw(room + 12, walls_at);
                if head != 0 {
                    let mut at = walls_at;
                    loop {
                        i += 1;
                        let wall = lw(at) + data;
                        sw(at, wall);
                        bgWallTransform(wall, a1, sp);
                        at = (i << 2) + walls_at;
                        if lw((i << 2) + walls_at) == 0 {
                            break;
                        }
                    }
                    table = lw(a0);
                }
            }
            let room = number * 44 + table;
            let glass = lw(room + 16);
            if glass != 0 {
                let glass_at = glass + data;
                let mut i = 0;
                let head = lw(glass_at);
                sw(room + 16, glass_at);
                if head != 0 {
                    let mut at = glass_at;
                    loop {
                        i += 1;
                        let pane = lw(at) + data;
                        sw(at, pane);
                        bgGlassTransform(pane, sp);
                        at = (i << 2) + glass_at;
                        if lw((i << 2) + glass_at) == 0 {
                            break;
                        }
                    }
                    table = lw(a0);
                }
            }
            bgRoombbTransform(number * 44 + table + 20, a1);
            number = next;
            table = lw(a0);
            sw(a0 + 16, lw(a0 + 16) + 1);
            if lw(number * 44 + table) == 0 {
                break;
            }
            offset = number * 44;
        }
    }
    let portal_table = lw(a0 + 4);
    sw(a0 + 20, 0);
    let mut number = 1;
    if lw(portal_table + 4) != 0 {
        let mut at = portal_table + 4;
        let mut count = lw(a0 + 20);
        loop {
            number += 1;
            let portal = lw(at) + data;
            sw(a0 + 20, count + 1);
            sw(at, portal);
            bgPortalTransform(portal, sp);
            at = (number << 2) + lw(a0 + 4);
            if lw(at) == 0 {
                break;
            }
            count = lw(a0 + 20);
        }
    }
    sw(a0 + 24, 0);
    let textures = lw(a0 + 8);
    if lw(textures) >= 0 {
        let mut entry = textures;
        let mut i = 1;
        loop {
            emLoadTextureNum(lw(entry), entry, entry);
            let textures = lw(a0 + 8);
            sw(a0 + 24, lw(a0 + 24) + 1);
            entry = (i << 4) + textures;
            if lw(entry) < 0 {
                break;
            }
            i += 1;
        }
    }
    let table = lw(a0);
    if lw(table + 44) != 0 {
        let mut room_object = lw(table + 44);
        let mut offset = 88;
        loop {
            sw(room_object + 36, 1);
            obPrecalcGfxData(room_object);
            room_object = lw(offset + lw(a0));
            offset += 44;
            if room_object == 0 {
                break;
            }
        }
    }
    sh(a0 + 52, lhu(a1 + 20));
    let pads = lw(pad_file + 16);
    if pads != 0 {
        padAddTilePads(pads, sp, a1);
    }
}

/// Play the death of the character `a0` (@ 0x2c3d40): wait until the animation of the fall has finished, then, by the kind of death, spawn the effects that appear when
/// the body disappears (the special effect of the body), restart the character in the story or delete it, play the sounds of the death animation by their times
/// and switch from the dying animations to the lying and getting up ones.
pub fn dying(a0: i32) {
    let state = lw(a0 + 352);
    enemySetMoveMode(a0, 32);
    let flags = lw(state + 2704);
    sw(state + 3004, 0);
    sw(state + 2704, flags & -3);
    let animating = curAnim(a0, 469);
    let mut kind;
    if animating == 0 {
        kind = lw(state + 8);
    } else {
        if curAnimComplete(lw(a0 + 32)) == 0 {
            return;
        }
        kind = lw(state + 8);
    }
    'b3: {
        'b4: {
            if kind == 0 || kind == 2 {
                let anim = lw(lw(a0 + 32) + 96);
                let matches = ((anim - 566) as u32) < (41 as u32) || anim == 472 || anim == 471 || anim == 510 || anim == 509;
                if matches {
                    if curAnimComplete(lw(a0 + 32)) != 0 {
                        if isRoomVisibleByAnyPlayer(lw(a0 + 12)) != 0 {
                            let spawned = lw(state + 0x11b0);
                            if spawned != 1 {
                                if spawned != 0 {
                                    kind = lw(state + 8);
                                    break 'b3;
                                }
                                specialfxNewSpawn(5, a0 + 48, -1, state, 0);
                                sw(state + 0x11b0, 1);
                                break 'b4;
                            }
                        }
                        let mode = lw(g::curgameparms + 72);
                        if mode <= 0 {
                            break 'b4;
                        }
                        if mode < 6 {
                            let flags = lw(a0 + 16);
                            sw(state + 2892, 0);
                            let f12 = lwc1(state + 2892);
                            sw(a0 + 16, flags & -0xf000001 | 32);
                            let old = lw(state + 2716);
                            sw(state + 2716, 32);
                            sw(state + 2720, old);
                            setAnim(a0, 3, f12, 2.0_f32);
                            break 'b3;
                        }
                        if mode >= 8 {
                            break 'b3;
                        }
                        enemyDeleteNPC(a0);
                        return;
                    }
                    kind = lw(state + 8);
                }
            }
            if kind != 0 {
                if kind != 2 {
                    break 'b3;
                }
            }
            let model = lw(a0 + 32);
            if ((lw(model + 96) - 566) as u32) >= (41 as u32) {
                break 'b3;
            }
            if curAnimComplete(model) != 0 {
                kind = lw(state + 8);
                break 'b3;
            }
            let model = lw(a0 + 32);
            sw(a0 + 16, lw(a0 + 16) & -33);
            let animation = lw(model + 96);
            let index = if lw(lw(model + 88) + 12) != 0 { animation - 587 } else { animation - 566 };
            let clamped = if -1 >= index { 0 } else { index };
            let slot = (if clamped >= 20 { 19 } else { clamped }) << 2;
            if lwc1(slot + g::deathSoundTime) < lwc1(model + 100) {
                let frames = running_x();
                if lw(state + 2728) != 99 {
                    if frames > 0 {
                        soundStart(lw(slot + g::deathSound), a0 + 48);
                    }
                    sw(state + 2728, 99);
                }
            }
        }
        kind = lw(state + 8);
    }
    if kind != 1 {
        return;
    }
    'first: {
        if curAnim(a0, 524) == 0 {
            if curAnim(a0, 525) == 0 {
                break 'first;
            }
        }
        if curAnimComplete(lw(a0 + 32)) != 0 {
            sw(a0 + 16, lw(a0 + 16) | 32);
            let mut mode = 0x10000;
            let old;
            if (lw(state + 2712) & 1024) == 0 {
                old = lw(state + 2716);
                mode = 8;
            } else {
                old = lw(state + 2716);
            }
            sw(state + 2716, mode);
            sw(state + 2720, old);
            sw(state + 2892, 0);
            return;
        }
    }
    'second: {
        if curAnim(a0, 524) == 0 {
            if curAnim(a0, 525) == 0 {
                break 'second;
            }
        }
        if curAnimComplete(lw(a0 + 32)) == 0 {
            sw(a0 + 16, lw(a0 + 16) | 32);
            return;
        }
    }
    'third: {
        if curAnim(a0, 577) == 0 {
            if curAnim(a0, 579) == 0 {
                break 'third;
            }
        }
        if curAnimComplete(lw(a0 + 32)) != 0 {
            if (lw(a0 + 16) & 0x10000000) != 0 {
                let visible = isRoomVisibleByAnyPlayer(lw(a0 + 12));
                let flags = lw(a0 + 16) & -33;
                sw(a0 + 16, flags);
                if visible != 0 {
                    let spawned = lw(state + 0x11b0);
                    if spawned != 1 {
                        if spawned != 0 {
                            return;
                        }
                        specialfxNewSpawn(5, a0 + 48, -1, state, 0);
                        sw(state + 0x11b0, 1);
                        return;
                    }
                }
                let params = g::curgameparms;
                if lw(params + 72) == 3 {
                    if (lw(params + 80) & 32) != 0 {
                        sw(state + 2892, 0);
                        let f12 = lwc1(state + 2892);
                        sw(a0 + 16, flags & -0xf000001 | 32);
                        let old = lw(state + 2716);
                        sw(state + 2716, 32);
                        sw(state + 2720, old);
                        setAnim(a0, 3, f12, 2.0_f32);
                        return;
                    }
                }
                enemyDeleteNPC(a0);
                return;
            }
        }
    }
    if curAnim(a0, 577) == 0 {
        if curAnim(a0, 579) == 0 {
            return;
        }
    }
    if curAnimComplete(lw(a0 + 32)) == 0 {
        let flags = lw(a0 + 16);
        let value = if (flags & 0x10000000) == 0 { flags | 32 } else { flags & -33 };
        sw(a0 + 16, value);
        let model = lw(a0 + 32);
        let animation = lw(model + 96);
        let index = if lw(lw(model + 88) + 12) != 0 { animation - 587 } else { animation - 566 };
        let clamped = if -1 >= index { 0 } else { index };
        let slot = (if clamped >= 20 { 19 } else { clamped }) << 2;
        if lwc1(slot + g::deathSoundTime) < lwc1(model + 100) {
            let frames = running_x();
            if lw(state + 2728) != 99 {
                if frames > 0 {
                    soundStart(lw(slot + g::deathSound), a0 + 48);
                }
                sw(state + 2728, 99);
            }
        }
    }
}

/// Draw the particles (@ 0x2a1908) of the current player: the groups that the player can see are drawn in two passes, the opaque kinds first (smoke, bursts, streams,
/// trails, glass and manual groups of the first blend) and then the additive ones (sparks, tracers, pulses, explosions, rays and the gun sparks).
pub fn particleGfx() {
    let frame = Frame::new(144);
    let sp = frame.sp();
    let player = lw(g::currentplayer);
    sceVu0MulMatrix(sp, lw(player + 1760), lw(player + 1764));
    let old_zb = dlSetZB(2);
    let number = lw(lw(g::currentplayer));
    let mut viewport = 0;
    if number == 1 {
        viewport = 16;
    } else if number >= 2 {
        if number == 2 {
            viewport = 32;
        } else if number == 3 {
            viewport = 64;
        }
    } else if number == 0 {
        viewport = 8;
    }
    dlSetBlend(1);
    let mut i = 0;
    loop {
        let group = lw(g::pgroups) + i * 0x17a0;
        if lw(group) != -1 {
            if particleGroupIsVisible(group, viewport) != 0 {
                let kind = lw(group);
                if (kind as u32) < (16 as u32) {
                    let info = lw(lw(group + 4) * 104 + (g::particleinfos + 4));
                    match kind {
                        2 => {
                            dlSelectTextureKick(lw(0x3b0168));
                            dlSetDitherMatrix(2);
                            particleSmokeGfx(group, g::scratchbuffer + 687, sp);
                        }
                        4 => {
                            if info == 1 {
                                particleBurstGfx(group, sp);
                            }
                        }
                        5 => {
                            if info == 1 {
                                particleStreamGfx(group, sp);
                            }
                        }
                        6 => {
                            if info == 1 {
                                particleTrailGfx(group, sp);
                            }
                        }
                        7 => particleGlassGfx(group),
                        11 => {
                            if info == 1 {
                                particleManualGfx(group, sp);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        i += 1;
        if i >= 100 {
            break;
        }
    }
    dlSetBlend(2);
    let mut i = 0;
    loop {
        let group = lw(g::pgroups) + i * 0x17a0;
        if lw(group) != -1 {
            if particleGroupIsVisible(group, viewport) != 0 {
                let kind = lw(group);
                if (kind as u32) < (16 as u32) {
                    let info = lw(lw(group + 4) * 104 + (g::particleinfos + 4));
                    match kind {
                        0 => {
                            dlSelectTextureKick(lw(0x3b0154));
                            dlSetDitherMatrix(2);
                            particleSparkGroupGfx(group, 0xffffff, sp, lwc1(0x3ad58c), lwc1(0x3ad590));
                        }
                        1 => {
                            dlSelectTextureKick(lw(0x3b0158));
                            dlSetDitherMatrix(0);
                            particleSpriteSparkGroupGfx(group, lw((lw(group + 928) << 3) + g::pulsedefs), sp);
                        }
                        3 => {
                            let mut skip = false;
                            if !(0.0_f32 < lwc1(group + 12)) {
                                let word = lw(group + 48);
                                if (word & (word - 1)) != 0 {
                                    skip = true;
                                }
                            }
                            if !skip {
                                dlSelectTextureKick(lw(0x3b0160));
                                dlSetDitherMatrix(2);
                                particleTracerGfx(group, 0xffffff, sp);
                            }
                        }
                        4 => {
                            if info == 2 {
                                particleBurstGfx(group, sp);
                            }
                        }
                        5 => {
                            if info == 2 {
                                particleStreamGfx(group, sp);
                            }
                        }
                        6 => {
                            if info == 2 {
                                particleTrailGfx(group, sp);
                            }
                        }
                        11 => {
                            if info == 2 {
                                particleManualGfx(group, sp);
                            }
                        }
                        12 => {
                            dlSelectTextureKick(lw(0x3b015c));
                            dlSetDitherMatrix(0);
                            particleExplosionGfx(group, lw((lw(group + 928) << 3) + g::explosiondefs), sp);
                        }
                        13 => {
                            dlSelectTextureKick(lw(0x3b0164));
                            dlSetDitherMatrix(0);
                            particleRayGfx(group, g::scratchbuffer + 119472, sp);
                        }
                        14 => {
                            let pulse;
                            let mut skip = false;
                            if 0.0_f32 < lwc1(group + 12) {
                                pulse = lw(group + 928);
                            } else {
                                let word = lw(group + 48);
                                if (word & (word - 1)) != 0 {
                                    skip = true;
                                }
                                pulse = lw(group + 928);
                            }
                            if !skip {
                                dlSelectTextureKick(lw((lw((pulse << 3) + (g::pulsedefs + 4)) << 4) + (g::particletextures + 12)));
                                dlSetDitherMatrix(0);
                                particlePulseGfx(group, sp);
                            }
                        }
                        15 => {
                            dlSelectTextureKick(lw(0x3b016c));
                            dlSetDitherMatrix(2);
                            particleGunSparkGroupGfx(group, 0xffffff, sp, lwc1(0x3ad594), lwc1(0x3ad598));
                        }
                        _ => {}
                    }
                }
            }
        }
        i += 1;
        if i >= 100 {
            break;
        }
    }
    dlSetZB(old_zb);
}

/// Tick of the on screen keyboard (@ 0x22f820): the grid of characters (11 columns, 5 rows plus a row of special keys) is moved with the directions (the cursor skips
/// over cells that hold the same character), accept adds the character under the cursor to the text (wrapping the line at a space when it would be too wide), and the
/// special keys switch the character table, delete, or finish. Back deletes the last character, the other button cancels.
pub fn stringeditTick(a0: i32) {
    let frame = Frame::new(96);
    let sp = frame.sp();
    stringeditSet(a0);
    let mut front = lw(g::front);
    if (lw(front + 12) & 60) != 0 {
        soundStart(141, 0);
        front = lw(g::front);
    }
    let buttons = lw(front + 12);
    let textbox = lw(g::textbox);
    'edit: {
        if (buttons & 16) != 0 {
            let table = lw(textbox + 28) + lw(textbox + 44) * 11;
            let wanted = lbu(table + lw(textbox + 40));
            let mut column = lw(textbox + 40);
            loop {
                let next = column - 1;
                sw(textbox + 40, next);
                if next < 0 {
                    sw(textbox + 40, 10);
                }
                if lbu(table + lw(textbox + 40)) != wanted {
                    break 'edit;
                }
                column = lw(textbox + 40);
            }
        }
        if (buttons & 32) != 0 {
            let table = lw(textbox + 28) + lw(textbox + 44) * 11;
            let wanted = lbu(table + lw(textbox + 40));
            let mut column = lw(textbox + 40);
            loop {
                let next = column + 1;
                sw(textbox + 40, next);
                if next >= 11 {
                    sw(textbox + 40, 0);
                }
                if lbu(table + lw(textbox + 40)) != wanted {
                    break 'edit;
                }
                column = lw(textbox + 40);
            }
        }
        if (buttons & 4) != 0 {
            let textbox = lw(g::textbox);
            let row = lw(textbox + 44) - 1;
            sw(textbox + 44, row);
            if row < 0 {
                sw(textbox + 44, 4);
            }
            break 'edit;
        }
        if (buttons & 8) != 0 {
            let row = lw(textbox + 44) + 1;
            sw(textbox + 44, row);
            if row >= 5 {
                sw(textbox + 44, 0);
            }
            break 'edit;
        }
        let length;
        let text_box;
        'delete: {
            let pressed = joyNewBut(a0);
            text_box = lw(g::textbox);
            if (pressed & 128) == 0 {
                let pressed = joyNewBut(a0);
                let text_box = lw(g::textbox);
                if (pressed & 64) == 0 {
                    let pressed = joyNewBut(a0);
                    let current = lw(g::textbox);
                    if (pressed & 16) == 0 {
                        break 'edit;
                    }
                    sw(current + 36, 1);
                    soundStart(142, 0);
                    break 'edit;
                }
                let sound;
                'key: {
                    let cell = lw(text_box + 44) * 11 + lw(text_box + 40);
                    if (cell as u32) >= (44 as u32) {
                        let characters = lw(text_box + 28);
                        let key = lbu(characters + cell);
                        if key == 254 {
                            length = lw(text_box + 24);
                            break 'delete;
                        }
                        if key != 253 {
                            if key != 252 {
                                if key != 255 {
                                    break 'edit;
                                }
                                let text = lw(text_box + 48);
                                let mut size = 0;
                                if lbu(text) != 0 {
                                    loop {
                                        size += 1;
                                        if lbu(text + size) == 0 {
                                            break;
                                        }
                                    }
                                }
                                let mut text_box = text_box;
                                'trim: {
                                    if size > 0 {
                                        if lbu(size + lw(text_box + 48) - 1) == 32 {
                                            loop {
                                                let end = size + lw(text_box + 48);
                                                size -= 1;
                                                sb(end - 1, 0);
                                                if size <= 0 {
                                                    break;
                                                }
                                                text_box = lw(g::textbox);
                                                if lbu(size + lw(text_box + 48) - 1) != 32 {
                                                    break 'trim;
                                                }
                                            }
                                            text_box = lw(g::textbox);
                                        }
                                    }
                                }
                                if size != 0 {
                                    sw(text_box + 32, 1);
                                } else {
                                    sw(text_box + 36, 1);
                                }
                                sound = 143;
                                break 'key;
                            }
                            let first = g::chartab1;
                            if characters != first {
                                sw(text_box + 28, first);
                            } else {
                                sw(text_box + 28, g::chartab2);
                            }
                            sound = 141;
                            break 'key;
                        }
                        sb(sp, 32);
                    } else {
                        sb(sp, lbu(lw(text_box + 28) + cell));
                    }
                    sb(sp + 1, 0);
                    let mut fitted = 1;
                    if lw(text_box + 24) < lw(text_box + 52) {
                        let mut line_start = lw(text_box + 48);
                        let mut lines = 1;
                        let mut index = 0;
                        let character;
                        let used;
                        if lbu(line_start) == 0 {
                            character = lbu(sp);
                            used = lw(text_box + 24);
                        } else {
                            character = lbu(sp);
                            let mut text = lw(text_box + 48);
                            loop {
                                let at = text + index;
                                index += 1;
                                if lbu(at) == 10 {
                                    line_start = at + 1;
                                    lines += 1;
                                }
                                if lbu(text + index) == 0 {
                                    break;
                                }
                                text = lw(text_box + 48);
                            }
                            used = lw(text_box + 24);
                        }
                        let mut old_font = 0;
                        'placed: {
                            sb(lw(text_box + 48) + used, character);
                            sw(text_box + 24, used + 1);
                            let current = lw(g::textbox);
                            sb(lw(current + 48) + lw(current + 24), 0);
                            old_font = textSelect(1);
                            let width = textWidth(line_start);
                            let current = lw(g::textbox);
                            if lw(current + 20) < width {
                                'cannot: {
                                    let typed = lbu(sp);
                                    if lines < lw(current + 16) {
                                        if typed != 32 {
                                            let mut walk;
                                            let mut found_back;
                                            let limit;
                                            'wrap: {
                                                'search: {
                                                    let length = lw(current + 24);
                                                    walk = lw(current + 48) + (length - 1);
                                                    found_back = ((walk as u32) < (line_start as u32)) as i32;
                                                    if found_back != 0 {
                                                        limit = lw(current + 52);
                                                    } else {
                                                        'space: {
                                                            if lbu(walk) != 32 {
                                                                walk -= 1;
                                                                loop {
                                                                    found_back = ((walk as u32) < (line_start as u32)) as i32;
                                                                    if found_back != 0 {
                                                                        break;
                                                                    }
                                                                    if lbu(walk) == 32 {
                                                                        break 'space;
                                                                    }
                                                                    walk -= 1;
                                                                }
                                                                limit = lw(current + 52);
                                                                break 'search;
                                                            }
                                                        }
                                                        if found_back == 0 {
                                                            break 'wrap;
                                                        }
                                                        limit = lw(current + 52);
                                                    }
                                                }
                                                let end = lw(current + 24);
                                                if end < limit {
                                                    sb(end + lw(current + 48) - 1, 10);
                                                    let box_now = lw(g::textbox);
                                                    let at = lw(box_now + 24);
                                                    sb(lw(box_now + 48) + at, lbu(sp));
                                                    sw(box_now + 24, at + 1);
                                                    let box_now = lw(g::textbox);
                                                    sb(lw(box_now + 48) + lw(box_now + 24), 0);
                                                    break 'placed;
                                                }
                                                if found_back != 0 {
                                                    break 'cannot;
                                                }
                                            }
                                            sb(walk, 10);
                                            break 'placed;
                                        }
                                        sb(lw(current + 24) + lw(current + 48) - 1, 10);
                                        break 'placed;
                                    }
                                }
                                fitted = 0;
                            }
                        }
                        if fitted == 0 {
                            let current = lw(g::textbox);
                            let length = lw(current + 24) - 1;
                            let at = lw(current + 48) + length;
                            sw(current + 24, length);
                            sb(at, 0);
                            soundStart(142, 0);
                        } else {
                            soundStart(140, 0);
                        }
                        textSelect(old_font);
                        break 'edit;
                    }
                    sound = 142;
                }
                soundStart(sound, 0);
                break 'edit;
            }
            length = lw(text_box + 24);
        }
        let text_box = lw(g::textbox);
        let _ = text_box;
        let current = lw(g::textbox);
        if length > 0 {
            let text = lw(current + 48);
            sw(current + 24, length - 1);
            sb(text + length - 1, 0);
            soundStart(142, 0);
        }
    }
}

/// Tick of the pad action page of the map maker (@ 0x24f520): the pad under the cursor (`aimpadinfo`) can have one item of an action attached (start, drop, health, gun,
/// armour): a press of the left button reads the pad's item into the page's values and a press of accept sets the item of the page's picked kind, after checking the
/// group limits, the map cost and the file size; the first item deletes the pad's item.
pub fn padact_pageTick(a0: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    sh(g::padactcuritem, lhu(a0 + 4));
    if (joyNewBut(0) & 16) != 0 {
        return;
    }
    let menu = g::padact_menu;
    'b2: {
        'b3: {
            let pressed = joyNewBut(0);
            let pad = lw(g::aimpadinfo);
            if (pressed & 128) == 0 {
                if (joyNewBut(0) & 64) != 0 {
                    'b4: {
                        let item = lhu(a0 + 4);
                        let kind = item - 2;
                        if item != 1 {
                            let mut value = -1;
                            if (kind as u32) < (7 as u32) {
                                match kind {
                                    0 => value = lw(g::gv_padactstart),
                                    1 => value = lw(g::gv_padactdrop) + 5,
                                    2 => value = lw(g::gv_padacthealth) + 15,
                                    3 => value = 14,
                                    4 => value = 15,
                                    5 => value = lw(g::gv_padactarmour) + 17,
                                    _ => value = lw(g::gv_padactgun) + 8,
                                }
                            }
                            let pad_now = lw(g::aimpadinfo);
                            if value == -1 {
                                break 'b3;
                            }
                            let existing = objFindItem(lw(g::zoomtileobj), lw(lw(pad_now)), -1);
                            let mut old_kind = -1;
                            if existing != 0 {
                                old_kind = lhu(existing + 2) as i32;
                            }
                            memset(sp, 0, 40);
                            let mut object = lw(g::mapobjects);
                            if object != 0 {
                                loop {
                                    mmItemGroupCount(lw(object + 52), sp);
                                    object = lw(object + 56);
                                    if object == 0 {
                                        break;
                                    }
                                }
                            }
                            let defs = g::itemdefs + 4;
                            let counter = sp + (lw((value << 4) + defs) << 2);
                            sw(counter, lw(counter) + 1);
                            if old_kind != 0 {
                                let counter = sp + (lw((old_kind << 4) + defs) << 2);
                                sw(counter, lw(counter) - 1);
                            }
                            'limits: {
                                let mut group = 0;
                                let mut limits = g::itemgrouplimits;
                                let mut counts = sp;
                                let mut limit_at = limits;
                                loop {
                                    let over = (lw(limits) < lw(counts)) as i32;
                                    counts += 4;
                                    if over != 0 {
                                        break;
                                    }
                                    group += 1;
                                    limits += 4;
                                    limit_at += 4;
                                    if group >= 10 {
                                        break 'limits;
                                    }
                                }
                                value = -1;
                                let strings = lw((lw(g::cur_language) << 2) + g::languages);
                                let name = mmItemGroupName(group);
                                let format = lw(strings + 3488);
                                let text = vafmt(&[0x3a2240, format as i64, lw(limit_at) as i64, name as i64]);
                                mmStartMessage(text, 0x6464647f, 0);
                            }
                            let flags = lw(g::editflags);
                            if value == -1 {
                                break 'b3;
                            }
                            if (flags & 0x2000) == 0 {
                                mmBackupMap();
                                sw(g::editflags, lw(g::editflags) | 0x2000);
                            }
                            let cost = mmMapCost();
                            let mut language = lw(g::cur_language);
                            if (cost + 1) < mm_cost_limit() {
                                let size = mmCalcFileSize();
                                let pad_now = lw(g::aimpadinfo);
                                if ((size + 4) as u32) < (0x1ffd as u32) {
                                    let found = objFindItem(lw(g::zoomtileobj), lw(lw(pad_now)), -1);
                                    let object = lw(g::zoomtileobj);
                                    if found != 0 {
                                        mmFreeItem(object, found);
                                    }
                                    let made = mmNewItem(lw(g::zoomtileobj), lhu(lw(lw(g::aimpadinfo))), value);
                                    let language = lw(g::cur_language);
                                    if made == 0 {
                                        mmStartMessage(lw(lw((language << 2) + g::languages) + 3484), 0x7f7f3c7f, 0);
                                        break 'b2;
                                    }
                                    break 'b4;
                                }
                                language = lw(g::cur_language);
                            }
                            mmStartMessage(lw(lw((language << 2) + g::languages) + 3484), 0x7f7f3c7f, 0);
                            break 'b2;
                        }
                        let found = objFindItem(lw(g::zoomtileobj), lw(lw(lw(g::aimpadinfo))), -1);
                        let flags = lw(g::editflags);
                        if found == 0 {
                            break 'b3;
                        }
                        let mut object = lw(g::zoomtileobj);
                        if (flags & 0x2000) == 0 {
                            mmBackupMap();
                            sw(g::editflags, lw(g::editflags) | 0x2000);
                            object = lw(g::zoomtileobj);
                        }
                        mmFreeItem(object, found);
                    }
                    mm_setpadicon(lw(g::aimpadinfo));
                    break 'b2;
                }
            } else {
                if pad != 0 {
                    let found = objFindItem(lw(g::zoomtileobj), lw(lw(pad)), -1);
                    if found != 0 {
                        let kind = lhu(found + 2);
                        if (kind as u32) < (20 as u32) {
                            let page_item;
                            match kind {
                                0 | 1 | 2 | 3 | 4 => {
                                    sh(a0 + 4, 2);
                                    gvSetValue(g::gv_padactstart, lhu(found + 2));
                                    break 'b2;
                                }
                                5 | 6 | 7 | 8 => {
                                    sh(a0 + 4, 3);
                                    gvSetValue(g::gv_padactdrop, lhu(found + 2) - 5);
                                    break 'b2;
                                }
                                9 | 10 | 11 | 12 | 13 => {
                                    sh(a0 + 4, 8);
                                    gvSetValue(g::gv_padactgun, lhu(found + 2) - 8);
                                    break 'b2;
                                }
                                14 => {
                                    page_item = 5;
                                }
                                15 => {
                                    page_item = 6;
                                }
                                16 | 17 => {
                                    sh(a0 + 4, 4);
                                    gvSetValue(g::gv_padacthealth, lhu(found + 2) - 15);
                                    break 'b2;
                                }
                                _ => {
                                    sh(a0 + 4, 7);
                                    gvSetValue(g::gv_padactarmour, lhu(found + 2) - 17);
                                    break 'b2;
                                }
                            }
                            sh(a0 + 4, page_item);
                        }
                    }
                }
            }
        }
    }
    let item = lhu(g::padactcuritem);
    let _ = item;
    menutick_24c3a8(a0, menu);
    let previous = lhu(g::padactcuritem);
    let list_row;
    'scroll: {
        if lhu(a0 + 4) != previous {
            let found = menuitem(menu, previous);
            let current = lhu(a0 + 4);
            let offset;
            if current != lhu(found + 16) {
                let other = lhu(found + 18);
                if current != other {
                    list_row = lw(a0 + 12);
                    break 'scroll;
                }
                offset = 10;
            } else {
                offset = -10;
            }
            sw(g::rolloffset, offset);
        }
        list_row = lw(a0 + 12);
    }
    let made = rollmenumake(lw(a0 + 8) + list_row * 24, g::padact_menu, lhu(a0 + 4), 0);
    sw(a0 + 12, lw(a0 + 12) + made);
}

/// Tick of the arcade options page (@ 0x239088): the menu items of the picked game mode are copied into the options page (with their neighbours linked), the items that the
/// team play setting hides are greyed out, the weapons and character set items open their pages, the start item checks that the game can go ahead and the limits
/// get their texts.
pub fn arcadeoptions_pageTick(a0: i32) {
    pageSet(lw(lw((lw(g::cur_language) << 2) + g::languages) + 496), g::arcadeoptions_page);
    frontfxTick(0);
    botsetInit(chrset_read());
    checkAllowedTeams();
    let language = lw(g::cur_language);
    let items = opt_items_or(g::i0410) + 32;
    let mut source = 0;
    let mut count = 0;
    let mut text = 0;
    let mut at = items;
    let mut left = 9;
    loop {
        copy_menu_item(at, g::null_menuitem);
        left -= 1;
        at += 32;
        if left < 0 {
            break;
        }
    }
    let mode = lw(0x1fc1228) - 1;
    if (mode as u32) < (6 as u32) {
        match mode {
            0 => {
                count = 8;
                source = g::bagtag_menuitems;
            }
            1 => {
                count = 6;
                source = g::ctf_menuitems;
            }
            2 => {
                count = 10;
                source = g::deathmatch_menuitems;
            }
            3 => {
                count = 4;
                source = g::escort_menuitems;
            }
            4 => {
                count = 6;
                source = g::knockout_menuitems;
            }
            _ => {
                count = 3;
                source = g::laststand_menuitems;
            }
        }
    }
    let mut i = 0;
    let total = count + 1;
    if count != 0 {
        let last = count - 1;
        let mut offset = i << 5;
        loop {
            let dst = offset + items;
            let src = offset + source;
            copy_menu_item(dst, src);
            sh(dst + 8, i + 2);
            let next = i + 1;
            if i <= 0 {
                sh(dst + 16, 14);
            } else {
                sh(dst + 16, next);
            }
            let item = offset + items;
            let down = if i >= last { 12 } else { i + 3 };
            sh(item + 18, down);
            i = next;
            offset = i << 5;
            if i >= count {
                break;
            }
        }
    }
    let state = 0x1fc11f8;
    let mode_data = lw(state + 48);
    let page = opt_items_or(g::i0410);
    sh(page + 368, total);
    let mut cursor = page + 44;
    let teamplay = lw(g::gv_teamplay);
    let mut j = 1;
    sw(opt_items_or(g::i0410), lw((lh(mode_data * 12 + g::gamemodedata) << 2) + lw((language << 2) + g::languages)));
    'teams: {
        loop {
            j += 1;
            if lw(cursor) == g::gv_teamplay {
                break;
            }
            cursor += 32;
            if j >= 11 {
                break 'teams;
            }
        }
        let mut item = page + 32;
        let mut left = 9;
        loop {
            'item: {
                let value = lw(item + 12);
                if value != g::gv_useteamstarts {
                    if value != g::gv_friendfire {
                        left -= 1;
                        break 'item;
                    }
                }
                let flags = lhu(item + 10);
                let flags = if teamplay == 0 { flags | 0x4040 } else { flags & 0xbfbf };
                sh(item + 10, flags);
                left -= 1;
            }
            item += 32;
            if left < 0 {
                break 'teams;
            }
        }
    }
    let page = opt_items_or(g::i0410);
    if language != 0 {
        let flags = lhu(page + 394) & 0xfeff;
        sh(page + 362, lhu(page + 362) & 0xfeff);
        sh(page + 394, flags);
    }
    let page = opt_items_or(g::i0410);
    sh(page + 358, 111);
    sh(page + 390, 125);
    sh(page + 422, 146);
    if count == 10 {
        sh(page + 358, 118);
        sh(page + 390, 132);
        sh(page + 422, 150);
    }
    let (flags_a, flags_b);
    if lw(0x1fc1228) != 6 {
        flags_a = lhu(page + 394) & 0xbdff;
        flags_b = lhu(page + 362) & 0xbdff;
    } else {
        if ((lhu(a0 + 4) - 12) as u32) < (2 as u32) {
            sh(a0 + 4, 14);
        }
        flags_a = lhu(page + 394) | 0x4200;
        flags_b = lhu(page + 362) | 0x4200;
    }
    sh(page + 394, flags_a);
    sh(page + 362, flags_b);
    let current = lhu(a0 + 4);
    if current >= 12 {
        if arcade_edit_row(current) {
            text = lw(lw((language << 2) + g::languages) + 924);
        } else if current == 14 {
            text = lw(lw((language << 2) + g::languages) + 512);
        }
    }
    let page = opt_items_or(g::i0410);
    let options = lw(state + 48);
    sw(page + 480, text);
    storeGameModeOptions(options);
    let mut go = false;
    let mut next_page = 0;
    'buttons: {
        let pressed = joyNewBut(lw(lw(g::front)));
        if (pressed & 64) == 0 {
            let pressed = joyNewBut(lw(lw(g::front)));
            if (pressed & 2048) == 0 {
                break 'buttons;
            }
            go = true;
        } else {
            let current = lhu(a0 + 4);
            if current == 13 {
                next_page = g::chrsets_page;
            } else if current < 14 {
                if current == 12 {
                    next_page = g::weapons_page;
                } else {
                    break 'buttons;
                }
            } else if current != 14 {
                break 'buttons;
            } else {
                go = true;
            }
        }
        if go {
            if checkOkToGo() == 0 {
                soundStart(142, 0);
                break 'buttons;
            }
            checkNoGuns();
            setStates(1);
            next_page = g::characterselect_page;
        }
        frontPageSet(0, next_page);
        pageIn(a0);
    }
    let current = lhu(a0 + 4);
    if current == 12 || current == 13 {
        let front = lw(g::front);
        let buttons = lw(front + 12);
        let mut front = front;
        let mut applied = true;
        if (buttons & 16) == 0 {
            if (buttons & 32) == 0 {
                applied = false;
            } else {
                if current == 12 {
                    weaponSetInc();
                } else {
                    chrsetInc();
                }
                front = lw(g::front);
            }
        } else {
            if current == 12 {
                weaponSetDec();
            } else {
                chrsetDec();
            }
            front = lw(g::front);
        }
        let _ = applied;
        sw(front + 12, lw(front + 12) & -49);
    }
    let menu = av_opt_ready_or(g::arcadeoptions_menu);
    menutick_230238(a0, menu);
    let made = std_menumake(lw(a0 + 8), menu);
    sw(a0 + 12, made);
    let time = if lw(g::gv_timelimit) <= 0 { 0 } else { lw(lw((lw(g::cur_language) << 2) + g::languages) + 916) };
    let score = lw(g::gv_scorelimit);
    sw(g::gv_timelimit + 24, time);
    let score_text = if score <= 0 { 0 } else { lw(lw((lw(g::cur_language) << 2) + g::languages) + 920) };
    let kills = lw(g::gv_killlimit);
    sw(g::gv_scorelimit + 24, score_text);
    let kill_text = if kills <= 0 { 0 } else { lw(lw((lw(g::cur_language) << 2) + g::languages) + 920) };
    sw(g::gv_killlimit + 24, kill_text);
}

/// The view of a player who is dead (@ 0x280b60): the eye drops towards the floor, and the camera looks at the prop that killed the player (or, after a while, at the
/// other characters in turn: every 8 frames one of them is tested and the one most in front of the current view with a clear line becomes the next to look at). The view
/// direction is turned towards the target at a rate that depends on the biggest component of the change, and normalised.
pub fn playerDeathView() {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let mut player = lw(g::currentplayer);
    let mut change = 0;
    let mut turned = 0;
    let rest = lwc1(player + 112);
    let target_y = lwc1(player + 104);
    let top = ps2float::add(target_y, lwc1(player + 124));
    let floor = lwc1(player + 108);
    let body = lw(player + 384);
    let data = lw(body + 352);
    sw((lw(player) << 2) + g::ipblur, 0);
    let eye = ps2float::add(rest, (ps2float::sub(target_y, rest)).pdiv(35.0_f32));
    swc1(player + 112, eye);
    if ps2float::sub(top, eye) < floor {
        swc1(player + 112, ps2float::sub(top, floor));
    }
    let z = lwc1(player + 148);
    let looking = lw(player + 1808);
    let height = ps2float::add(ps2float::add(lwc1(player + 144), lwc1(player + 116)), lwc1(player + 124));
    swc1(player + 152, lwc1(player + 140));
    swc1(player + 160, z);
    swc1(player + 156, height);
    if looking == 0 {
        sw(player + 1792, lw(player + 1792) + lw(g::updateframesi));
    }
    'killer: {
        let killer = lw(data + 256);
        if killer != 0 {
            if killer != body {
                let to_x;
                'direction: {
                    let eye_x = lwc1(player + 152);
                    let dx = ps2float::sub(lwc1(killer + 48), eye_x);
                    swc1(sp + 32, dx);
                    let dy = ps2float::sub(ps2float::add(lwc1(killer + 52), 1.5_f32), lwc1(player + 156));
                    swc1(sp + 36, dy);
                    let dz = ps2float::sub(lwc1(killer + 56), lwc1(player + 160));
                    swc1(sp + 40, dz);
                    if dx == 0.0_f32 {
                        if !(dy == 0.0_f32) {
                            to_x = lwc1(sp + 32);
                            break 'direction;
                        }
                        if dz == 0.0_f32 {
                            break 'killer;
                        }
                    }
                    to_x = lwc1(sp + 32);
                }
                turned = 1;
                let to_y = lwc1(sp + 36);
                let to_z = lwc1(sp + 40);
                let looking = lw(player + 1808);
                swc1(sp + 16, to_x);
                swc1(sp + 20, to_y);
                swc1(sp + 24, to_z);
                if looking == 0 {
                    if 24.0_f32 < lw(player + 1792) as f32 {
                        let killer_data = lw(killer + 352);
                        sw(player + 1792, 0);
                        change = 1;
                        if (lw(killer_data + 2708) & 32) == 0 {
                            sw(sp, 0);
                            sw(sp + 8, 0);
                            let hit = bgLineTest(lw(player + 796), player + 152, sp + 16, 0, 0, 0, 0, -3);
                            player = lw(g::currentplayer);
                            change = if hit != 0 { turned } else { 0 };
                        }
                    }
                }
            } else if lw(player + 1808) == 0 {
                if 24.0_f32 < lw(player + 1792) as f32 {
                    sw(player + 1792, 0);
                    change = 1;
                }
            }
        }
    }
    if change != 0 {
        let best = lwc1(g::nexttestto + 4);
        sw(player + 1808, 1);
        swc1(player + 1816, best);
        sw(player + 1792, 0);
        sw(player + 1812, 0);
    }
    if lw(player + 1808) != 0 {
        'search: {
            let mut chars = lw(g::numactivechars);
            let mut players = lw(g::lvnumplayers);
            let mut i = 0;
            if (chars + players) > 0 {
                let mut phase = lw(player + 1792);
                loop {
                    if (i & 7) != phase {
                        i += 1;
                    } else {
                        let chr = lw(g::chrinfo) + i * 0x1210;
                        if (lw(chr + 2708) & 32) != 0 {
                            i += 1;
                        } else {
                            let mode = lw(chr + 2716);
                            if (mode & 0x2010) != 0 {
                                i += 1;
                            } else if mode == 0 {
                                i += 1;
                            } else {
                                'candidate: {
                                    let prop = lw(chr + 3020);
                                    let dx = ps2float::sub(lwc1(prop + 48), lwc1(player + 152));
                                    swc1(sp + 32, dx);
                                    swc1(sp + 48, dx);
                                    let dy = ps2float::sub(ps2float::add(lwc1(prop + 52), 1.5_f32), lwc1(player + 156));
                                    swc1(sp + 36, dy);
                                    swc1(sp + 52, dy);
                                    let dz = ps2float::sub(lwc1(prop + 56), lwc1(player + 160));
                                    swc1(sp + 40, dz);
                                    swc1(sp + 56, dz);
                                    let inverse = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt());
                                    let ux = ps2float::mul(lwc1(sp + 48), inverse);
                                    let uz = ps2float::mul(lwc1(sp + 56), inverse);
                                    let uy = ps2float::mul(lwc1(sp + 52), inverse);
                                    swc1(sp + 48, ux);
                                    swc1(sp + 56, uz);
                                    swc1(sp + 52, uy);
                                    let facing = ps2float::add(ps2float::add(ps2float::mul(lwc1(player + 1796), ux), ps2float::mul(lwc1(player + 1800), uy)), ps2float::mul(lwc1(player + 1804), uz));
                                    if lwc1(player + 1816) < facing {
                                        sw(sp, 0);
                                        sw(sp + 8, 0);
                                        let hit = bgLineTest(lw(player + 796), player + 152, sp + 32, 0, 0, 0, 0, -3);
                                        player = lw(g::currentplayer);
                                        if hit == 0 {
                                            chars = lw(g::numactivechars);
                                            swc1(player + 1816, facing);
                                            sw(player + 1812, prop);
                                            break 'candidate;
                                        }
                                    }
                                    chars = lw(g::numactivechars);
                                }
                                players = lw(g::lvnumplayers);
                                i += 1;
                            }
                        }
                    }
                    if i >= chars + players {
                        break 'search;
                    }
                    phase = lw(player + 1792);
                }
            }
        }
        let phase = lw(player + 1792) + 1;
        sw(player + 1792, phase);
        if phase >= 8 {
            let chosen = lw(player + 1812);
            sw(player + 1792, 0);
            sw(player + 1808, 0);
            if chosen != 0 {
                sw(data + 256, chosen);
            }
        }
    }
    'turn: {
        let frames = lw(g::updateframesi);
        if turned != 0 {
            let mut step = 0;
            if frames > 0 {
                let speed = lwc1(0x3acfec);
                let mut tx = lwc1(sp + 16);
                let mut frames = frames;
                loop {
                    let ty = lwc1(sp + 20);
                    let tz = lwc1(sp + 24);
                    let length = ps2float::add(ps2float::add(ps2float::mul(tx, tx), ps2float::mul(ty, ty)), ps2float::mul(tz, tz)).abs().psqrt();
                    let inverse = (1.0_f32).pdiv(length);
                    let nz = ps2float::mul(tz, inverse);
                    let ny = ps2float::mul(ty, inverse);
                    let nx = ps2float::mul(tx, inverse);
                    swc1(sp + 24, nz);
                    swc1(sp + 20, ny);
                    swc1(sp + 16, nx);
                    let mut ax = ps2float::sub(nx, lwc1(player + 1796));
                    if !(0.0_f32 <= ax) {
                        ax = -ax;
                    }
                    let mut ay = ps2float::sub(ny, lwc1(player + 1800));
                    if !(0.0_f32 <= ay) {
                        ay = -ay;
                    }
                    let mut az = ps2float::sub(nz, lwc1(player + 1804));
                    if !(0.0_f32 <= az) {
                        az = -az;
                    }
                    let rate;
                    if ay < ax && az < ax {
                        let limit = lwc1(0x3acff0);
                        if limit < ps2float::mul(ax, speed) {
                            rate = (limit).pdiv(ax);
                        } else {
                            rate = lwc1(0x3acff4);
                        }
                    } else if ax < ay && az < ay {
                        let limit = lwc1(0x3acff8);
                        if limit < ps2float::mul(ay, speed) {
                            rate = (limit).pdiv(ay);
                        } else {
                            rate = lwc1(0x3acffc);
                        }
                    } else {
                        let limit = lwc1(0x3ad000);
                        if limit < ps2float::mul(az, speed) {
                            rate = (limit).pdiv(az);
                        } else {
                            rate = lwc1(0x3ad004);
                        }
                    }
                    let current_x = lwc1(player + 1796);
                    let current_y = lwc1(player + 1800);
                    let current_z = lwc1(player + 1804);
                    let new_x = ps2float::add(current_x, ps2float::mul(rate, ps2float::sub(lwc1(sp + 16), current_x)));
                    swc1(player + 1796, new_x);
                    let new_y = ps2float::add(current_y, ps2float::mul(rate, ps2float::sub(lwc1(sp + 20), current_y)));
                    swc1(player + 1800, new_y);
                    let new_z = ps2float::add(current_z, ps2float::mul(rate, ps2float::sub(lwc1(sp + 24), current_z)));
                    swc1(player + 1804, new_z);
                    'normalise: {
                        if new_x == 0.0_f32 {
                            if new_y == 0.0_f32 && new_z == 0.0_f32 {
                                frames = lw(g::updateframesi);
                                break 'normalise;
                            }
                        }
                        let x = lwc1(player + 1796);
                        let y = lwc1(player + 1800);
                        let z = lwc1(player + 1804);
                        let inverse = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(x, x), ps2float::mul(y, y)), ps2float::mul(z, z)).abs().psqrt());
                        let nz = ps2float::mul(lwc1(player + 1804), inverse);
                        let ny = ps2float::mul(lwc1(player + 1800), inverse);
                        swc1(player + 1796, ps2float::mul(lwc1(player + 1796), inverse));
                        swc1(player + 1804, nz);
                        swc1(player + 1800, ny);
                        frames = lw(g::updateframesi);
                    }
                    step += 1;
                    tx = lwc1(sp + 16);
                    if step >= frames {
                        break 'turn;
                    }
                }
            }
        }
    }
    let y = lwc1(player + 1800);
    let z = lwc1(player + 1804);
    swc1(player + 188, lwc1(player + 1796));
    swc1(player + 196, z);
    swc1(player + 192, y);
}
