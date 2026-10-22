//! Functions: the inflate decompressor of the packed files. It is the one of Info-ZIP and gzip (a method of 32 KB window, Huffman tables of 8 bytes an entry), with the state kept in globals:
//!
//! * 0x3afeb8 the input, 0x3afebc the position in it, 0x3afec0 the size of the input (reads past the end give -1)
//! * 0x3afec4 the window of 0x8000 bytes, 0x3afec8 the position `wp` in it (it is flushed with `FlushOut` when it is full)
//! * 0x3afed4 the bit buffer `bb`, 0x3afed8 the number of bits in it `bk`
//!
//! A Huffman table is an array of entries of 8 bytes: the extra bits `e` (+0; 16 is a literal, 15 the end of the block, 99 invalid, more than 16 a link to a table of `e - 16` bits), the bits to drop `b` (+1) and the value `n` (+4, 16 bits) or the link to the next table (+4).

use crate::*;

/// Take the next byte of the input, or -1 past its end.
fn next_byte() -> i32 {
    let at = lw(0x3afebc);
    if (at as u32) >= (lw(0x3afec0) as u32) {
        return -1;
    }
    let byte = lbu(lw(0x3afeb8) + at);
    sw(0x3afebc, at + 1);
    byte
}

/// Fill the bit buffer `b` of `k` bits until it has at least `n` bits.
fn need_bits(b: &mut i32, k: &mut i32, n: i32) {
    while (*k as u32) < (n as u32) {
        *b |= next_byte() << (*k & 31);
        *k += 8;
    }
}

/// Throw away the `n` lowest bits of the bit buffer.
fn dump_bits(b: &mut i32, k: &mut i32, n: i32) {
    *b = ((*b as u32) >> (n & 31)) as i32;
    *k -= n;
}

/// Inflate the whole stream: blocks until the last one, and then give back the whole bytes left in the bit buffer to the input and write what is in the window.
pub fn Inflate() {
    let frame = Frame::new(32);
    let sp = frame.sp();
    sw(0x3afec8, 0);
    sw(0x3afed8, 0);
    sw(0x3afed4, 0);
    loop {
        let last = if inflate_block(sp) != 0 { exit(1) } else { lw(sp) };
        if last != 0 {
            break;
        }
    }
    let mut bits = lw(0x3afed8);
    let mut position = lw(0x3afebc);
    if (bits as u32) >= 8 {
        loop {
            bits -= 8;
            position -= 1;
            sw(0x3afed8, bits);
            sw(0x3afebc, position);
            if (bits as u32) < 8 {
                break;
            }
        }
    }
    FlushOut();
}

/// Inflate one block: the first bit (put at `a0`) says if it is the last, the next two the kind: 0 stored, 1 fixed codes, 2 dynamic codes. Gives 0, or an error that ends the game for a kind that does not exist.
pub fn inflate_block(a0: i32) -> i32 {
    let mut k = lw(0x3afed8);
    let mut b = lw(0x3afed4);
    need_bits(&mut b, &mut k, 1);
    sw(a0, b & 1);
    dump_bits(&mut b, &mut k, 1);
    need_bits(&mut b, &mut k, 2);
    let kind = b & 3;
    sw(0x3afed8, k - 2);
    sw(0x3afed4, ((b as u32) >> 2) as i32);
    AllocReset();
    match kind {
        2 => inflate_dynamic(),
        0 => inflate_stored(),
        1 => inflate_fixed(),
        _ => exit(1),
    }
}

/// Inflate a block with the fixed codes of the format: literals and lengths are 8 bits (0 to 143), 9 (144 to 255), 7 (256 to 279) and 8 (280 to 287), and the distances 5 bits. Gives 0 if it is good.
pub fn inflate_fixed() -> i32 {
    let frame = Frame::new(1184);
    let sp = frame.sp();
    // The lengths of the codes are at sp, the table of literals and lengths at sp+1152 with its bits at sp+1156, the one of distances at sp+1160 with its bits at sp+1164.
    for i in 0..144 {
        sw(sp + i * 4, 8);
    }
    for i in 144..256 {
        sw(sp + i * 4, 9);
    }
    for i in 256..280 {
        sw(sp + i * 4, 7);
    }
    for i in 280..288 {
        sw(sp + i * 4, 8);
    }
    sw(sp + 1156, 7);
    let result = huft_build(sp, 288, 257, g::cplens, g::cplext, sp + 1152, sp + 1156);
    if result != 0 {
        return result;
    }
    for i in 0..30 {
        sw(sp + i * 4, 5);
    }
    sw(sp + 1164, 5);
    let result = huft_build(sp, 30, 0, g::cpdist, g::cpdext, sp + 1160, sp + 1164);
    if result >= 2 {
        return result;
    }
    let failed = inflate_codes(lw(sp + 1152), lw(sp + 1160), lw(sp + 1156), lw(sp + 1164));
    (failed != 0) as i32
}

/// Inflate a stored block: the rest of the byte is dropped, then comes the length (16 bits) and its complement, then the bytes themselves. The game ends if the complement is wrong.
pub fn inflate_stored() -> i32 {
    let mut w = lw(0x3afec8);
    let mut k = lw(0x3afed8);
    let mut b = lw(0x3afed4);
    let rest = k & 7;
    dump_bits(&mut b, &mut k, rest);
    need_bits(&mut b, &mut k, 16);
    let mut length = b & 0xffff;
    dump_bits(&mut b, &mut k, 16);
    need_bits(&mut b, &mut k, 16);
    if length == !b & 0xffff {
        dump_bits(&mut b, &mut k, 16);
    } else {
        exit(1);
    }
    while length != 0 {
        length -= 1;
        need_bits(&mut b, &mut k, 8);
        sb(lw(0x3afec4) + w, b);
        w += 1;
        if w == 0x8000 {
            sw(0x3afec8, w);
            FlushOut();
            w = 0;
        }
        dump_bits(&mut b, &mut k, 8);
    }
    sw(0x3afed4, b);
    sw(0x3afed8, k);
    sw(0x3afec8, w);
    0
}

/// Make the `count` entries from the entry `at` of the lengths of the codes (a table of words at `sp`) all `value`, and add `count` to `at`.
fn fill_lengths(sp: i32, at: &mut i32, count: i32, value: i32) {
    for _ in 0..count {
        sw(sp + (*at << 2), value);
        *at += 1;
    }
}

/// Inflate a block with the dynamic codes: it starts with the counts of the codes, then the lengths of the codes that give the lengths of the other codes, and then the lengths of the codes for the literals and lengths and for the distances. Gives 0 if it is good, 1 if the counts or the codes are wrong; the game ends if a table is not complete.
pub fn inflate_dynamic() -> i32 {
    let frame = Frame::new(1392);
    let sp = frame.sp();
    // The lengths are at sp, the table of codes at sp+1264 with its bits at sp+1268, then the tables of literals and lengths and of distances reuse those and sp+1272 and sp+1276.
    let mut k = lw(0x3afed8);
    let mut b = lw(0x3afed4);
    need_bits(&mut b, &mut k, 5);
    let literals = (b & 31) + 257;
    dump_bits(&mut b, &mut k, 5);
    need_bits(&mut b, &mut k, 5);
    let distances = (b & 31) + 1;
    dump_bits(&mut b, &mut k, 5);
    need_bits(&mut b, &mut k, 4);
    let code_lengths = (b & 15) + 4;
    dump_bits(&mut b, &mut k, 4);
    if (literals as u32) >= 287 || (distances as u32) >= 31 {
        return 1;
    }
    for j in 0..code_lengths {
        need_bits(&mut b, &mut k, 3);
        sw(sp + (lw(g::border + j * 4) << 2), b & 7);
        dump_bits(&mut b, &mut k, 3);
    }
    for j in code_lengths..19 {
        sw(sp + (lw(g::border + j * 4) << 2), 0);
    }
    sw(sp + 1268, 7);
    let result = huft_build(sp, 19, 19, 0, 0, sp + 1264, sp + 1268);
    if result != 0 {
        return result;
    }
    let total = literals + distances;
    let mut i = 0;
    let mut last = 0;
    let bits = lw(sp + 1268);
    let mask = lhu((bits << 1) + g::mask_bits);
    while (i as u32) < (total as u32) {
        need_bits(&mut b, &mut k, bits);
        let entry = lw(sp + 1264) + ((b & mask) << 3);
        dump_bits(&mut b, &mut k, lbu(entry + 1));
        let code = lhu(entry + 4);
        if code < 16 {
            last = code;
            sw(sp + (i << 2), last);
            i += 1;
        } else if code == 16 {
            need_bits(&mut b, &mut k, 2);
            let repeat = (b & 3) + 3;
            dump_bits(&mut b, &mut k, 2);
            if (total as u32) < (i + repeat) as u32 {
                return 1;
            }
            fill_lengths(sp, &mut i, repeat, last);
        } else if code == 17 {
            need_bits(&mut b, &mut k, 3);
            let repeat = (b & 7) + 3;
            dump_bits(&mut b, &mut k, 3);
            if (total as u32) < (i + repeat) as u32 {
                return 1;
            }
            fill_lengths(sp, &mut i, repeat, 0);
            last = 0;
        } else {
            need_bits(&mut b, &mut k, 7);
            let repeat = (b & 127) + 11;
            dump_bits(&mut b, &mut k, 7);
            if (total as u32) < (i + repeat) as u32 {
                return 1;
            }
            fill_lengths(sp, &mut i, repeat, 0);
            last = 0;
        }
    }
    sw(0x3afed4, b);
    sw(0x3afed8, k);
    sw(sp + 1268, lw(g::lbits));
    let result = huft_build(sp, literals, 257, g::cplens, g::cplext, sp + 1264, sp + 1268);
    if result != 0 {
        if result != 1 {
            return result;
        }
        exit(1);
    }
    sw(sp + 1276, lw(g::dbits));
    let result = huft_build(sp + (literals << 2), distances, 0, g::cpdist, g::cpdext, sp + 1272, sp + 1276);
    if result == 0 {
        let failed = inflate_codes(lw(sp + 1264), lw(sp + 1272), lw(sp + 1268), lw(sp + 1276));
        return (failed != 0) as i32;
    }
    if result != 1 {
        return result;
    }
    exit(1);
    1
}

/// Find the entry for the code in the bit buffer in the table `table` of `bits` bits (`mask` is the mask for them), following the links to the lower tables, and drop the bits of the code. Gives the entry and its `e`, or `None` for an invalid code.
fn lookup_code(table: i32, bits: i32, mask: i32, b: &mut i32, k: &mut i32) -> Option<(i32, i32)> {
    need_bits(b, k, bits);
    let mut entry = table + ((*b & mask) << 3);
    let mut extra = lbu(entry);
    while (extra as u32) >= 17 {
        if extra == 99 {
            return None;
        }
        dump_bits(b, k, lbu(entry + 1));
        extra -= 16;
        need_bits(b, k, extra);
        entry = lw(entry + 4) + ((*b & lhu((extra << 1) + g::mask_bits)) << 3);
        extra = lbu(entry);
    }
    dump_bits(b, k, lbu(entry + 1));
    Some((entry, extra))
}

/// Inflate the data of a block with the tables `a0` of literals and lengths (`a2` bits) and `a1` of distances (`a3` bits). The window of 0x8000 bytes is flushed each time it is full. Gives 0 when the end of the block is found, 1 for a code that is not valid.
pub fn inflate_codes(a0: i32, a1: i32, a2: i32, a3: i32) -> i32 {
    let literal_mask = lhu((a2 << 1) + g::mask_bits);
    let distance_mask = lhu((a3 << 1) + g::mask_bits);
    let mut b = lw(0x3afed4);
    let mut k = lw(0x3afed8);
    let mut w = lw(0x3afec8);
    loop {
        let Some((entry, extra)) = lookup_code(a0, a2, literal_mask, &mut b, &mut k) else {
            return 1;
        };
        if extra == 16 {
            sb(lw(0x3afec4) + w, lbu(entry + 4));
            w += 1;
            if w == 0x8000 {
                sw(0x3afec8, w);
                FlushOut();
                w = 0;
            }
            continue;
        }
        if extra == 15 {
            sw(0x3afec8, w);
            sw(0x3afed4, b);
            sw(0x3afed8, k);
            return 0;
        }
        // A length: the extra bits come after the code.
        need_bits(&mut b, &mut k, extra);
        let mut length = lhu(entry + 4) + (b & lhu((extra << 1) + g::mask_bits));
        dump_bits(&mut b, &mut k, extra);
        let Some((entry, extra)) = lookup_code(a1, a3, distance_mask, &mut b, &mut k) else {
            return 1;
        };
        need_bits(&mut b, &mut k, extra);
        let mut from = w - lhu(entry + 4) - (b & lhu((extra << 1) + g::mask_bits));
        dump_bits(&mut b, &mut k, extra);
        // Copy the length from the distance back in the window, in parts that do not go past its end.
        loop {
            from &= 0x7fff;
            let room = 0x8000 - if (w as u32) >= (from as u32) { w } else { from };
            let mut count = if (length as u32) < (room as u32) { length } else { room };
            length -= count;
            let slide = lw(0x3afec4);
            if ((w - from) as u32) >= (count as u32) {
                memcpy(slide + w, slide + from, count);
                w += count;
                from += count;
            } else {
                while count != 0 {
                    sb(slide + w, lbu(slide + from));
                    w += 1;
                    from += 1;
                    count -= 1;
                }
            }
            if w == 0x8000 {
                sw(0x3afec8, w);
                FlushOut();
                w = 0;
            }
            if length == 0 {
                break;
            }
        }
    }
}

/// Make the Huffman table for the `n` code lengths at `b` (words), of which the first `s` are for plain values (literals) and the rest use the lists `d` of base values and `e` of extra bits. `t` gets the table, `m` the bits to look at first (it is the most wanted, and gets the real one). The tables are allocated with `AllocMem`. Gives 0 if it is good, 1 if the set of codes is not complete, 2 if there are more codes than bits can have, 3 if the memory is out.
pub fn huft_build(b: i32, n: i32, s: i32, d: i32, e: i32, t: i32, m: i32) -> i32 {
    let frame = Frame::new(1744);
    let sp = frame.sp();
    // The entry being made is kept at sp+80: `e` at +0, `b` at +1 and the value `n` (16 bits) or the link at +4. It is copied whole to the tables, with the two bytes at +2 that nothing sets.
    let entry_at = sp + 80;
    let mut t = t;
    let mut count = [0_i32; 17];
    let mut offset = [0_i32; 17];
    let mut stack = [0_i32; 16];
    let mut values = [0_i32; 288];

    // Count the codes of each length.
    let mut p = b;
    let mut i = n;
    loop {
        count[lw(p) as usize] += 1;
        p += 4;
        i -= 1;
        if i == 0 {
            break;
        }
    }
    if count[0] == n {
        sw(m, 0);
        sw(t, 0);
        return 0;
    }

    // The shortest and the longest code, and the bits of the first table between them.
    let mut j = 1;
    let mut bits = lw(m);
    if count[1] == 0 {
        loop {
            j += 1;
            if (j as u32) >= 17 {
                break;
            }
            if count[j as usize] != 0 {
                break;
            }
        }
    }
    let mut k = j;
    if (bits as u32) < (j as u32) {
        bits = j;
    }
    i = 16;
    while i != 0 && count[i as usize] == 0 {
        i -= 1;
    }
    let longest = i;
    if (longest as u32) < (bits as u32) {
        bits = longest;
    }
    sw(m, bits);

    // Add the dummy codes needed to fill the last length.
    let mut spare = 1_i32 << (j & 31);
    if (j as u32) < (i as u32) {
        loop {
            spare -= count[j as usize];
            j += 1;
            if spare < 0 {
                return 2;
            }
            spare <<= 1;
            if (j as u32) >= (i as u32) {
                break;
            }
        }
    }
    spare -= count[i as usize];
    if spare < 0 {
        return 2;
    }
    count[i as usize] += spare;

    // The first place in the table of values for each length.
    j = 0;
    offset[1] = 0;
    let mut from = 1;
    let mut to = 2;
    for _ in 0..(i - 1) {
        j += count[from];
        from += 1;
        offset[to] = j;
        to += 1;
    }

    // The values in the order of their lengths.
    p = b;
    i = 0;
    loop {
        let length = lw(p) as usize;
        p += 4;
        if length != 0 {
            values[offset[length] as usize] = i;
            offset[length] += 1;
        }
        i += 1;
        if (i as u32) >= (n as u32) {
            break;
        }
    }

    // Make the tables for each code. Then `i` is the code of this length, `w` the bits that are done in the tables above, `h` the level of the table, `z` its size, `q` its start.
    offset[0] = 0;
    i = 0;
    let mut next_value = 0;
    let mut h = -1_i32;
    let mut w = -bits;
    stack[0] = 0;
    let mut z = 0_i32;
    let mut q = 0_i32;
    while k <= longest {
        let mut a = count[k as usize] - 1;
        while a != -1 {
            // Make the tables down to the level this code needs.
            while k > w + bits {
                h += 1;
                w += bits;
                let limit = if ((longest - w) as u32) > (bits as u32) { bits } else { longest - w };
                let mut jj = k - w;
                let mut f = 1_i32 << (jj & 31);
                if ((a + 1) as u32) < (f as u32) {
                    f -= a + 1;
                    let mut at = k as usize;
                    jj += 1;
                    loop {
                        at += 1;
                        if (jj as u32) >= (limit as u32) {
                            break;
                        }
                        f <<= 1;
                        let c = count[at];
                        if (c as u32) >= (f as u32) {
                            break;
                        }
                        jj += 1;
                        f -= c;
                    }
                }
                z = 1_i32 << (jj & 31);
                let table = AllocMem((z + 1) << 3);
                if table == 0 {
                    return 3;
                }
                sw(t, table + 8);
                sw(table + 4, 0);
                t = table + 4;
                q = table + 8;
                stack[h as usize] = q;
                if h != 0 {
                    offset[h as usize] = i;
                    sb(entry_at + 1, bits);
                    sb(entry_at, 16 + jj);
                    sw(entry_at + 4, q);
                    let at = stack[(h - 1) as usize] + ((((i as u32) >> ((w - bits) & 31)) as i32) << 3);
                    sd(at, ld(entry_at));
                }
            }

            // The entry for this code.
            sb(entry_at + 1, k - w);
            if (next_value as u32) >= (n as u32) {
                sb(entry_at, 99);
            } else {
                let value = values[next_value as usize];
                if (value as u32) < (s as u32) {
                    sb(entry_at, if (value as u32) < 256 { 16 } else { 15 });
                    sh(entry_at + 4, value);
                } else {
                    sb(entry_at, lbu(e + ((value - s) << 1)));
                    sh(entry_at + 4, lhu(d + ((value - s) << 1)));
                }
                next_value += 1;
            }

            // Fill all the places in the table that have this code.
            let step = 1_i32 << ((k - w) & 31);
            let mut place = ((i as u32) >> (w & 31)) as i32;
            while (place as u32) < (z as u32) {
                sd(q + (place << 3), ld(entry_at));
                place += step;
            }

            // Increment the code, from its top bit down.
            let mut bit = 1_i32 << ((k - 1) & 31);
            while i & bit != 0 {
                i ^= bit;
                bit = ((bit as u32) >> 1) as i32;
            }
            i ^= bit;

            // Go back up over the tables that are finished.
            while (i & ((1_i32 << (w & 31)) - 1)) != offset[h as usize] {
                h -= 1;
                w -= bits;
            }
            a -= 1;
        }
        k += 1;
    }
    (spare != 0 && longest != 1) as i32
}
