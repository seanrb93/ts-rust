//! CPU depth for the game's light glows.  The game reads back a 640x224 16-bit copy of the GS z-buffer, but only looks at the few points its glows test, so a frame's triangles are kept (`DepthScene`) and only those points are worked out
//! (`DepthScene::probe`).  A point gets the value it would have in the whole picture (`DepthScene::rasterize`, which what reads the whole buffer uses).

pub const W: usize = 640;
pub const H: usize = 224;

/// The triangles of a frame that write depth, in the order they are drawn.
#[derive(Default)]
pub struct DepthScene {
    pub triangles: Vec<[[f32; 3]; 3]>,
    /// For each run of triangles with the same test: the end of the run, whether the depth test is on, and the test.
    pub runs: Vec<(u32, bool, u8)>,
}

/// Z16 value stored for a GS depth: the game's probes compare against its own projected z (the same units as the vertex z), clamped to 16 bits.
fn z16(z: f32) -> u16 {
    z.max(0.0).min(65535.0) as u16
}

/// What a triangle is drawn into.
trait Target {
    /// Is any pixel wanted in these rows?
    fn wants_rows(&self, first: usize, last: usize) -> bool;
    fn wants_row(&self, y: usize) -> bool;
    /// The wanted pixels from `first` to `last` of a row.
    fn span(&mut self, y: usize, first: usize, last: usize, f: impl FnMut(usize, &mut u16));
}

struct Whole<'a>(&'a mut [u16]);

impl Target for Whole<'_> {
    fn wants_rows(&self, _: usize, _: usize) -> bool {
        true
    }
    fn wants_row(&self, _: usize) -> bool {
        true
    }
    #[inline]
    fn span(&mut self, y: usize, first: usize, last: usize, mut f: impl FnMut(usize, &mut u16)) {
        let row = y * W;
        for x in first..=last {
            f(x, &mut self.0[row + x]);
        }
    }
}

/// Some pixels: sorted by row, then by column.
struct Points<'a> {
    /// Where the pixels of each row start in `pixels`.
    rows: &'a [u32; H + 1],
    pixels: &'a [u32],
    values: &'a mut [u16],
}

impl Target for Points<'_> {
    #[inline]
    fn wants_rows(&self, first: usize, last: usize) -> bool {
        first <= last && self.rows[last + 1] > self.rows[first]
    }
    #[inline]
    fn wants_row(&self, y: usize) -> bool {
        self.rows[y + 1] > self.rows[y]
    }
    #[inline]
    fn span(&mut self, y: usize, first: usize, last: usize, mut f: impl FnMut(usize, &mut u16)) {
        let row = y * W;
        for i in self.rows[y] as usize..self.rows[y + 1] as usize {
            let x = self.pixels[i] as usize - row;
            if x >= first && x <= last {
                f(x, &mut self.values[i]);
            }
        }
    }
}

fn tri<T: Target>(target: &mut T, p: &[[f32; 3]; 3], zte: bool, ztst: u8) {
    let min_y = p.iter().map(|v| v[1]).fold(f32::MAX, f32::min).floor().max(0.0) as usize;
    let max_y = (p.iter().map(|v| v[1]).fold(f32::MIN, f32::max).ceil().min(H as f32 - 1.0)).max(-1.0);
    if max_y < 0.0 || !target.wants_rows(min_y.min(H), max_y as usize) {
        return;
    }
    let min_x = p.iter().map(|v| v[0]).fold(f32::MAX, f32::min).floor().max(0.0) as usize;
    let max_x = (p.iter().map(|v| v[0]).fold(f32::MIN, f32::max).ceil().min(W as f32 - 1.0)).max(-1.0);
    if max_x < 0.0 {
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
        if !target.wants_row(y) {
            continue;
        }
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
        target.span(y, lo.max(min_x as f32) as usize, hi as usize, |x, o| {
            let px = x as f32 + 0.5;
            let a = ev(&w0, px, py);
            let b = ev(&w1, px, py);
            let c = 1.0 - a - b;
            if a < 0.0 || b < 0.0 || c < 0.0 {
                return;
            }
            let z = z16(z2 + a * dz0 + b * dz1);
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
        });
    }
}

impl DepthScene {
    pub fn clear(&mut self) {
        self.triangles.clear();
        self.runs.clear();
    }

    fn draw<T: Target>(&self, target: &mut T) {
        let mut first = 0;
        for &(end, zte, ztst) in &self.runs {
            for t in &self.triangles[first..end as usize] {
                tri(target, t, zte, ztst);
            }
            first = end as usize;
        }
    }

    /// The whole picture.
    pub fn rasterize(&self) -> Vec<u16> {
        let mut buf = vec![0u16; W * H];
        self.draw(&mut Whole(&mut buf));
        buf
    }

    /// The depth at some pixels (a place in the picture, `y * W + x`); `pixels` is sorted and has no pixel twice here, and those beyond the picture are not asked for.
    pub fn probe(&self, pixels: &[u32], values: &mut Vec<u16>) {
        values.clear();
        values.resize(pixels.len(), 0);
        if pixels.is_empty() {
            return;
        }
        let mut rows = [0u32; H + 1];
        for p in pixels {
            rows[*p as usize / W + 1] += 1;
        }
        for y in 0..H {
            rows[y + 1] += rows[y];
        }
        self.draw(&mut Points { rows: &rows, pixels, values });
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

    fn numbers() -> impl FnMut(f32) -> f32 {
        let mut seed = 12345u32;
        move |m: f32| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / (1u32 << 24) as f32 * m
        }
    }

    #[test]
    fn matches_reference() {
        let mut r = numbers();
        let mut bad = 0usize;
        let mut total = 0usize;
        for _ in 0..300 {
            let p = [[r(800.0) - 80.0, r(300.0) - 40.0, r(60000.0)], [r(800.0) - 80.0, r(300.0) - 40.0, r(60000.0)], [r(800.0) - 80.0, r(300.0) - 40.0, r(60000.0)]];
            let mut a = vec![0u16; W * H];
            let mut b = vec![0u16; W * H];
            tri(&mut Whole(&mut a), &p, false, 1);
            reference(&mut b, p);
            total += a.len();
            bad += a.iter().zip(&b).filter(|(x, y)| x != y).count();
        }
        assert!(bad * 1000 < total, "{bad} of {total} pixels differ");
    }

    /// The points have just what the whole picture has there, whatever the tests of the triangles.
    #[test]
    fn points_match_the_whole_picture() {
        let mut r = numbers();
        let mut scene = DepthScene::default();
        for run in 0..40u32 {
            for _ in 0..25 {
                let mut corner = || [r(1200.0) - 280.0, r(600.0) - 180.0, r(80000.0) - 5000.0];
                scene.triangles.push([corner(), corner(), corner()]);
            }
            scene.runs.push((scene.triangles.len() as u32, run % 3 != 0, (run % 4) as u8));
        }
        // (a triangle with a bad corner must come out the same too)
        scene.triangles[7][1][0] = f32::NAN;
        scene.triangles[9][2][1] = f32::INFINITY;
        let whole = scene.rasterize();
        assert!(whole.iter().filter(|z| **z != 0).count() > 1000);
        let mut pixels: Vec<u32> = (0..3000).map(|_| r((W * H) as f32) as u32).collect();
        pixels.extend([0, (W * H - 1) as u32, W as u32 - 1, W as u32]);
        pixels.sort_unstable();
        pixels.dedup();
        let mut values = Vec::new();
        scene.probe(&pixels, &mut values);
        for (p, v) in pixels.iter().zip(&values) {
            assert_eq!(whole[*p as usize], *v, "pixel {p}");
        }
    }
}
