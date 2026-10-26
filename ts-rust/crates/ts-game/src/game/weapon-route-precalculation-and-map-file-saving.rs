//! The routes enemies take to the weapons, and the saving of a map maker's map file.

use crate::*;

/// For each pad work out which weapons the enemies go to from it: the number of pickups of each of the five gun combos (`lvGetGunComboNums`) is counted, five combos are chosen to
/// look for (the combo of a count above 0 and the last that is present for the rest), then for every pad the cost of the route (`findNextPad`, the route and the hall route)
/// to each weapon pickup of a wanted combo is found and the five cheapest are kept; the five pads written at +8 of the pad's extra record are those that score best by the combo's
/// worth (100 less the combo's number over 100, plus 1) over the square of the cost (@ 0x2cd250).
pub fn enemyPreCalcEnemyRoutes() {
    let frame = Frame::new(2336);
    let sp = frame.sp();
    let mut i = 0;
    let combos = lvGetGunComboNums();
    memset(sp + 2112, 0, 20);
    memset(sp + 2144, 0, 20);
    let mut weapons = lw(g::numPickupWeapons);
    let total = weapons;
    let mut pads = lw(g::numpads);
    if weapons > 0 {
        let mut offset = i << 3;
        loop {
            let next = i + 1;
            let mut at = sp + 2112;
            let mut combo = combos;
            let mut left = 4;
            let kind = lw(lw(offset + (g::weaponspawn + 4)) + 352);
            loop {
                if lw(kind + 12) == lw(combo) {
                    sw(at, lw(at) + 1);
                }
                at += 4;
                left -= 1;
                combo += 4;
                if left < 0 {
                    break;
                }
            }
            i = next;
            offset = i << 3;
            if i >= total {
                break;
            }
        }
    }
    let mut slot = 0;
    let mut offset = slot << 2;
    loop {
        let count_at = sp + 2112 + offset;
        let combo_at = offset + combos;
        if lw(count_at) != 0 {
            sw(sp + 2144 + offset, lw(combo_at));
            sw(count_at, lw(count_at) - 1);
        } else {
            let mut best = -1;
            let mut j = 0;
            let mut at = sp + 2112;
            loop {
                let count = lw(at);
                at += 4;
                best = if count >= 1 { j } else { best };
                j += 1;
                if j >= 5 {
                    break;
                }
            }
            sw(sp + 2144 + offset, lw((best << 2) + combos));
        }
        slot += 1;
        offset = slot << 2;
        if slot >= 5 {
            break;
        }
    }
    let mut pad = 0;
    if pads > 0 {
        loop {
            let unreachable = lwc1(0x3afd88);
            let next_pad = pad + 1;
            let mut cost_at = sp + 2048;
            let mut left = 4;
            let mut extra = pad * 28 + (lw(g::levelpadsextra) + 8);
            loop {
                swc1(cost_at, unreachable);
                left -= 1;
                sw(extra, 0);
                cost_at += 4;
                extra += 4;
                if left < 0 {
                    break;
                }
            }
            let mut w = 0;
            if weapons > 0 {
                let mut spawn = g::weaponspawn;
                loop {
                    let entry = w << 3;
                    let record = spawn + entry;
                    let weapon_pad = lw(entry + spawn);
                    let thing = lw(lw(record + 4) + 352);
                    let cost;
                    if pad == weapon_pad {
                        cost = 0.0_f32;
                    } else {
                        sw(sp + 36, weapon_pad);
                        sw(sp + 32, pad);
                        sw(sp + 1040, 0);
                        sw(sp + 2044, 0);
                        sw(g::lastroutecost, 0);
                        sw(g::lasthallroutecost, 0);
                        let found = findNextPad(sp + 32);
                        let route = lwc1(g::lastroutecost);
                        if found == -2 {
                            cost = lwc1(0x3afd88);
                            weapons = lw(g::numPickupWeapons);
                        } else {
                            weapons = lw(g::numPickupWeapons);
                            cost = route + lwc1(g::lasthallroutecost);
                        }
                    }
                    let combo = lw(thing + 12);
                    let mut chosen = -1;
                    let next_w = w + 1;
                    let mut searching = true;
                    if combo == lw(sp + 2144) && cost < lwc1(sp + 2048) {
                        chosen = 0;
                        searching = false;
                    }
                    if searching {
                        let mut k = 1;
                        loop {
                            let at = k << 2;
                            if k >= 5 {
                                break;
                            }
                            if combo != lw(sp + 2144 + at) {
                                k += 1;
                            } else {
                                if cost < lwc1(sp + 2048 + at) {
                                    chosen = k;
                                    break;
                                }
                                k += 1;
                            }
                        }
                    }
                    let at = chosen << 2;
                    if chosen != -1 {
                        let cost_at = sp + 2048 + at;
                        let spawn_now = g::weaponspawn;
                        if cost < lwc1(cost_at) {
                            let pad_at = sp + at;
                            swc1(cost_at, cost);
                            let prop = lw(spawn_now + entry + 4);
                            sw(pad_at, lw(entry + spawn_now));
                            sw(pad_at + 2080, prop);
                        }
                    }
                    w = next_w;
                    spawn = g::weaponspawn;
                    if w >= weapons {
                        break;
                    }
                }
                pads = lw(g::numpads);
            }
            let big = lwc1(0x3afd88);
            let mut order = 0;
            let target = pad * 28 + lw(g::levelpadsextra) + 8;
            let mut best_score = lwc1(0x3afd88);
            loop {
                let mut best = -1;
                let mut j = 0;
                let following = order + 1;
                let mut combo_at = sp + 2144;
                let mut cost_at = sp + 2048;
                loop {
                    let cost = lwc1(cost_at);
                    if cost == big {
                        j += 1;
                    } else {
                        let score = if cost == 0.0_f32 {
                            lwc1(0x3ada8c)
                        } else {
                            ((((100 - lh((lw(combo_at) << 3) + GUNCOMBOS)) as f32).pdiv(1e+02_f32) + 1.0_f32)).pdiv((cost * cost))
                        };
                        if !(score < best_score) {
                            j += 1;
                        } else {
                            best_score = score;
                            best = j;
                            j += 1;
                        }
                    }
                    combo_at += 4;
                    cost_at += 4;
                    if j >= 5 {
                        break;
                    }
                }
                let at = best << 2;
                if best != -1 {
                    let prop = lw(sp + 2080 + at);
                    swc1(sp + 2048 + at, big);
                    sw(target + (order << 2), prop);
                }
                order = following;
                if order >= 5 {
                    break;
                }
                best_score = lwc1(0x3afd88);
            }
            pad = next_pad;
            if pad >= pads {
                break;
            }
        }
    }
}

/// Save the map of the map maker into the buffer `buffer`: the chunks of the file (0x1324626 and 0x1324627 with the game modes, the number of objects, the tile set, the tune, the
/// name and the description; 0x1324628 for the tiles with a record of 36 bytes for each tile object (its pad number, its place in cells, its rotation, seals, colour, the sizes) and
/// the items of the object, 4 bytes each; 0x1324629 the end) are written by `mmWriteChunk`. With `keep` 0 the object being placed is freed first and the edit flag 0x1000
/// cleared afterwards; the length written is returned, 0 when there are no objects (@ 0x247c78).
pub fn mmSaveMapFile(buffer: i32, keep: i32) -> i32 {
    let frame = Frame::new(192);
    let sp = frame.sp();
    sw(sp + 16, keep);
    let count = mmObjCount(1);
    sw(sp + 20, lw(g::editlevel));
    if lw(g::nummapobjects) <= 0 {
        return 0;
    }
    if lw(sp + 16) == 0 {
        let used = lw(g::usedobj);
        if used != 0 {
            mmFreeObject(used);
            sw(g::usedobj, 0);
        }
    }
    let modes = mmCheckGameModes(0);
    let first = mmWriteChunk(buffer, 0, 0x1324626, 114);
    let second = mmWriteChunk(buffer, first, 0x1324627, 172);
    let header = buffer + second;
    sw(header, modes);
    let mut at = second + 172;
    let tile_set = lhu(g::gv_tileset_3567d8);
    sw(header + 4, count);
    sh(header + 8, tile_set);
    let tune = mmGetTuneNum();
    sh(header + 10, tune);
    strcpy(header + 12, g::mapname);
    strcpy(header + 44, g::mapdesc);
    let mut end = at;
    if count != 0 {
        let items = mmItemCount(-1);
        let written = mmWriteChunk(buffer, at, 0x1324628, count * 36 + (items << 2));
        let mut object = lw(g::mapobjects);
        at = written;
        if object != 0 {
            let mut kind = lw(object);
            loop {
                'tile: {
                    let record = buffer + at;
                    let mut across = 0;
                    let mut down = 0;
                    if kind == 1 {
                        let tile = lw(object + 36);
                        sw(record, ((tile - g::tileset1).wrapping_mul(0x4f72c235)) >> 2);
                        let wide_b = lw(tile + 4);
                        let x = lw(object + 12) >> 21;
                        let rotation = lw(object + 24);
                        let y = lw(object + 16) >> 21;
                        let wide_a = lw(tile);
                        match rotation {
                            1 => {
                                across = wide_b;
                            }
                            2 => {
                                across = wide_a;
                                down = wide_b;
                            }
                            3 => {
                                down = wide_a;
                                across = 0;
                            }
                            _ => {}
                        }
                        sb(record + 4, x + across);
                        at += 36;
                        let colour = lbu(object + 20);
                        sb(record + 6, y + down);
                        sb(record + 5, colour);
                        sb(record + 7, lbu(object + 24));
                        let seals = mmMakeSeals(object);
                        let flags = lw(object + 32);
                        sh(record + 8, seals);
                        sw(record + 12, ((flags << 1) & -0x1010200) + 255);
                        sb(record + 16, lbu(object + 28));
                        sb(record + 17, lbu(object + 29));
                        let list = lw(object + 52);
                        sw(record + 20, lw(object + 12) >> 21);
                        sw(record + 24, lw(object + 20));
                        sw(record + 28, lw(object + 16) >> 21);
                        sw(record + 32, lw(object + 24));
                        let listed = mmItemListCount(list, -1);
                        sh(record + 10, listed);
                        if (listed & 0xffff) != 0 {
                            let mut item = lw(object + 52);
                            let mut turn = lw(object + 24);
                            let pad = lw((lw(record) << 2) + g::tilepaddata);
                            let table = pad + 12;
                            let size = lw(pad + 4);
                            if item != 0 {
                                loop {
                                    let out = buffer + at;
                                    let first_key = lw(table);
                                    sh(out, lhu(item + 2));
                                    sh(out + 2, lhu(item));
                                    let wanted = lhu(item);
                                    let mut found = 0;
                                    at += 4;
                                    if first_key != wanted && 0 < size {
                                        found += 1;
                                        loop {
                                            let more = found < size;
                                            if lw(found * 28 + table) == wanted {
                                                break;
                                            }
                                            if !more {
                                                break;
                                            }
                                            found += 1;
                                        }
                                    }
                                    let entry = found * 28 + table;
                                    swc1(sp, lwc1(entry + 12));
                                    swc1(sp + 4, lwc1(entry + 16));
                                    swc1(sp + 8, lwc1(entry + 20));
                                    if turn != 0 {
                                        matrixVecMulAligned(&[((turn << 6) + g::tilerotmtx) as i64, sp as i64]);
                                    }
                                    item = lw(item + 4);
                                    if item == 0 {
                                        break 'tile;
                                    }
                                    turn = lw(object + 24);
                                }
                            }
                        }
                    }
                }
                object = lw(object + 56);
                if object == 0 {
                    break;
                }
                kind = lw(object);
            }
        }
        end = at;
    }
    let result = mmWriteChunk(buffer, end, 0x1324629, end);
    if lw(sp + 16) == 0 {
        sw(g::editflags, lw(g::editflags) & -0x1001);
    }
    sw(g::editlevel, lw(sp + 20));
    result
}
