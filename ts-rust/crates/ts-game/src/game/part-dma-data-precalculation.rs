//! The DMA tags of a part's point lists, made once and kept.

use crate::*;

/// Add a DMA tag that sends `quads` quad words from `address`.
fn dma_tag(quads: i32, address: i32) {
    sb(lw(g::gfxtag) + 3, 48);
    let tag = lw(g::gfxtag);
    sw(tag + 4, address);
    sh(tag, quads);
    sw(g::gfxtag, tag + 16);
}

/// The tag that picks the texture of the list `list` (its texture number at +0, the table `textures` of 16-byte records) when it has one.
fn texture_tag(list: i32, textures: i32) {
    let number = lw(list);
    if number >= 0 {
        let picture = lw((number << 4) + textures);
        let texture = lw((if -1 >= picture { 0 } else { picture }) * 40 + lw(lw(g::thistexbuf) + 4) + 32);
        dma_tag(8, texture);
    }
}

/// The number of quad words of padding-aligned bytes `size`: three words of 4 bytes for each 4 bytes, over 4.
fn three_quarters(size: i32) -> i32 {
    let rounded = size + 3 & -4;
    (((rounded << 1) + rounded) as u32 >> 2) as i32
}

/// Make the DMA data for one list of the part `bone_index` of the model `model` (the list at +8 of the part for `flag` 0, at +12 for 1): the part's lists of points are sent by
/// DMA tags kept in memory from the level area (16 bytes a tag), one run of tags for each list (the packet of the ucode, the points, the colours, the texture coordinates, the
/// normals when the ucode has them, the texture and the ucode's own program), and the run's start is stored in the model's table of the flag at the part's place. Parts of kind 0
/// use 80-byte records (as made by `partPrecalcVifData`) and 9 tags a list (11 for kinds 1 to 3 with their 96 bytes), plus those for a texture, the clip and the normals; on
/// level 102 nothing is made and 0 is stored (@ 0x25f408).
pub fn partPrecalcDmaData(model: i32, bone_index: i32, flag: i32) {
    let bone = model - lw(model) * 80 + bone_index * 80;
    let mut list = if flag == 0 { lw(bone + 8) } else { lw(bone + 12) };
    let mut textures = lw(model + 16);
    let records = if flag != 0 { bone + 44 } else { bone + 20 };
    let mut made = 0;
    if textures == 0 {
        textures = lw(model + 12);
    }
    let table_offset = flag << 2;
    if lw(g::lvlevelnum) == 102 {
        sw((bone_index << 2) + lw(model + (flag << 2) + 28), 0);
        return;
    }
    let place = bone_index << 2;
    let base = lw(place + lw(model + table_offset + 20));
    if list != 0 {
        let mut index = 0;
        let kind = lbu(bone);
        if lb(bone) == 0 {
            let mut kind_now = lw(list + 20);
            let mut walk = list;
            let mut total = 0;
            if kind_now < 0 {
                total = 1;
            } else {
                loop {
                    let mut added = total + 9;
                    let texture = lw(walk);
                    walk += 24;
                    if (kind_now ^ 3) == 0 {
                        added += 2;
                    }
                    let next_kind = lw(walk + 20);
                    total = if texture >= 0 { added + 1 } else { added };
                    kind_now = next_kind;
                    if next_kind < 0 {
                        break;
                    }
                }
                total += 1;
            }
            made = memAlloc(total << 4, 1);
            dlPushDmaTag(made);
            if lw(list + 20) >= 0 {
                loop {
                    let record = index * 80 + base;
                    dma_tag(1, record);
                    dma_tag(lw(list + 8), (lw(list + 4) << 4) + lw(records));
                    dma_tag(1, record + 16);
                    dma_tag(three_quarters(lw(list + 16)), lw(list + 12) * 12 + lw(records + 4));
                    dma_tag(1, record + 32);
                    dma_tag(lw(list + 16), (lw(list + 12) << 4) + lw(records + 8));
                    dma_tag(1, record + 48);
                    dma_tag((lw(list + 16) + 3) >> 2, (lw(list + 12) << 2) + lw(records + 12));
                    if lw(list + 20) == 3 {
                        dma_tag(1, record + 64);
                        dma_tag(three_quarters(lw(list + 16)), lw(list + 12) * 12 + lw(records + 20));
                    }
                    texture_tag(list, textures);
                    let ucode = lw(list + 20);
                    if ucode != 3 {
                        dma_tag(1, (ucode << 5) + g::startucode);
                    } else if lw(g::doclip) == 0 {
                        dma_tag(1, g::startucode + 160);
                    } else {
                        dma_tag(1, (ucode << 5) + g::startucode);
                    }
                    list += 24;
                    index += 1;
                    if lw(list + 20) < 0 {
                        break;
                    }
                }
            }
            dlPopDmaTag();
            sw(place + lw(model + 28 + table_offset), made);
            return;
        }
        if ((kind as i32 - 1 & 255) as u32) < 3 {
            let mut kind_now = lw(list + 20);
            let mut walk = list;
            let mut total = 0;
            if kind_now < 0 {
                total = 1;
            } else {
                loop {
                    let mut added = total + 11;
                    let texture = lw(walk);
                    walk += 24;
                    if (kind_now ^ 4) == 0 {
                        added += 2;
                    }
                    let next_kind = lw(walk + 20);
                    total = if texture >= 0 { added + 1 } else { added };
                    kind_now = next_kind;
                    if next_kind < 0 {
                        break;
                    }
                }
                total += 1;
            }
            made = memAlloc(total << 4, 1);
            dlPushDmaTag(made);
            if lw(list + 20) >= 0 {
                loop {
                    let record = index * 96 + base;
                    dma_tag(1, record);
                    dma_tag(lw(list + 8), (lw(list + 4) << 4) + lw(records));
                    dma_tag(1, record + 16);
                    dma_tag(three_quarters(lw(list + 16)), lw(list + 12) * 12 + lw(records + 4));
                    dma_tag(1, record + 32);
                    dma_tag((lw(list + 16) + 3) >> 2, (lw(list + 12) << 2) + lw(records + 16));
                    dma_tag(1, record + 48);
                    dma_tag(lw(list + 16), (lw(list + 12) << 4) + lw(records + 8));
                    dma_tag(1, record + 64);
                    dma_tag((lw(list + 16) + 3) >> 2, (lw(list + 12) << 2) + lw(records + 12));
                    if lw(list + 20) == 4 {
                        dma_tag(1, record + 80);
                        dma_tag(three_quarters(lw(list + 16)), lw(list + 12) * 12 + lw(records + 20));
                    }
                    texture_tag(list, textures);
                    let ucode = lw(list + 20);
                    index += 1;
                    dma_tag(1, (ucode << 5) + g::startucode);
                    list += 24;
                    if lw(list + 20) < 0 {
                        break;
                    }
                }
            }
            dlPopDmaTag();
            sw(place + lw(model + 28 + table_offset), made);
            return;
        }
    }
    sw(place + lw(model + 28 + table_offset), made);
}
