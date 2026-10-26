//! Function: spreading a new decal (a bullet hole, a scorch) over the faces of the level around where it hit.

use crate::*;

/// The bytes of a queued face: the room, the tile in it, the face in that tile (4 bytes each), then the point (3 floats) and the normal (3 floats) the decal has there.
const ENTRY_SIZE: i32 = 36;
/// How many faces one decal may be spread over.
const MAX_ENTRIES: i32 = 10;

/// The size of a number, whatever its sign.
fn magnitude(x: f32) -> f32 {
    if 0.0_f32 <= x { x } else { -x }
}

/// Put a face in the queue at `at`.
fn push_entry(at: i32, room: i32, tile: i32, face: i32, point: [f32; 3], normal: [f32; 3]) {
    sw(at, room);
    sw(at + 4, tile);
    sw(at + 8, face);
    swc1(at + 12, point[0]);
    swc1(at + 16, point[1]);
    swc1(at + 20, point[2]);
    swc1(at + 24, normal[0]);
    swc1(at + 28, normal[1]);
    swc1(at + 32, normal[2]);
}

/// Whether the face (room, tile, face) is in the first `count` entries of the queue already.
fn queue_has(queue: i32, count: i32, room: i32, tile: i32, face: i32) -> bool {
    let mut n = 0;
    while n < count {
        let at = queue + n * ENTRY_SIZE;
        if lw(at) == room && lw(at + 4) == tile && lw(at + 8) == face {
            return true;
        }
        n += 1;
    }
    false
}

/// Whether the edge `e` (the vector from its end to its start) of the decal runs along the edge of a face that goes from `d1` away from the end of the decal edge
/// to `d2` away from its start: the two edges share an end, or one of them lies on the line of the other (the cross product of the two is small enough), or they overlap.
fn shares_edge(e: [f32; 3], d1: [f32; 3], d2: [f32; 3]) -> bool {
    let first_same = d1[0] == 0.0_f32 && d1[1] == 0.0_f32 && d1[2] == 0.0_f32;
    let second_same = d2[0] == 0.0_f32 && d2[1] == 0.0_f32 && d2[2] == 0.0_f32;
    if first_same {
        if second_same {
            return true;
        }
        let x = magnitude(e[1] * d2[2] - e[2] * d2[1]);
        let y = magnitude(e[2] * d2[0] - e[0] * d2[2]);
        let z = magnitude(e[0] * d2[1] - e[1] * d2[0]);
        let limit = lwc1(0x3ad5ec);
        return x <= limit && y <= limit && z <= limit;
    }
    if second_same {
        let x = magnitude(e[1] * d1[2] - e[2] * d1[1]);
        let z = magnitude(e[0] * d1[1] - e[1] * d1[0]);
        let y = magnitude(e[2] * d1[0] - e[0] * d1[2]);
        let limit = lwc1(0x3ad5f0);
        return x <= limit && y <= limit && z <= limit;
    }
    let x = magnitude(d1[1] * d2[2] - d1[2] * d2[1]);
    let z = magnitude(d1[0] * d2[1] - d1[1] * d2[0]);
    let y = magnitude(d1[2] * d2[0] - d1[0] * d2[2]);
    let limit = lwc1(0x3ad5f4);
    if !(x <= limit) || !(y <= limit) || !(z <= limit) {
        return false;
    }
    (d1[0] * d2[0] + d1[1] * d2[1] + d1[2] * d2[2]) <= 0.0_f32
}

/// Tick of the decals (@ 0x2a4bd8): takes the next request from the list of the decals to make (one per call) and, from the face that was hit, goes over the faces of the
/// rooms nearby that the edges of each decal reach, putting each new decal on the queue (at most 10 faces), so that the decal wraps round the corners it crosses.
pub fn decalTick() {
    let frame = Frame::new(1056);
    let sp = frame.sp();
    let queue = sp + 112;
    let out = sp + 48;
    let read_at = lw(g::s_DecalCreateListRead);
    if read_at == lw(g::s_DecalCreateListWrite) {
        return;
    }
    let request = read_at * 44 + 0x1fe9a40;
    let prop = lw(request + 4);
    let bullet = lw(request);
    let first_room = lw(request + 8);
    let first_tile = lw(request + 12);
    let first_face = lw(request + 16);
    sw(g::s_DecalCreateListRead, (read_at + 1).wrapping_rem(15));
    if prop == 0 {
        return;
    }
    let header = lw(prop);
    let owner = lw(prop + 244);
    let rooms = header - lw(header) * 80;
    let prop_room = decalGetRoomFromProp(owner);
    let random = newrnd() & 3_i64;
    let variant = random as i32;
    let spin = if random == 0_i64 || variant == 2 {
        (((newrnd() & 1_i64) as i32) << 1) + 1
    } else {
        ((newrnd() & 1_i64) as i32) << 1
    };
    let point = [lwc1(request + 20), lwc1(request + 24), lwc1(request + 28)];
    let normal = [lwc1(request + 32), lwc1(request + 36), lwc1(request + 40)];
    sw(request + 4, 0);
    decalGetBulletTexScaleCol(bullet, sp + 640, sp + 644, sp + 648);
    let scale = lwc1(0x3ad5e0);
    push_entry(queue, first_room, first_tile, first_face, point, normal);
    let mut read = 0;
    let mut write = 1;
    loop {
        'entry: {
            let entry = queue + read * ENTRY_SIZE;
            let room = rooms + lw(entry) * 80;
            let tile_index = lw(entry + 4);
            let face_index = lw(entry + 8);
            let tile = lw(room + 76) + tile_index * 36;
            if tile == 0 {
                break 'entry;
            }
            let (vertex_list, tile_table) = if lw(tile + 24) == 0 { (lw(room + 8), room + 20) } else { (lw(room + 12), room + 44) };
            let first_vertex = lw(vertex_list + tile_index * 24 + 12);
            let mut flags = -1;
            if lw(prop + 4) != 0 {
                let b = lb(room + 1);
                if b >= 0 {
                    flags = b;
                }
            }
            let verts = lw(tile_table + 4) + first_vertex * 12;
            let face = face_index * 12 + lw(tile + 32);
            let face_vertices = lw(face + 8);
            sw(sp + 8, verts);
            sw(sp, entry + 24);
            sw(sp + 16, face_vertices);
            sw(sp + 24, lbu(face));
            sw(sp + 32, out);
            read += 1;
            let made = decalAddNewDecal(lw(sp + 640), variant, spin, lw(sp + 648), owner, flags, prop_room, entry + 12, lwc1(sp + 644));
            if made < 0 {
                break 'entry;
            }
            if lw(out) < 0 {
                break 'entry;
            }
            let mut edge = 0;
            if !(write < MAX_ENTRIES) {
                break 'entry;
            }
            let room_count = lw(header);
            loop {
                let start = verts + lbu(face_vertices + lw(out + edge * 8)) * 12;
                let end = verts + lbu(face_vertices + lw(out + edge * 8 + 4)) * 12;
                edge += 1;
                let e = [ps2float::sub(lwc1(start), lwc1(end)), ps2float::sub(lwc1(start + 4), lwc1(end + 4)), ps2float::sub(lwc1(start + 8), lwc1(end + 8))];
                let (min_x, max_x) = if lwc1(start) < lwc1(end) { (lwc1(start), lwc1(end)) } else { (lwc1(end), lwc1(start)) };
                let (min_y, max_y) = if lwc1(start + 4) < lwc1(end + 4) { (lwc1(start + 4), lwc1(end + 4)) } else { (lwc1(end + 4), lwc1(start + 4)) };
                let (min_z, max_z) = if lwc1(start + 8) < lwc1(end + 8) { (lwc1(start + 8), lwc1(end + 8)) } else { (lwc1(end + 8), lwc1(start + 8)) };
                let more_edges = edge < 8;
                if room_count > 0 && write < MAX_ENTRIES {
                    let mut scan = 0;
                    loop {
                        let this_room = rooms + scan * 80;
                        let tiles = lw(this_room + 76);
                        let next_scan = scan + 1;
                        'room: {
                            if tiles == 0 {
                                break 'room;
                            }
                            let (index_table, vertex_table) =
                                if lw(tiles + 24) == 0 { (lw(this_room + 8), this_room + 20) } else { (lw(this_room + 12), this_room + 44) };
                            let mask = lhu(this_room + 68);
                            if mask != 0 {
                                if (lhu(prop + 292) & mask) == 0 {
                                    break 'room;
                                }
                            }
                            if index_table == 0 {
                                break 'room;
                            }
                            let mut table = index_table;
                            if lw(table + 20) < 0 {
                                break 'room;
                            }
                            let mut t = tiles;
                            let mut tile_no = 0;
                            loop {
                                if min_x <= lwc1(t + 12)
                                    && lwc1(t) <= max_x
                                    && min_y <= lwc1(t + 16)
                                    && lwc1(t + 4) <= max_y
                                    && min_z <= lwc1(t + 20)
                                    && lwc1(t + 8) <= max_z
                                {
                                    let face_count = lw(t + 28);
                                    let tile_verts = lw(vertex_table + 4) + lw(table + 12) * 12;
                                    if face_count > 0 {
                                        let mut f = 0;
                                        loop {
                                            let rec = f * 12 + lw(t + 32);
                                            let nx = (lh(rec + 2) as f32).pdiv(scale);
                                            let ny = (lh(rec + 4) as f32).pdiv(scale);
                                            let nz = (lh(rec + 6) as f32).pdiv(scale);
                                            let mut across = ps2float::add(ps2float::add(ps2float::mul(nx, e[0]), ps2float::mul(ny, e[1])), ps2float::mul(nz, e[2]));
                                            'face: {
                                                if !(0.0_f32 <= across) {
                                                    across = -across;
                                                }
                                                if !(across <= lwc1(0x3ad5e4)) {
                                                    break 'face;
                                                }
                                                let inverse = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(nx, nx), ps2float::mul(ny, ny)), ps2float::mul(nz, nz)).abs().psqrt());
                                                let n = [ps2float::mul(nx, inverse), ps2float::mul(ny, inverse), ps2float::mul(nz, inverse)];
                                                let facing = ps2float::add(ps2float::add(ps2float::mul(n[0], normal[0]), ps2float::mul(n[1], normal[1])), ps2float::mul(n[2], normal[2]));
                                                if !(lwc1(0x3ad5e8) <= facing) {
                                                    break 'face;
                                                }
                                                if queue_has(queue, write, scan, tile_no, f) {
                                                    break 'face;
                                                }
                                                if lbu(rec) == 0 {
                                                    break 'face;
                                                }
                                                let mut k: i32 = 0;
                                                loop {
                                                    let count = lbu(rec);
                                                    let list = lw(rec + 8);
                                                    if count == 0 {
                                                        trap("break");
                                                    }
                                                    let next = (k + 1).wrapping_rem(count);
                                                    let vk = tile_verts + lbu(list + k) * 12;
                                                    let vn = tile_verts + lbu(list + next) * 12;
                                                    let d1 = [ps2float::sub(lwc1(vk), lwc1(end)), ps2float::sub(lwc1(vk + 4), lwc1(end + 4)), ps2float::sub(lwc1(vk + 8), lwc1(end + 8))];
                                                    let d2 = [ps2float::sub(lwc1(vn), lwc1(start)), ps2float::sub(lwc1(vn + 4), lwc1(start + 4)), ps2float::sub(lwc1(vn + 8), lwc1(start + 8))];
                                                    if shares_edge(e, d1, d2) {
                                                        let at = queue + write * ENTRY_SIZE;
                                                        if lwc1(0x3ad5f8) <= facing {
                                                            push_entry(at, scan, tile_no, f, point, n);
                                                            write += 1;
                                                            break 'face;
                                                        }
                                                        if lwc1(0x3ad5fc) <= facing {
                                                            let dx = ps2float::sub(point[0], lwc1(vk));
                                                            let dy = ps2float::sub(point[1], lwc1(vk + 4));
                                                            let dz = ps2float::sub(point[2], lwc1(vk + 8));
                                                            write += 1;
                                                            let along = -ps2float::add(ps2float::add(ps2float::mul(dx, n[0]), ps2float::mul(dy, n[1])), ps2float::mul(dz, n[2]));
                                                            let projected = [ps2float::add(point[0], ps2float::mul(along, n[0])), ps2float::add(point[1], ps2float::mul(along, n[1])), ps2float::add(point[2], ps2float::mul(along, n[2]))];
                                                            push_entry(at, scan, tile_no, f, projected, n);
                                                            break 'face;
                                                        }
                                                    }
                                                    k += 1;
                                                    if !(k < lbu(rec)) {
                                                        break 'face;
                                                    }
                                                }
                                            }
                                            f += 1;
                                            if f >= face_count {
                                                break;
                                            }
                                            if !(write < MAX_ENTRIES) {
                                                break;
                                            }
                                        }
                                    }
                                }
                                t += 36;
                                tile_no += 1;
                                table += 24;
                                if lw(table + 20) < 0 {
                                    break;
                                }
                                if !(write < MAX_ENTRIES) {
                                    break;
                                }
                            }
                        }
                        scan = next_scan;
                        if scan >= room_count {
                            break;
                        }
                        if !(write < MAX_ENTRIES) {
                            break;
                        }
                    }
                }
                if !more_edges {
                    break;
                }
                if lw(out + edge * 8) < 0 {
                    break;
                }
                if !(write < MAX_ENTRIES) {
                    break;
                }
            }
        }
        if !(read < write) {
            return;
        }
    }
}
