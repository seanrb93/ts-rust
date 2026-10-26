//! Adding the pads and links of a map maker tile to the level.

use crate::*;

/// Add the pads and the links of the map maker tile `tile` (its number of pads at +4, of links at +8, the pads of 28 bytes from +12, then the links of 20 bytes) to the level,
/// placed by the matrix `matrix` and turned by the quarter at +16 of `context` (halls: the pads of the tile belong to new halls, numbered on from `numhalls`): each pad is copied
/// with the new number 1001 and up, its direction turned (by 90 for each quarter, kept from 0 up), its place moved by the matrix, and, when the tile's table of plug pads
/// has an entry for it that the context's mask (+20) does not hide, the flag of its direction (128, 64, 32, 256 for 0, 90, 180, 270 degrees) is set; each link is copied
/// and its two ends are made the new pads; the counts of the links inside a hall and between halls are kept at 0x3b00a4 and 0x3b00a8 (@ 0x2650e8).
pub fn padAddTilePads(tile: i32, matrix: i32, context: i32) {
    let mut halls = 1;
    let pads_at = tile + 12;
    let pad_count = lw(tile + 4);
    let link_count = lw(tile + 8);
    let links_at = pads_at + pad_count * 28;
    if pad_count > 0 {
        let mut at = pads_at;
        let mut left = pad_count;
        loop {
            let hall = lh(at + 8);
            left -= 1;
            at += 28;
            let next = hall + 1;
            halls = if halls < next { next } else { halls };
            if left == 0 {
                break;
            }
        }
    }
    let mut pad = 0;
    if pad_count > 0 {
        let mut pads = lw(g::levelpads);
        let mut numpads = lw(g::numpads);
        loop {
            let new_index = numpads + pad;
            let source = pad * 28 + pads_at;
            let hall_base = lhu(g::numhalls);
            let new_pad = new_index * 28 + pads;
            swc1(new_pad + 12, lwc1(source + 12));
            let hall = lhu(source + 8);
            let y = lwc1(source + 16);
            sw(new_pad, new_index + 1001);
            swc1(new_pad + 16, y);
            let z = lwc1(source + 20);
            sw(new_pad + 4, lw(source + 4));
            swc1(new_pad + 20, z);
            let quarter = lw(context + 16);
            let direction = lwc1(source + 24);
            sh(new_pad + 8, hall_base + hall);
            swc1(new_pad + 24, direction);
            if quarter == 1 {
                swc1(new_pad + 24, ps2float::sub(direction, 90.0_f32));
            } else if quarter == 2 {
                swc1(new_pad + 24, ps2float::sub(direction, 180.0_f32));
            } else if quarter == 3 {
                swc1(new_pad + 24, ps2float::sub(direction, 270.0_f32));
            }
            let record = new_index * 28 + pads;
            let angle = lwc1(record + 24);
            if angle < 0.0_f32 {
                swc1(record + 24, ps2float::add(angle, 360.0_f32));
            }
            matrixVecMul(matrix, record + 12);
            pads = lw(g::levelpads);
            if lw(new_index * 28 + pads + 4) & 1 != 0 {
                sw(0x3b00a0, lw(0x3b00a0) + 1);
            }
            let table = lw((lw(context) << 2) + g::tileplugpads);
            let wanted = lw(pad * 28 + pads_at);
            let mut entry = table;
            let mut found = false;
            let first = lh(entry);
            if first > 0 {
                if first == wanted {
                    found = true;
                } else {
                    entry += 4;
                    loop {
                        let id = lh(entry);
                        if id <= 0 {
                            break;
                        }
                        if id == lw(pad * 28 + pads_at) {
                            found = true;
                            break;
                        }
                        entry += 4;
                    }
                }
            }
            let next_pad = pad + 1;
            if found {
                let value = (lhu(entry) as i32) << 16;
                if value > 0 && lhu(entry + 2) & lhu(context + 20) == 0 {
                    let target = new_index * 28 + pads;
                    let angle = lwc1(target + 24);
                    if angle == 0.0_f32 {
                        sw(target + 4, lw(target + 4) | 128);
                    } else if angle == 90.0_f32 {
                        sw(target + 4, lw(target + 4) | 64);
                    } else if angle == 180.0_f32 {
                        sw(target + 4, lw(target + 4) | 32);
                    } else if angle == 270.0_f32 {
                        sw(target + 4, lw(target + 4) | 256);
                    }
                }
            }
            pad = next_pad;
            numpads = lw(g::numpads);
            if !(pad < pad_count) {
                break;
            }
        }
    }
    let mut link = 0;
    if link_count > 0 {
        let links = lw(g::levellinks);
        let mut numlinks = lw(g::numlinks);
        loop {
            let new_index = numlinks + link;
            let source = link * 20 + links_at;
            let cost = lwc1(source + 12);
            let dest = new_index * 20 + links;
            let flags = lw(source);
            sw(dest + 8, -1);
            sw(dest, flags);
            swc1(dest + 12, cost);
            sw(dest + 4, -1);
            if pad_count > 0 {
                let mut k = 0;
                loop {
                    let source = link * 20 + links_at;
                    let id = lw(k * 28 + pads_at);
                    if id != lw(source + 4) {
                        if id == lw(source + 8) {
                            sw(new_index * 20 + links + 8, lw(g::numpads) + k);
                        }
                    } else {
                        sw(new_index * 20 + links + 4, lw(g::numpads) + k);
                    }
                    k += 1;
                    if k >= pad_count {
                        break;
                    }
                }
            }
            let dest = new_index * 20 + links;
            let from = lw(dest + 4);
            if lw(dest) & 1 != 0 {
                let pads = lw(g::levelpads);
                let hall_links = lw(0x3b00a8);
                if lh(from * 28 + pads + 8) != lh(lw(dest + 8) * 28 + pads + 8) {
                    sw(0x3b00a8, hall_links + 1);
                } else {
                    sw(0x3b00a4, lw(0x3b00a4) + 1);
                }
            }
            link += 1;
            numlinks = lw(g::numlinks);
            if link >= link_count {
                break;
            }
        }
    }
    let halls_total = lw(g::numhalls) + halls;
    let links_total = lw(g::numlinks) + link_count;
    sw(g::numpads, lw(g::numpads) + pad_count);
    sw(g::numlinks, links_total);
    sw(g::numhalls, halls_total);
}
