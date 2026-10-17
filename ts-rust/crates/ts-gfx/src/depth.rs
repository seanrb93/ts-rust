//! CPU depth-buffer reconstruction.  The game reads back a 640x224 16-bit copy of the GS z-buffer to test whether
//! light glows are occluded.  Only the values at a few probe points matter, so a simple software rasteriser over the
//! frame's draw list reproduces them without reading the GPU depth buffer back.

use crate::scene::{Frame, Topology};

pub const W: usize = 640;
pub const H: usize = 224;

/// Z16 value stored for a GS depth: the game's probes compare against its own projected z (the same units as the
/// vertex z), clamped to 16 bits.
fn z16(z: f32) -> u16 {
    z.max(0.0).min(65535.0) as u16
}

pub fn rasterize(frame: &Frame) -> Vec<u16> {
    let mut buf = vec![0u16; W * H];
    for d in &frame.draws {
        if d.state.zmsk || d.topology != Topology::Triangles {
            continue;
        }
        let ztst = ((d.state.test >> 17) & 3) as u8;
        let zte = (d.state.test >> 16) & 1 != 0;
        let verts = &frame.vertices[d.first as usize..(d.first + d.count) as usize];
        for t in verts.chunks_exact(3) {
            tri(&mut buf, [t[0].pos, t[1].pos, t[2].pos], zte, ztst);
        }
    }
    buf
}

fn tri(buf: &mut [u16], p: [[f32; 3]; 3], zte: bool, ztst: u8) {
    let min_x = p.iter().map(|v| v[0]).fold(f32::MAX, f32::min).floor().max(0.0) as usize;
    let max_x = (p.iter().map(|v| v[0]).fold(f32::MIN, f32::max).ceil().min(W as f32 - 1.0)).max(-1.0);
    let min_y = p.iter().map(|v| v[1]).fold(f32::MAX, f32::min).floor().max(0.0) as usize;
    let max_y = (p.iter().map(|v| v[1]).fold(f32::MIN, f32::max).ceil().min(H as f32 - 1.0)).max(-1.0);
    if max_x < 0.0 || max_y < 0.0 {
        return;
    }
    let area = (p[1][0] - p[0][0]) * (p[2][1] - p[0][1]) - (p[2][0] - p[0][0]) * (p[1][1] - p[0][1]);
    if area.abs() < 1e-6 {
        return;
    }
    // barycentric weights and depth are affine in (x, y): w = a*x + b*y + c, evaluated without per-pixel divisions
    let inv = 1.0 / area;
    let w0 = [(p[2][1] - p[1][1]) * -inv, (p[1][0] - p[2][0]) * -inv, (p[1][0] * p[2][1] - p[2][0] * p[1][1]) * inv];
    let w1 = [(p[0][1] - p[2][1]) * -inv, (p[2][0] - p[0][0]) * -inv, (p[2][0] * p[0][1] - p[0][0] * p[2][1]) * inv];
    let ev = |w: &[f32; 3], x: f32, y: f32| w[0] * x + w[1] * y + w[2];
    let (z0, z1, z2) = (p[0][2], p[1][2], p[2][2]);
    // z = z2 + w0*(z0-z2) + w1*(z1-z2)
    let (dz0, dz1) = (z0 - z2, z1 - z2);
    for y in min_y..=max_y as usize {
        let py = y as f32 + 0.5;
        // x interval where w0 >= 0, w1 >= 0 and w0 + w1 <= 1 (each is linear in x)
        let (mut lo, mut hi) = (min_x as f32, max_x);
        let mut empty = false;
        let k0 = w0[1] * py + w0[2];
        let k1 = w1[1] * py + w1[2];
        for (a, k) in [(w0[0], k0), (w1[0], k1), (-(w0[0] + w1[0]), 1.0 - k0 - k1)] {
            // require a * px + k >= 0 with px = x + 0.5
            if a.abs() < 1e-12 {
                if k < 0.0 {
                    empty = true;
                }
            } else {
                let xr = -k / a - 0.5; // x where the value crosses zero
                if a > 0.0 {
                    lo = lo.max((xr - 1.0).floor());
                } else {
                    hi = hi.min((xr + 1.0).ceil());
                }
            }
        }
        if empty || lo > hi {
            continue;
        }
        let row = y * W;
        for x in (lo.max(min_x as f32) as usize)..=(hi as usize) {
            let px = x as f32 + 0.5;
            let a = ev(&w0, px, py);
            let b = ev(&w1, px, py);
            let c = 1.0 - a - b;
            if a < 0.0 || b < 0.0 || c < 0.0 {
                continue;
            }
            let z = z16(z2 + a * dz0 + b * dz1);
            let o = &mut buf[row + x];
            let pass = !zte
                || match ztst {
                    0 => false,
                    1 => true,
                    2 => z >= *o,
                    _ => z > *o,
                };
            if pass {
                *o = z;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(buf: &mut [u16], p: [[f32; 3]; 3]) {
        let area = (p[1][0] - p[0][0]) * (p[2][1] - p[0][1]) - (p[2][0] - p[0][0]) * (p[1][1] - p[0][1]);
        if area.abs() < 1e-6 {
            return;
        }
        for y in 0..H {
            for x in 0..W {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let w0 = ((p[1][0] - px) * (p[2][1] - py) - (p[2][0] - px) * (p[1][1] - py)) / area;
                let w1 = ((p[2][0] - px) * (p[0][1] - py) - (p[0][0] - px) * (p[2][1] - py)) / area;
                let w2 = 1.0 - w0 - w1;
                if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                    continue;
                }
                buf[y * W + x] = z16(w0 * p[0][2] + w1 * p[1][2] + w2 * p[2][2]);
            }
        }
    }

    #[test]
    fn matches_reference() {
        let mut seed = 12345u32;
        let mut r = |m: f32| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / (1u32 << 24) as f32 * m
        };
        let mut bad = 0usize;
        let mut total = 0usize;
        for _ in 0..300 {
            let p = [[r(800.0) - 80.0, r(300.0) - 40.0, r(60000.0)], [r(800.0) - 80.0, r(300.0) - 40.0, r(60000.0)], [r(800.0) - 80.0, r(300.0) - 40.0, r(60000.0)]];
            let mut a = vec![0u16; W * H];
            let mut b = vec![0u16; W * H];
            tri(&mut a, p, false, 1);
            reference(&mut b, p);
            total += a.len();
            bad += a.iter().zip(&b).filter(|(x, y)| x != y).count();
        }
        assert!(bad * 1000 < total, "{bad} of {total} pixels differ");
    }
}
