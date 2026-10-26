//! Functions: the electricity props' tick, the front end menu tick, saving the pads, loading the background of a level, finding the bearings of a lost character, moving a
//! prop along its path and drawing a character on the select screens.

use super::particle_spawners::random_as_float;
use crate::*;

/// Tick of the electricity props (@ 0x27a518): the lightning bolts of the prop are turned on and off at random (at most 4 of the 12 lines are on, and each is on for a while
/// and switches off by chance), and every line that is on gets its vertices wiggled by a sine wave and its texture coordinates shifted, in the override chunks of the prop.
pub fn electricityPropTick(a0: i32) {
    let frame = Frame::new(208);
    let sp = frame.sp();
    let mut index = 0;
    let mut model = lw(a0 + 32);
    sw(sp + 4, 0);
    let first = lw(model);
    let chunks = lw(first);
    sw(sp + 8, first - chunks * 80);
    if chunks > 0 {
        loop {
            let next;
            let mut counted = lw(model);
            'chunk: {
                let line = index - 1;
                let offset = line << 2;
                let entry = lw(sp + 8) + index * 80;
                if lw(offset + g::electricityOn) != 0 {
                    allocateOverrideChunk(a0, index, 1, 3);
                    let vertices = getOverrideChunk(a0, index, 1, 1, sp);
                    let coordinates = getOverrideChunk(a0, index, 1, 2, sp);
                    if vertices != 0 {
                        if coordinates != 0 {
                            let mut vertex = vertices;
                            let mut coordinate = coordinates;
                            let mut source_uv = lw(entry + 52);
                            let mut source = lw(entry + 48);
                            let time = ps2float::mul(ps2float::mul(gameframe_x(), 12.0_f32), lwc1(0x3acef8));
                            if lw(g::updateframesi) != 0 {
                                swc1(offset + g::electricityTextureOffset, ps2float::mul(random_as_float(), 2.3283064e-10_f32));
                            }
                            let count = lw(sp);
                            let mut i = 0;
                            if count <= 0 {
                                model = lw(a0 + 32);
                                next = index + 1;
                                counted = lw(model);
                                break 'chunk;
                            }
                            model = lw(a0 + 32);
                            next = index + 1;
                            loop {
                                let x = lwc1(source);
                                let scale = lwc1(0x3acefc);
                                let amplitude = lwc1(0x3acf00);
                                i += 1;
                                let wave_a = lwc1((((ps2float::add(ps2float::mul(ps2float::add(ps2float::mul(x, 8.0_f32), time), scale), 0.5_f32) as i32) & 2047) << 2) + g::g_SinTable);
                                let wave_b = lwc1(((((ps2float::add(ps2float::mul(ps2float::add(ps2float::mul(lwc1(source + 8), 8.0_f32), time), scale), 0.5_f32) as i32) + 512) & 2047) << 2) + g::g_SinTable);
                                swc1(vertex, x);
                                swc1(vertex + 4, ps2float::add(ps2float::add(lwc1(source + 4), ps2float::mul(wave_a, amplitude)), ps2float::mul(wave_b, amplitude)));
                                let z = lwc1(source + 8);
                                source += 12;
                                swc1(vertex + 8, z);
                                vertex += 12;
                                swc1(coordinate, ps2float::add(lwc1(source_uv), lwc1((line << 2) + g::electricityTextureOffset)));
                                let v = lwc1(source_uv + 4);
                                swc1(coordinate + 12, 1.0_f32);
                                source_uv += 16;
                                swc1(coordinate + 4, v);
                                swc1(coordinate + 8, 1.0_f32);
                                coordinate += 16;
                                if i >= count {
                                    break;
                                }
                            }
                            counted = lw(model);
                            break 'chunk;
                        }
                    }
                    next = index + 1;
                    model = lw(a0 + 32);
                    counted = lw(model);
                    break 'chunk;
                }
                next = index + 1;
                counted = lw(model);
            }
            index = next;
            if index >= lw(counted) {
                break;
            }
        }
    }
    sh(model + 292, 0);
    let limit = lwc1(0x3acf04);
    let mut line = 0;
    let mut offset = line << 2;
    loop {
        let on = lw(offset + g::electricityOn);
        let next = line + 1;
        if on == 1 {
            let time = offset + g::electricityTime;
            let model = lw(a0 + 32);
            let step = (lwc1(g::updateframesf)).pdiv(6e+01_f32);
            let old = lwc1(time);
            let mask = lhu(model + 292);
            let frames = lw(g::updateframesi);
            sw(sp + 4, lw(sp + 4) + 1);
            sh(model + 292, mask | on << (next & 31));
            swc1(time, ps2float::add(old, step));
            if frames != 0 {
                let random = ps2float::mul(random_as_float(), 2.3283064e-10_f32);
                let at = offset + g::electricityOn;
                if ps2float::add(ps2float::mul((random).pdiv(lwc1(g::updateframesf)), limit), limit) <= lwc1(time) {
                    sw(at, 0);
                }
            }
        }
        line = next;
        offset = line << 2;
        if line >= 12 {
            break;
        }
    }
    let mut made = 0;
    if (4 - lw(sp + 4)) > 0 {
        loop {
            let next = made + 1;
            let mut at;
            loop {
                let random = newrnd();
                at = (((random as i32 as u32) % (12 as u32)) as i32) << 2;
                if lw(at + g::electricityOn) != 1 {
                    break;
                }
            }
            sw(at + g::electricityOn, 1);
            sw(at + g::electricityTime, 0);
            let random = newrnd();
            let kind = ((random as i32 as u32) % (3 as u32)) as i32;
            if kind == 1 {
                soundStartRange(19, a0 + 48, 0, 25.0_f32);
            } else if kind == 0 {
                soundStartRange(18, a0 + 48, 0, 25.0_f32);
            } else if kind == 2 {
                soundStartRange(20, a0 + 48, 0, 25.0_f32);
            }
            let active = lw(sp + 4);
            made = next;
            if made >= 4 - active {
                break;
            }
        }
    }
}

/// Tick of the front end menus (@ 0x230238): move the cursor off a disabled item, scroll the list to keep the item in view, change a value item with left and right,
/// select an item or open its page with accept and go back a page with back.
pub fn menutick_230238(a0: i32, a1: i32) {
    let id = lhu(a0 + 4);
    if id == 0 {
        return;
    }
    'tick: {
        let mut item = menuitem(a1, id);
        let mut front = lw(g::front);
        if item != 0 {
            let mut index = 0;
            let mut offset = 0;
            let mut direction = 0;
            let mut found = false;
            let mut cells = 0;
            loop {
                cells = item + 16;
                if (lw(front + 12) & (4 << (index & 31))) != 0 {
                    direction = offset;
                    if lhu(cells + offset) != 0 {
                        found = true;
                        break;
                    }
                }
                index += 1;
                offset += 2;
                if index >= 4 {
                    break;
                }
            }
            if found {
                let mut at = cells + direction;
                loop {
                    let value = lhu(at);
                    sh(a0 + 4, value);
                    item = menuitem(a1, value & 0xffff);
                    let list = item + 16;
                    if (lhu(item + 10) & 0x4000) == 0 {
                        break;
                    }
                    at = list + direction;
                    if lhu(list + direction) == 0 {
                        break;
                    }
                }
                soundStart(140, 0);
                let current = lw(g::front);
                sw(current + 12, lw(current + 12) & -61);
            }
            let list = isinlist(a1, lhu(a0 + 4));
            front = lw(g::front);
            let mut finished = false;
            let mut buttons = 0;
            if list != 0 {
                let base = list + 12;
                let row = (item - (list + 16)) >> 5;
                let scroll = base + lw(front);
                let relative = row - lbu(scroll);
                let rows;
                'scroll: {
                    if relative > 0 {
                        let visible = lhu(list + 2);
                        if relative < visible - 1 {
                            rows = lhu(list);
                            break 'scroll;
                        }
                        sb(scroll, row - lbu(list + 2) + 2);
                    } else {
                        let lower = row - 1;
                        sb(scroll, if -1 >= lower { 0 } else { lower });
                    }
                    front = lw(g::front);
                    rows = lhu(list);
                }
                if row < rows - 1 {
                    buttons = lw(front + 12);
                    finished = true;
                } else {
                    sb(base + lw(front), lbu(list) - lbu(list + 2));
                }
            }
            if !finished {
                front = lw(g::front);
                buttons = lw(front + 12);
            }
            if (buttons & 48) != 0 {
                let flags = lhu(item + 10);
                if (flags & 4) != 0 {
                    if (flags & 64) != 0 {
                        soundStart(142, 0);
                        front = lw(g::front);
                    } else {
                        let mut step = lw(front + 16);
                        if (lw(lw(item + 12) + 12) & 2) != 0 {
                            step = step * 655;
                        }
                        let delta = step >> 5;
                        if delta != 0 {
                            soundStart(141, 0);
                            let value = lw(item + 12);
                            gvSetValue(value, lw(value) + delta);
                            return;
                        }
                    }
                }
            }
            let pressed = joyNewBut(lw(front));
            front = lw(g::front);
            if (pressed & 64) != 0 {
                let entry = isinlist(a1, lhu(a0 + 4));
                if entry == 0 {
                    front = lw(g::front);
                } else if (lhu(entry + 8) & 1) == 0 {
                    front = lw(g::front);
                } else {
                    if (lhu(item + 10) & 64) == 0 {
                        menuitemSelect(a1, lhu(a0 + 4));
                        sh(entry + 10, lhu(a0 + 4));
                        soundStart(142, 0);
                        return;
                    }
                    front = lw(g::front);
                    sw(front + 12, lw(front + 12) & -61);
                }
            }
            let pressed = joyNewBut(lw(front));
            let _ = lw(g::front);
            if (pressed & 64) == 0 {
                break 'tick;
            }
            let entry = isinlist(a1, lhu(a0 + 4));
            let flags;
            if entry == 0 {
                flags = lhu(item + 10);
            } else {
                if (lhu(entry + 8) & 1) != 0 {
                    if (lhu(item + 10) & 64) != 0 {
                        soundStart(142, 0);
                        break 'tick;
                    }
                    menuitemSelect(a1, lhu(a0 + 4));
                    sh(entry + 10, lhu(a0 + 4));
                    soundStart(142, 0);
                    return;
                }
                flags = lhu(item + 10);
            }
            let disabled = flags & 64;
            if (flags & 1) != 0 {
                if disabled == 0 {
                    if lhu(item + 8) == 0x8000 {
                        pageOut();
                        return;
                    }
                    frontPageSet(0, lw(item + 12));
                    pageIn(a0);
                    return;
                }
                soundStart(142, 0);
            }
        }
    }
    let front = lw(g::front);
    if (joyNewBut(lw(front)) & 16) != 0 {
        pageOut();
    }
}

/// Save the pads and links of the level to the pad file (@ 0x2658f0): the number of used pads and links is counted (and the number kept for the map maker is updated), then
/// the header, one line per used pad (its id, flags, room, position and heading) and one per used link are written to a buffer that is saved to the file (the old file is
/// copied away first).
pub fn padSave() {
    let frame = Frame::new(1200);
    let sp = frame.sp();
    let name = padFileOutName();
    sw(sp + 1024, name);
    let buffer = malloc(0xf4240);
    let pads = lw(g::numpads);
    let mut i = 0;
    let mut used_pads = 0;
    let mut a4 = 0;
    let kept;
    let links;
    if pads <= 0 {
        kept = lw(0x3b00a0);
        links = lw(g::numlinks);
    } else {
        a4 = lw(g::levelpads);
        kept = lw(0x3b00a0);
        links = lw(g::numlinks);
        loop {
            let offset = i * 28;
            i += 1;
            used_pads += lw(offset + a4 + 4) & 1;
            if i >= pads {
                break;
            }
        }
    }
    if kept != used_pads {
        sw(0x3b00a0, used_pads);
    }
    let mut i = 0;
    let mut used_links = 0;
    if links > 0 {
        a4 = lw(g::levellinks);
        loop {
            let offset = i * 20;
            i += 1;
            used_links += lw(offset + a4) & 1;
            if i >= links {
                break;
            }
        }
    }
    sprintf(&[buffer as i64, 0x3a27d0, i as i64, used_links as i64, a4 as i64, links as i64, pads as i64, kept as i64]);
    let mut end = buffer + strlen(buffer);
    sprintf(&[end as i64, 0x3a2808, lw(0x3b00a0) as i64, used_links as i64]);
    end += strlen(end);
    sprintf(&[end as i64, 0x3a2830]);
    end += strlen(end);
    if lw(g::numpads) > 0 {
        let mut offset = 0;
        let mut count = 0;
        loop {
            let pad = offset + lw(g::levelpads);
            let flags = lw(pad + 4);
            sw(pad + 4, flags & -0x10001);
            let total;
            'pad: {
                if (flags & 1) != 0 {
                    total = lw(g::numpads);
                    if lw(pad) == -1 {
                        break 'pad;
                    }
                    let x = fptodp(lwc1(pad + 12));
                    let room = lh(pad + 8);
                    let y = fptodp(lwc1(pad + 16));
                    let z = fptodp(lwc1(offset + lw(g::levelpads) + 20));
                    let levelpads = lw(g::levelpads);
                    sd(sp, z);
                    let w = fptodp(lwc1(offset + levelpads + 24));
                    sd(sp + 8, w);
                    sprintf(&[end as i64, 0x3a2848, lw(g::pad_rev) as i64, lw(pad) as i64, lw(pad + 4) as i64, room as i64, x, y]);
                    end += strlen(end);
                }
            }
            let _ = total;
            let limit = lw(g::numpads);
            count += 1;
            offset += 28;
            if count >= limit {
                break;
            }
        }
    }
    sprintf(&[end as i64, 0x3a2870]);
    end += strlen(end);
    let mut remaining = lw(g::numlinks);
    if remaining > 0 {
        let mut offset = 0;
        let mut count = 0;
        loop {
            let link = offset + lw(g::levellinks);
            let pads = lw(g::levelpads);
            if (lw(link) & 1) != 0 {
                let length = fptodp(lwc1(link + 12));
                let to = lw(link + 8) * 28 + pads;
                let from = lw(link + 4) * 28 + pads;
                sprintf(&[end as i64, 0x3a2890, lw(g::pad_rev) as i64, lw(link) as i64, lw(from) as i64, lw(to) as i64, length]);
                end += strlen(end);
                remaining = lw(g::numlinks);
            }
            count += 1;
            offset += 20;
            if count >= remaining {
                break;
            }
        }
    }
    sprintf(&[(sp + 16) as i64, 0x3aeb28, lw(sp + 1024) as i64, 0, 0, remaining as i64]);
    let size = fileSize(lw(sp + 1024));
    if size <= 0 {
        fileSave(lw(sp + 1024), buffer, end - buffer);
    } else {
        fileCopy(lw(sp + 1024), sp + 16);
        fileSave(lw(sp + 1024), buffer, end - buffer);
    }
    free(buffer);
}

/// Load the background of the level from the preloaded file `a0` (@ 0x253b88): the offsets in the file are relocated into pointers (rooms, their objects, portals and the
/// lists of every room), the rooms are given the level's textures, and the tables for the rooms and portals are allocated.
pub fn bgLoad(a0: i32) {
    let file = preloadGetFile(a0);
    let base = lw(file + 16);
    let mut rooms = lw(base + 4) + base;
    let volumes = lw(base + 12);
    let portals = lw(base + 8) + base;
    sw(g::levelrooms, rooms);
    sw(g::levelportals, portals);
    if volumes != 0 {
        let at = volumes + base;
        let count = lw(at);
        sw(g::levelvolumes, at + 4);
        sw(g::bgNumVolumes, count);
    }
    let first = lw(rooms + 44);
    let mut room_count = 0;
    sw(g::bgNumRooms, 0);
    let mut number = 1;
    let mut portal_table;
    if first == 0 {
        portal_table = lw(g::levelportals);
    } else {
        loop {
            let offset = number * 44;
            let entry = number * 44 + rooms;
            let relative = lw(entry);
            let object = relative + base;
            if relative != 0 {
                sw(entry, object);
                sw(object + 36, 1);
                obRelocate(object, base);
                obCalcBounds(lw(offset + lw(g::levelrooms)), 0);
                rooms = lw(g::levelrooms);
            }
            let object = lw(offset + rooms);
            sw(object + 12, lw(g::leveltextures));
            propSetOb(number + 512, object);
            let rooms_now = lw(g::levelrooms);
            let room = offset + lw(g::levelrooms);
            let word = lw(room + 4);
            if word != 0 {
                sw(room + 4, word + base);
            }
            let next_number = number + 1;
            'list: {
                let relative = lw(room + 8);
                let list = relative + base;
                if relative != 0 {
                    let mut i = 0;
                    let head = lw(list);
                    sw(room + 8, list);
                    if head != 0 {
                        let mut at = list;
                        loop {
                            'wall: {
                                let wall = lw(at) + base;
                                let x = lwc1(wall + 12);
                                let equal = x == lwc1(wall + 24);
                                sw(at, wall);
                                if equal {
                                    if !(x == lwc1(wall + 36)) {
                                        i += 1;
                                        break 'wall;
                                    }
                                    sh(wall + 6, lhu(wall + 6) | 16);
                                }
                                i += 1;
                            }
                            at = (i << 2) + list;
                            if lw((i << 2) + list) == 0 {
                                break 'list;
                            }
                        }
                    }
                }
            }
            'second: {
                let room = number * 44 + rooms_now;
                let relative = lw(room + 12);
                let list = relative + base;
                if relative != 0 {
                    let mut i = 0;
                    let head = lw(list);
                    sw(room + 12, list);
                    if head != 0 {
                        let mut at = list;
                        loop {
                            i += 1;
                            sw(at, lw(at) + base);
                            at = (i << 2) + list;
                            if lw((i << 2) + list) == 0 {
                                break 'second;
                            }
                        }
                    }
                }
            }
            'third: {
                let room = number * 44 + rooms_now;
                let relative = lw(room + 16);
                let list = relative + base;
                if relative != 0 {
                    let mut i = 0;
                    let head = lw(list);
                    sw(room + 16, list);
                    if head != 0 {
                        let mut at = list;
                        loop {
                            i += 1;
                            sw(at, lw(at) + base);
                            at = (i << 2) + list;
                            if lw((i << 2) + list) == 0 {
                                break 'third;
                            }
                        }
                    }
                }
            }
            let offset = number * 44;
            number = next_number;
            obPrecalcGfxData(lw(offset + rooms_now));
            rooms = lw(g::levelrooms);
            room_count = lw(g::bgNumRooms) + 1;
            sw(g::bgNumRooms, room_count);
            if lw(number * 44 + rooms) == 0 {
                break;
            }
        }
        portal_table = lw(g::levelportals);
    }
    let mut portals_found = 0;
    sw(g::bgNumPortals, 0);
    let mut i = 1;
    if lw(portal_table + 4) != 0 {
        let mut at = portal_table + 4;
        loop {
            i += 1;
            portals_found += 1;
            sw(at, lw(at) + base);
            sw(g::bgNumPortals, portals_found);
            at = (i << 2) + portal_table;
            if lw((i << 2) + portal_table) == 0 {
                break;
            }
        }
    }
    let _ = &mut portal_table;
    let room_data = g::roomdata;
    let table = memAlloc((room_count + 1) * 48, 1);
    let count = lw(g::bgNumRooms);
    sw(g::roombb, table);
    let table = memAlloc(((count + 1) * 20 + 15) & -16, 1);
    let count = lw(g::bgNumRooms);
    sw(g::roomdata, table);
    let table = memAlloc((count * 20 + 35) & -16, 1);
    let count = lw(g::bgNumRooms);
    sw(room_data + 4, table);
    let table = memAlloc((count * 20 + 35) & -16, 1);
    let count = lw(g::bgNumRooms);
    sw(room_data + 8, table);
    let table = memAlloc((count * 20 + 35) & -16, 1);
    let count = lw(g::bgNumRooms);
    sw(room_data + 12, table);
    let table = memAlloc(((count << 2) + 19) & -16, 1);
    let count = lw(g::bgNumPortals);
    sw(0x3b0084, table);
    let table = memAlloc((count * 48 + 63) & -16, 1);
    let count = lw(g::bgNumRooms);
    sw(g::portaldata, table);
    let table = memAlloc(((count << 2) + 19) & -16, 1);
    sw(g::roomprops, table);
    bgRoomInitBB();
    bgPortalInitBB();
}

/// Find the bearings of the character `a0` that has got lost (@ 0x2c6018): when the character is not on screen for any player it is put at the nearest pad that no player
/// has in view; otherwise it is moved to a pad out of the lines of sight of the players, found by repeated `closestpad` searches (at most 75) with a line test, and the
/// pad, room and mode are set.
pub fn findbearings(a0: i32) {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let mut line = 0;
    let mut tries = 0;
    let state = lw(a0 + 352);
    let position = a0 + 48;
    'done: {
        if (lw(a0 + 16) & 0xf000000) == 0 {
            let mut seen = 0;
            let pad = closestpad(position, 0.0_f32);
            let players = lw(g::lvnumplayers);
            let mut player = 0;
            'seen: {
                if players > 0 {
                    let all = lw(g::players);
                    loop {
                        let mut i = 0;
                        if lw(player * 1820 + all + 908) <= 0 {
                            player += 1;
                        } else {
                            let pads = lw(g::levelpads);
                            let next = player + 1;
                            loop {
                                let offset = i << 2;
                                i += 1;
                                let at = player * 1820 + all;
                                seen = if (lw(at + offset + 912) ^ lh(pad * 28 + pads + 10)) == 0 { 1 } else { seen };
                                if i >= lw(at + 908) {
                                    break;
                                }
                            }
                            player = next;
                        }
                        if player >= players {
                            break 'seen;
                        }
                    }
                }
            }
            if seen == 0 {
                let pads = lw(g::levelpads);
                sw(state + 680, pad);
                sw(state + 1688, 0);
                sw(state + 2692, 0);
                let at = pad * 28 + pads;
                swc1(a0 + 48, lwc1(at + 12));
                swc1(a0 + 52, lwc1(at + 16));
                swc1(a0 + 56, lwc1(at + 20));
                let room = lh(at + 10);
                sw(state + 2892, 0);
                sw(a0 + 12, room);
                let mode = lw(state + 2716);
                sw(state + 2716, 512);
                sw(state + 2720, mode);
                enemySetRoom(a0, 0);
                let model = lw(a0 + 32);
                sw(model + 128, 0);
                sw(model + 132, 0);
                sw(model + 136, 0);
                swc1(model + 104, lwc1(a0 + 48));
                swc1(model + 108, lwc1(a0 + 52));
                swc1(model + 112, lwc1(a0 + 56));
                swc1(model + 116, lwc1(a0 + 48));
                swc1(model + 120, lwc1(a0 + 52));
                swc1(model + 124, lwc1(a0 + 56));
                break 'done;
            }
        }
        let mut pad;
        'nearest: {
            enemySetMoveMode(a0, 8);
            if lw(state + 3000) != 0 {
                let current = lw(state + 680);
                if current >= 0 {
                    let at = current * 28 + lw(g::levelpads);
                    let dy = ps2float::sub(lwc1(a0 + 52), lwc1(at + 16));
                    let dx = ps2float::sub(lwc1(a0 + 48), lwc1(at + 12));
                    let dz = ps2float::sub(lwc1(a0 + 56), lwc1(at + 20));
                    let distance = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt();
                    pad = closestpad(position, distance);
                    break 'nearest;
                }
            }
            pad = closestpad(position, 0.0_f32);
        }
        let mut blocked = 0;
        if pad != -1 {
            let y = lwc1(a0 + 52);
            let z = lwc1(a0 + 56);
            let room = lw(a0 + 12);
            let at = pad * 28 + lw(g::levelpads);
            swc1(sp + 16, ps2float::sub(lwc1(at + 12), lwc1(a0 + 48)));
            swc1(sp + 20, ps2float::sub(lwc1(at + 16), y));
            swc1(sp + 24, ps2float::sub(lwc1(at + 20), z));
            swc1(sp + 32, lwc1(state + 3008));
            swc1(sp + 36, lwc1(state + 3012));
            let height = lwc1(state + 3016);
            sw(sp, 0);
            swc1(sp + 40, height);
            sw(sp + 8, 0);
            blocked = bgLineTest(room, sp + 32, sp + 16, 0, sp + 64, sp + 48, 0, 56);
        }
        let mode;
        'chosen: {
            'give_up: {
                'clear: {
                    if blocked != 0 {
                        let more = (tries < 75) as i32;
                        if pad == -1 {
                            break 'give_up;
                        }
                        if more != 0 {
                            let mut offset = pad * 28;
                            loop {
                                let at = offset + lw(g::levelpads);
                                let dy = ps2float::sub(lwc1(a0 + 52), lwc1(at + 16));
                                let dx = ps2float::sub(lwc1(a0 + 48), lwc1(at + 12));
                                let dz = ps2float::sub(lwc1(a0 + 56), lwc1(at + 20));
                                let distance = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt();
                                pad = closestpad(position, distance);
                                let new_offset = pad * 28;
                                if pad != -1 {
                                    let y = lwc1(a0 + 52);
                                    let z = lwc1(a0 + 56);
                                    let at = new_offset + lw(g::levelpads);
                                    let room = lw(a0 + 12);
                                    swc1(sp + 16, ps2float::sub(lwc1(at + 12), lwc1(a0 + 48)));
                                    swc1(sp + 20, ps2float::sub(lwc1(at + 16), y));
                                    swc1(sp + 24, ps2float::sub(lwc1(at + 20), z));
                                    swc1(sp + 32, lwc1(state + 3008));
                                    swc1(sp + 36, lwc1(state + 3012));
                                    let height = lwc1(state + 3016);
                                    sw(sp, 0);
                                    swc1(sp + 40, height);
                                    sw(sp + 8, 0);
                                    blocked = bgLineTest(room, sp + 32, sp + 16, 0, sp + 64, sp + 48, 0, 56);
                                }
                                tries += 1;
                                if blocked == 0 {
                                    break 'clear;
                                }
                                let more = (tries < 75) as i32;
                                if pad == -1 {
                                    break 'give_up;
                                }
                                offset = pad * 28;
                                if more == 0 {
                                    break 'clear;
                                }
                            }
                        }
                    }
                }
                if pad != -1 {
                    mode = lw(state + 2716);
                    break 'chosen;
                }
            }
            pad = closestpad(position, 0.0_f32);
            let at = pad * 28 + lw(g::levelpads);
            swc1(a0 + 48, lwc1(at + 12));
            swc1(a0 + 52, lwc1(at + 16));
            swc1(a0 + 56, lwc1(at + 20));
            sw(a0 + 12, lh(at + 10));
            mode = lw(state + 2716);
        }
        sw(state + 2716, 512);
        sw(state + 680, pad);
        sw(state + 2720, mode);
        sw(state + 1688, 0);
        sw(state + 2692, 0);
        sw(state + 2892, 0);
    }
    let _ = &mut line;
}

/// Move the path following prop `a0` along its path by the distance `f12` (@ 0x275b88). The current path step (`+12` of the state) is one of: a straight move to a point
/// (kind 0), an arc round a point in either direction (1 and 2), a wait for a time (4) or nothing (3). Moves that complete advance to the next step (and `updateData`) and
/// the rest of the distance carries on into it; the heading, pitch and roll are blended towards the step's values by the fraction done.
pub fn pathPosUpdateRel(a0: i32, f12: f32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let mut remaining = f12;
    let state = lw(a0 + 352);
    if !(0.0_f32 < remaining) {
        return;
    }
    let mut step = lw(state + 12);
    loop {
        let mut fraction = 0.0_f32;
        let mut advance = 0;
        let mut kind = lw(step);
        'kind: {
            if (kind as u32) < (5 as u32) {
                'abort: {
                    'blend: {
                        let mut radius = 0.0_f32;
                        let mut scale = 0.0_f32;
                        let mut angle = 0.0_f32;
                        match kind {
                            0 => {
                                let px = lwc1(a0 + 48);
                                let py = lwc1(a0 + 52);
                                let pz = lwc1(a0 + 56);
                                swc1(sp + 32, ps2float::add(lwc1(state + 40), lwc1(step + 4)));
                                swc1(sp + 36, ps2float::add(lwc1(state + 44), lwc1(step + 8)));
                                swc1(sp + 40, ps2float::add(lwc1(state + 48), lwc1(step + 12)));
                                let rx = ps2float::sub(px, lwc1(state + 40));
                                swc1(sp + 16, rx);
                                let ry = ps2float::sub(py, lwc1(state + 44));
                                swc1(sp + 20, ry);
                                let rz = ps2float::sub(pz, lwc1(state + 48));
                                swc1(sp + 24, rz);
                                let dx = ps2float::sub(lwc1(step + 4), rx);
                                swc1(sp, dx);
                                let dy = ps2float::sub(lwc1(step + 8), ry);
                                swc1(sp + 4, dy);
                                let dz = ps2float::sub(lwc1(step + 12), rz);
                                let length = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt();
                                swc1(sp + 8, dz);
                                let target_x = lwc1(sp + 32);
                                if !(length < remaining) {
                                    let ratio = (remaining).pdiv(length);
                                    let mx = ps2float::mul(lwc1(sp), ratio);
                                    let my = ps2float::mul(lwc1(sp + 4), ratio);
                                    let mz = ps2float::mul(lwc1(sp + 8), ratio);
                                    swc1(sp + 48, mx);
                                    swc1(sp + 52, my);
                                    swc1(sp + 56, mz);
                                    fraction = (ps2float::add(ps2float::add(ps2float::mul(mx, mx), ps2float::mul(my, my)), ps2float::mul(mz, mz)).abs().psqrt()).pdiv(length);
                                    let new_y = ps2float::add(lwc1(a0 + 52), lwc1(sp + 52));
                                    step = lw(state + 12);
                                    let new_z = ps2float::add(lwc1(a0 + 56), lwc1(sp + 56));
                                    swc1(a0 + 48, ps2float::add(lwc1(a0 + 48), lwc1(sp + 48)));
                                    swc1(a0 + 52, new_y);
                                    swc1(a0 + 56, new_z);
                                    break 'blend;
                                }
                                remaining = ps2float::sub(remaining, length);
                                let target_y = lwc1(sp + 36);
                                advance = 1;
                                let target_z = lwc1(sp + 40);
                                swc1(a0 + 48, target_x);
                                swc1(a0 + 52, target_y);
                                swc1(a0 + 56, target_z);
                                fraction = 1.0_f32;
                                step = lw(state + 12);
                                kind = lw(step);
                                break 'kind;
                            }
                            1 => {
                                let end = lwc1(step + 16);
                                let current = lwc1(state + 72);
                                let mut turn = ps2float::sub(end, current);
                                radius = lwc1(step + 20);
                                if turn < 0.0_f32 {
                                    turn = ps2float::add(turn, 3.6e+02_f32);
                                }
                                let arc = ps2float::mul(ps2float::mul(radius, lwc1(0x3acdf0)), ((turn).pdiv(3.6e+02_f32)));
                                if !(arc < remaining) {
                                    let part = ps2float::mul(turn, remaining);
                                    fraction = (remaining).pdiv(arc);
                                    let new_angle = ps2float::add(current, (part).pdiv(arc));
                                    swc1(state + 72, new_angle);
                                    if 3.6e+02_f32 < new_angle {
                                        swc1(state + 72, ps2float::sub(new_angle, 3.6e+02_f32));
                                    }
                                    remaining = 0.0_f32;
                                } else {
                                    swc1(state + 72, end);
                                    remaining = ps2float::sub(remaining, arc);
                                    fraction = 1.0_f32;
                                    advance = 1;
                                }
                                scale = lwc1(0x3acdf4);
                                let phase = ps2float::mul(ps2float::sub(lwc1(state + 72), 9e+01_f32), scale).pdiv(1.8e+02_f32);
                                let sine = sinf(phase);
                                swc1(a0 + 48, ps2float::add(lwc1(state + 28), ps2float::mul(radius, sine)));
                                angle = ps2float::sub(lwc1(state + 72), 9e+01_f32);
                            }
                            2 => {
                                let end = lwc1(step + 16);
                                let current = lwc1(state + 72);
                                let mut turn = ps2float::sub(current, end);
                                radius = lwc1(step + 20);
                                if turn < 0.0_f32 {
                                    turn = ps2float::add(turn, 3.6e+02_f32);
                                }
                                let arc = ps2float::mul(ps2float::mul(radius, lwc1(0x3acde8)), ((turn).pdiv(3.6e+02_f32)));
                                if !(arc < remaining) {
                                    let part = ps2float::mul(turn, remaining);
                                    let heading = lwc1(state + 56);
                                    fraction = (remaining).pdiv(arc);
                                    swc1(state + 72, ps2float::sub(current, (part).pdiv(arc)));
                                    if heading < 0.0_f32 {
                                        swc1(state + 56, ps2float::add(heading, 3.6e+02_f32));
                                    }
                                    remaining = 0.0_f32;
                                } else {
                                    swc1(state + 72, end);
                                    remaining = ps2float::sub(remaining, arc);
                                    fraction = 1.0_f32;
                                    advance = 1;
                                }
                                scale = lwc1(0x3acdec);
                                let phase = ps2float::mul(ps2float::add(lwc1(state + 72), 9e+01_f32), scale).pdiv(1.8e+02_f32);
                                let sine = sinf(phase);
                                swc1(a0 + 48, ps2float::add(lwc1(state + 28), ps2float::mul(radius, sine)));
                                angle = ps2float::add(lwc1(state + 72), 9e+01_f32);
                            }
                            3 => {
                                break 'blend;
                            }
                            _ => {
                                let wait = lwc1(state + 64);
                                if !(wait < remaining) {
                                    let left = ps2float::sub(wait, remaining);
                                    remaining = 0.0_f32;
                                    swc1(state + 64, left);
                                    kind = lw(step);
                                    break 'kind;
                                }
                                remaining = ps2float::sub(remaining, wait);
                                sw(state + 64, 0);
                                advance = 1;
                                kind = lw(step);
                                break 'kind;
                            }
                        }
                        let cosine = cosf(ps2float::mul(angle, scale).pdiv(1.8e+02_f32));
                        step = lw(state + 12);
                        swc1(a0 + 56, ps2float::add(lwc1(state + 36), ps2float::mul(radius, cosine)));
                        kind = lw(step);
                        break 'kind;
                    }
                    let _ = &mut kind;
                    break 'abort;
                }
                remaining = 0.0_f32;
                kind = lw(step);
            }
        }
        if kind != 3 {
            let pitch = lwc1(state + 68);
            let mut heading = lwc1(state + 60);
            swc1(state + 68, ps2float::add(pitch, ps2float::mul(fraction, ps2float::sub(lwc1(step + 28), pitch))));
            if 1.8e+02_f32 <= heading {
                heading = ps2float::sub(heading, 3.6e+02_f32);
            }
            let heading_end = lwc1(step + 32);
            let delta = if !(1.8e+02_f32 <= heading_end) { ps2float::sub(heading_end, heading) } else { ps2float::sub(ps2float::sub(heading_end, 3.6e+02_f32), heading) };
            let blended = ps2float::add(heading, ps2float::mul(fraction, delta));
            swc1(state + 60, blended);
            if blended < 0.0_f32 {
                swc1(state + 60, ps2float::add(blended, 3.6e+02_f32));
            }
            let mut roll = lwc1(state + 52);
            let roll_end = lwc1(step + 36);
            if 1.8e+02_f32 <= roll {
                roll = ps2float::sub(roll, 3.6e+02_f32);
            }
            let delta = if !(1.8e+02_f32 <= roll_end) { ps2float::sub(roll_end, roll) } else { ps2float::sub(ps2float::sub(roll_end, 3.6e+02_f32), roll) };
            let blended = ps2float::add(roll, ps2float::mul(fraction, delta));
            swc1(state + 52, blended);
            if blended < 0.0_f32 {
                swc1(state + 52, ps2float::add(blended, 3.6e+02_f32));
            }
            let mut bank = lwc1(state + 76);
            let bank_end = lwc1(step + 40);
            if 1.8e+02_f32 <= bank {
                bank = ps2float::sub(bank, 3.6e+02_f32);
            }
            let delta = if !(1.8e+02_f32 <= bank_end) { ps2float::sub(bank_end, bank) } else { ps2float::sub(ps2float::sub(bank_end, 3.6e+02_f32), bank) };
            let blended = ps2float::add(bank, ps2float::mul(fraction, delta));
            swc1(state + 76, blended);
            if blended < 0.0_f32 {
                swc1(state + 76, ps2float::add(blended, 3.6e+02_f32));
            }
            let sum = ps2float::add(lwc1(state + 76), lwc1(state + 72));
            swc1(state + 56, sum);
            if 3.6e+02_f32 <= sum {
                swc1(state + 56, ps2float::sub(sum, 3.6e+02_f32));
            }
            if lw(a0 + 4) == 311 {
                swc1(
                    state + 80,
                    ps2float::mul(ps2float::mul(lwc1((((ps2float::add(ps2float::mul(ps2float::mul(lw(state + 84) as f32, lwc1(0x3acdf8)), lwc1(0x3acdfc)), 0.5_f32) as i32) & 2047) << 2) + g::g_SinTable), ps2float::sub(3e+01_f32, lwc1(state + 68))).pdiv(3e+01_f32), lwc1(0x3ace00)),
                );
            }
        }
        let next = step + 44;
        if advance == 1 {
            sw(state + 12, next);
            updateData(a0);
        }
        if !(0.0_f32 < remaining) {
            break;
        }
        step = lw(state + 12);
    }
}

/// Draw a character on the select screens (@ 0x2369a0): the character is turned by the heading and rotated (a character of the kinds 40, 46 and 60 by 198 degrees, the
/// others by 180), its gun, second gun, stage and stage prop are drawn after it (opaque pass, then transparent) when their draw flags are set, and the positions are
/// moved back.
pub fn drawChar(a0: i32) {
    let frame = Frame::new(208);
    let sp = frame.sp();
    let model = lw(a0 + 32);
    let degrees = ps2float::mul(lwc1(a0 + 76), lwc1(0x3ac9c4));
    swc1(sp + 64, lwc1(model + 128));
    let angle = (degrees).pdiv(1.8e+02_f32);
    swc1(sp + 68, lwc1(model + 132));
    swc1(sp + 72, lwc1(model + 136));
    matrixRotY(sp, angle);
    matrixVecRotAligned(&[sp as i64, (sp + 64) as i64]);
    let data = lw(a0 + 352);
    if !menu_skip_x() {
        swc1(a0 + 48, ps2float::add(lwc1(a0 + 48), lwc1(sp + 64)));
    }
    let kind = lh(data + 4);
    if kind == 46 || kind == 60 || kind == 40 {
        if !menu_skip_x() {
            rotateChar(a0, 198.0_f32);
        }
    } else {
        if !menu_skip_x() {
            rotateChar(a0, 1.8e+02_f32);
        }
    }
    let front = lw(g::front);
    let stage_props = g::stageprop;
    let slot = lw(front) << 2;
    let stage_prop = lw(slot + stage_props);
    if !menu_skip_x() {
        swc1(a0 + 56, ps2float::add(lwc1(a0 + 56), lwc1(sp + 72)));
    }
    if stage_prop != 0 {
        if lw(slot + g::propdrawflag) != 0 {
            let prop_model = lw(stage_prop + 32);
            let scale = lwc1(0x3ac9c8);
            let half_turn = 1.8e+02_f32;
            swc1(sp + 64, lwc1(prop_model + 128));
            swc1(sp + 68, lwc1(prop_model + 132));
            swc1(sp + 72, lwc1(prop_model + 136));
            let angle = ps2float::mul(lwc1(stage_prop + 76), scale).pdiv(half_turn);
            matrixRotY(sp, angle);
            matrixVecRotAligned(&[sp as i64, (sp + 64) as i64]);
            let front = lw(g::front);
            let prop = lw((lw(front) << 2) + stage_props);
            let z = lwc1(prop + 56);
            if !menu_skip_x() {
                swc1(prop + 48, ps2float::add(lwc1(prop + 48), lwc1(sp + 64)));
            }
            let dz = lwc1(sp + 72);
            swc1(prop + 76, half_turn);
            swc1(prop + 84, half_turn);
            if !menu_skip_x() {
                swc1(prop + 56, ps2float::add(z, dz));
            }
        }
    }
    if (lw(a0 + 16) & 1) != 0 {
        dlSetBlend(0);
        let old_zb = dlSetZB(3);
        obInstGfx(lw(a0 + 32), 0);
        let front = lw(g::front);
        let slot = lw(front) << 2;
        let gun = lw(slot + g::gun);
        if gun != 0 {
            if lw(slot + g::gundrawflag) != 0 {
                if (lw(gun + 16) & 256) == 0 {
                    obInstGfx(lw(gun + 32), 0);
                }
            }
        }
        let front = lw(g::front);
        let slot = lw(front) << 2;
        let gun2 = lw(slot + g::gun2);
        if gun2 != 0 {
            if lw(slot + g::gundrawflag2) != 0 {
                if (lw(gun2 + 16) & 256) == 0 {
                    obInstGfx(lw(gun2 + 32), 0);
                }
            }
        }
        let front = lw(g::front);
        let stage = lw((lw(front) << 2) + g::stage);
        if stage != 0 {
            obInstGfx(lw(stage + 32), 0);
        }
        let slot = lw(lw(g::front)) << 2;
        let prop = lw(slot + g::stageprop);
        if prop != 0 {
            if lw(slot + g::propdrawflag) != 0 {
                if (lw(prop + 16) & 256) == 0 {
                    obInstGfx(lw(prop + 32), 0);
                }
            }
        }
        dlSetBlend(1);
        dlSetZB(2);
        obInstGfx(lw(a0 + 32), 1);
        let front = lw(g::front);
        let slot = lw(front) << 2;
        let gun = lw(slot + g::gun);
        if gun != 0 {
            if lw(slot + g::gundrawflag) != 0 {
                if (lw(gun + 16) & 256) == 0 {
                    obInstGfx(lw(gun + 32), 1);
                }
            }
        }
        let front = lw(g::front);
        let slot = lw(front) << 2;
        let gun2 = lw(slot + g::gun2);
        if gun2 != 0 {
            if lw(slot + g::gundrawflag2) != 0 {
                if (lw(gun2 + 16) & 256) == 0 {
                    obInstGfx(lw(gun2 + 32), 1);
                }
            }
        }
        let slot = lw(lw(g::front)) << 2;
        let prop = lw(slot + g::stageprop);
        if prop != 0 {
            if lw(slot + g::propdrawflag) != 0 {
                if (lw(prop + 16) & 256) == 0 {
                    obInstGfx(lw(prop + 32), 1);
                }
            }
        }
        dlSetZB(old_zb);
    }
    let slot = lw(lw(g::front)) << 2;
    let shift = slot + 0x1fc1258;
    let stage = lw(slot + g::stage);
    let prop = lw(slot + g::stageprop);
    if !menu_skip_x() {
        swc1(a0 + 48, ps2float::add(lwc1(a0 + 48), lwc1(shift)));
        swc1(stage + 48, ps2float::add(lwc1(stage + 48), lwc1(shift)));
        if prop != 0 {
            swc1(prop + 48, ps2float::add(lwc1(prop + 48), lwc1(shift)));
        }
    }
}
