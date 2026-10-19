//! The window's display list packet and the palette of a 4-bit picture.

use crate::*;

/// Add the packet that draws the main window's frame: a quad of floats (position from the window's values, the constant 0x3ac81c, 1.0 and 32768.0)
/// followed by eight floats of 1.0 (@ 0x20c7e8).
pub fn windowGfx() {
    let window = g::window;
    let height_offset = lw(window + 36);
    let left = lw(window + 16) as f32;
    let x = lw(window) as f32;
    let y = lw(window + 20) as f32;
    let words: [f32; 8] = [
        left,
        (0 - height_offset) as f32,
        lwc1(0x3ac81c),
        1.0,
        x + 1728.0 + left,
        y + 1936.0 + height_offset as f32,
        32768.0,
        0.0,
    ];
    let mut all: Vec<i32> = words.iter().map(|f| f.to_bits() as i32).collect();
    all[7] = 0;
    all.extend(std::iter::repeat(1.0_f32.to_bits() as i32).take(8));
    all.extend([0x1400078b, 0, 0]);
    super::display_list_shapes::push_packet(80, 0x6c048010, 5, &all);
}

/// Rearrange a 256-entry palette (words) from `src` to `dst` in the order the graphics chip wants it: in each group of 32 the second and third blocks of 8 swap (@ 0x2b9050).
pub fn MakeCSM1Palette(src: i32, dst: i32) {
    let mut out = dst;
    for base in (0..256).step_by(32) {
        for block in [0, 16, 8, 24] {
            for k in 0..8 {
                sw(out, lw(src + 4 * (base + block + k)));
                out += 4;
            }
        }
    }
}
