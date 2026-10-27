//! Functions: ejecting cartridges, the tick of a character prop after the move, drawing the background rooms, the preload of the level setup and of the guns, copying a map
//! object in the map maker, the weapons page's tick and the test of a moving prop against the walls.

use crate::*;

/// Eject the cartridges of the gun `a2` of the prop `a0` (@ 0x28df28): the positions of the two ejection points (the cartridge hatch and a second point of the gun, each
/// transformed by the holder's bone matrices) are kept in the prop and a cartridge is added when the gun has a cartridge kind (`+4` of the gun's table entry) and the
/// hand is in use.
pub fn fireCartridges(a0: i32, a1: i32, a2: i32, a3: i32) {
    let frame = Frame::new(368);
    let sp = frame.sp();
    memset(sp + 160, 0, 12);
    sw(sp + 176, a3);
    if a0 == 0 {
        return;
    }
    let v0_1 = lw(a0 + 172);
    if v0_1 == 0 {
        return;
    }
    let v0_2 = lw(v0_1 + 64);
    if v0_2 == 0 {
        return;
    }
    let v0_3 = lw(v0_2 + 32);
    if v0_3 == 0 {
        return;
    }
    if lw(v0_3) == 0 {
        return;
    }
    let v0_4 = lw(a0 + 12);
    if v0_4 == 0 {
        return;
    }
    if lw(v0_4 + 32) == 0 {
        return;
    }
    let holder = lw(a1 + 64);
    let mut point = sp + 160;
    let f1 = lwc1(a0 + 72);
    let x = ps2float::add(lwc1(holder + 48), lwc1(a0 + 68));
    let f2 = lwc1(a0 + 76);
    let player = lw(g::currentplayer);
    let to_radians = lwc1(0x3ad2a8);
    swc1(sp + 144, x);
    let degrees = 1.8e+02_f32;
    let y = ps2float::add(lwc1(holder + 52), f1);
    let owner = lw(lw(player + 384) + 352);
    swc1(sp + 148, y);
    let z = ps2float::add(lwc1(holder + 56), f2);
    swc1(sp + 152, z);
    let heading = ps2float::mul(lwc1(holder + 76), to_radians).pdiv(degrees);
    matrixTransRotY(sp, x, y, z, heading);
    sceVu0MulMatrix(sp + 64, sp, lw(lw(holder + 32) + 4));
    matrixTrans(sp, lwc1(sp + 160), lwc1(point + 4), lwc1(point + 8));
    sceVu0MulMatrix(sp + 64, sp + 64, sp);
    let dy = lwc1(sp + 116);
    let dz = lwc1(sp + 120);
    swc1(a1 + 84, lwc1(sp + 112));
    swc1(a1 + 88, dy);
    swc1(a1 + 92, dz);
    let x = lwc1(sp + 144);
    let y = lwc1(sp + 148);
    let heading = ps2float::mul(lwc1(holder + 76), to_radians).pdiv(degrees);
    let z = lwc1(sp + 152);
    matrixTransRotY(sp, x, y, z, heading);
    sceVu0MulMatrix(sp + 64, sp, lw(lw(holder + 32) + 4));
    matrixTrans(sp, lwc1(sp + 160), lwc1(point + 4), lwc1(point + 8));
    sceVu0MulMatrix(sp + 64, sp + 64, sp);
    let prop = lw(a0 + 12);
    let ey = lwc1(sp + 116);
    let model = lw(lw(prop + 32));
    let ez = lwc1(sp + 120);
    let hatch = lw(a2 + 344);
    let base = lw(model) * 80;
    swc1(a1 + 96, lwc1(sp + 112));
    swc1(a1 + 100, ey);
    swc1(a1 + 104, ez);
    let bones = model - base;
    if hatch >= 0 {
        point = lw(bones + 72) + hatch * 12;
    }
    let px = lwc1(sp + 144);
    swc1(sp + 128, lwc1(point));
    let py = lwc1(sp + 148);
    let pz = lwc1(sp + 152);
    swc1(sp + 132, lwc1(point + 4));
    swc1(sp + 136, lwc1(point + 8));
    let rx = ps2float::mul(lwc1(prop + 72), to_radians).pdiv(degrees);
    let ry = ps2float::mul(lwc1(prop + 76), to_radians).pdiv(degrees);
    let rz = ps2float::mul(lwc1(prop + 88), to_radians).pdiv(degrees);
    matrixTransRotZXY(sp, px, py, pz, rx, ry, rz);
    matrixVecRotAligned(&[sp as i64, (sp + 128) as i64]);
    let vy = lwc1(sp + 132);
    let vz = lwc1(sp + 136);
    let current = lw(g::currentplayer);
    swc1(a0 + 28, ps2float::add(lwc1(sp + 128), lwc1(prop + 48)));
    let matrix = lw(current + 1772);
    swc1(a0 + 32, ps2float::add(vy, lwc1(prop + 52)));
    swc1(a0 + 36, ps2float::add(vz, lwc1(prop + 56)));
    matrixVecMul(matrix, a0 + 28);
    let second = lw(a2 + 348);
    let point = if second < 0 { sp + 160 } else { lw(bones + 72) + second * 12 };
    let prop = lw(a0 + 12);
    swc1(sp + 128, lwc1(point));
    let to_radians = lwc1(0x3ad2ac);
    let degrees = 1.8e+02_f32;
    swc1(sp + 132, lwc1(point + 4));
    let px = lwc1(sp + 144);
    let py = lwc1(sp + 148);
    swc1(sp + 136, lwc1(point + 8));
    let rz = ps2float::mul(lwc1(prop + 88), to_radians).pdiv(degrees);
    let rx = ps2float::mul(lwc1(prop + 72), to_radians).pdiv(degrees);
    let ry = ps2float::mul(lwc1(prop + 76), to_radians).pdiv(degrees);
    let pz = lwc1(sp + 152);
    matrixTransRotZXY(sp, px, py, pz, rx, ry, rz);
    matrixVecRotAligned(&[sp as i64, (sp + 128) as i64]);
    let vy = lwc1(sp + 132);
    let vz = lwc1(sp + 136);
    let current = lw(g::currentplayer);
    swc1(a0 + 40, ps2float::add(lwc1(sp + 128), lwc1(prop + 48)));
    let matrix = lw(current + 1772);
    swc1(a0 + 44, ps2float::add(vy, lwc1(prop + 52)));
    swc1(a0 + 48, ps2float::add(vz, lwc1(prop + 56)));
    matrixVecMul(matrix, a0 + 40);
    let kind = lw(lw(sp + 176) + 4);
    if kind != -1 {
        if lw(a2 + 344) >= 0 {
            addCartridge(lw(a0 + 180), 0, kind, a1, a0, a2, owner);
        }
    }
}

/// Tick of a character prop after it has moved (@ 0x26e8d8): the shadow or marker prop (`+536`) is put at the character's position, and for the boss of the kind 553
/// that is bright the special effect that follows the character is made, moved and coloured (a sine pulse from a timer), or freed when the character is no longer lit.
pub fn chrPropTickAfter(a0: i32) {
    let frame = Frame::new(400);
    let sp = frame.sp();
    let marker = lw(a0 + 536);
    if marker != 0 {
        let flags = lw(a0 + 16);
        let table = 0x3a7fa8;
        let third = lw(table + 8);
        sd(sp, ld(table));
        sw(sp + 8, third);
        let y;
        let top;
        'place: {
            if (flags & 0xf000000) == 0 {
                y = lwc1(a0 + 52);
            } else {
                if (ld(a0 + 16) & 0x80000180_i64) == 0_i64 {
                    let angle = ps2float::mul(lwc1(a0 + 80), lwc1(0x3acca0)).pdiv(1.8e+02_f32);
                    matrixRotY(sp + 16, angle);
                    matrixVecMul(sp + 16, sp);
                    let marker = lw(a0 + 536);
                    let step = lwc1(g::updateframesf);
                    let model = lw(a0 + 32);
                    let spin = lwc1(marker + 76);
                    swc1(marker + 48, ps2float::add(lwc1(a0 + 48), lwc1(sp)));
                    let bone = lw(lw(model + 92) + 20);
                    let z = ps2float::add(lwc1(a0 + 56), lwc1(sp + 8));
                    let matrices = lw(model + 4);
                    swc1(marker + 76, ps2float::add(spin, ps2float::add(step, step)));
                    swc1(marker + 56, z);
                    top = lwc1((bone << 6) + matrices + 52);
                    y = lwc1(a0 + 52);
                    break 'place;
                }
                y = lwc1(a0 + 52);
            }
            top = ps2float::add(y, 2.0_f32);
        }
        let marker = lw(a0 + 536);
        let height = ps2float::add(y, top);
        let cleared = lw(marker + 16) & -2;
        let offset = lwc1(0x3acca4);
        sw(marker + 16, cleared);
        let visible = lw(a0 + 16) & 1;
        swc1(marker + 52, ps2float::add(height, offset));
        sw(marker + 16, cleared | visible);
        sw(marker + 12, lw(a0 + 12));
    }
    'special: {
        let data = lw(a0 + 352);
        if lw(data + 8) == 2 {
            let model = lw(a0 + 32);
            let effect;
            if lw(model + 96) != 553 {
                effect = lw(data + 0x11a8);
            } else {
                let power = lwc1(model + 100);
                if !(0.0_f32 <= power) {
                    effect = lw(data + 0x11a8);
                } else {
                    if power < 3.5_f32 {
                        if (ld(a0 + 16) & 384_i64) != 0_i64 {
                            break 'special;
                        }
                        let flags = lw(a0 + 16);
                        if (flags & 0xf000000) == 0 {
                            break 'special;
                        }
                        if (flags & 1) == 0 {
                            break 'special;
                        }
                        let x = lwc1(a0 + 48);
                        let y = lwc1(a0 + 52);
                        let angle = ps2float::mul(lwc1(a0 + 76), lwc1(0x3acca8)).pdiv(1.8e+02_f32);
                        let z = lwc1(a0 + 56);
                        matrixTransRotY(sp + 208, x, y, z, angle);
                        let model = lw(a0 + 32);
                        sceVu0MulMatrix(sp + 80, sp + 208, lw(model + 4) + (lw(lw(model + 92) + 24) << 6));
                        let model = lw(a0 + 32);
                        sceVu0MulMatrix(sp + 144, sp + 208, lw(model + 4) + (lw(lw(model + 92) + 28) << 6));
                        let existing = lw(data + 0x11a8);
                        if existing != 0 {
                            let handle;
                            'colour: {
                                let model = lw(a0 + 32);
                                let start = lwc1(0x3accac);
                                let power = lwc1(model + 100);
                                let kind = lw(existing);
                                if start <= power {
                                    let half = 0x3fe0000000000000_i64;
                                    let phase = (ps2float::sub(power, start)).pdiv(lwc1(0x3accb0));
                                    let d = dpmul(fptodp(phase), ld(0x3a7fb8));
                                    let d = dpmul(d, half);
                                    let d = dpmul(d, ld(0x3a7fc0));
                                    let d = dpadd(d, half);
                                    let index = dptoli(d) as i32;
                                    let wave = lwc1((((index + 512) & 2047) << 2) + g::g_SinTable);
                                    if kind == 11 {
                                        obInstSetOverrideCol(lw(a0 + 32), 255, 220, 196, ps2float::mul(wave, lwc1(0x3accb4)));
                                        handle = lw(data + 0x11a8);
                                        break 'colour;
                                    }
                                    obInstSetOverrideCol(lw(a0 + 32), 128, 128, 255, ps2float::mul(wave, lwc1(0x3accb8)));
                                }
                                handle = lw(data + 0x11a8);
                            }
                            specialfxUpdatePos(handle, a0 + 48, sp + 128, sp + 192);
                            break 'special;
                        }
                        let random = newrnd();
                        let kind = if (random & 1_i64) != 0_i64 { 11 } else { 10 };
                        let made = specialfxNew(kind, a0 + 48, sp + 128, sp + 192, -1, -1, 0);
                        sw(data + 0x11a8, made);
                        break 'special;
                    }
                    effect = lw(data + 0x11a8);
                }
            }
            if effect != 0 {
                obInstSetOverrideCol(lw(a0 + 32), 0, 0, 0, 0.0_f32);
                specialfxFreePtr(lw(data + 0x11a8));
                sw(data + 0x11a8, 0);
            }
        }
    }
}

/// Draw the rooms of the level (@ 0x256830) for the current player: the rooms that the player can see are bucketed by their distance level, then drawn from the farthest
/// to the nearest, each with its opaque props, decals, transparent props and glass, with the sky and the always-drawn prop groups before and after. A string on the stack
/// keeps one character per room for the room debug view.
pub fn bgGfx() {
    let frame = Frame::new(1168);
    let sp = frame.sp();
    let old_zb = dlSetZB(0);
    let player = lw(g::currentplayer);
    let max_level = (lw(player) << 2) + g::maxlevel;
    if lw(max_level) >= 25 {
        sw(max_level, 24);
    }
    let mut level = 0;
    let rooms;
    let index;
    if lw((lw(player) << 2) + g::maxlevel) < 0 {
        rooms = lw(g::bgNumRooms);
        index = lw(player);
    } else {
        rooms = lw(g::bgNumRooms);
        let mut offset = level << 2;
        loop {
            sw(offset + 0x1fc4a10, 0);
            level += 1;
            offset = level << 2;
            if lw((lw(player) << 2) + g::maxlevel) < level {
                break;
            }
        }
        index = lw(player);
    }
    if index == 0 {
        sw(g::bgroomsdrawn, 0);
    }
    let mut room = 1;
    if room < rooms + 1 {
        let mut current = lw(player);
        loop {
            let distance = lw(room * 20 + lw((current << 2) + g::roomdata));
            if (distance as u32) < (25 as u32) {
                let counter = (distance << 2) + 0x1fc4a10;
                let count = lw(counter);
                sw(counter, count + 1);
                room_slot_write(count, distance, room);
            }
            let tens = room.wrapping_rem(10);
            let at = sp + (room - 1);
            let mut letter = 46;
            if tens == 9 {
                letter = 58;
            }
            room_letter_write(at, letter);
            room += 1;
            if room >= rooms + 1 {
                break;
            }
            current = lw(player);
        }
    }
    dlSetBlend(0);
    let window = g::window;
    dlSetClip(lw(window), lw(g::window + 20), lw(g::window + 4), lw(g::window + 24));
    let current = lw(g::currentplayer);
    if lw(current + 388) != 0 {
        propGfxRoom(lw(current) + 205, 0);
    }
    dlSetBlend(1);
    dlSetZB(2);
    propGfxRoom(211, 1);
    dlSetZB(3);
    propGfxRoom(209, 0);
    propGfxRoom(209, 1);
    dlSetBlend(0);
    dlSetZB(3);
    propGfxRoom(212, 0);
    dlSetBlend(1);
    dlSetZB(2);
    propGfxRoom(212, 1);
    let mut level = room_depth_limit(lw((lw(lw(g::currentplayer)) << 2) + g::maxlevel));
    if level >= 0 {
        loop {
            let count = lw((level << 2) + 0x1fc4a10);
            let mut i = 0;
            if count <= 0 {
                level -= 1;
            } else {
                let next_level = level - 1;
                loop {
                    'room: {
                        let number = room_slot_read(level, i);
                        let current = lw(lw(g::currentplayer));
                        let data = number * 20 + lw((current << 2) + g::roomdata);
                        let near = lw(data + 12);
                        if near < 0 {
                            i += 1;
                        } else {
                            let far = lw(data + 4);
                            if far < 640 {
                                if far >= near {
                                    i += 1;
                                    break 'room;
                                }
                                let drawn = lw(g::bgroomsdrawn);
                                if current == 0 {
                                    sw(g::bgroomsdrawn, drawn + 1);
                                }
                                dlSetBlend(0);
                                dlSetZB(3);
                                dlSetDitherMatrix(1);
                                propGfxRoom(number, 0);
                                decalGfx(number);
                                dlSetBlend(1);
                                dlSetZB(2);
                                let dither = lvGetDitherForXLU();
                                dlSetDitherMatrix(dither);
                                propGfxRoom(number, 1);
                                bgGlassGfxRoom(number);
                                sb(number + sp - 1, number.wrapping_rem(10) + 48);
                            }
                            i += 1;
                        }
                    }
                    if i >= count {
                        break;
                    }
                }
                level = next_level;
            }
            if level < 0 {
                break;
            }
        }
    }
    decalGfxMoving();
    dlSetBlend(1);
    dlSetZB(3);
    propGfxRoom(210, 0);
    propGfxRoom(210, 1);
    dlSetZB(old_zb);
    sb(sp + lw(g::bgNumRooms), 0);
}

/// Preload what the setup of the level needs (@ 0x228e70): the props of its object, weapon, pickup, door and window lists that are created in the game mode, the guns of the
/// combos for the pickups of the negative kinds, the bags and the extra objects. The map maker's level preloads its tile set and the challenge level 15 the prop 202. The
/// story levels (6, 101 to 105) preload nothing here.
pub fn lvLevelSetupPreload() {
    let setup = lvGetLevelSetup(lw(g::lvlevelnum));
    let level = lw(g::lvlevelnum);
    if level == 6 || level == 101 || level == 102 || level == 103 || level == 104 || level == 105 {
        return;
    }
    if level != 100 {
        if setup != 0 {
            let mut i = 0;
            let combos = lvGetGunComboNums();
            let count;
            if lw(setup + 12) <= 0 {
                count = lw(setup + 92);
            } else {
                loop {
                    let entry = lw(setup + 8) + i * 28;
                    if lvCheckCreateFlags(lw(entry)) != 0 {
                        propAddPreLoad(lw(entry + 4));
                    }
                    let total = lw(setup + 12);
                    i += 1;
                    if i >= total {
                        break;
                    }
                }
                count = lw(setup + 92);
            }
            let mut i = 0;
            if count > 0 {
                let mut offset = 0;
                loop {
                    let entry = lw(setup + 88) + offset;
                    if lvCheckCreateFlags(lw(entry)) != 0 {
                        propAddPreLoad(lw(entry + 4));
                    }
                    let total = lw(setup + 92);
                    i += 1;
                    offset += 28;
                    if i >= total {
                        break;
                    }
                }
            }
            let mut i = 0;
            if lw(setup + 20) > 0 {
                let mut offset = 0;
                loop {
                    let entry = lw(setup + 16) + offset;
                    if lvCheckCreateFlags(lw(entry)) != 0 {
                        let prop = lvGetPropNum(lw(entry + 4));
                        let mut preload = prop;
                        let mut skip = false;
                        if prop < 0 {
                            let kind = lw(entry + 4);
                            if kind >= -20 || kind < -25 {
                                skip = true;
                            } else {
                                preload = lw(lh((lw(combos - ((kind + 20) << 2) - 4) << 3) + (GUNCOMBOS + 4)) * 400 + (g::guns + 12));
                            }
                        }
                        if !skip {
                            propAddPreLoad(preload);
                        }
                        let kind = lw(entry + 4);
                        let slot = -6 - kind;
                        if (slot as u32) < (4 as u32) {
                            let params = g::curgameparms;
                            if lbu(params + slot + 100) != 0 {
                                if lw(params + 72) == 2 {
                                    propAddPreLoad(314 - kind);
                                }
                            }
                        }
                    }
                    i += 1;
                    offset += 48;
                    if i >= lw(setup + 20) {
                        break;
                    }
                }
            }
            let mut i = 0;
            if lw(setup + 28) > 0 {
                let mut list = lw(setup + 24);
                loop {
                    let offset = i << 4;
                    i += 1;
                    propAddPreLoad(lw(list + offset + 4));
                    if i >= lw(setup + 28) {
                        break;
                    }
                    list = lw(setup + 24);
                }
            }
            let count;
            if lw(g::lvlevelnum) == 27 && lw(g::curgameparms + 72) != 7 {
                count = lw(setup + 84);
            } else {
                let mut i = 0;
                if lw(setup + 76) > 0 {
                    let mut list = lw(setup + 72);
                    loop {
                        let entry = list + (i << 5);
                        if lvCheckCreateFlags(lw(entry)) != 0 {
                            propAddPreLoad(lw(entry + 4));
                        }
                        let total = lw(setup + 76);
                        i += 1;
                        if i >= total {
                            break;
                        }
                        list = lw(setup + 72);
                    }
                }
                count = lw(setup + 84);
            }
            let mut i = 0;
            if count > 0 {
                let mut list = lw(setup + 80);
                loop {
                    let offset = i << 6;
                    i += 1;
                    propAddPreLoad(lw(list + offset + 4));
                    if i >= lw(setup + 84) {
                        break;
                    }
                    list = lw(setup + 80);
                }
            }
        }
    } else {
        lvTileSetLevelPreload();
    }
    let params = g::curgameparms;
    if (lw(params + 80) & 32) != 0 {
        if lw(params + 68) == 15 {
            propAddPreLoad(202);
        }
    }
}

/// Copy the 60 bytes of a map object the way the compiled code does.
fn copy_map_object(dst: i32, src: i32) {
    let b = ld(src + 8);
    let c = ld(src + 16);
    let d = ld(src + 24);
    sd(dst, ld(src));
    sd(dst + 8, b);
    sd(dst + 16, c);
    sd(dst + 24, d);
    let f = ld(src + 40);
    let g_ = ld(src + 48);
    let h = lw(src + 56);
    sd(dst + 32, ld(src + 32));
    sd(dst + 40, f);
    sd(dst + 48, g_);
    sw(dst + 56, h);
}

/// Copy (or, when `a0` is set, cut) the map object under the cursor into the pasted object (@ 0x245cd0). A marked object copies the whole chain of marked objects
/// (each with its duplicated items) and puts them relative to the cursor; a single object is copied by itself.
pub fn mmDoCopy(a0: i32) {
    let under = mmObjectUnderCursor();
    if under == 0 {
        return;
    }
    mmBackupMap();
    let used = lw(g::usedobj);
    if used != 0 {
        mmFreeObject(used);
    }
    let flags = lw(under + 4);
    sw(g::usedobj, 0);
    if (flags & 1) != 0 {
        let mut source = lw(g::mapobjects);
        loop {
            if source == 0 {
                return;
            }
            if (lw(source + 4) & 1) != 0 {
                break;
            }
            source = lw(source + 56);
        }
        let mut target = mmAllocObject();
        sw(g::usedobj, target);
        copy_map_object(target, source);
        let first = lw(g::usedobj);
        sw(first + 56, 0);
        if a0 == 0 {
            sw(first + 52, 0);
            mmDuplicateItems(source, first);
        } else {
            sw(source + 52, 0);
        }
        loop {
            let mut next = lw(source + 56);
            let mut copy = 0;
            let word;
            'copied: {
                loop {
                    copy = 0;
                    if next == 0 {
                        word = lw(target + 4);
                        break 'copied;
                    }
                    if (lw(next + 4) & 1) != 0 {
                        break;
                    }
                    next = lw(next + 56);
                }
                if next == 0 {
                    word = lw(target + 4);
                    break 'copied;
                }
                copy = mmAllocObject();
                if copy == 0 {
                    word = lw(target + 4);
                    break 'copied;
                }
                copy_map_object(copy, next);
                if a0 == 0 {
                    sw(copy + 52, 0);
                    mmDuplicateItems(next, copy);
                    word = lw(target + 4);
                    break 'copied;
                }
                sw(next + 52, 0);
                word = lw(target + 4);
            }
            source = next;
            sw(target + 56, copy);
            sw(target + 4, word & -2);
            target = copy;
            if next == 0 {
                break;
            }
        }
        let scale_x = lw(g::scale_wtomx);
        let wx = (lh(lw(g::front) + 34) - lw(g::w_x)) << 16;
        let moved_x = FMul(wx, scale_x);
        let scale_y = lw(g::scale_wtomy);
        let x = lw(g::m_x) + moved_x as i32;
        let wy = (lh(lw(g::front) + 38) - lw(g::w_y)) << 16;
        let moved_y = FMul(wy, scale_y);
        let mut object = lw(g::usedobj);
        let y = lw(g::m_y) + moved_y as i32;
        if object != 0 {
            let level = lw(g::editlevel);
            let mut value = lw(object + 12);
            loop {
                let z = lw(object + 20);
                let relative_y = lw(object + 16) - y;
                sw(object + 12, value - x);
                sw(object + 16, relative_y);
                sw(object + 20, z - level);
                object = lw(object + 56);
                if object == 0 {
                    break;
                }
                value = lw(object + 12);
            }
        }
        if a0 != 0 {
            mmDeleteAllMarked();
        }
    } else {
        let copy = mmAllocObject();
        sw(g::usedobj, copy);
        if copy != 0 {
            copy_map_object(copy, under);
            let front = lw(g::front);
            sw(lw(g::usedobj) + 56, 0);
            let scale_x = lw(g::scale_wtomx);
            let wx = (lh(front + 34) - lw(g::w_x)) << 16;
            let moved_x = FMul(wx, scale_x);
            let front = lw(g::front);
            let window_y = lw(g::w_y);
            sw(lw(g::usedobj) + 12, lw(under + 12) - (lw(g::m_x) + moved_x as i32));
            let scale_y = lw(g::scale_wtomy);
            let wy = (lh(front + 38) - window_y) << 16;
            let moved_y = FMul(wy, scale_y);
            let used = lw(g::usedobj);
            let level = lw(g::editlevel);
            sw(used + 16, lw(under + 16) - (lw(g::m_y) + moved_y as i32));
            let z = lw(under + 20) - level;
            sw(used + 4, lw(used + 4) & -2);
            sw(used + 20, z);
            if a0 == 0 {
                sw(used + 52, 0);
                mmDuplicateItems(under, used);
                return;
            }
            sw(used + 52, lw(under + 52));
            sw(under + 52, 0);
            mmFreeSingleObject(under);
        }
    }
}

/// Preload one gun: its two props and the sounds of its shot (the ones at +100 and +236 when the cheat that swaps the shot sounds is on, else +96 and +232).
fn preload_gun(index: i32) {
    let gun = index * 400 + g::guns;
    propAddPreLoad(lw(gun));
    propAddPreLoad(lw(gun + 4));
    let (first, second) = if (lw(g::allowedcheats) & 32 & (lw(g::unlockedcheats) & lw(g::selectedcheats))) != 0 { (100, 236) } else { (96, 232) };
    let sound = lw(gun + first);
    if sound >= 0 {
        soundLoad(sound);
    }
    let sound = lw(gun + second);
    if sound >= 0 {
        soundLoad(sound);
    }
}

/// Preload the guns (@ 0x28d460): the guns of the challenge's combo and of the combos the players carry (the right and the left hand gun of each) and the props that
/// every gun may spawn.
pub fn gunPreload() {
    let mut combos = lvGetGunComboNums();
    if (lw(g::curgameparms + 80) & 32) != 0 {
        let setup = challengeGetSetup();
        let combo = lw(setup + 56);
        if ((combo - 1) as u32) < (32 as u32) {
            let entry = (combo << 3) + GUNCOMBOS;
            let right = lh(entry + 2);
            if right >= 0 {
                preload_gun(right);
            }
            let left = lh(entry + 4);
            if left >= 0 {
                preload_gun(left);
            }
        }
    }
    let mut left = 4;
    let mut combo = lw(combos);
    loop {
        let entry = (combo << 3) + GUNCOMBOS;
        let right = lh(entry + 2);
        if right >= 0 {
            preload_gun(right);
        }
        let left_gun = lh(entry + 4);
        if left_gun >= 0 {
            preload_gun(left_gun);
        }
        combos += 4;
        left -= 1;
        if left < 0 {
            break;
        }
        combo = lw(combos);
    }
    propAddPreLoad(218);
    propAddPreLoad(219);
    propAddPreLoad(220);
    propAddPreLoad(221);
    propAddPreLoad(222);
    propAddPreLoad(223);
    propAddPreLoad(224);
    propAddPreLoad(225);
    propAddPreLoad(230);
    propAddPreLoad(231);
    propAddPreLoad(226);
    propAddPreLoad(227);
    propAddPreLoad(228);
    propAddPreLoad(229);
    propAddPreLoad(232);
    propAddPreLoad(421);
    propAddPreLoad(424);
    propAddPreLoad(233);
}

/// Tick of the weapons page (@ 0x2333f0): the list of the weapons that can be picked is built (the last one only when the weapon set 8 is available), up and down move
/// the cursor in the list, accept fills the first free slot of the picked weapons and the second button clears the last one, and left and right change the weapon set.
pub fn weapons_pageTick(a0: i32) {
    #[cfg(feature = "extensions")]
        weapons_randomise(a0);
    pageSet(lw(lw((lw(g::cur_language) << 2) + g::languages) + 500), g::weapons_page);
    frontfxTick(0);
    sw(0x3aff4c, 0);
    let mut weapon = 1;
    loop {
        'listed: {
            let mut count = lw(0x3aff4c);
            if weapon == 32 {
                let available = weaponsetAvail(8);
                count = lw(0x3aff4c);
                if available == 0 {
                    break 'listed;
                }
            }
            sw((count << 2) + 0x1fc2a00, weapon);
            sw(0x3aff4c, count + 1);
        }
        weapon += 1;
        if weapon >= 33 {
            break;
        }
    }
    weapons_list_extend_hook();
    let mut language;
    'text: {
        'plain: {
            let item;
            'input: {
                let front = lw(g::front);
                let buttons = lw(front + 12);
                if (buttons & 8) == 0 {
                    if (buttons & 4) == 0 {
                        if (buttons & 16) == 0 {
                            if (buttons & 32) == 0 {
                                let mut sound = 0;
                                if (joyNewBut(lw(front)) & 64) == 0 {
                                    if (joyNewBut(lw(lw(g::front))) & 128) == 0 {
                                        if (joyNewBut(lw(lw(g::front))) & 16) != 0 {
                                            if lhu(a0 + 4) != 1 {
                                                item = lhu(a0 + 4);
                                                break 'input;
                                            }
                                            soundStart(140, 0);
                                            sh(a0 + 4, 2);
                                            joyClearAll(0);
                                        }
                                        item = lhu(a0 + 4);
                                        break 'input;
                                    }
                                    if lhu(a0 + 4) != 1 {
                                        item = lhu(a0 + 4);
                                        break 'input;
                                    }
                                    let mut filled = 0;
                                    if lw(g::gunzpack) >= 0 {
                                        let mut at = g::gunzpack;
                                        filled = 1;
                                        loop {
                                            at += 4;
                                            if filled >= 5 {
                                                break;
                                            }
                                            if lw(at) < 0 {
                                                break;
                                            }
                                            filled += 1;
                                        }
                                    }
                                    if filled <= 0 {
                                        item = lhu(a0 + 4);
                                        break 'input;
                                    }
                                    sound = 142;
                                    sw(((filled - 1) << 2) + g::gunzpack, -1);
                                } else {
                                    let current = lhu(a0 + 4);
                                    let id = current & 0xffff;
                                    if id == 2 {
                                        if weaponset_read() != 0 {
                                            item = current;
                                            break 'input;
                                        }
                                        soundStart(140, 0);
                                        sh(a0 + 4, 1);
                                        item = 1;
                                        break 'input;
                                    }
                                    if id != 1 {
                                        item = current;
                                        break 'input;
                                    }
                                    let mut slot = 0;
                                    let mut found = false;
                                    if lw(g::gunzpack) < 0 {
                                        found = true;
                                    } else {
                                        let mut at = g::gunzpack;
                                        slot = 1;
                                        loop {
                                            let more = slot < 5;
                                            at += 4;
                                            if !more {
                                                break;
                                            }
                                            if lw(at) < 0 {
                                                found = true;
                                                break;
                                            }
                                            slot += 1;
                                        }
                                    }
                                    if found {
                                        let row = lw(g::gunzlistrow);
                                        sw((slot << 2) + g::gunzpack, lw((row << 2) + 0x1fc2a00));
                                        soundStart(143, 0);
                                        item = lhu(a0 + 4);
                                        break 'input;
                                    }
                                    sound = 142;
                                }
                                soundStart(sound, 0);
                                item = lhu(a0 + 4);
                            } else {
                                language = lw(g::cur_language);
                                if lhu(a0 + 4) != 2 {
                                    break 'plain;
                                }
                                weaponSetInc();
                                item = lhu(a0 + 4);
                            }
                        } else {
                            language = lw(g::cur_language);
                            if lhu(a0 + 4) != 2 {
                                break 'plain;
                            }
                            weaponSetDec();
                            item = lhu(a0 + 4);
                        }
                    } else {
                        item = lhu(a0 + 4);
                        if item == 1 {
                            let row = lw(g::gunzlistrow) - 1;
                            sw(g::gunzlistrow, row);
                            let mut row = row;
                            if row < 0 {
                                let count = lw(0x3aff4c);
                                row = count - 1;
                                sw(g::gunzlistrowtop, count - 10);
                                sw(g::gunzlistrow, row);
                            }
                            if row < lw(g::gunzlistrowtop) {
                                sw(g::gunzlistrowtop, row);
                            }
                        }
                    }
                } else {
                    item = lhu(a0 + 4);
                    if item == 1 {
                        let mut row = lw(g::gunzlistrow) + 1;
                        let inside = (row < lw(0x3aff4c)) as i32;
                        sw(g::gunzlistrow, row);
                        if inside == 0 {
                            sw(g::gunzlistrow, 0);
                            row = 0;
                            sw(g::gunzlistrowtop, 0);
                        }
                        if row >= lw(g::gunzlistrowtop) + 10 {
                            sw(g::gunzlistrowtop, row - 9);
                        }
                    }
                }
            }
            language = lw(g::cur_language);
            if item == 2 {
                let text = if weaponset_read() != 0 { 0 } else { weapons_legend_or(lw(lw((lw(g::cur_language) << 2) + g::languages) + 1656)) };
                let front = lw(g::front);
                sw(g::i0600 + 64, text);
                sw(front + 12, lw(front + 12) & -49);
                break 'text;
            }
        }
        sw(g::i0600 + 64, lw(lw((language << 2) + g::languages) + 1660));
    }
    let menu = g::weapons_menu;
    menutick_230238(a0, menu);
    let made = std_menumake(lw(a0 + 8), menu);
    sw(a0 + 12, made);
}

/// Test a moving prop `a1` against the walls of the prop it may collide with (@ 0x270950). The walls near the prop's height are classified by which side of each wall the
/// moving point is on; when more sit on one side the prop is considered to be inside and the whole wall list is tested (`MoveTestWall`), else the nearest wall's normal
/// is found and the prop is moved out along it with `moveTest`. Returns the fraction of the move that was done.
pub fn MoveTestPropWalls(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, f12: f32, f13: f32) -> f32 {
    let frame = Frame::new(288);
    let sp = frame.sp();
    let mut left_count = 0;
    let mut right_count = 0;
    sw(sp + 84, a0);
    swc1(a6, f12);
    sw(sp + 88, a4);
    let less;
    if propDoCollisionsCanPush(lw(a1 + 4)) == 0 {
        less = (right_count < left_count) as i32;
    } else {
        let walls = lw(a1 + 484);
        let mut wall = lw(walls);
        let mut i = 0;
        if wall != 0 {
            let y = lwc1(a3 + 4);
            let mut top = ps2float::add(y, f13);
            loop {
                if !(lwc1(wall + 8) < top) {
                    i += 1;
                } else if !(y < ps2float::sub(lwc1(wall + 16), lwc1(0x3acd10))) {
                    i += 1;
                } else {
                    let ax = lwc1(wall + 12);
                    let z = lwc1(a3 + 8);
                    let mut side_a = 1;
                    if !(z <= ax) {
                        side_a = 0;
                    }
                    let bx = lwc1(wall + 28);
                    let mut side_b = 1;
                    if !(z <= bx) {
                        side_b = 0;
                    }
                    if side_a == side_b {
                        i += 1;
                    } else {
                        let cx = lwc1(wall + 20);
                        let dx = ps2float::sub(cx, lwc1(a3));
                        let cross = ps2float::mul(ps2float::sub(lwc1(wall + 4), cx), ps2float::sub(bx, z));
                        let mut side_c = 1;
                        if !(ps2float::mul(dx, ps2float::sub(ax, bx)) <= cross) {
                            side_c = 0;
                        }
                        if side_c != side_b {
                            i += 1;
                        } else {
                            if !(0.0_f32 < ps2float::add(ps2float::mul(ps2float::sub(bx, ax), dx), cross)) {
                                right_count += 1;
                            } else {
                                left_count += 1;
                            }
                            i += 1;
                        }
                    }
                }
                wall = lw((i << 2) + walls);
                if wall == 0 {
                    break;
                }
                top = ps2float::add(y, f13);
            }
        }
        less = (right_count < left_count) as i32;
    }
    if less == 0 {
        sw(sp, sp + 64);
        swc1(sp + 80, 1.0_f32);
        MoveTestWall(lw(a1 + 484), 0, a2, a3, lw(sp + 88), sp + 80, sp + 16, a6, f12, f13);
        let fraction = lwc1(sp + 80);
        if a5 != 0 {
            let mut x = lwc1(sp + 16);
            let y;
            if !(fraction < 1.0_f32) {
                x = lwc1(sp + 64);
                y = lwc1(sp + 68);
            } else {
                y = lwc1(sp + 20);
            }
            swc1(a5, x);
            swc1(a5 + 4, y);
        }
        return fraction;
    }
    let walls = lw(a1 + 484);
    let mut nearest = lwc1(0x3af938);
    let mut wall = lw(walls);
    let mut i = 0;
    if wall != 0 {
        let mut y = lwc1(a3 + 4);
        loop {
            'wall: {
                if !(lwc1(wall + 8) < ps2float::add(y, f13)) {
                    i += 1;
                } else if !(y < lwc1(wall + 16)) {
                    i += 1;
                } else {
                    let dx = ps2float::sub(lwc1(wall + 28), lwc1(wall + 12));
                    swc1(sp + 32, dx);
                    let dz = ps2float::sub(lwc1(wall + 4), lwc1(wall + 20));
                    let length = ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dz, dz)).abs().psqrt();
                    swc1(sp + 36, dz);
                    let nx = (lwc1(sp + 32)).pdiv(length);
                    let px = lwc1(a3);
                    let nz = (lwc1(sp + 36)).pdiv(length);
                    let pz = lwc1(a3 + 8);
                    swc1(sp + 32, nx);
                    swc1(sp + 36, nz);
                    let ox = ps2float::sub(lwc1(wall + 4), px);
                    swc1(sp + 48, ox);
                    let oz = ps2float::sub(lwc1(wall + 12), pz);
                    let distance = ps2float::add(ps2float::mul(nx, ox), ps2float::mul(nz, oz));
                    swc1(sp + 52, oz);
                    if 0.0_f32 < distance {
                        if !(distance < nearest) {
                            i += 1;
                            break 'wall;
                        }
                        swc1(sp + 16, nx);
                        nearest = distance;
                        swc1(sp + 20, nz);
                    }
                    i += 1;
                }
            }
            wall = lw((i << 2) + walls);
            if wall == 0 {
                break;
            }
            y = lwc1(a3 + 4);
        }
    }
    let ny = lwc1(sp + 20);
    if nearest < lwc1(0x3af938) {
        let push = ps2float::add(nearest, lwc1(0x3acd14));
        let x = lwc1(a3);
        let dx = ps2float::mul(push, lwc1(sp + 16));
        let y = lwc1(a3 + 4);
        let dz = ps2float::add(lwc1(a3 + 8), ps2float::mul(push, ny));
        sw(a1 + 16, lw(a1 + 16) & -33);
        swc1(sp + 36, y);
        swc1(sp + 40, dz);
        swc1(sp + 32, ps2float::add(x, dx));
        let result = moveTest(lw(sp + 84), a3, sp + 32, 0, 0, 0, f13, f12);
        sw(a1 + 16, lw(a1 + 16) | 32);
        if (result & 2) == 0 {
            let x = lwc1(sp + 32);
            let y = lwc1(sp + 36);
            let z = lwc1(sp + 40);
            swc1(a3, x);
            swc1(a3 + 8, z);
            swc1(a3 + 4, y);
            let ox = ps2float::sub(x, lwc1(a1 + 48));
            swc1(a5, ox);
            let oz = ps2float::sub(lwc1(a3 + 8), lwc1(a1 + 56));
            swc1(a5 + 4, oz);
            let length = ps2float::add(ps2float::mul(ox, ox), ps2float::mul(oz, oz)).abs().psqrt();
            let first = lwc1(a5);
            let second = (lwc1(a5 + 4)).pdiv(length);
            let slowed = ps2float::mul(f12, lwc1(0x3acd18));
            swc1(a5, (first).pdiv(length));
            swc1(a5 + 4, second);
            swc1(a6, slowed);
            return 0.0_f32;
        }
    }
    1.0_f32
}
