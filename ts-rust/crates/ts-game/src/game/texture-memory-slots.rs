//! Finding room for a texture in the texture memory.

use crate::*;

/// The number of slots of the texture buffer (at +0x1a10) and its first slot number (+0x1a0c); the slots are records of 12 bytes from +8: a count or mask of what is
/// used (+8), flags (+12; 1 = partly used by small textures, 2 = used by a large one) and the size entry of the large texture (+16).
const SLOT_COUNT: i32 = 0x1a10;
const SLOT_FIRST: i32 = 0x1a0c;

/// Find room for the texture `tex` in the texture buffer and write where it goes (the place, as the slot number shifted by 5 and the position inside it, at +24 for a small
/// texture, at +16 for a large one) and mark the room as used; returns 1 if there was room, 0 if not.
///
/// With `small` set the texture is one of the small kind: the slots that are partly used (flag 1 and fewer than 8 used) are tried from the last, then the last free slot (flags 1
/// and 2 clear). Otherwise the texture's size (+2 the size as a power of 2, +4 the width, +36 flag 1 whether the width is doubled) is looked up in the table `tsizes` (28 entries
/// of 44 bytes: width, height, the mask of one block, then the bit numbers of the blocks): the table gives the number of 128 by 64 blocks it needs; a slot already used by
/// a texture of the same size entry (flag 2) is searched for a free place for its mask, and if none, the last slot that is not used at all is taken; with no entry in the table
/// it goes at 0 (@ 0x2b6c20).
pub fn emGetNextPos(tex: i32, small: i32) -> i32 {
    let buffer = lw(g::thistexbuf);
    let slots = tex_slots(buffer);
    if small != 0 {
        let mut slot = lw(buffer + SLOT_COUNT) - 1;
        // a slot that is partly used with room for another
        while slot >= 0 {
            let at = slots + slot * 12;
            if (lw(at + 12) & 1) != 0 && (lw(at + 8) as u32) < 8 {
                break;
            }
            slot -= 1;
        }
        if slot < 0 {
            // else the last free slot
            slot = lw(buffer + SLOT_COUNT) - 1;
            while slot >= 0 && (lw(slots + slot * 12 + 12) & 3) != 0 {
                slot -= 1;
            }
            if slot < 0 {
                return 0;
            }
        }
        let flags_at = slots + 12 + slot * 12;
        let used_at = slots + 8 + slot * 12;
        sw(flags_at, lw(flags_at) | 1);
        sw(tex + 24, ((lw(buffer + SLOT_FIRST) + slot) << 5) + (lw(used_at) << 2));
        sw(used_at, lw(used_at) + 1);
        return 1;
    }
    let (width, power_shift);
    if (lbu(tex + 36) & 1) == 0 {
        width = lw(tex + 4) << 1;
        power_shift = 2 << (PowerOf2(lhu(tex + 2)) & 31);
    } else {
        width = lw(tex + 4);
        power_shift = 1 << (PowerOf2(lhu(tex + 2)) & 31);
    }
    let wanted_width = if 15 >= width { 16 } else { width };
    let wanted_height = if 15 >= power_shift { 16 } else { power_shift };
    let mut entry = 0;
    let mut mask = 0;
    let mut at = g::tsizes;
    let mut index = 0;
    loop {
        index += 1;
        if wanted_width == lw(at) && wanted_height == lw(at + 4) {
            entry = at;
            mask = lw(at + 8);
            break;
        }
        at += 44;
        if !((index as u32) < 28) {
            break;
        }
    }
    if entry == 0 {
        sw(tex + 16, 0);
        return 1;
    }
    let size_x = lw(entry);
    let mut blocks = 1;
    if 128 < size_x as u32 {
        blocks = ((size_x as u32) / 128) as i32;
    }
    let size_y = lw(entry + 4);
    let buffer = lw(g::thistexbuf);
    let slots = tex_slots(buffer);
    if 64 < size_y as u32 {
        blocks *= ((size_y as u32) / 64) as i32;
    }
    let count = lw(buffer + SLOT_COUNT);
    // a slot already used by this size, with a free place for the mask
    let mut slot = 0;
    while slot < count {
        let at = slots + slot * 12;
        if (lw(at + 12) & 2) != 0 && lw(at + 16) == entry && lw(at + 8) != -1 {
            let mut bit = 0;
            loop {
                let position = lbu(entry + 12 + bit);
                let place = ((mask as u32) >> (position & 31)) as i32;
                if (lw(slots + 8 + slot * 12) & place) == 0 {
                    sw(tex + 16, ((lw(buffer + SLOT_FIRST) + slot) << 5) + position);
                    let mut flags_at = slots + 12 + slot * 12;
                    for _ in 0..blocks {
                        let flags = lw(flags_at);
                        let used = lw(flags_at - 4) | place;
                        sw(flags_at + 4, entry);
                        sw(flags_at - 4, used);
                        sw(flags_at, flags | 2);
                        flags_at += 12;
                    }
                    return 1;
                }
                bit += 1;
                if bit >= 32 {
                    break;
                }
            }
        }
        slot += 1;
    }
    // else a slot that is not used at all
    let mut slot = 0;
    if lw(buffer + SLOT_COUNT) <= 0 {
        return 0;
    }
    loop {
        if (lw(slots + 12 + slot * 12) & 3) == 0 {
            sw(tex + 16, (lw(buffer + SLOT_FIRST) + slot) << 5);
            let mut flags_at = slots + 12 + slot * 12;
            for _ in 0..blocks {
                sw(flags_at - 4, mask);
                sw(flags_at + 4, entry);
                sw(flags_at, 2);
                flags_at += 12;
            }
            return 1;
        }
        let count = lw(buffer + SLOT_COUNT);
        slot += 1;
        if !(slot < count) {
            return 0;
        }
    }
}
