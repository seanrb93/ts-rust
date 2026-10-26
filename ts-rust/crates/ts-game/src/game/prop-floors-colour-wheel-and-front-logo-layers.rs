//! The floors and walls a prop carries with it, the colour wheel of the light editor, and a layer of the front end's logo.

use crate::*;

/// Make the floors of the prop `prop` (its table at +480) from the model's original ones (`propGetOriginalFloors` gives the list and a table of bone numbers): each floor's count and
/// the two halfwords are copied, and each of its points (12 bytes each, the first at +8 and the second and third at +12 and +16 in the original's layout) is moved to the prop's place
/// by the matrix of the prop's place and angle, and when `by_bones` is set also by the matrix of its bone in the model (@ 0x2675b8).
pub fn propCalculateFloors(prop: i32, by_bones: i32) {
    let frame = Frame::new(112);
    let sp = frame.sp();
    sw(sp + 72, by_bones);
    if lw(prop + 480) == 0 {
        return;
    }
    propGetOriginalFloors(lw(prop + 4), sp + 64, sp + 68);
    matrixTransRotY(sp, lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56), ps2float::mul(lwc1(prop + 76), lwc1(0x3acbc8)).pdiv(1.8e+02_f32));
    let mut originals = lw(sp + 64);
    let mut source = lw(originals);
    let mut index = 0;
    if source != 0 {
        let mut offset = 0;
        let mut floors = lw(prop + 480);
        loop {
            let mut point = 0;
            let count = lw(source);
            let floor = lw(offset + floors);
            sh(floor + 4, lhu(source + 4));
            sw(floor, count);
            sh(floor + 6, lhu(source + 6));
            let next = index + 1;
            if count != 0 {
                sw(sp + 76, source + 16);
                let first = source + 8;
                sw(sp + 80, floor + 8);
                let second = source + 12;
                loop {
                    let at = point * 12;
                    let third = lw(sp + 76);
                    swc1(lw(sp + 80) + point * 12, lwc1(first + point * 12));
                    swc1(floor + 12 + point * 12, lwc1(second + point * 12));
                    swc1(floor + 16 + point * 12, lwc1(third + point * 12));
                    let bones = lw(sp + 68);
                    if lw(sp + 72) != 0 {
                        let model_matrices = lw(lw(prop + 32) + 4);
                        matrixVecMul(model_matrices + (lw(offset + bones) << 6), floor + (at + 8));
                    }
                    point += 1;
                    matrixVecMul(sp, floor + (at + 8));
                    if (point as u32) >= (lw(floor) as u32) {
                        break;
                    }
                }
                originals = lw(sp + 64);
            }
            index = next;
            offset = index << 2;
            source = lw(offset + originals);
            if source == 0 {
                break;
            }
            floors = lw(prop + 480);
        }
    }
}

/// Make the memory for the walls and floors of the prop `prop` from the model's table (`PWFTAB`, 16 bytes a model: a list of wall sets at +0 and of floor sets at +8, both ended
/// by 0): one block of the wall memory holds the pointers to the wall records (36 bytes each), the pointers to the floor records and the floors' points (12 bytes each and 8 more
/// for the record); the pointers are put at +484 and +480 of the prop; none for a model with no sets (@ 0x2682d0).
pub fn propAllocateWallsFloors(prop: i32) {
    let entry = (lw(prop + 4) << 4) + PWFTAB;
    let walls = lw(entry);
    let mut points = 0;
    if walls != 0 {
        let floors = lw(entry + 8);
        if floors != 0 {
            let mut wall_count = 0;
            let wall_bytes;
            let mut at = walls;
            if lw(at) == 0 {
                wall_bytes = 0;
            } else {
                let mut bytes = 0;
                loop {
                    at += 4;
                    bytes += 4;
                    wall_count += 1;
                    if lw(at) == 0 {
                        break;
                    }
                }
                wall_bytes = bytes;
            }
            let mut floor_count = 0;
            let mut set = lw(floors);
            let floor_bytes;
            if set == 0 {
                floor_bytes = 0;
            } else {
                loop {
                    floor_count += 1;
                    let bytes = floor_count << 2;
                    points += lw(set);
                    set = lw(bytes + floors);
                    if set == 0 {
                        floor_bytes = bytes;
                        break;
                    }
                }
            }
            let size = ((wall_count + (floor_count + 2)) << 2) + points * 12 + ((floor_count << 3) + wall_count * 36);
            let memory = propAllocWallMem(size);
            sw(prop + 484, memory);
            let floor_table = memory + ((wall_count + 1) << 2);
            sw(prop + 480, floor_table);
            let mut records = floor_table + ((floor_count + 1) << 2);
            let floor_pointers = lw(prop + 480);
            let mut i = 0;
            if wall_count > 0 {
                loop {
                    let slot = (i << 2) + memory;
                    i += 1;
                    sw(slot, records);
                    records += 36;
                    if i >= wall_count {
                        break;
                    }
                }
            }
            sw(wall_bytes + memory, 0);
            let mut i = 0;
            if floor_count > 0 {
                let sets = lw(entry + 8);
                loop {
                    let slot = i << 2;
                    i += 1;
                    let set = lw(slot + sets);
                    sw(slot + floor_pointers, records);
                    records += lw(set) * 12 + 8;
                    if i >= floor_count {
                        break;
                    }
                }
            }
            sw(floor_bytes + floor_pointers, 0);
            return;
        }
    }
    sw(prop + 480, 0);
    sw(prop + 484, 0);
}

/// Draw a layer of the front end's logo: layer `layer` (a texture of those at 0x1fbd1a8) as a rectangle centred on the logo's place, 320 by `width` and 56 by `height` times
/// half sizes, in the colour from the four floats (alpha `alpha`, red `red`, green `green`, blue `blue`, each 0 to 1); layer -1 is the two halves of the logo's top, to the left and
/// right of the middle (@ 0x22b340).
pub fn frontfxRenderLayer(layer: i32, width: f32, height: f32, alpha: f32, red: f32, green: f32, blue: f32) {
    let frame = Frame::new(48);
    let sp = frame.sp();
    let full = 127.0_f32;
    let a = fptoui(ps2float::mul(alpha, full));
    let top = lw(g::logoposy);
    let b = fptoui(ps2float::mul(blue, full));
    let gr = fptoui(ps2float::mul(green, full));
    let r = fptoui(ps2float::mul(red, full));
    let colour = a << 24 | b & 255 | ((gr & 255) << 8 | (r & 255) << 16);
    if layer != -1 {
        let y = top as f32;
        let x = lw(g::logoposx) as f32;
        let half_width = ps2float::mul(width, 3.2e+02_f32);
        let half_height = ps2float::mul(height, 56.0_f32);
        let slot = (layer << 2) + 0x1fbd1a8;
        dlSelectTextureKick(lw(slot));
        let left = ps2float::sub(x, half_width) as i32;
        let up = ps2float::sub(y, half_height) as i32;
        let down = ps2float::add(y, half_height) as i32;
        let right = ps2float::add(x, half_width) as i32;
        let texture = lw(slot) * 40 + lw(lw(g::thistexbuf) + 4);
        let size_x = lhu(texture);
        let size_y = lhu(texture + 2) - 1;
        sw(sp, colour);
        dlTextureRectangle(left, up, right, down, 0, 0, (size_x - 1) << 4, size_y << 4);
    } else {
        let y = top as f32;
        let half_height = ps2float::mul(height, 56.0_f32);
        let down = ps2float::add(y, half_height) as i32;
        let up = ps2float::sub(y, half_height) as i32;
        dlSelectTextureKick(lw(g::logotop1num));
        let middle = lw(g::logoposx);
        let half_width = ps2float::mul(width, 256.0_f32);
        let texture = lw(g::logotop1num) * 40 + lw(lw(g::thistexbuf) + 4);
        let left = ps2float::sub(middle as f32, half_width) as i32;
        let size_y = (lhu(texture + 2) - 1) << 4;
        let size_x = (lhu(texture) - 1) << 4;
        sw(sp, colour);
        dlTextureRectangle(left, up, middle, down, 0, 0, size_x, size_y);
        dlSelectTextureKick(lw(g::logotop2num));
        let middle = lw(g::logoposx);
        let texture = lw(g::logotop2num) * 40 + lw(lw(g::thistexbuf) + 4);
        let right = ps2float::add(middle as f32, half_width) as i32;
        let size_x = lhu(texture);
        let size_y = lhu(texture + 2) - 1;
        sw(sp, colour);
        dlTextureRectangle(middle, up, right, down, 0, 0, (size_x - 1) << 4, size_y << 4);
    }
}

/// Draw the colour wheel of the box `wheel` (x, y, width, height as halfwords): a fan of 25 triangles round the middle, each corner on the edge in the colour of its hue (hue
/// going up by 15 degrees), and the cursor image 14 at the place of the light's hue and saturation (@ 0x2512b0).
pub fn draw_colourwheel(wheel: i32) {
    let frame = Frame::new(656);
    let sp = frame.sp();
    let table = g::g_SinTable;
    let mut i = 0;
    let mut at = sp;
    let half_width = ((lhu(wheel + 4) as u32) >> 1) as i32 as f32;
    let half_height = ((lhu(wheel + 6) as u32) >> 1) as i32 as f32;
    let to_radians = lwc1(0x3aca7c);
    let radius_x = ps2float::sub(half_width, 4.0_f32);
    let centre_x = ps2float::add(lhu(wheel) as f32, half_width);
    let one = 1.0_f32;
    let scale = lwc1(0x3aca80);
    let radius_y = ps2float::sub(half_height, 2.0_f32);
    let centre_y = ps2float::add(lhu(wheel + 2) as f32, half_height);
    let mut hue = 0.0_f32;
    loop {
        let angle = ps2float::add(ps2float::mul(ps2float::mul(hue, to_radians).pdiv(1.8e+02_f32), scale), 0.5_f32) as i32;
        let sine = lwc1(((angle & 2047) << 2) + table);
        let cosine = lwc1(((angle + 512 & 2047) << 2) + table);
        if i != 24 {
            sw(at, 0x7f7f7f80);
            sw(at + 4, centre_x as i32);
            sw(at + 8, centre_y as i32);
            let colour = hsv2col(hue, one, one);
            sw(at + 12, colour);
            sw(at + 16, ps2float::add(centre_x, ps2float::mul(cosine, radius_x)) as i32);
            sw(at + 20, ps2float::sub(centre_y, ps2float::mul(sine, radius_y)) as i32);
            at += 24;
        } else {
            let second = ld(sp + 8);
            let third = ld(sp + 16);
            sd(at, ld(sp));
            sd(at + 8, second);
            sd(at + 16, third);
        }
        i += 1;
        hue = ps2float::add(hue, 15.0_f32);
        if i >= 25 {
            break;
        }
    }
    dlDrawTriangleStrip(sp, 50, 0);
    let angle = ps2float::add(ps2float::mul(ps2float::mul(lwc1(g::lighthue), lwc1(0x3aca84)).pdiv(1.8e+02_f32), lwc1(0x3aca88)), 0.5_f32) as i32;
    let sine = lwc1(((angle & 2047) << 2) + table);
    let x = ps2float::add(ps2float::add(lw(g::window) as f32, centre_x), ps2float::mul(ps2float::mul(lwc1(((angle + 512 & 2047) << 2) + table), lwc1(g::lightsaturation)), radius_x));
    let x = dptofp(dpsub(fptodp(x), 0x4000000000000000_i64));
    let y = ps2float::sub(ps2float::add(lw(g::window + 20) as f32, centre_y), ps2float::mul(ps2float::mul(sine, lwc1(g::lightsaturation)), radius_y));
    let y = dptofp(dpsub(fptodp(y), 0x3ff0000000000000_i64));
    guiDrawImage(14, 0x7f7f7f80, x, y);
}
