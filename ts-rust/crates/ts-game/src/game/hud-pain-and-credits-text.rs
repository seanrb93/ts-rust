//! The ring of pain at the edge of the screen, and the texture coordinates of the credits.

use crate::*;

/// Draw the ring of pain (a strip of 38 vertices, 19 pairs, round the screen in an ellipse, the colour `colour` clear on the inside) with the pain texture (@ 0x2b2948).
pub fn hudPainGfx(colour: i32) {
    let frame = Frame::new(832);
    let sp = frame.sp();
    let window = g::window;
    let height = lw(window + 32);
    let centre_y = lw(window + 28) as f32;
    let mut angle = lwc1(0x3b0190);
    let to_table = lwc1(0x3ad7ac);
    let centre_x = lw(window + 8) as f32;
    let degrees = 180.0_f32;
    let outer_x = lw(window + 12) as f32 * 0.75_f32;
    let table_scale = lwc1(0x3ad7b0);
    let half = 0.5_f32;
    let mut inner_angle = 0.0_f32;
    let sin_table = g::g_SinTable;
    let sixty_four = 64.0_f32;
    let sixteen = 16.0_f32;
    let step = 20.0_f32;
    let mut a5 = sp;
    let mut a4 = sp + 4;
    let mut i = 0;
    loop {
        let outer_step = angle * to_table;
        sw(a5, colour & -256);
        sw(a4 + 16, colour);
        angle += step;
        let a1 = ((outer_step).pdiv(degrees) * table_scale + half) as i32;
        let v1 = ((inner_angle * to_table).pdiv(degrees) * table_scale + half) as i32;
        let cos_inner = lwc1((((v1 + 512) & 2047) << 2) + sin_table);
        let sin_inner = lwc1(((v1 & 2047) << 2) + sin_table);
        let f5 = lwc1(((a1 & 2047) << 2) + sin_table) * sixty_four + cos_inner * sixteen;
        let x0 = (centre_x + cos_inner * ((height + ((height as u32) >> 31) as i32) >> 1) as f32) as i32;
        let v1_3 = (lwc1((((a1 + 512) & 2047) << 2) + sin_table) * sixty_four + sin_inner * sixteen) as i32;
        let y0 = (centre_y - sin_inner * ((if -1 < height { height } else { height + 3 }) >> 2) as f32) as i32;
        sw(a4, x0);
        let v0 = f5 as i32;
        sw(a4 + 4, y0);
        let x1 = (centre_x + cos_inner * outer_x) as i32;
        let y1 = (centre_y - sin_inner * (height as f32 * 0.75_f32)) as i32;
        sw(a4 + 12, (y0 + v0 + 1024) << 3);
        sw(a4 + 8, (x0 + v1_3 + 1024) << 2);
        sw(a4 + 28, (x1 + ((v1_3 >> 1) + 1024)) << 2);
        sw(a4 + 20, x1);
        sw(a4 + 24, y1);
        sw(a5 + 36, (y1 + ((v0 >> 1) + 1024)) << 3);
        a4 += 40;
        let end = a4 + 28;
        a5 += 40;
        if i == 18 {
            sw(end - 68, lw(sp + 4));
            sw(end - 64, lw(sp + 8));
            sw(end - 48, lw(sp + 24));
            sw(end - 44, lw(sp + 28));
        }
        i += 1;
        inner_angle += step;
        if i >= 19 {
            break;
        }
    }
    dlSelectTextureKick(lw(0x3b0198));
    let dither = dlSetDitherMatrix(2);
    dlDrawTriangleStrip(sp, 38, 1);
    dlSetDitherMatrix(dither);
}

/// Set the texture coordinates and the alpha of the three layers of the credits (each of 12 quads of four vertices) from the camera height and the two positions `first` and
/// `second` (in rows of 5): the alpha of the layer (`creditalpha`) times the square of how near the position is to the camera, in the top byte of the vertex colours (@ 0x2af868).
pub fn creditsSetST(base: i32, first: i32, second: i32) {
    let camera_y = lwc1(g::creditscampos + 8);
    let positions = [base + 2304, base + 2308, base + 2320, base + 2324, base + 2336, base + 2340, base + 2352, base + 2356];
    let colours = [base + 0x1200, base + 0x1204, base + 0x1208, base + 0x120c];
    let near_a = 1.0_f32 - ((first as f32 * 5.0_f32 - camera_y)).pdiv(200.0_f32);
    let near_b = 1.0_f32 - ((second as f32 * 5.0_f32 - camera_y)).pdiv(200.0_f32);
    let mut layer = 0;
    loop {
        let rotation = (layer << 2) + g::layerrot;
        let alpha = lw((layer << 2) + g::creditalpha) as f32;
        let layer_colours = layer * 192;
        let layer_positions = layer * 768;
        let top_b = ((alpha * (near_b * near_b)) as i32) << 24;
        let top_a = ((alpha * (near_a * near_a)) as i32) << 24;
        let mut quad = 0;
        loop {
            let low = quad as f32;
            let colour_at = (quad << 4) + layer_colours;
            let position_at = (quad << 6) + layer_positions;
            let low_s = (low).pdiv(6.0_f32);
            let c0 = colours[0] + colour_at;
            let c1 = colours[1] + colour_at;
            let c2 = colours[2] + colour_at;
            let c3 = colours[3] + colour_at;
            let next = quad + 1;
            let high_s = (next as f32).pdiv(6.0_f32);
            let t_first = (first.wrapping_rem(6) as f32).pdiv(6.0_f32);
            quad = next;
            let mut t_second = (second.wrapping_rem(6) as f32).pdiv(6.0_f32);
            let more = quad < 12;
            if t_second < t_first {
                t_second += 1.0_f32;
            }
            let v0 = lw(c0) & 0xffffff | top_a;
            swc1(positions[0] + position_at, low_s + lwc1(rotation));
            swc1(positions[1] + position_at, t_first);
            sw(c0, v0);
            let v1 = lw(c1) & 0xffffff | top_a;
            swc1(positions[2] + position_at, high_s + lwc1(rotation));
            swc1(positions[3] + position_at, t_first);
            sw(c1, v1);
            let v2 = lw(c2) & 0xffffff | top_b;
            swc1(positions[4] + position_at, low_s + lwc1(rotation));
            swc1(positions[5] + position_at, t_second);
            sw(c2, v2);
            let v3 = lw(c3) & 0xffffff | top_b;
            swc1(positions[6] + position_at, high_s + lwc1(rotation));
            swc1(positions[7] + position_at, t_second);
            sw(c3, v3);
            if !more {
                break;
            }
        }
        layer += 1;
        if layer >= 3 {
            break;
        }
    }
}
