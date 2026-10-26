//! Merging the polygons of two models that share an edge.
//!
//! A polygon is a record of 12 bytes: the number of vertices (0 for a polygon that is not there any more) at +0, its normal as 3 halfwords at +2 (divided by the scale at 0x3acb48), and at +8 the pointer to the list of its vertex numbers, one byte for each (they go into the array of vertex positions of 12 bytes). A model has its count of polygons at +28 and the pointer to them at +32.

use crate::*;

/// The vertex after `index` in a polygon of `count` vertices.
fn next_vertex(index: i32, count: i32) -> i32 {
    let next = index + 1;
    if next >= count { 0 } else { next }
}

/// The vertex before `index` in a polygon of `count` vertices.
fn previous_vertex(index: i32, count: i32) -> i32 {
    let before = index - 1;
    if before < 0 { count - 1 } else { before }
}

/// The address of the position of vertex `index` of the polygon `poly`.
fn vertex_position(poly: i32, index: i32, positions: i32) -> i32 {
    lbu(lw(poly + 8) + index) * 12 + positions
}

/// Whether the two positions are the same point.
fn same_point(a: i32, b: i32) -> bool {
    lwc1(a) == lwc1(b) && lwc1(a + 4) == lwc1(b + 4) && lwc1(a + 8) == lwc1(b + 8)
}

/// The normal of the polygon `poly` as three floats.
fn polygon_normal(poly: i32, scale: f32) -> [f32; 3] {
    [(lh(poly + 2) as f32).pdiv(scale), (lh(poly + 4) as f32).pdiv(scale), (lh(poly + 6) as f32).pdiv(scale)]
}

/// The unit vector of the direction from the point `from` to the point `to`.
fn direction(from: i32, to: i32) -> [f32; 3] {
    let x = ps2float::sub(lwc1(to), lwc1(from));
    let y = ps2float::sub(lwc1(to + 4), lwc1(from + 4));
    let z = ps2float::sub(lwc1(to + 8), lwc1(from + 8));
    let length = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(x, x), ps2float::mul(y, y)), ps2float::mul(z, z)).abs().psqrt());
    [ps2float::mul(x, length), ps2float::mul(y, length), ps2float::mul(z, length)]
}

/// Look at the corner at the point `corner` of the polygon that would be made, between the edges to `before` and `after`: gives whether the edges go in opposite directions (a vertex that is on a straight line, whose cosine is at most the constant at 0x3acb50) and whether the corner is not a concave one for the polygon with the normal `normal`.
fn corner_is_good(corner: i32, after: i32, before: i32, normal: [f32; 3]) -> (bool, bool) {
    let toward_after = direction(corner, after);
    let toward_before = direction(corner, before);
    let straight_limit = lwc1(0x3acb50);
    let cosine = toward_after[0] * toward_before[0] + toward_after[1] * toward_before[1] + toward_after[2] * toward_before[2];
    let straight = cosine <= straight_limit;
    let cross_x = toward_after[1] * toward_before[2] - toward_after[2] * toward_before[1];
    let cross_y = toward_after[2] * toward_before[0] - toward_after[0] * toward_before[2];
    let cross_z = toward_after[0] * toward_before[1] - toward_after[1] * toward_before[0];
    let length = (1.0_f32).pdiv((cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).abs().psqrt());
    let unit_x = ps2float::mul(cross_x, length);
    let unit_y = ps2float::mul(cross_y, length);
    let unit_z = ps2float::mul(cross_z, length);
    let turn = ps2float::add(ps2float::add(ps2float::mul(normal[0], unit_x), ps2float::mul(normal[1], unit_y)), ps2float::mul(normal[2], unit_z));
    (straight, 0.0_f32 <= turn)
}

/// The vertices of the polygon `p` and of `q` that are at the same positions, found as an edge: `(a, b, c, d)`, where the edge from the vertex `a` to `c` of `p` is the edge from `b` to `d` of `q` the other way. `None` if they share no edge.
fn shared_edge(p: i32, q: i32, positions: i32) -> Option<(i32, i32, i32, i32)> {
    let count_p = lbu(p);
    let mut first_p = -1;
    let mut first_q = -1;
    let mut second_p = -1;
    let mut second_q = -1;
    let mut vi = 0;
    while vi < count_p {
        // Once the first shared vertex is known, only its neighbours can be the second.
        let try_this = first_p < 0 || vi == next_vertex(first_p, count_p) || vi == previous_vertex(first_p, count_p);
        if try_this {
            let count_q = lbu(q);
            let position = vertex_position(p, vi, positions);
            let known = first_p;
            let mut vj = 0;
            let mut matched = false;
            if count_q > 0 {
                loop {
                    let try_that = known < 0 || vj == next_vertex(first_q, count_q) || vj == previous_vertex(first_q, count_q);
                    if try_that && same_point(position, vertex_position(q, vj, positions)) {
                        matched = true;
                        break;
                    }
                    vj += 1;
                    if vj >= count_q {
                        break;
                    }
                }
            }
            if matched {
                if known >= 0 {
                    second_q = vj;
                    second_p = vi;
                } else {
                    first_q = vj;
                    first_p = vi;
                }
            }
        }
        vi += 1;
        if second_p >= 0 {
            break;
        }
    }
    if first_p < 0 || second_p < 0 {
        return None;
    }
    // The edge has to go the other way in the two polygons: swap the ends if it is the other way round.
    let count_q = lbu(q);
    let p_forward = second_p == next_vertex(first_p, count_p);
    let q_backward = second_q == previous_vertex(first_q, count_q);
    if p_forward {
        if q_backward {
            return Some((first_p, first_q, second_p, second_q));
        }
        return None;
    }
    if !q_backward {
        return Some((second_p, second_q, first_p, first_q));
    }
    None
}

/// Try to make one polygon of `p` and `q` (of the models with the vertex positions at `positions`) for the output record `out` (its count of polygons at +24 and of vertex numbers at +28, written to the polygons of `first`). Gives whether it was done, which clears both of the polygons.
fn merge_pair(out: i32, first: i32, p: i32, q: i32, positions: i32, scale: f32) -> bool {
    let normal_p = polygon_normal(p, scale);
    let normal_q = polygon_normal(q, scale);
    let same_direction = lwc1(0x3acb4c);
    if !(same_direction <= normal_p[0] * normal_q[0] + normal_p[1] * normal_q[1] + normal_p[2] * normal_q[2]) {
        return false;
    }
    let Some((a, b, c, d)) = shared_edge(p, q, positions) else {
        return false;
    };
    // The corner at the start of the edge (a of p, b of q) and the one at its end (c of p, d of q).
    let count_p = lbu(p);
    let count_q = lbu(q);
    let before_a = if previous_vertex(a, count_p) != c { previous_vertex(a, count_p) } else { next_vertex(a, count_p) };
    let after_b = if previous_vertex(b, count_q) != d { previous_vertex(b, count_q) } else { next_vertex(b, count_q) };
    let after_c = if previous_vertex(c, count_p) != a { previous_vertex(c, count_p) } else { next_vertex(c, count_p) };
    let before_d = if previous_vertex(d, count_q) != b { previous_vertex(d, count_q) } else { next_vertex(d, count_q) };
    let corner_a = vertex_position(p, a, positions);
    let end_c = vertex_position(q, d, positions);
    let start_straight;
    {
        let (straight, convex) = corner_is_good(
            corner_a,
            vertex_position(q, after_b, positions),
            vertex_position(p, before_a, positions),
            normal_p,
        );
        if !convex && !straight {
            return false;
        }
        start_straight = straight;
    }
    let end_straight;
    {
        let (straight, convex) = corner_is_good(
            end_c,
            vertex_position(p, after_c, positions),
            vertex_position(q, before_d, positions),
            normal_p,
        );
        if !convex && !straight {
            return false;
        }
        end_straight = straight;
    }

    // The new polygon: the vertices of `p` from `c` round to just before `a`, and then those of `q` from `b` round to just before `d`; a vertex on a straight line is left out.
    let polygons = lw(first + 32);
    let number = lw(out + 24);
    let record = number * 12 + polygons;
    sw(record + 8, lw(polygons + 8) + lw(out + 28));
    let mut written = 0;
    if c != a {
        let mut vi = c;
        loop {
            if !(end_straight && vi == c) {
                sb(lw(record + 8) + written, lbu(lw(p + 8) + vi));
                written += 1;
            }
            vi = next_vertex(vi, lbu(p));
            if vi == a {
                break;
            }
        }
    }
    if b != d {
        let mut vj = b;
        loop {
            if !(start_straight && vj == b) {
                sb(lw(record + 8) + written, lbu(lw(q + 8) + vj));
                written += 1;
            }
            vj = next_vertex(vj, lbu(q));
            if vj == d {
                break;
            }
        }
    }
    sh(record + 2, lhu(p + 2));
    sh(record + 4, lhu(p + 4));
    sh(record + 6, lhu(p + 6));
    sb(record, written);
    sw(out + 28, lw(out + 28) + written);
    sb(p, 0);
    sb(q, 0);
    sw(out + 24, lw(out + 24) + 1);
    true
}

/// Merge the polygons of the models `second` (`a2`) and `third` (`a3`) into the list of polygons of `first` (`a1`), through the output record `out` (`a0`: the number of polygons written at +24 and of indices at +28): a polygon of `a2` and one of `a3` with almost the same normal (their dot product at least the game's constant) that share an edge (the same vertex positions, in the opposite order, from the vertices at `a4`), and whose join stays convex, are made one polygon (both are then cleared), and the others are copied; returns 1 if any were joined.
pub fn obMergePolys(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32) -> i32 {
    sw(a0 + 24, 0);
    sw(a0 + 28, 0);
    let mut joined = 0;
    let mut i = 0;
    if lw(a2 + 28) > 0 {
        let scale = lwc1(0x3acb48);
        loop {
            if lbu(i * 12 + lw(a2 + 32)) != 0 {
                let mut merged = false;
                let count_third = lw(a3 + 28);
                let mut j = 0;
                while j < count_third {
                    let q = j * 12 + lw(a3 + 32);
                    let p = lw(a2 + 32) + i * 12;
                    if lbu(q) != 0 && p != q && merge_pair(a0, a1, p, q, a4, scale) {
                        merged = true;
                        joined = 1;
                        break;
                    }
                    j += 1;
                }
                if !merged {
                    // The polygon is copied as it is.
                    let polygons = lw(a1 + 32);
                    let number = lw(a0 + 24);
                    let source = lw(a2 + 32) + i * 12;
                    sw(number * 12 + polygons + 8, lw(polygons + 8) + lw(a0 + 28));
                    let mut k = 0;
                    if lbu(source) != 0 {
                        loop {
                            let vertex = lbu(lw(i * 12 + lw(a2 + 32) + 8) + k);
                            sb(lw(number * 12 + lw(a1 + 32) + 8) + k, vertex);
                            k += 1;
                            if k >= lbu(i * 12 + lw(a2 + 32)) {
                                break;
                            }
                        }
                    }
                    let record = number * 12 + lw(a1 + 32);
                    let source = i * 12 + lw(a2 + 32);
                    sh(record + 2, lhu(source + 2));
                    sh(record + 4, lhu(source + 4));
                    sh(record + 6, lhu(source + 6));
                    sb(record, lbu(source));
                    let written = lw(a0 + 28);
                    sw(a0 + 24, lw(a0 + 24) + 1);
                    sw(a0 + 28, written + lbu(i * 12 + lw(a2 + 32)));
                }
            }
            i += 1;
            if i >= lw(a2 + 28) {
                break;
            }
        }
    }
    sw(a1 + 28, lw(a0 + 24));
    joined
}
