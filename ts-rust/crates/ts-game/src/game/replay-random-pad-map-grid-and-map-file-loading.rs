//! The replay of a demo, picking a random pad, the grid of the map maker's view, and loading a map file.

use crate::*;
use super::random_as_float;

/// The tick of a demo replay: with a button of the first controller pressed (2384) and the replay not ended it is ended; for each player in a replay being played (mode 1) the
/// next record of the demo buffer (`0x3afe1c` is its cursor) gives the tick and update frame counts, the game frame, the player's controller state (the four axes and
/// four buttons at +284 to +312) and a check on the random number seed; a replay with no more records ends with the controllers cleared and the tick count 1; when it ends the
/// players' positions are printed and the demo is ended; a seed that does not match ends it too (@ 0x202d90).
pub fn replayTick() {
    let mut ended = 0;
    let pressed = joyBut(0);
    let end = lw(g::replayend);
    if (pressed & 2384) != 0 {
        ended = ((end as u32) < 1) as i32;
    }
    let mut players = lw(g::replaynumplayers);
    let mut i = 0;
    if players > 0 {
        loop {
            let mode = lw(g::replaymode);
            let controller = lw((i << 2) + g::playercontrollers);
            if mode != 0 {
                if mode != 1 {
                    i += 1;
                } else {
                    if lw(g::replayend) == 0 && (lw(0x3afe1c) as u32) < ((lw(g::replaybuf) + lw(g::replaybuflen)) as u32) {
                        let ticks = BuffRemoveu8(0x3afe1c);
                        sw(g::tickframesi, ticks);
                        swc1(g::tickframesf, ticks as f32);
                        let updates = BuffRemoveu8(0x3afe1c);
                        sw(g::updateframesi, updates);
                        swc1(g::updateframesf, updates as f32);
                        let frame = BuffRemoveu32(0x3afe1c);
                        sw(g::gameframe, frame);
                        let record = controller * 384 + g::controllers;
                        sw(record + 284, BuffRemoveu16(0x3afe1c));
                        sw(record + 288, BuffRemoveu16(0x3afe1c));
                        sw(record + 292, BuffRemoveu16(0x3afe1c));
                        sw(record + 296, BuffRemoveu16(0x3afe1c));
                        sw(record + 300, BuffRemoveu8(0x3afe1c));
                        sw(record + 304, BuffRemoveu8(0x3afe1c));
                        sw(record + 308, BuffRemoveu8(0x3afe1c));
                        sw(record + 312, BuffRemoveu8(0x3afe1c));
                        let seed = BuffRemoveu32(0x3afe1c);
                        BuffRemoveu32(0x3afe1c);
                        players = lw(g::replaynumplayers);
                        ended = if (seed as i64 ^ (lwu(g::newrndseedval) << 32) >> 32) != 0_i64 { mode } else { ended };
                    } else {
                        ended = 1;
                        sw(g::updateframesf, 0);
                        let record = controller * 384 + g::controllers;
                        sw(record + 312, 128);
                        sw(g::tickframesi, ended);
                        sw(g::updateframesi, 0);
                        swc1(g::tickframesf, ended as f32);
                        sw(record + 284, 0);
                        sw(record + 288, 0);
                        sw(record + 292, 0);
                        sw(record + 296, 0);
                        sw(record + 300, 128);
                        sw(record + 304, 128);
                        sw(record + 308, 128);
                    }
                    let already = lw(g::replayend);
                    if ended != 0 {
                        if already != 0 {
                            i += 1;
                        } else {
                            let mut k = 0;
                            sw(g::replayend, 1);
                            i += 1;
                            if players > 0 {
                                loop {
                                    playerprintpos();
                                    k += 1;
                                    if k >= lw(g::replaynumplayers) {
                                        break;
                                    }
                                }
                            }
                            demoEnd();
                            players = lw(g::replaynumplayers);
                        }
                    } else {
                        i += 1;
                    }
                }
            } else {
                i += 1;
            }
            if i >= players {
                break;
            }
        }
    }
}

/// A random pad for the character `chr` among the pads of the level's set-up (records of 16 bytes, their count at +36, the pad references at +12, the flags at +8): those that pass
/// the create flags (the challenge's when the game flag 32 is set and the challenge number is below 2) and have the flags `wanted` (and the character's team bit in the
/// team games) are listed; one is chosen at random and, if another character is on it (or goes for it), the following one of the list, and so on; if all are taken any is
/// chosen at random. Without a set-up or such pads any pad number is chosen at random (@ 0x2280b0).
pub fn lvGetRandomPad(chr: i32, wanted: i32) -> i32 {
    let mut wanted = wanted;
    let setup = lvGetLevelSetup(lw(g::lvlevelnum));
    if setup != 0 {
        let count = lw(setup + 36);
        if count != 0 {
            let parms = g::curgameparms;
            let teams = lw(parms + 80) & 0x20001;
            if teams == 1 {
                wanted |= teams << ((lw(chr + 24) + 1) & 31);
            }
            let mut chosen = 0;
            let list = 0x1fbcc78;
            let mut i = 0;
            if count > 0 {
                let mut offset = 0;
                let mut put = list;
                loop {
                    let records = lw(setup + 32);
                    let created = if (lw(parms + 80) & 32) == 0 {
                        lvCheckCreateFlags(lw(offset + records))
                    } else if (lw(parms + 68) as u32) < 2 {
                        lvCheckCreateFlagsChallenge(lw(offset + records))
                    } else {
                        lvCheckCreateFlags(lw(offset + records))
                    };
                    if created != 0 && (lw(offset + lw(setup + 32) + 8) & wanted) == wanted {
                        sw(put, i);
                        chosen += 1;
                        put += 4;
                    }
                    i += 1;
                    offset += 16;
                    if i >= lw(setup + 36) {
                        break;
                    }
                }
            }
            if chosen != 0 {
                let first = (newrnd() as i32 as u32 % chosen as u32) as i32;
                let records = lw(setup + 32);
                let others = lw(g::lvnumplayers) + lw(g::numactivechars);
                let mut try_at = first;
                loop {
                    let pad = lw((lw((try_at << 2) + list) << 4) + records + 12);
                    let mut taken = false;
                    if others > 0 {
                        let characters = lw(g::chrinfo);
                        let mut j = 0;
                        loop {
                            let other = j * 0x1210 + characters;
                            if chr == other {
                                j += 1;
                            } else {
                                if lw(other + 680) == pad {
                                    taken = true;
                                    break;
                                }
                                j += 1;
                            }
                            if j >= others {
                                break;
                            }
                        }
                    }
                    if !taken {
                        return pad;
                    }
                    let following = try_at + 1;
                    try_at = if (following ^ chosen) == 0 { 0 } else { following };
                    if try_at == first {
                        break;
                    }
                }
                let any = (newrnd() as i32 as u32 % chosen as u32) as i32;
                return lw((lw((any << 2) + list) << 4) + lw(setup + 32) + 12);
            }
            let r = newrnd();
            let pads = lw(g::numpads);
            if pads == 0 {
                trap("break");
            }
            return (r as i32 as u32 % pads as u32) as i32;
        }
    }
    let unit = random_as_float();
    let pads = lw(g::numpads) as f32;
    ps2float::mul(pads, ps2float::mul(unit, 2.3283064e-10_f32)) as i32
}

/// Draw the grid of the map maker's view: a vertical line for each multiple of 0x200000 across the map's width and a horizontal one for each down its height (the colour from the
/// grid's intensity and the ratio of the view to the window), and a black rectangle over what is outside the 0x2800000 square about the middle (@ 0x24a710).
pub fn mmDrawGrid() {
    let mut map_x = lw(g::m_x);
    let mask = 0x1fffff;
    let mut start_x = map_x;
    if (map_x & mask) != 0 {
        start_x = map_x + 0x200000 - (map_x & mask);
    }
    let map_y = lw(g::m_y);
    let mut start_y = map_y;
    let end_x = map_x + lw(g::m_w);
    if (map_y & mask) != 0 {
        start_y = map_y + 0x200000 - (map_y & mask);
    }
    let mut map_h = lw(g::m_h);
    let window_h = lw(g::w_h);
    let columns = (end_x - start_x) >> 21;
    let window_w = lw(g::w_w);
    let end_y = map_y + map_h;
    let rows = (end_y - start_y) >> 21;
    let mut at = start_x;
    if window_w.wrapping_mul(window_h) == 0 {
        trap("break");
    }
    let share = columns
        .wrapping_mul(window_h)
        .wrapping_add(rows.wrapping_mul(window_w))
        .wrapping_sub(rows.wrapping_mul(columns))
        << 8;
    let ratio = share.wrapping_div(window_w.wrapping_mul(window_h));
    let colour = (((!ratio & 255) as u32 >> 1) as i32) << 16 | lbu(g::gv_gridintensity);
    if at < end_x {
        loop {
            let scale_x = lw(g::scale_mtowx);
            let from = at - map_x;
            let scale_y = lw(g::scale_mtowy);
            let top = lw(g::w_y);
            at += 0x200000;
            let x = lw(g::w_x) + ((FMul(from, scale_x) as i32) >> 16);
            dlFillRectangle(x, top, x + 1, top + ((FMul(map_h, scale_y) as i32) >> 16), colour);
            map_x = lw(g::m_x);
            if at >= end_x {
                break;
            }
            map_h = lw(g::m_h);
        }
    }
    let mut line = start_y;
    if line < end_y {
        loop {
            mmDrawLine(map_x, line, map_x + lw(g::m_w), line, colour);
            line += 0x200000;
            map_x = lw(g::m_x);
            if line >= end_y {
                break;
            }
        }
    }
    let width = lw(g::m_w);
    if map_x < -0x2800000 {
        let y = lw(g::m_y);
        if -0x2800000 < map_x + width {
            mmDrawRect(map_x, y, -0x2800000, y + lw(g::m_h), 128);
            map_x = lw(g::m_x);
        }
    }
    let mut map_y = lw(g::m_y);
    if 0x27fffff >= map_x {
        let right = map_x + lw(g::m_w);
        let y = lw(g::m_y);
        if 0x2800000 < right {
            mmDrawRect(0x2800000, y, right, y + lw(g::m_h), 128);
            map_y = lw(g::m_y);
        }
    }
    let height = lw(g::m_h);
    if map_y < -0x2800000 {
        let x = lw(g::m_x);
        if -0x2800000 < map_y + height {
            mmDrawRect(x, map_y, x + lw(g::m_w), -0x2800000, 128);
            map_y = lw(g::m_y);
        }
    }
    if 0x27fffff >= map_y {
        let bottom = map_y + lw(g::m_h);
        if 0x2800000 < bottom {
            let x = lw(g::m_x);
            mmDrawRect(x, 0x2800000, x + lw(g::m_w), bottom, 128);
        }
    }
}

/// Load the map file at `file` into the map maker: cleared first, then the chunks of the file (0x1324627 the map's header: tileset, tune, name at +12 and description at +44,
/// the tile count at +4; 0x1324628 the tiles, each a record of 36 bytes with its position at +20, +28, the turn, and 4-byte items after it, made with `mmNewTile`, `mmPlaceObject`
/// and `mmNewItem`; 0x1324629 the end). Not a map file (the magic 0x1324626 and version 114 at the start) gives 0. When `keep_level` is 0 the edit level is the lowest of the
/// tiles' (at most 4) and the other level the one below it (@ 0x248060).
pub fn mmLoadMapFile(file: i32, keep_level: i32) -> i32 {
    let mut count = -1;
    mmClearMap();
    if lw(file) != 0x1324626 || lw(file + 4) != 114 {
        return 0;
    }
    let mut chunk = file + 8;
    let mut tag = lw(chunk);
    let mut data = chunk + 8;
    if tag != 0x1324629 {
        loop {
            if tag == 0x1324627 {
                gvSetValue(g::gv_tileset_3567d8, lhu(data + 8));
                mmSetTuneNum(lhu(data + 10));
                sb(data + 43, 0);
                sb(data + 171, 0);
                strcpy(g::mapname, data + 12);
                strcpy(g::mapdesc, data + 44);
                count = lw(data + 4);
            } else if tag == 0x1324628 {
                let mut made = 0;
                if count > 0 {
                    let mut record = data;
                    loop {
                        let mut item = record + 36;
                        let tile = mmNewTile(lw(record), lw(record + 20) << 21, lw(record + 28) << 21, lw(record + 24), lw(record + 32));
                        sw(tile + 32, (((lw(record + 12) as u32) >> 1) as i32 & 0x7f7f7f00) + 128);
                        sb(tile + 28, lbu(record + 16));
                        sb(tile + 29, lbu(record + 17));
                        mmPlaceObject(tile);
                        let items = lhu(record + 10);
                        if items != 0 {
                            let mut k = 0;
                            made += 1;
                            let mut kind = lhu(item + 2);
                            loop {
                                mmNewItem(tile, kind, lh(item));
                                k += 1;
                                item += 4;
                                if k >= lhu(record + 10) {
                                    break;
                                }
                                kind = lhu(item + 2);
                            }
                        } else {
                            made += 1;
                        }
                        record = item;
                        if made >= count {
                            break;
                        }
                    }
                }
            }
            let size = lw(chunk + 4);
            chunk = data + size;
            tag = lw(chunk);
            data = chunk + 8;
            if tag == 0x1324629 {
                break;
            }
        }
    }
    if keep_level == 0 {
        let mut level = 4;
        let mut object = lw(g::mapobjects);
        sw(g::editlevel, 4);
        let flags = lw(g::editflags);
        if object != 0 {
            let mut kind = lw(object);
            loop {
                if kind != 1 {
                    object = lw(object + 56);
                } else {
                    let tile_level = lw(object + 20);
                    if tile_level >= level {
                        object = lw(object + 56);
                    } else {
                        level = tile_level;
                        sw(g::editlevel, tile_level);
                        object = lw(object + 56);
                    }
                }
                if object == 0 {
                    break;
                }
                kind = lw(object);
            }
        }
        sw(g::otherlevel, if !(level < -2) { level - 1 } else { -3 });
        sw(g::editflags, flags & -0x1101);
    }
    1
}
