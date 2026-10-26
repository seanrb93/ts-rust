//! A line against a pane of glass, and the lists of rooms each player can see.

use crate::*;

/// Whether the line from `start` along `direction` hits the quad of glass `a`, `b`, `c`, `d` (points of three floats) closer than the square root of `limit2`: the plane is met
/// in front of the start, the point is on the inside of the four edges; the point goes to `hit` and the normal (one unit long) to `normal`, when not 0 (@ 0x259328).
pub fn bgHittestDistGlass(start: i32, direction: i32, a: i32, b: i32, c: i32, d: i32, hit: i32, normal: i32, limit2: f32) -> i32 {
    let bx = lwc1(b);
    let ax = lwc1(a);
    let az = lwc1(a + 8);
    let ay = lwc1(a + 4);
    let abx = ps2float::sub(bx, ax);
    let bz = lwc1(b + 8);
    let cz = lwc1(c + 8);
    let by = lwc1(b + 4);
    let abz = ps2float::sub(bz, az);
    let cy = lwc1(c + 4);
    let acz = ps2float::sub(cz, az);
    let cx = lwc1(c);
    let aby = ps2float::sub(by, ay);
    let acy = ps2float::sub(cy, ay);
    let dz = lwc1(direction + 8);
    let acx = ps2float::sub(cx, ax);
    let dy = lwc1(direction + 4);
    let dx = lwc1(direction);
    let ny = ps2float::sub(ps2float::mul(abz, acx), ps2float::mul(abx, acz));
    let nx = ps2float::sub(ps2float::mul(aby, acz), ps2float::mul(abz, acy));
    let nz = ps2float::sub(ps2float::mul(abx, acy), ps2float::mul(aby, acx));
    let facing = ps2float::add(ps2float::add(ps2float::mul(nx, dx), ps2float::mul(ny, dy)), ps2float::mul(nz, dz));
    if facing == 0.0_f32 {
        return 0;
    }
    let sx = lwc1(start);
    let sy = lwc1(start + 4);
    let sz = lwc1(start + 8);
    let t = (ps2float::sub(ps2float::add(ps2float::add(ps2float::mul(nx, ax), ps2float::mul(ny, ay)), ps2float::mul(nz, az)), ps2float::add(ps2float::add(ps2float::mul(nx, sx), ps2float::mul(ny, sy)), ps2float::mul(nz, sz)))).pdiv(facing);
    if t < 0.0_f32 {
        return 0;
    }
    let hz = ps2float::add(sz, ps2float::mul(t, dz));
    let hx = ps2float::add(sx, ps2float::mul(t, dx));
    let hy = ps2float::add(sy, ps2float::mul(t, dy));
    // the edge from b to c
    let bcx = ps2float::sub(cx, bx);
    let bcz = ps2float::sub(cz, bz);
    let chx = ps2float::sub(cx, hx);
    let chz = ps2float::sub(cz, hz);
    let bcy = ps2float::sub(cy, by);
    let chy = ps2float::sub(cy, hy);
    let bhz = ps2float::sub(bz, hz);
    let bhx = ps2float::sub(bx, hx);
    let bhy = ps2float::sub(by, hy);
    let e1_y = ps2float::sub(ps2float::mul(abz, bhx), ps2float::mul(abx, bhz));
    let e1_z_part = ps2float::sub(ps2float::mul(bcz, chx), ps2float::mul(bcx, chz));
    let e1_x_part = ps2float::sub(ps2float::mul(bcy, chz), ps2float::mul(bcz, chy));
    let e1_x = ps2float::sub(ps2float::mul(aby, bhz), ps2float::mul(abz, bhy));
    let e2_y = ps2float::sub(ps2float::mul(abx, bhy), ps2float::mul(aby, bhx));
    let e2_z = ps2float::sub(ps2float::mul(bcx, chy), ps2float::mul(bcy, chx));
    if ps2float::add(ps2float::add(ps2float::mul(e1_x, e1_x_part), ps2float::mul(e1_y, e1_z_part)), ps2float::mul(e2_y, e2_z)) < 0.0_f32 {
        return 0;
    }
    // the edge from c to d
    let dpz = lwc1(d + 8);
    let dpx = lwc1(d);
    let dpy = lwc1(d + 4);
    let cdz = ps2float::sub(dpz, cz);
    let cdx = ps2float::sub(dpx, cx);
    let dhx = ps2float::sub(dpx, hx);
    let dhz = ps2float::sub(dpz, hz);
    let dhy = ps2float::sub(dpy, hy);
    let cdy = ps2float::sub(dpy, cy);
    let f_z = ps2float::sub(ps2float::mul(cdz, dhx), ps2float::mul(cdx, dhz));
    let f_x = ps2float::sub(ps2float::mul(cdy, dhz), ps2float::mul(cdz, dhy));
    let f_y = ps2float::sub(ps2float::mul(cdx, dhy), ps2float::mul(cdy, dhx));
    if ps2float::add(ps2float::add(ps2float::mul(e1_x, f_x), ps2float::mul(e1_y, f_z)), ps2float::mul(e2_y, f_y)) < 0.0_f32 {
        return 0;
    }
    // the edge from d to a
    let dax = ps2float::sub(ax, dpx);
    let daz = ps2float::sub(az, dpz);
    let ahx = ps2float::sub(ax, hx);
    let ahz = ps2float::sub(az, hz);
    let ahy = ps2float::sub(ay, hy);
    let day = ps2float::sub(ay, dpy);
    let g_z = ps2float::sub(ps2float::mul(daz, ahx), ps2float::mul(dax, ahz));
    let g_x = ps2float::sub(ps2float::mul(day, ahz), ps2float::mul(daz, ahy));
    let g_y = ps2float::sub(ps2float::mul(dax, ahy), ps2float::mul(day, ahx));
    if ps2float::add(ps2float::add(ps2float::mul(e1_x, g_x), ps2float::mul(e1_y, g_z)), ps2float::mul(e2_y, g_y)) < 0.0_f32 {
        return 0;
    }
    let ox = ps2float::sub(hx, sx);
    let oy = ps2float::sub(hy, sy);
    let oz = ps2float::sub(hz, sz);
    if !(limit2 <= ps2float::add(ps2float::add(ps2float::mul(ox, ox), ps2float::mul(oy, oy)), ps2float::mul(oz, oz))) {
        if hit != 0 {
            swc1(hit + 8, hz);
            swc1(hit, hx);
            swc1(hit + 4, hy);
        }
        if normal != 0 {
            swc1(normal, nx);
            swc1(normal + 4, ny);
            swc1(normal + 8, nz);
            let inv = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(nx, nx), ps2float::mul(ny, ny)), ps2float::mul(nz, nz)).abs().psqrt());
            let z = ps2float::mul(lwc1(normal + 8), inv);
            let y = ps2float::mul(lwc1(normal + 4), inv);
            swc1(normal, ps2float::mul(lwc1(normal), inv));
            swc1(normal + 8, z);
            swc1(normal + 4, y);
        }
        return 1;
    }
    0
}

/// Make the list of rooms for each player (up to 212 room numbers from +912 of the player's record, their number at +908): the rooms are put in a table by their level in the
/// player's room data (up to 24; 200 bytes a level at 0x1fc4a78, their counts at 0x1fc4a10); then from the highest level down those with a clip in view (a number at +12 of
/// the room data that is not negative, and a distance at +4 under 640 and under that number); then the neighbours of the player's own room that are not in the list yet (@ 0x256c58).
pub fn bgGetRoomsTick() {
    let frame = Frame::new(144);
    let sp = frame.sp();
    let mut player = 0;
    if lw(g::lvnumplayers) > 0 {
        loop {
            let players = lw(g::players);
            let offset = player << 2;
            let max_at = offset + g::maxlevel;
            sw(player * 1820 + players + 908, 0);
            let mut max_level = lw(max_at);
            if max_level >= 25 {
                max_level = 24;
                sw(max_at, 24);
            }
            let mut i = 0;
            let rooms = lw(g::bgNumRooms);
            let next_player = player + 1;
            if max_level >= 0 {
                loop {
                    let at = (i << 2) + 0x1fc4a10;
                    i += 1;
                    let done = lw(offset + g::maxlevel) < i;
                    sw(at, 0);
                    if done {
                        break;
                    }
                }
            }
            let mut room = 1;
            if room < rooms + 1 {
                loop {
                    let level = lw(room * 20 + lw(offset + g::roomdata));
                    if (level as u32) < 25 {
                        let count_at = (level << 2) + 0x1fc4a10;
                        let count = lw(count_at);
                        sw(count_at, count + 1);
                        sb(count + level * 200 + 0x1fc4a78, room);
                    }
                    room += 1;
                    if room >= rooms + 1 {
                        break;
                    }
                }
            }
            let mut level = lw(offset + g::maxlevel);
            if level >= 0 {
                loop {
                    let count = lw((level << 2) + 0x1fc4a10);
                    let mut k = 0;
                    if count <= 0 {
                        level -= 1;
                    } else {
                        let below = level - 1;
                        loop {
                            let room = lbu(level * 200 + k + 0x1fc4a78);
                            let data = room * 20 + lw(offset + g::roomdata);
                            let clip = lw(data + 12);
                            if clip < 0 {
                                k += 1;
                            } else {
                                let distance = lw(data + 4);
                                if distance < 640 && distance < clip {
                                    let record = player * 1820 + players;
                                    sw(record + (lw(record + 908) << 2) + 912, room);
                                    sw(record + 908, lw(record + 908) + 1);
                                }
                                k += 1;
                            }
                            if k >= count {
                                break;
                            }
                        }
                        level = below;
                    }
                    if level < 0 {
                        break;
                    }
                }
            }
            let base = player * 1820;
            let record = base + players;
            let view = lw(record + 384);
            if lw(view + 12) == 0 {
                sw(view + 12, lw(record + 796));
            }
            let view = lw(record + 384);
            let neighbours = bgGetNeighbours(lw(view + 12), sp, 20);
            let mut n = 0;
            if neighbours > 0 {
                let mut players_at = lw(g::players);
                loop {
                    let mut found = false;
                    let record = base + players_at;
                    if lw(record + 908) > 0 {
                        let wanted = lw(sp + (n << 2));
                        if lw(record + 912) != wanted {
                            let mut k = 0;
                            loop {
                                k += 1;
                                if k >= lw(record + 908) {
                                    break;
                                }
                                if lw(record + (k << 2) + 912) == wanted {
                                    found = true;
                                    break;
                                }
                            }
                        } else {
                            found = true;
                        }
                    }
                    if !found {
                        let record = base + players_at;
                        let count = lw(record + 908);
                        if count < 212 {
                            sw(record + (count << 2) + 912, lw(sp + (n << 2)));
                            sw(record + 908, count + 1);
                        }
                    }
                    n += 1;
                    players_at = lw(g::players);
                    if n >= neighbours {
                        break;
                    }
                }
            }
            player = next_player;
            if player >= lw(g::lvnumplayers) {
                break;
            }
        }
    }
}
