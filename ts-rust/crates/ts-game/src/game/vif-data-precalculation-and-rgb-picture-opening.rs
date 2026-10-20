//! The VIF packets of a part's point lists, and the opening of an RGB picture.

use crate::*;

/// Make the VIF data for one list of the part `bone_index` of the model `model` (the list at +8 for `flag` 0, at +12 for 1): for each of the lists of the chain (the count
/// is taken up to the one whose +20 is negative) a record is made of the packets that load the ucode's data (gif, xyzw, [bind], st, rgba, [norm]) with sizes from the list's
/// fields; parts of kind 0 take 80 bytes a record (the norm packet when the list's ucode is 3), kinds 1 to 3 take 96 (with the bind packet, and the norm one for ucode 4); the
/// memory is taken from the level area, and its address stored in the model's table of the flag at the part's place (@ 0x25eec0).
pub fn partPrecalcVifData(model: i32, bone_index: i32, flag: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let bone = model - lw(model) * 80 + bone_index * 80;
    let mut list = if flag == 0 { lw(bone + 8) } else { lw(bone + 12) };
    let mut count = 0;
    sw(sp, 0);
    let table_at = model + 20;
    let flag_offset = flag << 2;
    let place = bone_index << 2;
    if list != 0 {
        let mut walk = list;
        let kind;
        if lw(list + 20) < 0 {
            kind = lbu(bone);
        } else {
            kind = lbu(bone);
            loop {
                walk += 24;
                count += 1;
                if lw(walk + 20) < 0 {
                    break;
                }
            }
        }
        if kind != 0 {
            if ((kind as i32 - 1) as u32) < 3 {
                let memory = memAlloc(count * 96, 1);
                sw(sp, memory);
                let mut i = 0;
                if count > 0 {
                    loop {
                        let base = lw(sp) + i * 96;
                        let marker = 0x1000404;
                        sw(lw(sp) + i * 96, 0);
                        let mut at = base + 4;
                        sw(at, 0);
                        at += 4;
                        sw(at, marker);
                        at += 4;
                        sw(at, lw(list + 8) << 16 | (lw((lw(list + 20) << 2) + g::ucode_loadgif) | 0x6c008000));
                        at += 4;
                        sw(at, 0);
                        at += 4;
                        sw(at, 0);
                        at += 4;
                        sw(at, marker);
                        at += 4;
                        sw(at, (lw(list + 16) + 3 & -4) << 16 | (lw((lw(list + 20) << 2) + g::ucode_loadxyzw) | 0x68008000));
                        at += 4;
                        sw(at, 0);
                        at += 4;
                        sw(at, 0);
                        at += 4;
                        sw(at, marker);
                        at += 4;
                        sw(at, (lw(list + 16) + 3 & -4) << 16 | (lw((lw(list + 20) << 2) + g::ucode_loadbind) | 0x6e008000));
                        at += 4;
                        sw(at, 0);
                        at += 4;
                        sw(at, 0);
                        at += 4;
                        sw(at, marker);
                        at += 4;
                        sw(at, lw(list + 16) << 16 | (lw((lw(list + 20) << 2) + g::ucode_loadst) | 0x6c008000));
                        at += 4;
                        sw(at, 0);
                        at += 4;
                        sw(at, 0);
                        at += 4;
                        sw(at, marker);
                        at += 4;
                        sw(at, (lw(list + 16) + 3 & -4) << 16 | (lw((lw(list + 20) << 2) + g::ucode_loadrgba) | 0x6e008000));
                        at += 4;
                        if lw(list + 20) == 4 {
                            sw(at, 0);
                            at += 4;
                            sw(at, 0);
                            at += 4;
                            sw(at, marker);
                            sw(at + 4, (lw(list + 16) + 3 & -4) << 16 | (lw((lw(list + 20) << 2) + g::ucode_loadnorm) | 0x68008000));
                        }
                        i += 1;
                        list += 24;
                        if i >= count {
                            break;
                        }
                    }
                }
            }
        } else {
            let memory = memAlloc(count * 80, 1);
            sw(sp, memory);
            let mut i = 0;
            if count > 0 {
                loop {
                    let base = lw(sp) + i * 80;
                    let marker = 0x1000404;
                    sw(lw(sp) + i * 80, 0);
                    let mut at = base + 4;
                    sw(at, 0);
                    at += 4;
                    sw(at, marker);
                    at += 4;
                    sw(at, lw(list + 8) << 16 | (lw((lw(list + 20) << 2) + g::ucode_loadgif) | 0x6c008000));
                    at += 4;
                    sw(at, 0);
                    at += 4;
                    sw(at, 0);
                    at += 4;
                    sw(at, marker);
                    at += 4;
                    sw(at, (lw(list + 16) + 3 & -4) << 16 | (lw((lw(list + 20) << 2) + g::ucode_loadxyzw) | 0x68008000));
                    at += 4;
                    sw(at, 0);
                    at += 4;
                    sw(at, 0);
                    at += 4;
                    sw(at, marker);
                    at += 4;
                    sw(at, lw(list + 16) << 16 | (lw((lw(list + 20) << 2) + g::ucode_loadst) | 0x6c008000));
                    at += 4;
                    sw(at, 0);
                    at += 4;
                    sw(at, 0);
                    at += 4;
                    sw(at, marker);
                    at += 4;
                    sw(at, (lw(list + 16) + 3 & -4) << 16 | (lw((lw(list + 20) << 2) + g::ucode_loadrgba) | 0x6e008000));
                    at += 4;
                    if lw(list + 20) == 3 {
                        sw(at, 0);
                        at += 4;
                        sw(at, 0);
                        at += 4;
                        sw(at, marker);
                        sw(at + 4, (lw(list + 16) + 3 & -4) << 16 | (lw((lw(list + 20) << 2) + g::ucode_loadnorm) | 0x68008000));
                    }
                    i += 1;
                    list += 24;
                    if i >= count {
                        break;
                    }
                }
            }
        }
    }
    sw(place + lw(table_at + flag_offset), lw(sp));
}

/// Open the SGI picture in the big buffer (magic 0xda01 read in either byte order; sizes at +6 width, +8 height, +10 channels) into the texture record `record`: the channel
/// rows are read (`RGBGetRow`, plain or from the run length table that follows the 512-byte header) into a buffer for each channel and the pixels are written to the download
/// buffer (+20) as bytes of red, green, blue and alpha (halved; 127 when there is no alpha channel), the rest of the power of two width filled with white; the width and height
/// go to +0 and +2 and the width as a power of two to +4. A picture over 0x10000 bytes or an allocation that fails ends the program with 1 (@ 0x2b9650).
pub fn OpenRGB(_unused: i32, record: i32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    memset(sp, 0, 44);
    let big = lw(g::bigbuffer);
    let tail = lw(big + 8);
    sd(sp, ld(big));
    sw(sp + 8, tail);
    let mut channels = lhu(sp + 10);
    if lhu(sp) == 0xda01 {
        for at in [sp + 2, sp + 4, sp + 6, sp + 8, sp + 10] {
            let value = lhu(at);
            sh(at, (value & 255) << 8 | ((value as u32) >> 8) as i32);
        }
        channels = lhu(sp + 10);
    }
    let mut i = 0;
    if channels >= 0 {
        let mut width = lhu(sp + 6);
        loop {
            let buffer = memAlloc(width << 8, 0);
            sw(sp + 12 + (i << 2), buffer);
            if buffer == 0 {
                exit(1);
            }
            channels = lhu(sp + 10);
            i += 1;
            if channels < i {
                break;
            }
            width = lhu(sp + 6);
        }
    }
    let wide = lhu(sp + 6);
    if (lhu(sp + 2) & 0xff00) == 256 {
        let rows = lhu(sp + 8) * channels;
        let bytes = rows << 2;
        let size = bytes + 15 & -16;
        let starts = memAlloc(size, 0);
        sw(sp + 36, starts);
        let lengths = memAlloc(size, 0);
        let starts_now = lw(sp + 36);
        sw(sp + 40, lengths);
        if starts_now == 0 || lengths == 0 {
            exit(1);
        }
        let source = lw(g::bigbuffer);
        sw(sp + 32, (rows << 3) + 512);
        memcpy(starts_now, source + 512, bytes);
        memcpy(lw(sp + 40), bytes + lw(g::bigbuffer) + 512, bytes);
        if lhu(sp) == 0xda01 {
            let mut first = lw(sp + 36);
            let mut left = ((bytes as u32) >> 2) as i32 - 1;
            let mut second = lw(sp + 40);
            if left != -1 {
                loop {
                    let value = lw(first);
                    left -= 1;
                    sw(first, value.swap_bytes());
                    first += 4;
                    let value = lw(second);
                    sw(second, value.swap_bytes());
                    second += 4;
                    if left == -1 {
                        break;
                    }
                }
            }
        }
    }
    let height = lhu(sp + 8);
    sh(record, wide);
    sh(record + 2, height);
    let power = PowerOf2(wide & 0xffff);
    let rows = lhu(sp + 8);
    let across = 1 << (power & 31);
    sw(record + 4, across);
    let mut out = lw(g::downloadbuffer);
    if (0x10000_u32) < (((across * rows) << 2) as u32) {
        exit(1);
    }
    sw(record + 20, out);
    let mut row = 0;
    if rows != 0 {
        let mut channels = lhu(sp + 10);
        loop {
            let mut channel = 0;
            let following;
            let mut size;
            if channels == 0 {
                following = row + 1;
                size = lhu(sp + 6);
            } else {
                following = row + 1;
                let mut next = channel + 1;
                loop {
                    RGBGetRow(sp, lw(sp + 12 + (next << 2)), row, channel);
                    channels = lhu(sp + 10);
                    channel = next;
                    next = channel + 1;
                    if channel >= channels {
                        break;
                    }
                }
                size = lhu(sp + 6);
            }
            let mut x = 0;
            if size != 0 {
                loop {
                    let mut c = 0;
                    let next_x;
                    let count;
                    if channels == 0 {
                        next_x = x + 1;
                        count = lhu(sp + 10);
                    } else {
                        next_x = x + 1;
                        loop {
                            let index = c + 1;
                            let is_alpha = c ^ 3;
                            c = index;
                            let value = lbu(lw(sp + 12 + (index << 2)) + x);
                            sb(out, if is_alpha != 0 { value } else { value >> 1 });
                            out += 1;
                            if c >= lhu(sp + 10) {
                                break;
                            }
                        }
                        count = lhu(sp + 10);
                    }
                    size = lhu(sp + 6);
                    if count == 3 {
                        sb(out, 127);
                        out += 1;
                        size = lhu(sp + 6);
                    }
                    x = next_x;
                    channels = lhu(sp + 10);
                    if x >= size {
                        break;
                    }
                }
            }
            let mut pad = out + 1;
            if (x as u32) < (lw(record + 4) as u32) {
                let mut last = out + 3;
                loop {
                    sb(last - 3, 255);
                    out += 4;
                    sb(pad + 1, 255);
                    x += 1;
                    sb(pad, 255);
                    sb(last, 127);
                    pad = out + 1;
                    last += 4;
                    if (x as u32) >= (lw(record + 4) as u32) {
                        break;
                    }
                }
            }
            row = following;
            channels = lhu(sp + 10);
            if row >= lhu(sp + 8) {
                break;
            }
        }
    }
    let lengths = lw(sp + 40);
    if lengths != 0 {
        memFree(lengths);
    }
    let starts = lw(sp + 36);
    let mut last = lhu(sp + 10);
    if starts != 0 {
        memFree(starts);
        last = lhu(sp + 10);
    }
    if last >= 0 {
        let mut offset = last << 2;
        loop {
            last -= 1;
            memFree(lw(sp + 12 + offset));
            offset = last << 2;
            if last < 0 {
                break;
            }
        }
    }
}
