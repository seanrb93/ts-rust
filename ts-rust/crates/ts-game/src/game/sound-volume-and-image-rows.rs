//! The volume of a sound by its distance, and a row of an image.

use crate::*;

/// Work out the volume of the sound `sound` (loudness as a percentage at +20) into `*left` and `*right`: from `base` up to 0x1fff by the percentage when `position` is 0, else less
/// the farther the nearest player is from `position` (nothing is changed when no player is nearer than the game's far value); with one player whose camera matrix (+1764) is
/// known the sound is moved to the side it comes from (the side farther from it is made quieter). The volumes are scaled by the sound volume setting (@ 0x2049c0).
pub fn soundCalcVol(sound: i32, position: i32, left: i32, right: i32, base: i32, range: f32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let far = lwc1(0x3afd00);
    let mut nearest = far;
    let loudness;
    if position == 0 {
        loudness = lhu(sound + 20) as f32 / 1e+02_f32 * (0x1fff - base) as f32;
    } else {
        let count = lw(g::lvnumplayers);
        let mut i = 0;
        if count > 0 {
            let players = lw(g::players);
            let x = lwc1(position);
            let y = lwc1(position + 4);
            let z = lwc1(position + 8);
            loop {
                let player = i * 1820 + players;
                let dx = x - lwc1(player + 152);
                swc1(sp, dx);
                let dy = y - lwc1(player + 156);
                swc1(sp + 4, dy);
                let dz = z - lwc1(player + 160);
                let squared = dx * dx + dy * dy + dz * dz;
                swc1(sp + 8, dz);
                if squared < nearest {
                    nearest = squared;
                }
                i += 1;
                if i >= count {
                    break;
                }
            }
        }
        if nearest == lwc1(0x3afd00) {
            return;
        }
        let mut distance = nearest.abs().sqrt();
        if range < distance {
            distance = range;
        }
        loudness = lhu(sound + 20) as f32 / 1e+02_f32 * (0x1fff - base) as f32 * ((range - distance) / range);
    }
    let volume = base + loudness as i32;
    sw(left, volume);
    sw(right, volume);
    let mut shifted = false;
    if position != 0 && lw(g::lvnumplayers) == 1 {
        let matrix = lw(lw(g::currentplayer) + 1764);
        if matrix != 0 {
            matrixVecRotAligned(&[matrix as i64, sp as i64]);
            let x = lwc1(sp);
            let z = lwc1(sp + 8);
            let side = x / (x * x + z * z).abs().sqrt();
            if 0.0_f32 < side {
                sw(left, (volume as f32 * (1.0_f32 - side)) as i32);
                shifted = true;
            } else {
                let full = volume as f32;
                sw(right, (side * full + full) as i32);
            }
        }
    }
    let _ = shifted;
    let level = lw(left);
    sw(left, (level * lw(g::gv_soundvolume)) >> 4);
    sw(right, (lw(right) * lw(g::gv_soundvolume)) >> 4);
}

/// Read one row of an image into `out`: the image `image` (a record whose halfword at +2 has 0x100 in its top byte when the rows are packed, the width at +6, the height at +8, the
/// buffer at +12, the tables of row positions and sizes at +36 and +40) has its row `row` of tile `tile` copied out from the big buffer (512 bytes in for the plain image); a
/// packed row is first copied to the image's buffer and then unpacked: a header byte with its top bit clear is a run of that many copies of the next byte, with it set a run of
/// that many bytes as they are; a header of 0 ends the row (@ 0x2b94f8).
pub fn RGBGetRow(image: i32, out: i32, tile: i32, row: i32) {
    if (lhu(image + 2) & 0xff00) != 256 {
        let width = lhu(image + 6);
        memcpy(out, row * width * lhu(image + 8) + tile * width + lw(g::bigbuffer) + 512, width);
        return;
    }
    let at = (row * lhu(image + 8) + tile) << 2;
    let buffer = lw(image + 12);
    let size = lw(at + lw(image + 40));
    let source = lw(g::bigbuffer) + lw(at + lw(image + 36));
    memcpy(buffer, source, size);
    let mut header_at = lw(image + 12);
    let mut put = out;
    let mut after_header = header_at + 1;
    'rows: loop {
        let mut header = lbu(header_at);
        'step: {
            loop {
                let count = header & 127;
                header_at = after_header;
                if count == 0 {
                    break 'rows;
                }
                if (header & 128) != 0 {
                    break;
                }
                let value = lbu(after_header);
                let mut left = count - 1;
                header_at = after_header + 1;
                if left == -1 {
                    break 'step;
                }
                after_header += 2;
                loop {
                    sb(put, value);
                    left -= 1;
                    put += 1;
                    if left == -1 {
                        break;
                    }
                }
                header = lbu(header_at);
            }
            let mut left = (header & 127) - 1;
            after_header += 1;
            if left == -1 {
                continue 'rows;
            }
            loop {
                let byte = lbu(header_at);
                left -= 1;
                header_at += 1;
                sb(put, byte);
                put += 1;
                if left == -1 {
                    break;
                }
            }
            after_header = header_at + 1;
            continue 'rows;
        }
        after_header += 2;
    }
}
