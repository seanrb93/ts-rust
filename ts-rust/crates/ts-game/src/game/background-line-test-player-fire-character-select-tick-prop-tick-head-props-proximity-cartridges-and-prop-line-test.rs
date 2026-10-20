//! Functions: the line test against the background and the props, a player firing a gun, the character select effect tick, the tick of the props before they are moved, the
//! head props, the proximity props, adding a cartridge and the line test of the props of a room.

use super::particle_spawners::random_as_float;
use crate::*;

/// Test the line from `a1` along the vector `a2` against the level (@ 0x259038): starting at the room `a0` the props and glass of every room the line passes through are
/// tested (`propLineTestRoom`, `bgGlassLineTestRoom`), following the portals until a hit is found or the line ends. The hit position goes to `a4`, the normal to
/// `a5`, the room reached to `a6` and the glass hits to the ninth argument. `a3` says that the vector is already a unit step (else its length limits the test).
/// Returns what was hit (a prop pointer or a flag), or 0.
pub fn bgLineTest(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32) -> i32 {
    let frame = Frame::new(288);
    let sp = frame.sp();
    let mut room = a0;
    let mut previous = 0;
    sw(sp + 84, a5);
    sw(sp + 88, a6);
    sw(sp + 92, a7);
    memset(sp, 0, 12);
    sw(sp + 96, 0);
    let tail = lw(0x3a2668);
    sd(sp + 16, ld(0x3a2660));
    sw(sp + 24, tail);
    let mut limit = lwc1(0x3aea4c);
    sw(sp + 80, 0);
    sw(g::glasshitreverse, 0);
    if a3 == 0 {
        let x = lwc1(a2);
        let y = lwc1(a2 + 4);
        let z = lwc1(a2 + 8);
        limit = x * x + y * y + z * z;
    }
    let mut best = limit;
    if room != 0 {
        let mut mask = lw(sp + 92);
        loop {
            let hit = propLineTestRoom(room, a1, a2, a3, sp + 32, sp + 48, mask, lw(sp + 288));
            let hx = lwc1(sp + 32);
            if hit != 0 {
                let hy = lwc1(sp + 36);
                let dx = hx - lwc1(a1);
                let start_z = lwc1(a1 + 8);
                let dy = hy - lwc1(a1 + 4);
                let hz = lwc1(sp + 40);
                swc1(sp + 64, dx);
                let dz = hz - start_z;
                swc1(sp + 68, dy);
                let distance = dx * dx + dy * dy + dz * dz;
                swc1(sp + 72, dz);
                if distance < best {
                    let nx = lwc1(sp + 48);
                    if distance < limit {
                        best = distance;
                        let ny = lwc1(sp + 52);
                        let nz = lwc1(sp + 56);
                        sw(sp + 96, hit);
                        swc1(sp, hx);
                        swc1(sp + 4, hy);
                        swc1(sp + 8, hz);
                        swc1(sp + 16, nx);
                        swc1(sp + 20, ny);
                        swc1(sp + 24, nz);
                    }
                }
            }
            bgGlassLineTestRoom(room, a1, a2, sp + 80);
            let before = previous;
            let next;
            if !(best == limit) {
                next = room;
                previous = room;
            } else {
                next = bgPortalCrossed(a1, a2, room, before, a3);
                previous = room;
            }
            room = next;
            mask = lw(sp + 92);
            if room == previous {
                break;
            }
        }
    }
    let glass = lw(sp + 80);
    if lw(sp + 296) != 0 {
        sw(lw(sp + 296), glass);
    }
    let room_out = lw(sp + 88);
    if room_out != 0 {
        sw(room_out, room);
    }
    let mut result = 0;
    if best < limit {
        let x = lwc1(sp);
        if a4 != 0 {
            let y = lwc1(sp + 4);
            let z = lwc1(sp + 8);
            swc1(a4, x);
            swc1(a4 + 8, z);
            swc1(a4 + 4, y);
        }
        let nx = lwc1(sp + 16);
        if lw(sp + 84) != 0 {
            let ny = lwc1(sp + 20);
            let nz = lwc1(sp + 24);
            let normal = lw(sp + 84);
            let inverse = 1.0_f32 / (nx * nx + ny * ny + nz * nz).abs().sqrt();
            swc1(normal + 8, nz * inverse);
            swc1(normal, nx * inverse);
            swc1(normal + 4, ny * inverse);
        }
        result = lw(sp + 96);
    }
    result
}

/// Fire the player's gun (@ 0x28f010), the gun combo `a0` in the gun state `a1` and the hand `a2`: when the clip holds enough ammo the shot is started (a clip that is empty
/// reloads or changes to a combo with ammo) and the ammo is used up (unless the cheat of endless ammo is on), the state is marked as firing and the shots are fired;
/// the remote mine launcher (combo 15) detonates the mines with the second hand instead.
pub fn gunPlayerFire(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32) {
    let player = lw(g::currentplayer);
    let combo = lw(a0);
    let owner = lw(lw(player + 384) + 352);
    let gun = combo * 400 + g::guns;
    let slot = lw(gun + 16);
    let stats = if a2 == 0 { gun + 52 } else { gun + 188 };
    if lw(g::updateframesi) <= 0 {
        return;
    }
    if combo != 15 {
        sw(a1 + 152, 0);
    } else {
        if a2 == 1 {
            gunDetonateRemoteMines(owner);
            return;
        }
        sw(a1 + 152, 0);
    }
    sw(a1 + 160, 1);
    sw(a1 + 52, a2);
    sw(a1 + 56, a3);
    sw(a1 + 164, 0);
    let cost = if a3 == 0 { lw(stats + 36) } else { lw(stats + 40) };
    let offset = slot << 2;
    let clip_base = a0 + 4;
    let clip = lw(gun + 20);
    let mut cheats;
    let end_state;
    'fire: {
        'after: {
            'enough: {
                if lw(clip_base + offset) >= cost {
                    cheats = lw(g::allowedcheats);
                    if clip > 0 {
                        break 'enough;
                    }
                } else if clip > 0 {
                    sw(a1 + 152, -1);
                    if a3 == 0 {
                        if lw(owner + offset + 620) >= cost {
                            gunReload(owner, a0, a1, 1, a4);
                            return;
                        }
                    }
                    sw(a1 + 56, 1);
                    let kind = lw(a0);
                    if kind != 8 {
                        if kind != 11 {
                            if kind != 13 {
                                swc1(a1 + 80, lwc1(stats + 72) * lwc1(0x3ad2d4));
                            }
                        }
                    }
                    let delay;
                    'delay: {
                        if kind != 23 {
                            if kind != 9 {
                                delay = lw(a1 + 148);
                                break 'delay;
                            }
                        }
                        swc1(a1 + 128, lwc1(stats + 108));
                        delay = lw(a1 + 148);
                    }
                    if delay < 6 {
                        end_state = lw(gun + 392);
                        break 'fire;
                    }
                    soundStart(187, 0);
                    sw(a1 + 148, 0);
                    break 'after;
                }
                let ammo = lw(owner + offset + 620);
                if ammo < cost {
                    if ammo > 0 {
                        let delay = lw(a1 + 148);
                        if delay < 6 {
                            sw(a1 + 160, 0);
                            return;
                        }
                        soundStart(187, 0);
                        sw(a1 + 148, 0);
                        sw(a1 + 160, 0);
                        return;
                    }
                    sw(a1 + 160, 0);
                    changeToComboWithAmmo(owner);
                    return;
                }
                cheats = lw(g::allowedcheats);
            }
            if (cheats & 8 & (lw(g::unlockedcheats) & lw(g::selectedcheats))) != 0 {
                sw(a1 + 148, 0);
            } else if (lw(g::curgameparms + 80) & 128) != 0 {
                sw(a1 + 148, 0);
            } else {
                let mut at = clip_base + offset;
                if clip <= 0 {
                    at = owner + 620 + offset;
                }
                sw(at, lw(at) - cost);
                sw(a1 + 148, 0);
            }
            sw(a0 + 68, 0);
            sw(a1 + 176, 1);
            let value = lwc1(stats + 80);
            let number = lw(player);
            sw(a1 + 156, 0);
            sw(a1 + 180, number);
            let kind = lw(a0);
            sw(a1 + 168, lw(player + 384));
            swc1(a1 + 88, value);
            sw(a1 + 172, a0);
            let remaining;
            if kind != 18 {
                remaining = lw(gun + 388);
            } else {
                if a2 == 0 {
                    sw(a1 + 4, 0);
                    sw(a1, 15);
                    break 'after;
                }
                remaining = lw(gun + 388);
            }
            if remaining > 0 {
                end_state = lw(gun + 392);
                break 'fire;
            }
            playerFireShots(a1);
        }
        end_state = lw(gun + 392);
    }
    if end_state != 0 {
        sw(a1 + 124, 1);
    }
}

/// Tick of the character select effect (@ 0x2385b0): the character slides to the left or right (state -1 or 1 of the player) with its position moving at a speed that
/// is faster once the button is held, the character is changed when it has gone out of view and the next one comes in from the other side, and the spin of the
/// platform, the flash and the angle of the light are updated.
pub fn fxSelectCharTick(a0: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    sw(sp, a0);
    let front0 = lw(g::front);
    let signon = signonGet(lw(front0));
    let mut front = lw(g::front);
    let base = front;
    let offset0 = lw(front) << 2;
    let state = lw(offset0 + 0x1fc1268);
    sw(sp + 4, lw(g::signonarray) + signon * 2936 + 32);
    let mut index;
    'b1: {
        'b2: {
            'b3: {
                'b4: {
                    'b5: {
                        if state != -1 {
                            if state != 1 {
                                index = lw(front);
                                break 'b1;
                            }
                            let speed = if lw(offset0 + 0x1fc12d8) == 0 { 3.0_f32 } else { 4.5_f32 };
                            swc1(offset0 + 0x1fc1278, speed);
                            let player = lw(base);
                            let at = player << 2;
                            let position_at = at + 0x1fc1248;
                            let wanted = lw(at + 0x1fc1298);
                            let current = lw(at + 0x1fc1288);
                            let position = lwc1(position_at) + lwc1(at + 0x1fc1278) * lwc1(g::updateframesf) * lwc1(0x3ac9d4);
                            swc1(position_at, position);
                            if wanted == current {
                                let flag_at = at + 0x1fc12d8;
                                let slot;
                                if lw(flag_at) == 0 {
                                    slot = lw(base);
                                } else if (lw(front + 12) & 512) != 0 {
                                    slot = lw(base);
                                } else {
                                    sw(flag_at, 0);
                                    let own = lw(front) << 2;
                                    let mut value = fptodp(lwc1(own + 0x1fc1248));
                                    if dpcmp(value, 0_i64) < 0 {
                                        value = dpsub(0_i64, value);
                                    }
                                    let inverse = dptofp(dpdiv(0x3ff0000000000000_i64, value));
                                    swc1(own + 0x1fc12e8, inverse);
                                    slot = lw(base);
                                }
                                let at2 = slot << 2;
                                if !(0.0_f32 <= lwc1(at2 + 0x1fc1248)) {
                                    break 'b2;
                                }
                                if lw(at2 + 0x1fc12d8) == 0 {
                                    // the slot is cleared below
                                    sw(at2 + 0x1fc1298, -1);
                                    sw((lw(front) << 2) + 0x1fc1268, 0);
                                    swc1((lw(front) << 2) + 0x1fc1248, 0.0_f32);
                                    break 'b2;
                                }
                                fxSelectCharRight(lw(sp));
                                let slot_data = lw(sp + 4);
                                front = lw(g::front);
                                sw((lw(front) << 2) + 0x1fc1298, lw(slot_data + 84));
                                break 'b2;
                            }
                            if !(1.0_f32 <= position) {
                                index = lw(front);
                                break 'b1;
                            }
                            selectChar(player, wanted);
                            front = lw(g::front);
                            let own = lw(front) << 2;
                            sw(own + 0x1fc1288, lw(own + 0x1fc1298));
                            let own = lw(front) << 2;
                            let held = lw(front + 12) & 512;
                            swc1(own + 0x1fc1248, -1.0_f32);
                            if held == 0 {
                                sw(own + 0x1fc12d8, 0);
                                break 'b2;
                            }
                            sw(own + 0x1fc12d8, 1);
                            break 'b2;
                        } else {
                            let speed = if lw(offset0 + 0x1fc12d8) == 0 { 3.0_f32 } else { 4.5_f32 };
                            swc1(offset0 + 0x1fc1278, speed);
                            let player = lw(base);
                            let at = player << 2;
                            let position_at = at + 0x1fc1248;
                            let wanted = lw(at + 0x1fc1298);
                            let current = lw(at + 0x1fc1288);
                            let position = lwc1(position_at) - lwc1(at + 0x1fc1278) * lwc1(g::updateframesf) * lwc1(0x3ac9d0);
                            swc1(position_at, position);
                            if wanted == current {
                                let flag_at = at + 0x1fc12d8;
                                let slot;
                                if lw(flag_at) == 0 {
                                    slot = lw(base);
                                } else if (lw(front + 12) & 256) != 0 {
                                    slot = lw(base);
                                } else {
                                    sw(flag_at, 0);
                                    let own = lw(front) << 2;
                                    swc1(own + 0x1fc12e8, 1.0_f32 / lwc1(own + 0x1fc1248));
                                    slot = lw(base);
                                }
                                let at2 = slot << 2;
                                if !(lwc1(at2 + 0x1fc1248) <= 0.0_f32) {
                                    break 'b2;
                                }
                                if lw(at2 + 0x1fc12d8) != 1 {
                                    sw(at2 + 0x1fc1298, -1);
                                    sw((lw(front) << 2) + 0x1fc1268, 0);
                                    swc1((lw(front) << 2) + 0x1fc1248, 0.0_f32);
                                    break 'b2;
                                }
                                fxSelectCharLeft(lw(sp));
                                let slot_data = lw(sp + 4);
                                front = lw(g::front);
                                sw((lw(front) << 2) + 0x1fc1298, lw(slot_data + 84));
                                break 'b2;
                            }
                            if !(position <= -1.0_f32) {
                                index = lw(front);
                                break 'b1;
                            }
                            selectChar(player, wanted);
                            front = lw(g::front);
                            let own = lw(front) << 2;
                            sw(own + 0x1fc1288, lw(own + 0x1fc1298));
                            let own = lw(front) << 2;
                            let held = lw(front + 12) & 256;
                            swc1(own + 0x1fc1248, 1.0_f32);
                            if held == 0 {
                                sw(own + 0x1fc12d8, 0);
                                break 'b2;
                            }
                            sw(own + 0x1fc12d8, 1);
                            break 'b2;
                        }
                    }
                }
            }
        }
        index = lw(front);
    }
    let at = index << 2;
    if lw(at + 0x1fc12d8) == 0 {
        let positions = 0x1fc1248;
        let one = fptodp(lwc1(at + positions));
        let mut other = fptodp(lwc1((lw(front) << 2) + positions));
        if dpcmp(other, 0_i64) < 0 {
            other = dpsub(0_i64, other);
        }
        let product = dpmul(one, other);
        let product = dpmul(product, ld(0x3a1ce8));
        let scale = fptodp(lwc1((lw(front) << 2) + 0x1fc12e8));
        let value = dptofp(dpmul(product, scale));
        swc1(at + 0x1fc1258, value);
    } else {
        swc1(at + 0x1fc1258, lwc1(at + 0x1fc1248) * lwc1(0x3ac9d8));
    }
    let at = lw(front) << 2;
    let flash = at + 0x1fc12b8;
    let step = lwc1(g::updateframesf);
    if lw(flash) != 0 {
        let progress_at = at + 0x1fc12c8;
        let progress = lwc1(progress_at) + step * lwc1(0x3ac9dc) * lwc1(0x3ac9e0);
        swc1(progress_at, progress);
        if 1.0_f32 <= progress {
            sw(flash, 0);
            sw((lw(front) << 2) + 0x1fc12c8, 0);
        }
    }
    let player = lw(front);
    let degrees = 1.8e+02_f32;
    let turn;
    let target = player * 12 + g::phi;
    if lw(0x3aff20) < 2 {
        turn = lwc1((player << 2) + 0x1fc1248) * 6e+01_f32;
    } else {
        turn = lwc1((player << 2) + 0x1fc1248) * degrees;
    }
    swc1(target, degrees - turn);
}

/// Tick of the props before they are moved (@ 0x26f500). First every used prop gets the buffers for its animation (the matrices and the override pointers; the props that
/// have none get zero); then each prop is ticked by its kind (door, character, gun, special, car, remote, water, corpse, dropping pickups, heads and pickups), its animation
/// is updated, and when it moved or turned its walls and floors are recalculated; finally the position of the prop relative to its rotation is stored.
pub fn propTickBefore() {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let mut count = lw(g::num_props);
    let mut i = 0;
    if count > 0 {
        loop {
            let prop = lw(g::levelprops) + i * 592;
            if lw(prop + 16) < 0 {
                i += 1;
            } else {
                let mut model = lw(prop + 32);
                let parts = lw(lw(model) + 4);
                if parts != 0 {
                    let size = ((parts << 6) + 15) & -16;
                    let memory = memdbAlloc(size);
                    sw(lw(prop + 32) + 4, memory);
                    let memory = memdbAlloc(size);
                    sw(lw(prop + 32) + 8, memory);
                    let memory = memdbAlloc(size);
                    sw(lw(prop + 32) + 12, memory);
                    let memory = memdbAlloc(size);
                    sw(lw(prop + 32) + 16, memory);
                    let memory = memdbAlloc(size);
                    model = lw(prop + 32);
                    let flags = lw(prop + 16) & -257;
                    sw(model + 20, memory);
                    sw(prop + 16, flags);
                } else {
                    sw(model + 4, 0);
                    sw(model + 8, 0);
                    sw(model + 12, 0);
                    sw(model + 16, 0);
                    sw(model + 20, 0);
                }
                let base = lw(model);
                let overrides;
                if lw(base + 8) == 0 {
                    overrides = lw(model + 248);
                } else {
                    let matrices = lw(base + 4);
                    let size;
                    'size: {
                        if matrices != 0 {
                            if (lw(prop + 16) & 0xf000000) != 0 {
                                size = ((matrices << 6) + 15) & -16;
                                break 'size;
                            }
                        }
                        size = 64;
                    }
                    let memory = memdbAlloc(size);
                    sw(lw(prop + 32) + 24, memory);
                    let memory = memdbAlloc(64);
                    let model = lw(prop + 32);
                    sw(model + 28, memory);
                    overrides = lw(model + 248);
                }
                count = lw(g::num_props);
                if overrides != 0 {
                    nullOverridePointers(prop);
                    count = lw(g::num_props);
                }
                i += 1;
            }
            if i >= count {
                break;
            }
        }
    }
    let mut i = 0;
    if count > 0 {
        loop {
            let prop = lw(g::levelprops) + i * 592;
            let mut flags = lw(prop + 16);
            let model = lw(prop + 32);
            let heading;
            'prop: {
                if (flags & -0x7fffff00) != 0 {
                    heading = lwc1(prop + 76);
                } else {
                    if lw(prop + 188) == 0 {
                        let kind = lw(prop + 8);
                        if kind == 64 {
                            droppingPickupPropTick(prop);
                            flags = lw(prop + 16);
                        } else if kind == 2048 {
                            headPropTick(prop);
                            flags = lw(prop + 16);
                        }
                    }
                    if (flags & -0x7fffff00) != 0 {
                        heading = lwc1(prop + 76);
                    } else {
                        if lw(prop + 188) == 0 {
                            if lw(prop + 8) == 64 {
                                if (flags & 2) != 0 {
                                    let mut visible = flags & 1;
                                    if lw(lw(prop + 352)) == 16 {
                                        visible = flags & 1;
                                        if (flags & 256) == 0 {
                                            visible = flags & 1;
                                            if lw(g::gameframe) >= 6 {
                                                flags |= 1;
                                                sw(prop + 16, flags);
                                                visible = flags & 1;
                                            }
                                        }
                                    }
                                    if visible != 0 {
                                        pickupPropTick(prop);
                                        flags = lw(prop + 16);
                                    }
                                }
                            }
                        }
                        if (flags & -0x7fffff00) != 0 {
                            heading = lwc1(prop + 76);
                        } else if lw(prop + 188) != 0 {
                            heading = lwc1(prop + 76);
                        } else {
                            animUpdate(model);
                            match lw(prop + 8) {
                                4 => doorPropTick(prop),
                                8 => chrPropTick(prop),
                                16 => gunPropTick(),
                                128 => specialPropTick(prop),
                                256 => carPropTick(prop),
                                512 => remotePropTick(prop),
                                1024 => waterPropTick(prop),
                                0x1000 => corpsePropTick(prop),
                                _ => {}
                            }
                            let x = lwc1(prop + 48);
                            let still;
                            sw(prop + 28, 0);
                            if x == lwc1(prop + 112) {
                                if !(lwc1(prop + 52) == lwc1(prop + 116)) {
                                    still = false;
                                } else if !(lwc1(prop + 56) == lwc1(prop + 120)) {
                                    still = false;
                                } else if lwc1(prop + 76) == lwc1(prop + 128) {
                                    heading = lwc1(prop + 76);
                                    break 'prop;
                                } else {
                                    still = false;
                                }
                            } else {
                                still = false;
                            }
                            let _ = still;
                            propCalculateWalls(prop, 1);
                            propCalculateFloors(prop, 1);
                            heading = lwc1(prop + 76);
                            sw(prop + 28, 0);
                        }
                    }
                }
            }
            let angle = heading * lwc1(0x3acccc) / 1.8e+02_f32;
            i += 1;
            matrixRotY(sp, angle);
            let table = 0x1fd0df0;
            let at = table + lw(prop + 4) * 12;
            let y = lwc1(at + 4);
            let z = lwc1(at + 8);
            swc1(sp + 64, lwc1(at));
            swc1(sp + 68, y);
            swc1(sp + 72, z);
            matrixVecMulAligned(&[sp as i64, (sp + 64) as i64, at as i64, at as i64]);
            let limit = lw(g::num_props);
            swc1(prop + 508, lwc1(sp + 64));
            swc1(prop + 512, lwc1(sp + 68));
            swc1(prop + 516, lwc1(sp + 72));
            if i >= limit {
                break;
            }
        }
    }
}

/// Tick of a head prop (@ 0x26b710), the head that is shot off a character: it fades out as its timer (`+68`) runs down, is attached to the character's neck bone, then
/// released with the character's last velocity, falls under gravity bouncing off the level (a line test reflects its velocity) while it tumbles, and is deleted when the
/// timer is over.
pub fn headPropTick(a0: i32) {
    let frame = Frame::new(272);
    let sp = frame.sp();
    let state = lw(a0 + 352);
    let timer = lwc1(state + 68) - lwc1(g::updateframesf) / 6e+01_f32;
    sw(a0 + 16, lw(a0 + 16) | 0x40000000);
    swc1(state + 68, timer);
    if timer < 1.0_f32 {
        let alpha = (timer * 127.0_f32) as i32;
        let alpha_byte = alpha & 255;
        if alpha >= 0 {
            obInstSetOverrideStates(lw(a0 + 32), 3, -1, -1, alpha_byte);
        }
    }
    let stage = lw(state + 64);
    if (stage as u32) < (5 as u32) {
        match stage {
            0 => {
                let body = lw(lw(state + 96) + 32);
                let bone = lw(lw(body + 92) + 20);
                propSetAttached(a0, body, bone, 0);
                sw(state + 64, 1);
            }
            1 => {
                let model = lw(a0 + 32);
                sw(state + 64, 2);
                sceVu0CopyMatrix(state, lw(model + 4));
                let owner = lw(state + 96);
                let model = lw(a0 + 32);
                let flags = lw(owner + 16);
                swc1(a0 + 72, lwc1(owner + 72));
                let heading = lwc1(owner + 76);
                sw(owner + 16, flags | 0x10000000);
                swc1(a0 + 76, heading);
                swc1(a0 + 88, lwc1(owner + 88));
                swc1(a0 + 48, lwc1(owner + 48));
                swc1(a0 + 52, lwc1(owner + 52));
                swc1(a0 + 56, lwc1(owner + 56));
                obInstSetAnim(model, 1, 0.0_f32);
                sw(a0 + 28, lw(a0 + 28) | 4);
            }
            2 => {
                let matrix = lw(lw(a0 + 32) + 4);
                sw(a0 + 16, lw(a0 + 16) | 1);
                sceVu0CopyMatrix(matrix, state);
                let step = lwc1(g::updateframesf);
                swc1(a0 + 60, lwc1(state + 100));
                if step == 0.0_f32 {
                    swc1(a0 + 64, lwc1(state + 104) + 0.5_f32);
                } else {
                    swc1(a0 + 64, lwc1(state + 104) + 0.5_f32 / step);
                }
                let vx = lwc1(a0 + 60);
                let vy = lwc1(a0 + 64);
                let vz = lwc1(state + 108);
                swc1(a0 + 68, vz);
                let z = lwc1(a0 + 68);
                sw(state + 64, 3);
                sw(a0 + 140, 2);
                sw(g::headtick, 0);
                let scale = 1.0_f32 / (vx * vx + vy * vy + vz * vz).abs().sqrt() * 12.0_f32 / 6e+01_f32;
                swc1(a0 + 68, z * scale);
                swc1(a0 + 60, vx * scale);
                swc1(a0 + 64, vy * scale);
                swc1(state + 72, 15.0_f32);
                sw(state + 80, 0);
                sw(state + 76, 0);
            }
            3 => {
                sceVu0CopyMatrix(lw(lw(a0 + 32) + 4), state);
                if lwc1(state + 68) < 0.0_f32 {
                    sw(state + 64, 4);
                }
                {
                    let step = lwc1(g::updateframesf);
                    let x = lwc1(a0 + 48);
                    let dy = lwc1(a0 + 64) * step;
                    let matrix = lw(lw(a0 + 32) + 4);
                    let dz = lwc1(a0 + 68) * step;
                    swc1(sp + 80, lwc1(a0 + 60) * step);
                    swc1(sp + 84, dy);
                    swc1(sp + 88, dz);
                    let owner = lw(state + 96);
                    let y = lwc1(a0 + 52);
                    let flags = lw(owner + 16);
                    let z = lwc1(a0 + 56);
                    swc1(sp + 160, x + lwc1(matrix + 48));
                    swc1(sp + 164, y + lwc1(matrix + 52));
                    let mz = lwc1(matrix + 56);
                    sw(owner + 16, flags & -33);
                    sw(sp, 0);
                    sw(sp + 8, 0);
                    swc1(sp + 168, z + mz);
                    let hit = bgLineTest(lw(a0 + 12), sp + 160, sp + 80, 0, sp + 96, sp + 112, a0 + 12, 48);
                    sw(lw(state + 96) + 16, flags);
                    if hit == 0 {
                        let step = lwc1(g::updateframesf);
                        let vy = lwc1(a0 + 64) + step * lwc1(0x3acc20);
                        let ny = lwc1(a0 + 52) + lwc1(sp + 84);
                        let damp = lwc1(0x3acc24);
                        let nz = lwc1(a0 + 56) + lwc1(sp + 88);
                        let vz = lwc1(a0 + 68);
                        let vx = lwc1(a0 + 60) * damp;
                        swc1(a0 + 48, lwc1(a0 + 48) + lwc1(sp + 80));
                        swc1(a0 + 52, ny);
                        swc1(a0 + 56, nz);
                        swc1(a0 + 60, vx);
                        swc1(a0 + 68, vz * damp);
                        swc1(a0 + 64, vy * damp);
                        let spin = lwc1(state + 84) + lwc1(state + 72) * step;
                        swc1(state + 84, spin);
                        if 3.6e+02_f32 < spin {
                            swc1(state + 84, spin - 3.6e+02_f32);
                        }
                        let spin = lwc1(state + 84);
                        if spin < 0.0_f32 {
                            swc1(state + 84, spin + 3.6e+02_f32);
                        }
                        let spin = lwc1(state + 88) + lwc1(state + 76) * lwc1(g::updateframesf);
                        swc1(state + 88, spin);
                        if 3.6e+02_f32 < spin {
                            swc1(state + 88, spin - 3.6e+02_f32);
                        }
                        let spin = lwc1(state + 88);
                        if spin < 0.0_f32 {
                            swc1(state + 88, spin + 3.6e+02_f32);
                        }
                        let spin = lwc1(state + 92) + lwc1(state + 80) * lwc1(g::updateframesf);
                        swc1(state + 92, spin);
                        if 3.6e+02_f32 < spin {
                            swc1(state + 92, spin - 3.6e+02_f32);
                        }
                        let spin = lwc1(state + 92);
                        if spin < 0.0_f32 {
                            swc1(state + 92, spin + 3.6e+02_f32);
                        }
                    } else {
                        let damp = lwc1(0x3acc10);
                        let sy = lwc1(state + 76) * damp;
                        let sz = lwc1(state + 80) * damp;
                        swc1(state + 72, lwc1(state + 72) * damp);
                        swc1(state + 76, sy);
                        swc1(state + 80, sz);
                        let vx = lwc1(sp + 80);
                        let vy = lwc1(sp + 84);
                        let vz = lwc1(sp + 88);
                        let nx = lwc1(sp + 112);
                        let ny = lwc1(sp + 116);
                        let nz = lwc1(sp + 120);
                        let mut length = (vx * vx).abs().sqrt() + vy * vy + vz * vz;
                        let inverse = 1.0_f32 / length;
                        let slow = length < lwc1(0x3acc14);
                        let ux = vx * inverse;
                        let uy = vy * inverse;
                        let uz = vz * inverse;
                        swc1(sp + 128, ux);
                        swc1(sp + 132, uy);
                        swc1(sp + 136, uz);
                        let dot = ux * nx + uy * ny + uz * nz;
                        let twice = dot + dot;
                        swc1(sp + 152, uz - twice * nz);
                        swc1(sp + 144, ux - twice * nx);
                        swc1(sp + 148, uy - twice * ny);
                        if slow {
                            swc1(state + 72, 0.0_f32);
                            length = 0.0_f32;
                            swc1(state + 76, 0.0_f32);
                            swc1(state + 80, 0.0_f32);
                        }
                        let step = lwc1(g::updateframesf);
                        let rx = lwc1(sp + 144);
                        if step == 0.0_f32 {
                            let factor = lwc1(0x3acc1c);
                            let ry = lwc1(sp + 148) * length * factor;
                            let rz = lwc1(sp + 152) * length * factor;
                            swc1(a0 + 60, rx * length * factor);
                            swc1(a0 + 64, ry);
                            swc1(a0 + 68, rz);
                        } else {
                            let factor = lwc1(0x3acc18);
                            let ry = lwc1(sp + 148) * length * factor / step;
                            let rz = lwc1(sp + 152) * length * factor / step;
                            swc1(a0 + 60, rx * length * factor / step);
                            swc1(a0 + 64, ry);
                            swc1(a0 + 68, rz);
                        }
                    }
                }
                let model = lw(a0 + 32);
                let matrix = lw(model + 4);
                let radians = lwc1(0x3acc28);
                let keep_x = lwc1(matrix + 48);
                let keep_y = lwc1(matrix + 52);
                swc1(matrix + 48, -lwc1(a0 + 508));
                let keep_z = lwc1(matrix + 56);
                swc1(lw(lw(a0 + 32) + 4) + 52, -lwc1(a0 + 512));
                swc1(lw(lw(a0 + 32) + 4) + 56, -lwc1(a0 + 516));
                let rz = lwc1(state + 92) * radians / 1.8e+02_f32;
                let rx = lwc1(state + 84) * radians / 1.8e+02_f32;
                let ry = lwc1(state + 88) * radians / 1.8e+02_f32;
                matrixRotZXY(sp + 16, rx, ry, rz);
                let matrix = lw(lw(a0 + 32) + 4);
                sceVu0MulMatrix(matrix, matrix, sp + 16);
                swc1(lw(lw(a0 + 32) + 4) + 48, keep_x);
                swc1(lw(lw(a0 + 32) + 4) + 52, keep_y);
                swc1(lw(lw(a0 + 32) + 4) + 56, keep_z);
            }
            _ => {
                propDelete(a0);
            }
        }
    }
}

/// Update the props that are shown near a bullet that is a proximity mine, a thrown or planted explosive (@ 0x289638): the mine's parts (`+140` and `+144`) follow the
/// bullet's owner bone, and as the stage (`+120`) advances the first part is thrown off, the second falls and bounces with a line test, and when it hits something the
/// bullet explodes (sound, explosion, smoke, decal).
pub fn updateProximityProps(a0: i32, a1: i32) {
    let frame = Frame::new(272);
    let sp = frame.sp();
    let scale = lwc1(0x3ad1bc);
    let owner = lw(a0 + 96);
    let model = lw(lw(owner + 32));
    let z = lwc1(owner + 56);
    let rx = lwc1(owner + 72) * scale / 1.8e+02_f32;
    let ry = lwc1(owner + 76) * scale / 1.8e+02_f32;
    let x = lwc1(owner + 48);
    let rz = lwc1(owner + 88) * scale / 1.8e+02_f32;
    let y = lwc1(owner + 52);
    let bones = model - lw(model) * 80;
    matrixTransRotZXY(sp + 16, x, y, z, rx, ry, rz);
    if lw(a0 + 120) >= 2 {
        let part = lw(a0 + 140);
        let step = lwc1(g::updateframesf);
        let vy = lwc1(part + 64);
        let vx = lwc1(part + 60);
        let vz = lwc1(part + 68);
        let px = lwc1(part + 48);
        let spin = lwc1(part + 88) - step * 12.0_f32;
        let pz = lwc1(part + 56);
        let new_vy = vy - step * lwc1(0x3ad1c0);
        let new_y = lwc1(part + 52) + vy * step;
        swc1(part + 72, lwc1(part + 72) + step * 16.0_f32);
        swc1(part + 88, spin);
        swc1(part + 64, new_vy);
        swc1(part + 52, new_y);
        swc1(part + 48, px + vx * step);
        swc1(part + 56, pz + vz * step);
        let flags;
        if !(lwc1(0x3ad1c4) < lwc1(a0 + 132)) {
            flags = lw(part + 16);
        } else {
            let floor = lwc1(a0 + 148);
            if !(new_y < floor) {
                flags = lw(part + 16);
            } else {
                swc1(part + 52, floor);
                swc1(part + 68, vz * 0.5_f32);
                swc1(part + 64, new_vy * -0.5_f32);
                swc1(part + 60, vx * 0.5_f32);
                flags = lw(part + 16);
            }
        }
        sw(part + 16, flags & -0x40000001);
    } else {
        let source = lw(bones + 72);
        let part = lw(a0 + 140);
        swc1(part + 48, lwc1(source));
        swc1(part + 52, lwc1(source + 4));
        swc1(part + 56, lwc1(source + 8));
        matrixVecMul(sp + 16, part + 48);
        let owner = lw(a0 + 96);
        let part = lw(a0 + 140);
        let room = lw(owner + 12);
        swc1(part + 72, lwc1(owner + 72));
        sw(part + 12, room);
        swc1(part + 76, lwc1(owner + 76));
        swc1(part + 88, lwc1(owner + 88));
    }
    'done: {
        'moved: {
            if lw(a0 + 120) >= 4 {
                let part = lw(a0 + 144);
                let step = lwc1(g::updateframesf);
                swc1(sp + 112, lwc1(part + 60) * step);
                swc1(sp + 116, lwc1(part + 64) * step);
                let vz = lwc1(part + 68);
                sw(sp + 8, sp + 144);
                sw(sp, 0);
                let room = lw(part + 12);
                swc1(sp + 120, vz * step);
                let hit = bgLineTest(room, part + 48, sp + 112, 0, sp + 80, sp + 96, part + 12, 48);
                if lw(sp + 144) != 0 {
                    let source = 0x3a8a60;
                    let word = lw(source + 8);
                    sd(sp + 128, ld(source));
                    sw(sp + 136, word);
                    bgProcessGlasshits(lw(a0), lw(sp + 144), sp + 128, lwc1(a0 + 80));
                }
                let sound;
                if lwc1(0x3ad1c8) < lwc1(a0 + 72) {
                    sound = lw(a0 + 256);
                } else {
                    if hit == 0 {
                        let part = lw(a0 + 144);
                        let step = lwc1(g::updateframesf);
                        let y = lwc1(part + 52);
                        let gravity = lwc1(0x3ad1cc);
                        let z = lwc1(part + 56);
                        swc1(part + 48, lwc1(part + 48) + lwc1(sp + 112));
                        let vy = lwc1(part + 64) - step * gravity;
                        swc1(part + 52, y + lwc1(sp + 116));
                        let dz = lwc1(sp + 120);
                        swc1(part + 64, vy);
                        swc1(part + 56, z + dz);
                        break 'moved;
                    }
                    sound = lw(a0 + 256);
                }
                if sound >= 0 {
                    soundStop(sound);
                }
                let random = newrnd();
                let at = lw(a0 + 144) + 48;
                soundStart(((random as i32 as u32) % (6 as u32)) as i32 + 98, at);
                particleNewExplosion(0, 0, lw(a0 + 144) + 48, lw(a0), a0 + 24, a0 + 48, a1, 0);
                particleNewSmoke(0, lw(a0 + 144) + 48);
                let mut normal = sp + 80;
                if hit == 0 {
                    normal = lw(a0 + 144) + 48;
                }
                decalNewDecal(lw(a0), lw(a0 + 100), hit, normal, sp + 96);
                bulletFree(a0);
                break 'done;
            }
            let source = lw(bones + 72);
            let part = lw(a0 + 144);
            let next = source + 12;
            swc1(part + 48, lwc1(source + 12));
            swc1(part + 52, lwc1(next + 4));
            swc1(part + 56, lwc1(next + 8));
            matrixVecMul(sp + 16, part + 48);
            let owner = lw(a0 + 96);
            let part = lw(a0 + 144);
            let room = lw(owner + 12);
            swc1(part + 72, lwc1(owner + 72));
            sw(part + 12, room);
            swc1(part + 76, lwc1(owner + 76));
            swc1(part + 88, lwc1(owner + 88));
        }
        let stage = lw(a0 + 120);
        if (stage as u32) < (5 as u32) {
            let next_stage;
            match stage {
                0 => {
                    next_stage = stage + 1;
                    if !(3.0_f32 < lwc1(a0 + 72)) {
                        break 'done;
                    }
                    sw(a0 + 72, 0);
                }
                1 => {
                    if propMovingPropInRange(a0 + 24, lw(a0 + 100), 5.0_f32) == 0 {
                        break 'done;
                    }
                    next_stage = lw(a0 + 120) + 1;
                }
                2 => {
                    swc1(a0 + 148, lwc1(lw(a0 + 96) + 52));
                    let random = random_as_float();
                    let part = lw(a0 + 140);
                    let base = lwc1(a0 + 128) * lwc1(0x3ad1d0);
                    swc1(part + 60, base + (random * 2.3283064e-10_f32 - 0.5_f32) * lwc1(0x3ad1d4));
                    let random = random_as_float();
                    let part = lw(a0 + 140);
                    let range = lwc1(0x3ad1d8);
                    swc1(part + 64, random * 2.3283064e-10_f32 * range + range);
                    let random = random_as_float();
                    let part = lw(a0 + 140);
                    let base = lwc1(a0 + 136) * range;
                    let spread = random * 2.3283064e-10_f32;
                    let third = lwc1(0x3ad1dc);
                    sw(a0 + 120, lw(a0 + 120) + 1);
                    swc1(part + 68, base + (spread - 0.5_f32) * third);
                    break 'done;
                }
                3 => {
                    if !(0.75_f32 < lwc1(a0 + 72)) {
                        break 'done;
                    }
                    let scale_a = lwc1(0x3ad1e0);
                    let scale_b = lwc1(0x3ad1e4);
                    let vx = lwc1(a0 + 128) * scale_a;
                    sw(a0 + 120, stage + 1);
                    let part = lw(a0 + 144);
                    swc1(part + 60, vx);
                    swc1(part + 64, lwc1(a0 + 132) * scale_b);
                    swc1(part + 68, lwc1(a0 + 136) * scale_a);
                    sw(a0 + 72, 0);
                    break 'done;
                }
                _ => {
                    next_stage = stage + 1;
                }
            }
            sw(a0 + 120, next_stage);
        }
    }
}

/// Add a cartridge (an ejected shell) to the ring of 10 cartridge props (@ 0x295318): the cartridge kind `a2` must be one of the four in the cartridge table. The oldest
/// cartridge is replaced when the ring is full. The cartridge is made at the gun (`a1` set: from the muzzle data of `a3` and `a5`) or at the position of the shot `a4` with a
/// random velocity added to the shooter's velocity, and is shown to the viewports of the players by `a0`.
pub fn addCartridge(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32) {
    let frame = Frame::new(368);
    let sp = frame.sp();
    sw(sp + 176, a1);
    let mut kind = 0;
    let mut found = false;
    if lw(g::cartridge_table) == a2 {
        found = true;
    } else {
        kind = 1;
        loop {
            let more = (kind < 4) as i32;
            if lw((kind << 2) + g::cartridge_table) == a2 {
                found = more != 0;
                break;
            }
            if more == 0 {
                break;
            }
            kind += 1;
        }
    }
    if !found {
        return;
    }
    let groups = g::cartridge_groups;
    let mut used = lw(groups + 964);
    let first;
    if used < 10 {
        first = lw(groups + 960);
    } else {
        let head = lw(groups + 960);
        sw(groups + 964, used - 1);
        let old = lw(head * 96 + groups + 64);
        let mut next = head + 1;
        if old != 0 {
            propDelete(old);
            let head = lw(groups + 960);
            sw(head * 96 + groups + 64, 0);
            next = head + 1;
        }
        sw(groups + 960, next);
        if next >= 10 {
            sw(groups + 960, 0);
        }
        used = lw(groups + 964);
        first = lw(groups + 960);
    }
    sw(groups + 964, used + 1);
    let slot = first + used;
    let cartridge = (if slot >= 10 { slot - 10 } else { slot }) * 96 + groups;
    if lw(sp + 176) != 0 {
        let prop = propNew(lw((kind << 2) + g::cartridge_table), lwc1(a3 + 84), lwc1(a3 + 88), lwc1(a3 + 92), 0.0_f32);
        let holder = lw(a6 + 3020);
        swc1(prop + 88, 0.0_f32);
        swc1(prop + 72, 0.0_f32);
        let degrees = 1.8e+02_f32;
        let scale = lwc1(0x3ad468);
        let ry = lwc1(holder + 76) + degrees;
        let rz = lwc1(holder + 88) * scale;
        let player = lw(g::currentplayer);
        let rx = lwc1(holder + 72) * scale;
        sw(prop + 140, 1);
        let body = lw(player + 384);
        sw(prop + 16, 0);
        sw(cartridge + 64, prop);
        sw(prop + 12, lw(body + 12));
        matrixRotZXY(sp + 112, rx / degrees, ry * scale / degrees, rz / degrees);
        sceVu0MulMatrix(cartridge, lw(lw(g::currentplayer) + 1772), sp + 112);
        swc1(cartridge + 56, 0.0_f32);
        swc1(cartridge + 48, 0.0_f32);
        swc1(cartridge + 52, 0.0_f32);
        let factor = lwc1(a5 + 336);
        let sz = lwc1(a5 + 332);
        let dx = (lwc1(a3 + 96) - lwc1(a3 + 84)) * factor;
        let dy = (lwc1(a3 + 100) - lwc1(a3 + 88)) * factor;
        let sy = lwc1(a5 + 328);
        let dz = (lwc1(a3 + 104) - lwc1(a3 + 92)) * factor;
        swc1(sp + 16, lwc1(a5 + 324));
        swc1(sp, dx);
        swc1(sp + 4, dy);
        swc1(sp + 8, dz);
        swc1(sp + 20, sy);
        swc1(sp + 24, sz);
        matrixVecRotAligned(&[cartridge as i64, (sp + 16) as i64]);
    } else {
        let prop = propNew(lw((kind << 2) + g::cartridge_table), lwc1(a4 + 28), lwc1(a4 + 32), lwc1(a4 + 36), 0.0_f32);
        sw(cartridge + 64, prop);
        swc1(prop + 88, 0.0_f32);
        let source = lw(a4 + 12);
        swc1(prop + 72, 0.0_f32);
        let scale = lwc1(0x3ad460);
        let rz = lwc1(source + 88) * scale;
        let rx = lwc1(source + 72) * scale;
        let degrees = 1.8e+02_f32;
        let ry = lwc1(source + 76) * scale;
        let player = lw(g::currentplayer);
        sw(prop + 140, 1);
        let body = lw(player + 384);
        sw(prop + 16, 0);
        sw(prop + 12, lw(body + 12));
        matrixRotZXY(sp + 48, rx / degrees, ry / degrees, rz / degrees);
        sceVu0MulMatrix(cartridge, lw(lw(g::currentplayer) + 1772), sp + 48);
        swc1(cartridge + 56, 0.0_f32);
        swc1(cartridge + 48, 0.0_f32);
        swc1(cartridge + 52, 0.0_f32);
        let factor = lwc1(a5 + 336);
        let sx = lwc1(a5 + 324);
        let dx = (lwc1(a4 + 40) - lwc1(a4 + 28)) * factor;
        let sz = lwc1(a5 + 332);
        let dy = (lwc1(a4 + 44) - lwc1(a4 + 32)) * factor;
        let dz = (lwc1(a4 + 48) - lwc1(a4 + 36)) * factor;
        let shot_kind = lw(a3);
        swc1(sp + 20, lwc1(a5 + 328));
        swc1(sp, dx);
        swc1(sp + 4, dy);
        swc1(sp + 8, dz);
        swc1(sp + 24, sz);
        swc1(sp + 16, sx);
        if shot_kind == 23 {
            swc1(sp + 16, sx - lwc1(0x3ad464));
        }
        matrixVecRotAligned(&[cartridge as i64, (sp + 16) as i64]);
    }
    let random = random_as_float() * 2.3283064e-10_f32 - 0.5_f32;
    let prop = lw(cartridge + 64);
    swc1(prop + 60, lwc1(sp) + (random + random) * lwc1(sp + 16));
    let random = random_as_float() * 2.3283064e-10_f32 - 0.5_f32;
    let prop = lw(cartridge + 64);
    swc1(prop + 64, lwc1(sp + 4) + (random + random) * lwc1(sp + 20));
    let random = random_as_float();
    let prop = lw(cartridge + 64);
    let step = lwc1(g::updateframesf);
    let random = random * 2.3283064e-10_f32 - 0.5_f32;
    let holder = lw(a6 + 3020);
    let thousand = 1e+03_f32;
    swc1(prop + 68, lwc1(sp + 8) + (random + random) * lwc1(sp + 24));
    let vx = (lwc1(holder + 48) - lwc1(holder + 112)) / step;
    swc1(sp + 32, vx);
    swc1(sp + 36, (lwc1(holder + 52) - lwc1(holder + 116)) / step);
    swc1(sp + 40, (lwc1(holder + 56) - lwc1(holder + 120)) / step);
    let vy = lwc1(prop + 64);
    let new_vx = lwc1(prop + 60) + vx;
    let vz = lwc1(prop + 68);
    let px = lwc1(prop + 48);
    let py = lwc1(prop + 52);
    swc1(prop + 60, new_vx);
    let pz = lwc1(prop + 56);
    let new_vy = vy + lwc1(sp + 36);
    swc1(prop + 64, new_vy);
    let extra = lwc1(sp + 40);
    swc1(prop + 48, px + new_vx * step);
    let new_vz = vz + extra;
    swc1(prop + 68, new_vz);
    swc1(prop + 52, py + new_vy * step);
    swc1(prop + 56, pz + new_vz * step);
    swc1(cartridge + 68, new_vx * thousand);
    sw(cartridge + 72, 0);
    swc1(cartridge + 76, lwc1(prop + 68) * thousand);
    swc1(cartridge + 84, lwc1(holder + 52));
    sw(cartridge + 80, 0);
    let flags;
    if lw(sp + 176) != 0 {
        let mask;
        if (a0 as u32) >= (5 as u32) {
            mask = 0xf000000;
        } else if a0 == 1 {
            mask = 0xd000000;
        } else if a0 >= 2 {
            if a0 == 2 {
                mask = 0xb000000;
            } else {
                if a0 != 3 {
                    return;
                }
                mask = 0x7000000;
            }
        } else {
            if a0 != 0 {
                return;
            }
            mask = 0xe000000;
        }
        flags = getPropViewportDrawFlags() & mask;
    } else {
        if (a0 as u32) >= (4 as u32) {
            return;
        }
        if a0 == 1 {
            flags = 0x2000000;
        } else if a0 >= 2 {
            if a0 == 2 {
                flags = 0x4000000;
            } else {
                if a0 != 3 {
                    return;
                }
                flags = 0x8000000;
            }
        } else {
            if a0 != 0 {
                return;
            }
            flags = 0x1000000;
        }
    }
    sw(cartridge + 88, flags | 1);
}

/// Test the line from `a1` along `a2` against the props of the room `a0` (@ 0x270430): the room's own object, then every prop and character of the two prop lists that is in
/// the room (or, for a door or a car, linked to it) and is hittable; the nearest hit is kept. The hit position goes to `a4` and the normal to `a5`; `a6` and `a7` are
/// the masks of the kinds to skip and the flags of the test. Returns the prop that was hit, or 0.
pub fn propLineTestRoom(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32) -> i32 {
    let frame = Frame::new(336);
    let sp = frame.sp();
    sw(sp + 128, a4);
    sw(sp + 132, a5);
    sw(sp + 136, a6);
    sw(sp + 140, a7);
    sw(sp + 144, 0);
    memset(sp, 0, 12);
    memset(sp + 16, 0, 12);
    let dx = lwc1(a2);
    let dy = lwc1(a2 + 4);
    let dz = lwc1(a2 + 8);
    let end_x = lwc1(a1) + dx;
    let start_z = lwc1(a1 + 8);
    let mut limit = lwc1(g::headtick + 4);
    let end_y = lwc1(a1 + 4) + dy;
    swc1(sp + 32, dx);
    swc1(sp + 48, end_x);
    swc1(sp + 36, dy);
    swc1(sp + 52, end_y);
    swc1(sp + 56, start_z + dz);
    swc1(sp + 40, dz);
    let inverse = 1.0_f32 / (dx * dx + dy * dy + dz * dz).abs().sqrt();
    let nz = lwc1(sp + 40) * inverse;
    let ny = lwc1(sp + 36) * inverse;
    swc1(sp + 32, lwc1(sp + 32) * inverse);
    swc1(sp + 40, nz);
    swc1(sp + 36, ny);
    if a3 == 0 {
        let x = lwc1(a2);
        let y = lwc1(a2 + 4);
        let z = lwc1(a2 + 8);
        limit = x * x + y * y + z * z;
    }
    let mut mask;
    'room: {
        let room = lw((a0 << 2) + lw(g::roomprops));
        mask = lw(sp + 136);
        if (lw(room + 16) & 32) != 0 {
            mask = lw(sp + 136);
            if (lw(room + 8) & mask) != 0 {
                break 'room;
            }
            let hit = obInstLineTest(lw(room + 32), a1, a2, a3, sp + 64, sp + 80);
            let hx = lwc1(sp + 64);
            if hit != 0 {
                let ox = hx - lwc1(a1);
                let sz = lwc1(a1 + 8);
                let oy = lwc1(sp + 68) - lwc1(a1 + 4);
                let hz = lwc1(sp + 72);
                swc1(sp + 96, ox);
                let oz = hz - sz;
                swc1(sp + 100, oy);
                let distance = ox * ox + oy * oy + oz * oz;
                swc1(sp + 104, oz);
                if distance < limit {
                    limit = distance;
                    obConfirmLastHitInst();
                    let px = lwc1(sp + 64);
                    let py = lwc1(sp + 68);
                    let pz = lwc1(sp + 72);
                    let qx = lwc1(sp + 80);
                    let qy = lwc1(sp + 84);
                    let qz = lwc1(sp + 88);
                    sw(sp + 144, room);
                    swc1(sp, px);
                    swc1(sp + 4, py);
                    swc1(sp + 8, pz);
                    swc1(sp + 16, qx);
                    swc1(sp + 20, qy);
                    swc1(sp + 24, qz);
                }
            }
        }
        mask = lw(sp + 136);
    }
    sw(sp + 136, mask | 112);
    let mut list = 0;
    let mut found;
    loop {
        let count;
        let table;
        if list != 0 {
            count = lw(0x3b00ec);
            table = 0x1fd3410;
        } else {
            count = lw(0x3b00e8);
            table = 0x1fd2f60;
        }
        sw(sp + 148, table);
        let mut i = 0;
        if count <= 0 {
            list += 1;
        } else {
            let next_list = list + 1;
            let mut table = lw(sp + 148);
            loop {
                'prop: {
                    let prop = lw((i << 2) + table);
                    let next;
                    if prop == 0 {
                        next = i + 1;
                    } else {
                        let kind;
                        let flags;
                        let mut in_room;
                        'room_check: {
                            kind = lw(prop + 8);
                            if kind != 256 {
                                if kind != 4 {
                                    let prop_room = lw(prop + 12);
                                    next = i + 1;
                                    flags = lw(prop + 16);
                                    in_room = ((prop_room ^ a0) as u32 == 0) as i32;
                                    break 'room_check;
                                }
                            }
                            let rooms = lw(prop + 184);
                            in_room = 0;
                            if rooms <= 0 {
                                flags = lw(prop + 16);
                                next = i + 1;
                            } else {
                                flags = lw(prop + 16);
                                if lw(prop + 144) != a0 {
                                    next = i + 1;
                                    let mut k = 1;
                                    loop {
                                        let offset = k << 2;
                                        if k >= rooms {
                                            break 'room_check;
                                        }
                                        k += 1;
                                        if lw(prop + offset + 144) == a0 {
                                            break;
                                        }
                                    }
                                    in_room = 1;
                                } else {
                                    next = i + 1;
                                    in_room = 1;
                                }
                            }
                        }
                        i = next;
                        if (flags & 32) == 0 {
                            break 'prop;
                        }
                        let skip = lw(sp + 136);
                        if in_room != 0 {
                            i = next;
                            if (kind & skip) != 0 {
                                break 'prop;
                            }
                            let mut hit_prop = 0;
                            'test: {
                                let hit;
                                if kind != 8 {
                                    hit = obInstLineTest(lw(prop + 32), a1, a2, a3, sp + 64, sp + 80);
                                } else {
                                    let hidden = flags & 128;
                                    if (lw(sp + 140) & 4) != 0 {
                                        if (lw(lw(prop + 352) + 2708) & 32) != 0 {
                                            break 'test;
                                        }
                                    }
                                    let options = lw(sp + 140);
                                    if hidden != 0 {
                                        break 'test;
                                    }
                                    hit = chrLineTestChr(prop, a1, sp + 32, sp + 48, a3, sp + 64, sp + 80, options);
                                }
                                hit_prop = if hit != 0 { prop } else { 0 };
                            }
                            let hx = lwc1(sp + 64);
                            if hit_prop != 0 {
                                let ox = hx - lwc1(a1);
                                let sz = lwc1(a1 + 8);
                                let oy = lwc1(sp + 68) - lwc1(a1 + 4);
                                let hz = lwc1(sp + 72);
                                swc1(sp + 112, ox);
                                let oz = hz - sz;
                                swc1(sp + 116, oy);
                                let distance = ox * ox + oy * oy + oz * oz;
                                swc1(sp + 120, oz);
                                if distance < limit {
                                    let px;
                                    if lw(hit_prop + 8) != 8 {
                                        obConfirmLastHitInst();
                                        px = lwc1(sp + 64);
                                    } else {
                                        chrConfirmLastHitPart();
                                        px = lwc1(sp + 64);
                                    }
                                    limit = distance;
                                    let py = lwc1(sp + 68);
                                    let pz = lwc1(sp + 72);
                                    let qx = lwc1(sp + 80);
                                    let qy = lwc1(sp + 84);
                                    let qz = lwc1(sp + 88);
                                    sw(sp + 144, hit_prop);
                                    swc1(sp, px);
                                    swc1(sp + 4, py);
                                    swc1(sp + 8, pz);
                                    swc1(sp + 16, qx);
                                    swc1(sp + 20, qy);
                                    swc1(sp + 24, qz);
                                }
                            }
                        }
                        break 'prop;
                    }
                    i = next;
                }
                table = lw(sp + 148);
                if i >= count {
                    break;
                }
            }
            list = next_list;
        }
        found = lw(sp + 144);
        if list >= 2 {
            break;
        }
    }
    let out_position = lw(sp + 128);
    if found != 0 {
        let x = lwc1(sp);
        if out_position != 0 {
            let y = lwc1(sp + 4);
            let z = lwc1(sp + 8);
            swc1(out_position, x);
            swc1(out_position + 8, z);
            swc1(out_position + 4, y);
        }
        let out_normal = lw(sp + 132);
        let nx = lwc1(sp + 16);
        if out_normal != 0 {
            let ny = lwc1(sp + 20);
            let nz = lwc1(sp + 24);
            swc1(out_normal, nx);
            swc1(out_normal + 8, nz);
            swc1(out_normal + 4, ny);
        }
        found = lw(sp + 144);
    }
    found
}
