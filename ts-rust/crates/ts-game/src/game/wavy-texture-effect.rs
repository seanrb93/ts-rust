//! The wavy picture of the front end's background.

use crate::*;

/// The remainder of `a` by `b`; for a `b` of 0 the dividend (what the original's divide leaves in the remainder register).
fn remainder(a: i32, b: i32) -> i32 {
    if b == 0 {
        a
    } else {
        a.wrapping_rem(b)
    }
}

/// The sine of the angle in the table `table` for the angle `x` (the table index is the angle times the constant at 0x3ac948, rounded, taken modulo 2048).
fn sine_at(table: i32, x: f32, scale: f32, half: f32) -> f32 {
    lwc1(((((x * scale + half) as i32) & 2047) << 2) + table)
}

/// Draw the picture of the texture `texture` as a grid of 15 by 7 waving cells (stack arguments: the strengths `size_x` and `size_y` of the waves, and the offsets `shift_x` and
/// `shift_y` of the picture; `colour` is the colour of every point, `cols` and `rows` the number of cells that the picture is cut into). A grid of 16 by 8 points is made first,
/// the points moved across and down by sums of two sines of the grid's place and of the background time (`frontbgtime`) with the frequencies `x_*`, `y_*` and the speeds
/// `*_time_*`, and every cell is then sent as a packet of 224 bytes (a quad of four points with the texture coordinates of the cell) (@ 0x22b6b8).
pub fn frontfxWaveyTexture(
    texture: i32,
    colour: i32,
    cols: i32,
    rows: i32,
    x_freq_x: f32,
    x_freq_y: f32,
    y_freq_x: f32,
    y_freq_y: f32,
    x_time_x: f32,
    x_time_y: f32,
    y_time_x: f32,
    y_time_y: f32,
) {
    let frame = Frame::new(2912);
    let sp = frame.sp();
    let four = 4.0_f32;
    let size_y = lwc1(sp + 2920);
    let size_x = lwc1(sp + 2912);
    let cols_f = cols as f32;
    let record = texture * 40 + lw(lw(g::thistexbuf) + 4);
    let time = lwc1(g::frontbgtime);
    let table = g::g_SinTable;
    let shift_x = lwc1(sp + 2928);
    let shift_y = lwc1(sp + 2936);
    let height = (lhu(record + 2) << 4) as f32;
    let scale = lwc1(0x3ac948);
    let width = (lhu(record) << 4) as f32;
    let half = 0.5_f32;
    sw(sp + 2648, sp + 2560);
    let mut row = 0;
    loop {
        let next = row + 1;
        let mut col = 0;
        let mut point = (row << 4) * 20 + sp;
        let y = row as f32 * (((size_y * four + 224.0_f32)).pdiv(7.0_f32)) - (size_y + size_y);
        let wave_x = sine_at(table, y * y_freq_x + time * y_time_x, scale, half);
        let wave_y = sine_at(table, y * y_freq_y + time * y_time_y, scale, half);
        loop {
            let column = col as f32;
            swc1(point + 8, y);
            sw(point, colour);
            col += 1;
            let x = column * (((size_x * four + 6.4e+02_f32)).pdiv(15.0_f32)) - (size_x + size_x);
            swc1(point + 4, x);
            let across = (sine_at(table, x * x_freq_x + time * x_time_x, scale, half) + wave_x) * size_x;
            swc1(point + 16, (sine_at(table, x * x_freq_y + time * x_time_y, scale, half) + wave_y) * size_y);
            swc1(point + 12, across);
            point += 20;
            if col >= 16 {
                break;
            }
        }
        row = next;
        if row >= 8 {
            break;
        }
    }
    dlSelectTextureKick(texture);
    let one = 1.0_f32;
    let sixteen = 16.0_f32;
    let dither = dlSetDitherMatrix(1);
    sw(sp + 2640, dither);
    let mut cell_row = 0;
    let mut base = cell_row << 4;
    loop {
        let following = cell_row + 1;
        let next_base = following << 4;
        sw(sp + 2644, base + 1);
        let below = next_base + 1;
        let mut cell = 0;
        loop {
            let across = remainder(cell, cols);
            let down = remainder(cell_row, rows);
            let first = cell + base;
            if cols == 0 {
                trap("break");
            }
            let right = cell + lw(sp + 2644);
            let across_f = across as f32;
            let down_f = down as f32;
            let top_left = first * 20 + sp;
            let sum_x = lwc1(top_left + 4) + lwc1(top_left + 12);
            let bottom_left = (cell + next_base) * 20 + sp;
            let sum_y = lwc1(top_left + 8) + lwc1(top_left + 16);
            sw(sp + 2560, lw(top_left));
            let bl_y2 = lwc1(bottom_left + 16);
            let bl_x1 = lwc1(bottom_left + 4);
            let u0 = (((across_f).pdiv(cols_f) + shift_x) * width) as i32;
            let bl_x2 = lwc1(bottom_left + 12);
            let bl_y1 = lwc1(bottom_left + 8);
            let v0 = (((down_f).pdiv(rows as f32) + shift_y) * height) as i32;
            sw(sp + 2572, u0);
            swc1(sp + 2564, f32::from_bits((sum_x * sixteen) as i32 as u32));
            swc1(sp + 2568, f32::from_bits((sum_y * sixteen) as i32 as u32));
            sw(sp + 2576, v0);
            let colour_bl = lw(bottom_left);
            sw(sp + 2592, u0);
            sw(sp + 2580, colour_bl);
            let v1 = ((((down_f + one)).pdiv(rows as f32) + shift_y) * height - one) as i32;
            swc1(sp + 2584, f32::from_bits(((bl_x1 + bl_x2) * sixteen) as i32 as u32));
            let top_right = right * 20 + sp;
            swc1(sp + 2588, f32::from_bits(((bl_y1 + bl_y2) * sixteen) as i32 as u32));
            sw(sp + 2596, v1);
            let tr_x = lwc1(top_right + 4) + lwc1(top_right + 12);
            let tr_y = lwc1(top_right + 8) + lwc1(top_right + 16);
            let bottom_right = (cell + below) * 20 + sp;
            sw(sp + 2600, lw(top_right));
            sw(sp + 2616, v0);
            let u1 = ((((across_f + one)).pdiv(cols_f) + shift_x) * width - one) as i32;
            let br_x1 = lwc1(bottom_right + 4);
            let br_x2 = lwc1(bottom_right + 12);
            swc1(sp + 2604, f32::from_bits((tr_x * sixteen) as i32 as u32));
            sw(sp + 2612, u1);
            swc1(sp + 2608, f32::from_bits((tr_y * sixteen) as i32 as u32));
            sw(sp + 2620, lw(bottom_right));
            swc1(sp + 2624, f32::from_bits(((br_x1 + br_x2) * sixteen) as i32 as u32));
            let br_y2 = lwc1(bottom_right + 16);
            let br_y1 = lwc1(bottom_right + 8);
            sw(sp + 2632, u1);
            sw(sp + 2636, v1);
            sq(sp + 2656, below as u128);
            swc1(sp + 2628, f32::from_bits(((br_y1 + br_y2) * sixteen) as i32 as u32));
            let packet = memdbAlloc(224);
            sb(lw(g::gfxtag) + 3, 48);
            sw(packet, 0x6c0d8000);
            let tag = lw(g::gfxtag);
            sw(tag + 4, packet);
            let mut put = packet + 4;
            sw(g::gfxtag, tag + 16);
            sw(put, 0x8004);
            put += 4;
            let following_cell = cell + 1;
            sw(put, 0x30ae4000);
            let mut left = 3;
            let vertices = lw(sp + 2648);
            put += 4;
            sw(put, 1299);
            put += 4;
            sh(tag, 14);
            sw(put, 0);
            let mut colour_at = vertices + 16;
            put += 4;
            let mut coord_at = vertices + 4;
            let mut word = put + 4;
            let _ = lq(sp + 2656);
            let mut out = word;
            loop {
                let x = lw(coord_at) as f32;
                let y = lw(coord_at + 4) as f32;
                let rgba = lw(colour_at - 16);
                let depth = lw(coord_at + 8) + 8;
                coord_at += 20;
                sw(out - 4, depth);
                sw(word, lw(colour_at) + 8);
                sw(word + 12, ((rgba as u32) >> 24) as i32);
                colour_at += 20;
                sw(word + 16, ((rgba as u32) >> 16) as i32 & 255);
                put += 48;
                sw(word + 20, ((rgba as u32) >> 8) as i32 & 255);
                left -= 1;
                sw(word + 24, rgba & 255);
                sw(word + 28, x as i32 + 0x6c00);
                sw(word + 32, y as i32 + 0x7900);
                sw(word + 36, -1);
                sw(word + 4, 0);
                sw(word + 8, 0);
                sw(out + 40, 0);
                out += 48;
                word = out;
                if left < 0 {
                    break;
                }
            }
            cell = following_cell;
            sw(put, 0x140007fc);
            let end = put + 4;
            sw(end, 0x11000000);
            sw(end + 4, 0);
            if cell >= 15 {
                break;
            }
        }
        cell_row = following;
        base = cell_row << 4;
        if cell_row >= 7 {
            break;
        }
    }
    dlSetDitherMatrix(lw(sp + 2640));
}
