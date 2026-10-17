//! Frame interpolation by motion vectors.  The game produces a new display list 60 times a second.  To show more
//! pictures per second the renderer draws each game frame once, together with a buffer that tells for every pixel how far
//! the surface there has moved on screen since the previous game frame, and then shows in-between pictures by moving
//! the finished image back along those vectors (see `Renderer::present_scene`).  Nothing is re-triangulated, so no
//! cracks open between neighbouring polygons, and the expensive part (drawing the scene) happens once per game frame.
//!
//! The vectors come from pairing the vertices of the current frame with those of the previous one: mesh batches (VU1
//! kicks) are paired in order, and a batch is used only when it has the same size, program and state in both frames and
//! its vertices did not jump (a cut or a teleport).  Everything unpaired gets no vector and stays where it is.

use ts_gfx::scene::*;

/// Largest on-screen movement (in 640x224 field pixels) per game frame that still counts as the same object moving.
const MAX_STEP: f32 = 96.0;
/// How far ahead in the previous list a matching batch is searched for (objects appearing / disappearing shift the lists).
const LOOKAHEAD: usize = 24;

/// How many pictures were prepared / how many of them had moving world geometry (diagnostics).
pub static STAT_TOTAL: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
pub static STAT_BLENDED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
pub static FAILS: [std::sync::atomic::AtomicU32; 3] = [std::sync::atomic::AtomicU32::new(0), std::sync::atomic::AtomicU32::new(0), std::sync::atomic::AtomicU32::new(0)];
/// vertices of opaque world meshes seen / given a motion vector (diagnostics)
pub static VERTS_TOTAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static VERTS_PAIRED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// use the camera-motion fallback for unpaired meshes (switched off by the diagnostics to compare)
pub static FIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
pub static VERTS_MATCHED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static NO_BACKDROP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn same(a: &Draw, b: &Draw) -> bool {
    // draws that read the frame buffer (tbp0 0 / 1280) are image effects
    if matches!(a.state.tex, Some(k) if k.tbp0 == 0 || k.tbp0 == 1280) {
        return false;
    }
    a.count == b.count && a.topology == b.topology && a.prog == b.prog && a.world == b.world && a.state == b.state
}

/// The current frame with the on-screen movement of its vertices since `prev` attached.  (`_t` is unused: the in-between
/// position is chosen when the picture is shown.)
pub fn interpolate(prev: &Frame, cur: &Frame, _t: f32) -> Frame {
    let mut out = cur.clone();
    // only the running game world moves smoothly: menus use effects (screen feedback, animated textures) that must not be
    // touched, and a frame whose neighbour was a menu has nothing to compare with
    if !cur.smooth || !prev.smooth {
        return out;
    }
    let mut pj = 0usize;
    let mut unpaired: Vec<(u32, u32, usize)> = Vec::new();
    let mut samples: Vec<[f32; 4]> = Vec::new();
    let mut world_total = 0u32;
    let mut world_paired = 0u32;
    for (si, d) in cur.segs.iter().enumerate() {
        let two_d = cur.vertices[d.first as usize].pos[2] >= 4.0e9;
        let world = !d.state.zmsk && d.topology == Topology::Triangles && !two_d;
        if two_d {
            continue;
        }
        let end = (pj + LOOKAHEAD).min(prev.segs.len());
        let found = (pj..end).find(|&j| same(&prev.segs[j], d));
        let mut ok = false;
        if let Some(j) = found {
            pj = j + 1;
            let p = &prev.segs[j];
            let a = &prev.vertices[p.first as usize..(p.first + p.count) as usize];
            let b = &cur.vertices[d.first as usize..(d.first + d.count) as usize];
            ok = !a.iter().zip(b).any(|(x, y)| (x.pos[0] - y.pos[0]).abs() > MAX_STEP || (x.pos[1] - y.pos[1]).abs() > MAX_STEP);
            if ok {
                let o = &mut out.vertices[d.first as usize..(d.first + d.count) as usize];
                for ((x, y), r) in a.iter().zip(b).zip(o.iter_mut()) {
                    r.vel = [y.pos[0] - x.pos[0], y.pos[1] - x.pos[1]];
                }
                if world {
                    let stride = (b.len() / 24).max(1);
                    for (x, y) in a.iter().zip(b).step_by(stride) {
                        samples.push([y.pos[0], y.pos[1], y.pos[0] - x.pos[0], y.pos[1] - x.pos[1]]);
                    }
                }
            }
        }
        if world && !ok {
            unpaired.push((d.first, d.count, si));
        }
        if world {
            world_total += 1;
            VERTS_TOTAL.fetch_add(d.count as u64, std::sync::atomic::Ordering::Relaxed);
            if ok {
                world_paired += 1;
                VERTS_PAIRED.fetch_add(d.count as u64, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }
    // Meshes that could not be paired as a whole (clipped differently from one frame to the next) are matched vertex by
    // vertex: a vertex keeps its texture coordinates from frame to frame, so the same texture position in the previous
    // frame's meshes is the same point of the world.  Vertices without a partner take the typical motion of their mesh,
    // and meshes without any partner the camera motion fitted to the paired ones.
    if !unpaired.is_empty() {
        let fit = if FIT.load(std::sync::atomic::Ordering::Relaxed) && samples.len() >= 60 { fit_motion(&samples) } else { None };
        let index = UvIndex::build(prev);
        let mut tmp: Vec<[f32; 2]> = Vec::new();
        for (first, count, si) in unpaired {
            let seg = &cur.segs[si];
            let verts = &cur.vertices[first as usize..(first + count) as usize];
            let mut found: Vec<Option<[f32; 2]>> = Vec::with_capacity(verts.len());
            tmp.clear();
            for v in verts {
                let m = index.find(seg, v);
                if let Some(m) = m {
                    tmp.push(m);
                }
                found.push(m);
            }
            VERTS_MATCHED.fetch_add(tmp.len() as u64, std::sync::atomic::Ordering::Relaxed);
            let typical = if tmp.len() >= 3 {
                let mut xs: Vec<f32> = tmp.iter().map(|m| m[0]).collect();
                let mut ys: Vec<f32> = tmp.iter().map(|m| m[1]).collect();
                xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                Some([xs[xs.len() / 2], ys[ys.len() / 2]])
            } else {
                None
            };
            let o = &mut out.vertices[first as usize..(first + count) as usize];
            for (r, m) in o.iter_mut().zip(found) {
                let m = m.or(typical).or_else(|| fit.as_ref().map(|f| f.at(r.pos[0], r.pos[1])));
                if let Some(m) = m {
                    r.vel = [m[0].clamp(-MAX_STEP, MAX_STEP), m[1].clamp(-MAX_STEP, MAX_STEP)];
                }
            }
        }
    }
    FAILS[0].fetch_add(world_total - world_paired, std::sync::atomic::Ordering::Relaxed);
    FAILS[1].fetch_add(world_total, std::sync::atomic::Ordering::Relaxed);
    STAT_TOTAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if world_paired > 0 {
        STAT_BLENDED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    out
}

/// Screen motion as a quadratic function of the position: v = c0 + c1 dx + c2 dy + c3 dx^2 + c4 dx dy + c5 dy^2.
struct Motion {
    cx: [f32; 6],
    cy: [f32; 6],
}

impl Motion {
    fn at(&self, x: f32, y: f32) -> [f32; 2] {
        let (dx, dy) = ((x - 320.0) / 320.0, (y - 112.0) / 112.0);
        let b = [1.0, dx, dy, dx * dx, dx * dy, dy * dy];
        let f = |c: &[f32; 6]| c.iter().zip(&b).map(|(c, b)| c * b).sum::<f32>();
        [f(&self.cx), f(&self.cy)]
    }
}

fn basis(x: f32, y: f32) -> [f64; 6] {
    let (dx, dy) = (((x - 320.0) / 320.0) as f64, ((y - 112.0) / 112.0) as f64);
    [1.0, dx, dy, dx * dx, dx * dy, dy * dy]
}

/// Robust (iteratively reweighted) least-squares fit of the motion field to (x, y, vx, vy) samples.
fn fit_motion(samples: &[[f32; 4]]) -> Option<Motion> {
    let mut w = vec![1.0f64; samples.len()];
    let mut out = Motion { cx: [0.0; 6], cy: [0.0; 6] };
    for iter in 0..6 {
        let mut ata = [[0.0f64; 6]; 6];
        let mut bx = [0.0f64; 6];
        let mut by = [0.0f64; 6];
        for (s, &wt) in samples.iter().zip(&w) {
            let b = basis(s[0], s[1]);
            for i in 0..6 {
                for j in 0..6 {
                    ata[i][j] += wt * b[i] * b[j];
                }
                bx[i] += wt * b[i] * s[2] as f64;
                by[i] += wt * b[i] * s[3] as f64;
            }
        }
        for (i, row) in ata.iter_mut().enumerate() {
            row[i] += 1e-3 * samples.len() as f64 * if i < 3 { 0.0 } else { 1.0 } + 1e-6;
        }
        let cx = solve6(ata, bx)?;
        let cy = solve6(ata, by)?;
        for i in 0..6 {
            out.cx[i] = cx[i] as f32;
            out.cy[i] = cy[i] as f32;
        }
        // reweight: samples far from the fit (moving objects, mismatched meshes) count little
        let scale = if iter < 2 { 4.0 } else { 1.5 };
        for (k, s) in samples.iter().enumerate() {
            let m = out.at(s[0], s[1]);
            let r = ((m[0] - s[2]).powi(2) + (m[1] - s[3]).powi(2)).sqrt() as f64;
            w[k] = 1.0 / (1.0 + (r / scale).powi(2));
        }
    }
    Some(out)
}

fn solve6(mut a: [[f64; 6]; 6], mut b: [f64; 6]) -> Option<[f64; 6]> {
    for c in 0..6 {
        let p = (c..6).max_by(|&i, &j| a[i][c].abs().partial_cmp(&a[j][c].abs()).unwrap_or(std::cmp::Ordering::Equal))?;
        if a[p][c].abs() < 1e-12 {
            return None;
        }
        a.swap(c, p);
        b.swap(c, p);
        for r in c + 1..6 {
            let f = a[r][c] / a[c][c];
            for k in c..6 {
                a[r][k] -= f * a[c][k];
            }
            b[r] -= f * b[c];
        }
    }
    let mut x = [0.0f64; 6];
    for r in (0..6).rev() {
        let mut v = b[r];
        for k in r + 1..6 {
            v -= a[r][k] * x[k];
        }
        x[r] = v / a[r][r];
    }
    Some(x)
}

/// The vertices of the previous frame's opaque world meshes, looked up by texture and texture position.
struct UvIndex {
    map: std::collections::HashMap<(u32, u32, u32, bool, i32, i32), Vec<[f32; 2]>>,
}

const UV_CELL: f32 = 1.0 / 32.0;

fn uv_of(d: &Draw, v: &Vertex) -> Option<(f32, f32)> {
    if d.state.use_stq {
        let q = v.st[2];
        if q.abs() < 1e-12 {
            return None;
        }
        // normalised texture coordinates
        Some((v.st[0] / q, v.st[1] / q))
    } else {
        // texel positions: scale to the same range as the normalised ones
        let k = d.state.tex.map(|t| (1u32 << t.tw.min(12)) as f32).unwrap_or(1.0);
        let kh = d.state.tex.map(|t| (1u32 << t.th.min(12)) as f32).unwrap_or(1.0);
        Some((v.st[0] / k, v.st[1] / kh))
    }
}

impl UvIndex {
    fn build(prev: &Frame) -> UvIndex {
        let mut map: std::collections::HashMap<_, Vec<[f32; 2]>> = std::collections::HashMap::new();
        for d in &prev.segs {
            let Some(first) = prev.vertices.get(d.first as usize) else { continue };
            if first.pos[2] >= 4.0e9 || d.state.zmsk || d.topology != Topology::Triangles {
                continue;
            }
            let Some(t) = d.state.tex else { continue };
            for v in &prev.vertices[d.first as usize..(d.first + d.count) as usize] {
                let Some((u, w)) = uv_of(d, v) else { continue };
                map.entry((t.tbp0, t.tw, t.th, d.state.use_stq, (u / UV_CELL).floor() as i32, (w / UV_CELL).floor() as i32)).or_default().push([v.pos[0], v.pos[1]]);
            }
        }
        UvIndex { map }
    }

    /// Movement of a vertex since the previous frame, from the nearest previous vertex with the same texture position.
    fn find(&self, d: &Draw, v: &Vertex) -> Option<[f32; 2]> {
        let t = d.state.tex?;
        let (u, w) = uv_of(d, v)?;
        let (cu, cw) = ((u / UV_CELL).floor() as i32, (w / UV_CELL).floor() as i32);
        let mut best: Option<([f32; 2], f32)> = None;
        for du in -1..=1 {
            for dw in -1..=1 {
                if let Some(list) = self.map.get(&(t.tbp0, t.tw, t.th, d.state.use_stq, cu + du, cw + dw)) {
                    for p in list {
                        let (dx, dy) = (v.pos[0] - p[0], v.pos[1] - p[1]);
                        let dist = dx * dx + dy * dy;
                        if dx.abs() <= MAX_STEP && dy.abs() <= MAX_STEP && best.map_or(true, |b| dist < b.1) {
                            best = Some(([dx, dy], dist));
                        }
                    }
                }
            }
        }
        best.map(|b| b.0)
    }
}

/// The frame shown `t` (0..1) of the way from `prev` to `cur`: the vertices (already projected to the screen) of draws that
/// both frames contain are blended; draws that exist in only one of them, or that jumped (a cut, a teleport), are shown as
/// `cur` has them.
///
/// Draws are paired in order; a draw is blended only when both lists contain the same draw (same primitive type, vertex
/// count, VU1 program and GS state) and nothing in it moved further than a few dozen pixels.
pub fn blend(prev: &Frame, cur: &Frame, t: f32) -> Frame {
    let mut out = cur.clone();
    // menus use effects (screen feedback, animated textures) that must not be touched
    if !cur.smooth || !prev.smooth || t >= 0.999 {
        return out;
    }
    let mut pj = 0usize;
    for d in &cur.draws {
        let end = (pj + LOOKAHEAD).min(prev.draws.len());
        let Some(j) = (pj..end).find(|&j| same(&prev.draws[j], d)) else { continue };
        pj = j + 1;
        let p = &prev.draws[j];
        let a = &prev.vertices[p.first as usize..(p.first + p.count) as usize];
        let b = &cur.vertices[d.first as usize..(d.first + d.count) as usize];
        // the HUD stays where it is
        if b.first().is_some_and(|v| v.pos[2] >= 4.0e9) {
            continue;
        }
        if a.iter().zip(b).any(|(x, y)| (x.pos[0] - y.pos[0]).abs() > MAX_STEP || (x.pos[1] - y.pos[1]).abs() > MAX_STEP) {
            continue;
        }
        let o = &mut out.vertices[d.first as usize..(d.first + d.count) as usize];
        for ((x, y), r) in a.iter().zip(b).zip(o.iter_mut()) {
            for k in 0..3 {
                r.pos[k] = x.pos[k] + (y.pos[k] - x.pos[k]) * t;
                r.st[k] = x.st[k] + (y.st[k] - x.st[k]) * t;
            }
            for k in 0..4 {
                r.color[k] = x.color[k] + (y.color[k] - x.color[k]) * t;
            }
            r.fog = x.fog + (y.fog - x.fog) * t;
        }
    }
    out
}
