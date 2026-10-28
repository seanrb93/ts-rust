//! wgpu renderer for the draw lists produced by `ts-gfx`.
//!
//! Draws are issued in the original 640x224 field coordinate system and scaled by the viewport, so the output
//! resolution is independent of the original frame buffer.  Fixed-function GS state (blending, depth test, alpha
//! test, colour write masks, scissor) is mapped to GPU pipeline state / shader parameters.

use bytemuck::{Pod, Zeroable};
mod overlay;
pub mod interp;
pub use interp::{blend, interpolate};

pub fn interp_fails() -> [u32; 3] {
    [0, 1, 2].map(|i| interp::FAILS[i].swap(0, std::sync::atomic::Ordering::Relaxed))
}

pub fn interp_set_no_backdrop(v: bool) {
    interp::NO_BACKDROP.store(v, std::sync::atomic::Ordering::Relaxed);
}

pub fn interp_total() -> u32 {
    interp::STAT_TOTAL.swap(0, std::sync::atomic::Ordering::Relaxed)
}

pub fn interp_blended() -> u32 {
    interp::STAT_BLENDED.swap(0, std::sync::atomic::Ordering::Relaxed)
}
use std::collections::HashMap;
use ts_gfx::scene::*;
use wgpu::util::DeviceExt;

pub const LOGICAL_W: f32 = 640.0;
pub const LOGICAL_H: f32 = 224.0;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
/// screen movement per pixel (x, y in field pixels) for frame interpolation
const VEL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;

/// Texture filtering requested by the user (the game itself asks for bilinear or nearest per draw).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Filter {
    /// point sampling everywhere
    Nearest,
    /// as the game asked (bilinear, no mip maps)
    Bilinear,
    /// mip maps with trilinear blending
    Trilinear,
}

/// How 2D elements (HUD, menus) are placed when the picture is wider than the original 4:3.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HudMode {
    /// stretched with the picture (the original layout, distorted)
    Stretch,
    /// original proportions, centred
    Center,
    /// original proportions; in-game HUD pieces stick to the nearest screen edge, menus stay centred
    Edges,
}

/// Image-quality options of the renderer.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    /// internal resolution relative to the output area (1.0 = native, 2.0 = 4x the pixels, 0.5 = quarter)
    pub render_scale: f32,
    /// multisample anti-aliasing: 1 (off), 2, 4 or 8 samples
    pub msaa: u32,
    pub filter: Filter,
    /// anisotropic filtering, 1 (off) to 16; anything above 1 implies mip maps
    pub aniso: u16,
    /// display aspect ratio (width / height); `None` fills the window
    pub aspect: Option<f32>,
    /// placement of the 2D elements in a picture wider than 4:3
    pub hud: HudMode,
    /// internal resolution as a height in pixels (the width follows the picture's aspect ratio): 0 = the output area times `render_scale`
    pub internal_height: u32,
    /// scaling the internal image up to the window shows its pixels as they are (nearest) instead of smoothed
    pub sharp: bool,
    /// the console's own picture size, 640x448 (overrides `render_scale` and `internal_height`)
    pub native: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { render_scale: 1.0, msaa: 1, filter: Filter::Bilinear, aniso: 1, aspect: Some(4.0 / 3.0), hud: HudMode::Edges, internal_height: 0, sharp: false, native: false }
    }
}

impl Settings {
    /// `msaa` limited to the sample counts the GPU offers (`mask` has bit n set when n samples work), rounded down.
    pub fn msaa_for(&self, mask: u32) -> u32 {
        [8u32, 4, 2, 1].into_iter().find(|&n| n <= self.msaa.max(1) && (mask >> n) & 1 != 0).unwrap_or(1)
    }

    fn mips(&self) -> bool {
        self.filter == Filter::Trilinear || (self.filter != Filter::Nearest && self.aniso > 1)
    }
}

/// The image the game is drawn into (possibly multisampled and at a different resolution than the window).
struct SceneTarget {
    size: (u32, u32),
    msaa: u32,
    ms_view: Option<wgpu::TextureView>,
    _ms_tex: Option<wgpu::Texture>,
    resolve: wgpu::Texture,
    resolve_view: wgpu::TextureView,
    depth_view: wgpu::TextureView,
    _depth: wgpu::Texture,
    vel_ms_view: Option<wgpu::TextureView>,
    _vel_ms: Option<wgpu::Texture>,
    _vel_resolve: wgpu::Texture,
    vel_resolve_view: wgpu::TextureView,
    warp_buf: wgpu::Buffer,
    blit_bind: wgpu::BindGroup,
    sharp: bool,
}

const BLIT_SHADER: &str = r#"
@group(0) @binding(0) var t: texture_2d<f32>;
@group(0) @binding(1) var s: sampler;
@group(0) @binding(2) var vt: texture_2d<f32>;
// x: how far from the current game frame towards the previous one (0 = current), y,z: field size in pixels, w: 1 = warp
@group(0) @binding(3) var<uniform> warp: vec4<f32>;
struct VO { @builtin(position) p: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs(@builtin(vertex_index) i: u32) -> VO {
    var o: VO;
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    o.p = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    o.uv = vec2<f32>(x, y);
    return o;
}
fn velocity(uv: vec2<f32>) -> vec2<f32> {
    let d = vec2<f32>(textureDimensions(vt));
    let p = vec2<i32>(clamp(uv, vec2<f32>(0.0), vec2<f32>(0.9999)) * d);
    return textureLoad(vt, p, 0).xy;
}
@fragment fn fs(i: VO) -> @location(0) vec4<f32> {
    var uv = i.uv;
    if (warp.w > 0.5) {
        // The picture shows the game's current frame.  An object that moved by v since the previous frame stood at
        // (current - v); `warp.x` of the way back it is at (current - warp.x * v), so every output pixel reads the
        // current frame where that object is, found with two fixed-point steps.
        let inv = vec2<f32>(1.0 / warp.y, 1.0 / warp.z);
        var q = uv;
        for (var k = 0; k < 4; k = k + 1) {
            q = clamp(uv + warp.x * velocity(q) * inv, vec2<f32>(0.0), vec2<f32>(1.0));
        }
        // Where the vectors around here disagree (an object's edge, something uncovered, a mis-paired mesh) the search does
        // not land on a pixel that moves back to this one; keep the current frame there rather than smear it.
        let back = q - warp.x * velocity(q) * inv;
        let err = (back - uv) * vec2<f32>(warp.y, warp.z);
        if (dot(err, err) > 2.25) {
            q = uv;
        }
        uv = q;
    }
    return vec4<f32>(textureSampleLevel(t, s, uv, 0.0).rgb, 1.0);
}
"#;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuVertex {
    pos: [f32; 3],
    color: [f32; 4],
    st: [f32; 3],
    fog: f32,
    /// UV bounds (texels) of the triangle: u min, v min, u max, v max; sample centres are clamped into them
    uvb: [f32; 4],
    /// screen movement since the previous game frame (field pixels)
    vel: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuParams {
    mode: [u32; 4],
    misc: [f32; 4],
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct PipelineKey {
    blend: BlendKind,
    depth_cmp: u8,
    depth_write: bool,
    color_mask: u8,
    topology: u8,
    /// 1: the draw writes the screen-velocity buffer (opaque 3D geometry); 2: the draw is 2D (HUD, menus) and clears the
    /// velocity where it covers pixels, so that it stays put when the picture is warped
    vel: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum BlendKind {
    Opaque,
    /// general (A - B) * C + D: source factor, destination factor, operation (see `gen_blend`)
    Gen(u8, u8, u8),
    /// (Cs - Cd) * As + Cd
    Alpha,
    /// Cs * As + Cd
    Additive,
    /// Cd - Cs * As
    Subtractive,
    /// (Cs - Cd) * FIX + Cd, constant supplied per draw
    Constant,
    /// (Cd - Cs) * As + Cs
    Inverse,
}

/// A game texture on the GPU.  The sampler (filter, repeat or clamp) is chosen per draw, so the texture keeps one bind group
/// for each combination that was asked for: draws of the same texture can differ in it.
struct GpuTexture {
    _tex: wgpu::Texture,
    view: wgpu::TextureView,
    binds: HashMap<(bool, bool, bool), wgpu::BindGroup>,
    sig: u64,
    size: (u32, u32),
}

struct ScreenCopy {
    tex: wgpu::Texture,
    bind: wgpu::BindGroup,
    size: (u32, u32),
}

/// Frame buffer pages of the original (FBP 0 and 40, in 32-block units) -> texture base pointers that mean "the screen".
///
/// Returns 0 = ordinary texture, 1 = the screen being drawn this frame (frame buffer page `cur_fbp`), 2 = the other
/// (previous) frame buffer.
fn screen_kind(tex: &Option<TexKey>, cur_fbp: u32) -> u8 {
    match tex {
        Some(k) if k.tbp0 == 0 || k.tbp0 == 1280 => {
            if k.tbp0 / 32 == cur_fbp { 1 } else { 2 }
        }
        _ => 0,
    }
}

/// Box-filter one mip level; colour is weighted by alpha so transparent texels do not darken or tint the edges.
/// (Alpha is the GS byte: 128 = opaque.)
fn downsample(src: &[u8], w: u32, h: u32, nw: u32, nh: u32) -> Vec<u8> {
    let mut out = vec![0u8; (nw * nh * 4) as usize];
    for y in 0..nh {
        for x in 0..nw {
            let (mut r, mut g, mut b, mut a, mut n) = (0u32, 0u32, 0u32, 0u32, 0u32);
            let (mut ur, mut ug, mut ub) = (0u32, 0u32, 0u32);
            for dy in 0..2 {
                for dx in 0..2 {
                    let (sx, sy) = (((x * 2 + dx).min(w - 1)) as usize, ((y * 2 + dy).min(h - 1)) as usize);
                    let i = (sy * w as usize + sx) * 4;
                    let al = src[i + 3] as u32;
                    r += src[i] as u32 * al;
                    g += src[i + 1] as u32 * al;
                    b += src[i + 2] as u32 * al;
                    ur += src[i] as u32;
                    ug += src[i + 1] as u32;
                    ub += src[i + 2] as u32;
                    a += al;
                    n += 1;
                }
            }
            let o = ((y * nw + x) * 4) as usize;
            if a > 0 {
                out[o] = (r / a) as u8;
                out[o + 1] = (g / a) as u8;
                out[o + 2] = (b / a) as u8;
            } else {
                out[o] = (ur / n) as u8;
                out[o + 1] = (ug / n) as u8;
                out[o + 2] = (ub / n) as u8;
            }
            out[o + 3] = (a / n) as u8;
        }
    }
    out
}

/// Some screen effects (the fisheye lens) use one thin strip of the frame buffer as scratch: quad A copies a source
/// row (or column) into the scratch strip and quad B immediately reads it back, resampled, into its destination.
/// On the GS that is a read-after-write through live memory.  Fuse each such pair: B reads A's source directly and A
/// is dropped (made degenerate).  `quads` holds 6 vertices per quad.
fn fuse_scratch_strips(quads: &mut [GpuVertex]) {
    #[derive(Clone, Copy)]
    struct Strip {
        row: bool,
        lo: f32,    // dest coordinate of the strip (y for rows, x for columns)
        start: f32, // source coordinate at the strip's first pixel centre
        quad: usize,
    }
    let n = quads.len() / 6;
    let bbox = |q: &[GpuVertex]| {
        let f = |g: fn(&GpuVertex) -> f32| (q.iter().map(g).fold(f32::MAX, f32::min), q.iter().map(g).fold(f32::MIN, f32::max));
        (f(|v| v.pos[0]), f(|v| v.pos[1]), f(|v| v.st[0]), f(|v| v.st[1]))
    };
    let mut cur: Option<Strip> = None;
    for qi in 0..n {
        let q = &mut quads[qi * 6..qi * 6 + 6];
        let ((x0, x1), (y0, y1), (u0, u1), (v0, v1)) = bbox(q);
        let (w, h) = (x1 - x0, y1 - y0);
        let near = |a: f32, b: f32| (a - b).abs() < 0.02;
        // a scratch copy: 1 pixel thick, identity along its length
        if h <= 1.01 && w >= 64.0 && near(u0, x0 + 0.5) && near(u1, x1 + 0.5) && near(v1 - v0, 1.0) {
            cur = Some(Strip { row: true, lo: y0, start: v0, quad: qi });
            continue;
        }
        if w <= 1.01 && h >= 64.0 && near(v0, y0 + 0.5) && near(v1, y1 + 0.5) && near(u1 - u0, 1.0) {
            cur = Some(Strip { row: false, lo: x0, start: u0, quad: qi });
            continue;
        }
        if let Some(s) = cur {
            let reads_strip = if s.row { near(v0, s.lo + 0.5) && near(v1, s.lo + 1.5) } else { near(u0, s.lo + 0.5) && near(u1, s.lo + 1.5) };
            if reads_strip {
                for v in q.iter_mut() {
                    if s.row {
                        v.st[1] += s.start - (s.lo + 0.5);
                    } else {
                        v.st[0] += s.start - (s.lo + 0.5);
                    }
                }
                let dead = &mut quads[s.quad * 6..s.quad * 6 + 6];
                for v in dead.iter_mut() {
                    v.pos = [0.0; 3];
                }
                cur = None;
            }
        }
    }
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    shader: wgpu::ShaderModule,
    pipelines: HashMap<PipelineKey, wgpu::RenderPipeline>,
    pipeline_layout: wgpu::PipelineLayout,
    frame_layout: wgpu::BindGroupLayout,
    tex_layout: wgpu::BindGroupLayout,
    textures: HashMap<TexKey, GpuTexture>,
    white: Option<GpuTexture>,
    samplers: HashMap<(bool, bool, bool), wgpu::Sampler>,
    scene: Option<SceneTarget>,
    blit: Option<(wgpu::RenderPipeline, wgpu::BindGroupLayout, wgpu::Sampler, wgpu::Sampler)>,
    /// Image-quality options; changing them takes effect on the next frame.
    pub settings: Settings,
    built_with: (Filter, u16, u32),
    /// multisample counts that work with the colour and depth formats in use (bit n set = n samples)
    pub max_msaa: u32,
    scene_count: u64,
    /// GPU time still outstanding right after the CPU finished submitting a scene (measured every 30th scene), ms
    pub gpu_ms: f32,
    /// multisample count allowed by the memory budget at the current internal size
    msaa_cap: u32,
    /// Copies of the rendered image used by draws that sample the frame buffer (image-processing effects):
    /// the current frame so far, and the previous frame.
    screen_copy: Option<ScreenCopy>,
    last_copy: Option<ScreenCopy>,
    screen_sampler: wgpu::Sampler,
    pub format: wgpu::TextureFormat,
    pub stats: RenderStats,
    overlay: Option<overlay::Overlay>,
    /// Debug text drawn over the finished frame (empty = nothing).
    pub overlay_text: String,
    /// Debug: clamp texture alpha to 1.0 (the first version's behaviour) for A/B comparisons.
    pub legacy_alpha: bool,
    /// Debug: do not apply the half-pixel position shift.
    pub no_shift: bool,
    /// the graphics adapter and API in use (for the diagnostics overlay)
    pub gpu_name: String,
    /// the scene on show was drawn for a 4:3 demo: it is shown in a 4:3 rectangle
    force_43: bool,
    /// Keep this render as the "previous screen" for image effects that blend with it (off for in-between frames).
    pub update_last: bool,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct RenderStats {
    pub draws: usize,
    pub vertices: usize,
    pub textures_created: usize,
    pub missing_textures: usize,
}

fn decode_blend(alpha: u64) -> (BlendKind, f32) {
    let a = (alpha & 3) as u8;
    let b = ((alpha >> 2) & 3) as u8;
    let c = ((alpha >> 4) & 3) as u8;
    let d = ((alpha >> 6) & 3) as u8;
    let fix = ((alpha >> 32) & 0xff) as f32 / 128.0;
    // A,B,D: 0 = Cs, 1 = Cd, 2 = 0.   C: 0 = As, 1 = Ad, 2 = FIX
    match (a, b, d, c) {
        (0, 1, 1, 0) => (BlendKind::Alpha, 0.0),
        (0, 2, 1, 0) => (BlendKind::Additive, 0.0),
        (2, 0, 1, 0) => (BlendKind::Subtractive, 0.0),
        (0, 1, 1, 2) => (BlendKind::Constant, fix),
        (0, 2, 1, 2) => (BlendKind::Additive, 0.0),
        (1, 0, 0, 0) => (BlendKind::Inverse, 0.0),
        (0, 0, 0, _) => (BlendKind::Opaque, 0.0),
        // Cs * C with nothing added: only opaque when C is a fixed 1.0
        (0, 2, 2, 2) if fix >= 0.99 => (BlendKind::Opaque, 0.0),
        _ => gen_blend(a, b, c, d, fix),
    }
}

/// Map the GS blend equation `(A - B) * C + D` (A,B,D: 0 = Cs, 1 = Cd, 2 = 0; C: 0 = As, 1 = Ad, 2 = FIX) onto
/// `src * sf (op) dst * df` where possible.
fn gen_blend(a: u8, b: u8, c: u8, d: u8, fix: f32) -> (BlendKind, f32) {
    // factor codes: 0 zero, 1 one, 2 src alpha, 3 1-src alpha, 4 dst alpha, 5 1-dst alpha, 6 constant, 7 1-constant
    let (pos, neg) = match c {
        0 => (2u8, 3u8),
        1 => (4, 5),
        _ => (6, 7),
    };
    // returns (factor, negative)
    let coef = |x: u8| -> (u8, bool) {
        let k = (a == x) as i32 - (b == x) as i32;
        let m = (d == x) as i32;
        match (k, m) {
            (0, 0) => (0, false),
            (0, _) => (1, false),
            (1, 0) => (pos, false),
            (-1, 0) => (pos, true),
            (-1, _) => (neg, false),
            _ => (1, false),
        }
    };
    let (sf, sneg) = coef(0);
    let (df, dneg) = coef(1);
    let op = match (sneg, dneg) {
        (false, false) => 0,
        (false, true) => 1,
        (true, false) => 2,
        (true, true) => 0,
    };
    (BlendKind::Gen(sf, df, op), fix)
}

fn blend_state(k: BlendKind) -> Option<wgpu::BlendState> {
    use wgpu::{BlendComponent, BlendFactor as F, BlendOperation as O};
    let comp = |s: F, d: F, op: O| BlendComponent { src_factor: s, dst_factor: d, operation: op };
    let rgb = match k {
        BlendKind::Opaque => return None,
        BlendKind::Gen(sf, df, op) => {
            let f = |c: u8| match c {
                0 => F::Zero,
                1 => F::One,
                2 => F::SrcAlpha,
                3 => F::OneMinusSrcAlpha,
                4 => F::DstAlpha,
                5 => F::OneMinusDstAlpha,
                6 => F::Constant,
                _ => F::OneMinusConstant,
            };
            let o = match op {
                0 => O::Add,
                1 => O::Subtract,
                _ => O::ReverseSubtract,
            };
            comp(f(sf), f(df), o)
        }
        BlendKind::Alpha => comp(F::SrcAlpha, F::OneMinusSrcAlpha, O::Add),
        BlendKind::Additive => comp(F::SrcAlpha, F::One, O::Add),
        BlendKind::Subtractive => comp(F::SrcAlpha, F::One, O::ReverseSubtract),
        BlendKind::Constant => comp(F::Constant, F::OneMinusConstant, O::Add),
        BlendKind::Inverse => comp(F::OneMinusSrcAlpha, F::SrcAlpha, O::Add),
    };
    Some(wgpu::BlendState { color: rgb, alpha: comp(F::One, F::OneMinusSrcAlpha, O::Add) })
}

impl Renderer {
    /// Create a renderer.  `surface` (if any) selects a compatible adapter.
    pub fn new(instance: &wgpu::Instance, surface: Option<&wgpu::Surface>, format: wgpu::TextureFormat) -> Renderer {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: surface,
            force_fallback_adapter: false,
        }))
        .expect("no suitable GPU adapter");
        let gpu_name = {
            let i = adapter.get_info();
            format!("{} ({:?})", i.name, i.backend)
        };
        eprintln!("[render] graphics adapter: {gpu_name}");
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("ts-device"),
                required_features: adapter.features() & wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                required_limits: wgpu::Limits { max_texture_dimension_2d: adapter.limits().max_texture_dimension_2d.min(16384), ..wgpu::Limits::default() },
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ))
        .expect("device creation failed");
        device.on_uncaptured_error(Box::new(|e| {
            eprintln!("[render] GPU error: {e}");
            if let Ok(exe) = std::env::current_exe() {
                use std::io::Write;
                if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(exe.with_file_name("ts-rust-crash.log")) {
                    let _ = writeln!(f, "[GPU error] {e}
");
                }
            }
        }));
        // Sample counts other than 1 and 4 need the adapter-specific format features (a wgpu rule, not a GPU limit)
        let max_msaa = {
            let c = adapter.get_texture_format_features(format).flags;
            let d = adapter.get_texture_format_features(DEPTH_FORMAT).flags;
            let extended = device.features().contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES);
            let mut mask = 1u32 << 1;
            for n in [2u32, 4, 8] {
                if c.sample_count_supported(n) && d.sample_count_supported(n) && (extended || n == 4) {
                    mask |= 1 << n;
                }
            }
            mask
        };
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ts-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("frame"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let tex_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tex"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ts-layout"),
            bind_group_layouts: &[&frame_layout, &tex_layout],
            push_constant_ranges: &[],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let mut r = Renderer {
            device,
            queue,
            shader,
            pipelines: HashMap::new(),
            pipeline_layout,
            frame_layout,
            tex_layout,
            textures: HashMap::new(),
            white: None,
            samplers: HashMap::new(),
            scene: None,
            blit: None,
            settings: Settings::default(),
            built_with: (Filter::Bilinear, 1, 1),
            max_msaa,
            scene_count: 0,
            gpu_ms: 0.0,
            msaa_cap: 8,
            screen_copy: None,
            last_copy: None,
            screen_sampler: sampler,
            format,
            stats: RenderStats::default(),
            overlay: None,
            overlay_text: String::new(),
            legacy_alpha: false,
            no_shift: false,
            gpu_name: gpu_name.clone(),
            force_43: false,
            update_last: true,
        };
        r.white = Some(r.make_texture(1, 1, &[255, 255, 255, 128], (false, false, false), 0));
        r
    }

    fn make_screen_copy(&self, size: (u32, u32)) -> ScreenCopy {
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("screen-copy"),
            size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = tex.create_view(&Default::default());
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen-bind"),
            layout: &self.tex_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.screen_sampler) },
            ],
        });
        ScreenCopy { tex, bind, size }
    }

    fn ensure_sampler(&mut self, key: (bool, bool, bool)) {
        if self.samplers.contains_key(&key) {
            return;
        }
        let (linear, rep_s, rep_t) = key;
        let st = &self.settings;
        let linear = linear && st.filter != Filter::Nearest;
        let f = if linear { wgpu::FilterMode::Linear } else { wgpu::FilterMode::Nearest };
        let mips = linear && st.mips();
        let m = |rep: bool| if rep { wgpu::AddressMode::Repeat } else { wgpu::AddressMode::ClampToEdge };
        let smp = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: m(rep_s),
            address_mode_v: m(rep_t),
            mag_filter: f,
            min_filter: f,
            mipmap_filter: if mips { wgpu::FilterMode::Linear } else { wgpu::FilterMode::Nearest },
            anisotropy_clamp: if mips && st.aniso > 1 { st.aniso.min(16) } else { 1 },
            ..Default::default()
        });
        self.samplers.insert(key, smp);
    }

    fn make_texture(&mut self, w: u32, h: u32, rgba: &[u8], samp: (bool, bool, bool), sig: u64) -> GpuTexture {
        // a picture bigger than the graphics card takes is made smaller
        let max = self.device.limits().max_texture_dimension_2d.max(1);
        let (mut w, mut h) = (w, h);
        let mut shrunk: Option<Vec<u8>> = None;
        while w > max || h > max {
            let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
            let next = downsample(shrunk.as_deref().unwrap_or(rgba), w, h, nw, nh);
            shrunk = Some(next);
            w = nw;
            h = nh;
        }
        let rgba: &[u8] = shrunk.as_deref().unwrap_or(rgba);
        let mut chain: Vec<u8> = Vec::new();
        let mut levels = 1u32;
        if self.settings.mips() && w * h > 1 {
            let (mut cw, mut ch) = (w, h);
            let mut cur = rgba.to_vec();
            chain.extend_from_slice(&cur);
            while cw > 1 || ch > 1 {
                let (nw, nh) = ((cw / 2).max(1), (ch / 2).max(1));
                cur = downsample(&cur, cw, ch, nw, nh);
                chain.extend_from_slice(&cur);
                cw = nw;
                ch = nh;
                levels += 1;
            }
        }
        let data: &[u8] = if levels > 1 { &chain } else { rgba };
        let tex = self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: Some("game-tex"),
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            data,
        );
        let view = tex.create_view(&Default::default());
        let mut t = GpuTexture { _tex: tex, view, binds: HashMap::new(), sig, size: (w, h) };
        self.ensure_bind(&mut t, samp);
        t
    }

    /// Bind group of `t` with the sampler `samp`.
    fn ensure_bind(&mut self, t: &mut GpuTexture, samp: (bool, bool, bool)) {
        if t.binds.contains_key(&samp) {
            return;
        }
        self.ensure_sampler(samp);
        let sampler = &self.samplers[&samp];
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tex-bind"),
            layout: &self.tex_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&t.view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) },
            ],
        });
        t.binds.insert(samp, bind);
    }

    fn pipeline(&mut self, key: PipelineKey) -> &wgpu::RenderPipeline {
        if !self.pipelines.contains_key(&key) {
            let t_pipe = std::time::Instant::now();
            let layout = wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<GpuVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x3, 3 => Float32, 4 => Float32x4, 5 => Float32x2],
            };
            let cmp = match key.depth_cmp {
                0 => wgpu::CompareFunction::Never,
                1 => wgpu::CompareFunction::Always,
                2 => wgpu::CompareFunction::GreaterEqual,
                _ => wgpu::CompareFunction::Greater,
            };
            let mut mask = wgpu::ColorWrites::empty();
            if key.color_mask & 1 != 0 {
                mask |= wgpu::ColorWrites::RED;
            }
            if key.color_mask & 2 != 0 {
                mask |= wgpu::ColorWrites::GREEN;
            }
            if key.color_mask & 4 != 0 {
                mask |= wgpu::ColorWrites::BLUE;
            }
            if key.color_mask & 8 != 0 {
                mask |= wgpu::ColorWrites::ALPHA;
            }
            let topo = match key.topology {
                0 => wgpu::PrimitiveTopology::TriangleList,
                1 => wgpu::PrimitiveTopology::LineList,
                _ => wgpu::PrimitiveTopology::PointList,
            };
            let p = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ts-pipeline"),
                layout: Some(&self.pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.shader,
                    entry_point: "vs_main",
                    buffers: &[layout],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.shader,
                    entry_point: "fs_main",
                    targets: &[
                        Some(wgpu::ColorTargetState { format: self.format, blend: blend_state(key.blend), write_mask: mask }),
                        Some(wgpu::ColorTargetState {
                            format: VEL_FORMAT,
                            blend: if key.vel == 2 {
                                let c = wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::SrcAlpha,
                                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                    operation: wgpu::BlendOperation::Add,
                                };
                                Some(wgpu::BlendState { color: c, alpha: c })
                            } else {
                                None
                            },
                            write_mask: if key.vel != 0 { wgpu::ColorWrites::ALL } else { wgpu::ColorWrites::empty() },
                        }),
                    ],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState { topology: topo, cull_mode: None, ..Default::default() },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: key.depth_write,
                    depth_compare: cmp,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState { count: self.msaa(), mask: !0, alpha_to_coverage_enabled: false },
                multiview: None,
                cache: None,
            });
            self.pipelines.insert(key, p);
            if std::env::var_os("TS_PIPELOG").is_some() {
                eprintln!("[render] new pipeline #{} took {:.1} ms", self.pipelines.len(), t_pipe.elapsed().as_secs_f64() * 1e3);
            }
        }
        &self.pipelines[&key]
    }

    /// (Re)create the scene target when its size or sample count changed.
    fn ensure_scene(&mut self, size: (u32, u32)) {
        let msaa = self.msaa();
        let sharp = self.settings.sharp;
        if self.scene.as_ref().map(|d| (d.size, d.msaa, d.sharp)) == Some((size, msaa, sharp)) {
            return;
        }
        let ext = wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 };
        let mk = |label, format, samples, usage| {
            self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: ext,
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let resolve = mk("scene", self.format, 1, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC);
        let resolve_view = resolve.create_view(&Default::default());
        let (ms_tex, ms_view) = if msaa > 1 {
            let t = mk("scene-msaa", self.format, msaa, wgpu::TextureUsages::RENDER_ATTACHMENT);
            let v = t.create_view(&Default::default());
            (Some(t), Some(v))
        } else {
            (None, None)
        };
        let depth = mk("depth", DEPTH_FORMAT, msaa, wgpu::TextureUsages::RENDER_ATTACHMENT);
        let depth_view = depth.create_view(&Default::default());
        let vel_resolve = mk("velocity", VEL_FORMAT, 1, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING);
        let vel_resolve_view = vel_resolve.create_view(&Default::default());
        let (vel_ms, vel_ms_view) = if msaa > 1 {
            let t = mk("velocity-msaa", VEL_FORMAT, msaa, wgpu::TextureUsages::RENDER_ATTACHMENT);
            let v = t.create_view(&Default::default());
            (Some(t), Some(v))
        } else {
            (None, None)
        };
        let warp_buf = self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("warp"), size: 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        self.ensure_blit();
        let (_, layout, lin, near) = self.blit.as_ref().unwrap();
        let upscale = if sharp { near } else { lin };
        let blit_bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blit"),
            layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&resolve_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(upscale) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&vel_resolve_view) },
                wgpu::BindGroupEntry { binding: 3, resource: warp_buf.as_entire_binding() },
            ],
        });
        self.scene = Some(SceneTarget { size, msaa, ms_view, _ms_tex: ms_tex, resolve, resolve_view, depth_view, _depth: depth, vel_ms_view, _vel_ms: vel_ms, _vel_resolve: vel_resolve, vel_resolve_view, warp_buf, blit_bind, sharp });
    }

    fn msaa(&self) -> u32 {
        let capped = Settings { msaa: self.settings.msaa.min(self.msaa_cap), ..self.settings.clone() };
        capped.msaa_for(self.max_msaa)
    }

    fn ensure_blit(&mut self) {
        if self.blit.is_some() {
            return;
        }
        let layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("blit"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ],
        });
        let module = self.device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("blit"), source: wgpu::ShaderSource::Wgsl(BLIT_SHADER.into()) });
        let pl = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("blit"), bind_group_layouts: &[&layout], push_constant_ranges: &[] });
        let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blit"),
            layout: Some(&pl),
            vertex: wgpu::VertexState { module: &module, entry_point: "vs", compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: "fs",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: self.format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let lin = self.device.create_sampler(&wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });
        let near = self.device.create_sampler(&wgpu::SamplerDescriptor::default());
        self.blit = Some((pipeline, layout, lin, near));
    }

    /// Drop everything that was built for other image-quality settings.
    fn apply_settings(&mut self) {
        let now = (self.settings.filter, self.settings.aniso, self.msaa());
        if now == self.built_with {
            return;
        }
        if now.2 != self.built_with.2 {
            self.pipelines.clear();
        }
        if now.0 != self.built_with.0 || now.1 != self.built_with.1 {
            self.textures.clear();
            self.samplers.clear();
            self.white = Some(self.make_texture(1, 1, &[255, 255, 255, 128], (false, false, false), 0));
        }
        self.built_with = now;
        self.scene = None;
    }

    fn texture_for(&mut self, key: &TexKey, samp: (bool, bool, bool), cache: &mut HashMap<(TexKey, (bool, bool, bool)), Option<(u32, u32)>>, snaps: &HashMap<TexKey, std::sync::Arc<DecodedTex>>) -> Option<(u32, u32)> {
        if let Some(r) = cache.get(&(*key, samp)) {
            return *r;
        }
        let r = match snaps.get(key) {
            Some(d) => {
                let need = self.textures.get(key).map_or(true, |t| t.sig != d.sig);
                if need {
                    let t = self.make_texture(d.w, d.h, &d.rgba, samp, d.sig);
                    self.textures.insert(*key, t);
                    self.stats.textures_created += 1;
                }
                self.bind_for(key, samp);
                // (the game's coordinates address the console's size, not the picture's)
                Some((d.lw, d.lh))
            }
            None => self.texture_for_uncached(key, samp),
        };
        cache.insert((*key, samp), r);
        r
    }

    /// Make sure the cached texture `key` has a bind group for `samp`.
    fn bind_for(&mut self, key: &TexKey, samp: (bool, bool, bool)) {
        if let Some(mut t) = self.textures.remove(key) {
            self.ensure_bind(&mut t, samp);
            self.textures.insert(*key, t);
        }
    }

    fn texture_for_uncached(&mut self, key: &TexKey, samp: (bool, bool, bool)) -> Option<(u32, u32)> {
        let (w, h, sig) = ts_gfx::vram::with_vram(|v| ts_gfx::texture::signature(key, v))?;
        let need = match self.textures.get(key) {
            Some(t) => t.sig != sig,
            None => true,
        };
        if need {
            let (w, h, rgba, sig) = ts_gfx::vram::with_vram(|v| ts_gfx::texture::decode(key, v))?;
            let t = self.make_texture(w, h, &rgba, samp, sig);
            self.textures.insert(*key, t);
            self.stats.textures_created += 1;
        }
        self.bind_for(key, samp);
        Some((w, h))
    }

    /// Render `frame` into `target_tex` (size in pixels): the game is drawn into an internal image (resolution scale,
    /// anti-aliasing) which is then scaled into the largest rectangle of the requested aspect ratio.
    pub fn render(&mut self, target_tex: &wgpu::Texture, wsize: (u32, u32), frame: &Frame) {
        self.render_scene(wsize, frame);
        self.present_scene(target_tex, wsize, 0.0, false);
    }

    /// Draw the game's frame into the internal image (and its screen-velocity buffer).
    pub fn render_scene(&mut self, wsize: (u32, u32), frame: &Frame) {
        self.stats = RenderStats { draws: frame.draws.len(), vertices: frame.vertices.len(), ..Default::default() };
        self.apply_settings();
        self.force_43 = frame.demo;
        // output rectangle inside the window
        let (tw_, th_) = (wsize.0 as f32, wsize.1 as f32);
        let aspect = if frame.demo { 4.0 / 3.0 } else { self.settings.aspect.unwrap_or(tw_ / th_).max(0.1) };
        let ww = tw_.min(th_ * aspect).floor().max(1.0);
        let wh = (ww / aspect).min(th_).floor().max(1.0);
        let (wx, wy) = (((tw_ - ww) * 0.5).floor(), ((th_ - wh) * 0.5).floor());
        // internal resolution
        let sc = self.settings.render_scale.clamp(0.25, 8.0);
        let mut size = (((ww * sc).round() as u32).max(16), ((wh * sc).round() as u32).max(16));
        if self.settings.internal_height > 0 {
            let h = self.settings.internal_height.clamp(120, 8192);
            size = (((h as f32 * ww / wh).round() as u32).max(16), h);
        }
        if self.settings.native {
            size = (640, 448);
        }
        // keep the internal image inside the GPU's limits and a sane memory budget (colour + depth per sample)
        let max_dim = self.device.limits().max_texture_dimension_2d.min(8192);
        let fit = (max_dim as f32 / size.0 as f32).min(max_dim as f32 / size.1 as f32).min(1.0);
        size = (((size.0 as f32 * fit) as u32).max(16), ((size.1 as f32 * fit) as u32).max(16));
        const MAX_PIXELS: f64 = 36_000_000.0;
        let px = size.0 as f64 * size.1 as f64;
        if px > MAX_PIXELS {
            let f = (MAX_PIXELS / px).sqrt();
            size = (((size.0 as f64 * f) as u32).max(16), ((size.1 as f64 * f) as u32).max(16));
        }
        // 8x / 4x multisampling of a big image needs hundreds of MB: lower the sample count to stay under ~40M samples
        let cap = [8u32, 4, 2, 1].into_iter().find(|&n| size.0 as u64 * size.1 as u64 * n as u64 <= 40_000_000).unwrap_or(1);
        if cap != self.msaa_cap {
            self.msaa_cap = cap;
            self.apply_settings();
        }
        self.ensure_scene(size);
        // per-frame buffers
        let mut verts: Vec<GpuVertex> = frame
            .vertices
            .iter()
            .map(|v| GpuVertex { pos: v.pos, color: v.color, st: v.st, fog: v.fog, uvb: [-1e9, -1e9, 1e9, 1e9], vel: v.vel })
            .collect();
        let mut params: Vec<GpuParams> = Vec::with_capacity(frame.draws.len());
        let mut prepared: Vec<(Option<TexKey>, PipelineKey, f32)> = Vec::new();
        let mut screen_kinds: Vec<u8> = Vec::with_capacity(frame.draws.len());
        let mut tex_cache: HashMap<(TexKey, (bool, bool, bool)), Option<(u32, u32)>> = HashMap::new();
        let mut samps: Vec<(bool, bool, bool)> = Vec::with_capacity(frame.draws.len());
        for d in &frame.draws {
            let s = &d.state;
            let (mut tw, mut th) = (1.0, 1.0);
            let mut textured = false;
            let mut samp = (false, false, false);
            let sk = screen_kind(&s.tex, CURRENT_FBP.load(std::sync::atomic::Ordering::Relaxed));
            screen_kinds.push(sk);
            if sk != 0 {
                tw = LOGICAL_W;
                th = LOGICAL_H;
                textured = true;
            } else if let Some(k) = s.tex {
                let linear = (s.tex1 >> 5) & 1 != 0;
                let mut rep_s = (s.clamp & 3) == 0 || (s.clamp & 3) == 3;
                let mut rep_t = ((s.clamp >> 2) & 3) == 0 || ((s.clamp >> 2) & 3) == 3;
                if (rep_s || rep_t) && d.count > 0 {
                    let (sw, sh) = if s.use_stq { (1.0, 1.0) } else { ((1u32 << k.tw.min(12)) as f32, (1u32 << k.th.min(12)) as f32) };
                    // A texture that repeats but is only used from edge to edge (a glow sprite) would blend its first texels into
                    // the last ones where the filter reaches over the edge: a thin line along the sprite's far edge.  The console
                    // drew that edge off screen; in a wider picture it can be seen.  Such a draw is clamped.
                    let (mut u0, mut u1, mut v0, mut v1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
                    for v in &frame.vertices[d.first as usize..(d.first + d.count) as usize] {
                        if v.st[2].abs() > 1e-6 {
                            let q = if s.use_stq { v.st[2] } else { 1.0 };
                            let (u, w) = (v.st[0] / q / sw, v.st[1] / q / sh);
                            u0 = u0.min(u);
                            u1 = u1.max(u);
                            v0 = v0.min(w);
                            v1 = v1.max(w);
                        }
                    }
                    if u0 >= -0.001 && u1 <= 1.001 && v0 >= -0.001 && v1 <= 1.001 {
                        rep_s = false;
                        rep_t = false;
                    }
                }
                samp = (linear, rep_s, rep_t);
                if let Some((w, h)) = self.texture_for(&k, samp, &mut tex_cache, &frame.textures) {
                    tw = w as f32;
                    th = h as f32;
                    textured = true;
                } else {
                    self.stats.missing_textures += 1;
                }
            }
            let (kind, fix) = if s.blend.enabled { decode_blend(s.blend.alpha) } else { (BlendKind::Opaque, 0.0) };
            let test = s.test;
            let ate = test & 1 != 0;
            let atst = ((test >> 1) & 7) as u32;
            let aref = ((test >> 4) & 0xff) as f32;
            let afail = ((test >> 12) & 3) as u32;
            let zte = (test >> 16) & 1 != 0;
            let ztst = ((test >> 17) & 3) as u8;
            let depth_cmp = if zte { ztst } else { 1 };
            let tfx = s.tex.map(|k| k.tfx).unwrap_or(0);
            let tcc = s.tex.map(|k| k.tcc).unwrap_or(0);
            let flags = (textured as u32) | ((s.use_stq as u32) << 1) | (tcc << 2) | ((ate as u32) << 3) | (((sk != 0) as u32) << 4);
            params.push(GpuParams { mode: [atst, tfx, flags, afail], misc: [aref, tw, th, if self.legacy_alpha { 1.0 } else { 0.0 }] });
            let fbmsk = s.fbmsk;
            let mut cm = 0u8;
            if fbmsk & 0xff == 0 {
                cm |= 1;
            }
            if (fbmsk >> 8) & 0xff == 0 {
                cm |= 2;
            }
            if (fbmsk >> 16) & 0xff == 0 {
                cm |= 4;
            }
            if (fbmsk >> 24) & 0xff == 0 {
                cm |= 8;
            }
            let topo = match d.topology {
                Topology::Triangles => 0,
                Topology::Lines => 1,
                Topology::Points => 2,
            };
            samps.push(samp);
            prepared.push((
                if textured && sk == 0 { s.tex } else { None },
                PipelineKey {
                    blend: kind,
                    depth_cmp,
                    depth_write: !s.zmsk && depth_cmp != 0,
                    color_mask: cm,
                    topology: topo,
                    vel: match frame.vertices.get(d.first as usize) {
                        Some(v) if v.pos[2] >= 4.0e9 || (depth_cmp == 0 && !d.world && sk == 0) => 2,
                        Some(_) if !s.zmsk && depth_cmp != 0 && d.topology == Topology::Triangles => 1,
                        _ => 0,
                    },
                },
                fix,
            ));
        }
        for p in &prepared {
            self.pipeline(p.1);
        }
        // UV-addressed (2D) textured triangles: keep sample centres inside the triangle's UV box so linear filtering
        // never reaches into neighbouring glyphs / atlas entries when the picture is scaled up.
        for d in frame.draws.iter() {
            if d.state.use_stq || d.state.tex.is_none() || d.topology != Topology::Triangles {
                continue;
            }
            let r = &mut verts[d.first as usize..(d.first + d.count) as usize];
            for t in r.chunks_exact_mut(3) {
                let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
                for v in t.iter() {
                    for k in 0..2 {
                        lo[k] = lo[k].min(v.st[k]);
                        hi[k] = hi[k].max(v.st[k]);
                    }
                }
                for v in t.iter_mut() {
                    v.uvb = [lo[0], lo[1], hi[0], hi[1]];
                }
            }
        }
        // Widescreen: 2D elements (HUD, menus) keep their original proportions instead of being stretched with the
        // picture.  A draw is 2D when it ignores depth (test "always") and every vertex sits at the far plane; 3D
        // sprites (light glows, particles) are depth-tested or carry a real depth and are left alone.
        let k = aspect / (4.0 / 3.0);
        let mut hud_scissor: Vec<Option<[i32; 4]>> = vec![None; frame.draws.len()];
        // A split screen of three or four players has four panes of half the width: the 2D art is placed per pane
        let panes = frame.players >= 3;
        // The developer menu is made for a whole screen: in a pane it is scaled down to fit (its backdrop is the black box of alpha
        // 96; the draws that follow it in the pane's scissor are the menu's lines and outline)
        let mut menu_done = vec![false; frame.draws.len()];
        if frame.menu_open && frame.players >= 2 {
            let kk = if k > 1.001 { k } else { 1.0 };
            let n = frame.draws.len();
            let flat = |d: &Draw, verts: &[GpuVertex]| -> bool { verts[d.first as usize..(d.first + d.count) as usize].iter().all(|v| v.pos[2] > 4.0e9) };
            let mut i = 0;
            while i < n {
                let d = &frame.draws[i];
                let r = &verts[d.first as usize..(d.first + d.count) as usize];
                let (mut bx0, mut bx1, mut by0, mut by1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
                for v in r {
                    bx0 = bx0.min(v.pos[0]);
                    bx1 = bx1.max(v.pos[0]);
                    by0 = by0.min(v.pos[1]);
                    by1 = by1.max(v.pos[1]);
                }
                let backdrop = d.topology == Topology::Triangles && !d.world && d.state.tex.is_none() && r.len() >= 6 && r[0].color[3] == 96.0 && bx1 - bx0 >= 200.0 && by1 - by0 >= 100.0 && flat(d, &verts);
                if !backdrop {
                    i += 1;
                    continue;
                }
                let sc = d.state.scissor;
                let mut j = i;
                while j < n && frame.draws[j].state.scissor == sc {
                    j += 1;
                }
                let mask: Vec<bool> = frame.draws[i..j].iter().map(|e| !e.world && flat(e, &verts)).collect();
                let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
                for (e, _) in frame.draws[i..j].iter().zip(&mask).filter(|(_, m)| **m) {
                    for v in &verts[e.first as usize..(e.first + e.count) as usize] {
                        x0 = x0.min(v.pos[0]);
                        x1 = x1.max(v.pos[0]);
                        y0 = y0.min(v.pos[1]);
                        y1 = y1.max(v.pos[1]);
                    }
                }
                let (px0, py0, pw, ph) = (sc[0] as f32, sc[1] as f32, (sc[2] - sc[0] + 1) as f32, (sc[3] - sc[1] + 1) as f32);
                let s = ((ph - 6.0) / (y1 - y0)).min(1.0);
                let (mxc, cx, ty) = ((x0 + x1) * 0.5, px0 + pw * 0.5, py0 + 3.0);
                for ((e, done), m) in frame.draws[i..j].iter().zip(menu_done[i..j].iter_mut()).zip(&mask) {
                    if !*m {
                        continue;
                    }
                    *done = true;
                    for v in &mut verts[e.first as usize..(e.first + e.count) as usize] {
                        v.pos[0] = cx + (v.pos[0] - mxc) * s / kk;
                        v.pos[1] = ty + (v.pos[1] - y0) * s;
                    }
                }
                i = j;
            }
        }
        let mut extra_lines: Vec<(usize, Vec<GpuVertex>)> = Vec::new();
        // MapMaker is recognised by its background grid (a row of tall thin lines).  Its screens are either a HUD over the
        // map (pieces anchored to the screen thirds like the in-game HUD) or a panel / pop-up menu, in which case all
        // of the 2D art stays together in the middle.
        let (mut mm_grid, mut mm_modal) = (false, false);
        if k > 1.001 && self.settings.hud != HudMode::Stretch && !frame.in_game {
            for (i, d) in frame.draws.iter().enumerate() {
                if screen_kinds[i] != 0 || d.topology != Topology::Triangles || d.world {
                    continue;
                }
                let r = &verts[d.first as usize..(d.first + d.count) as usize];
                if r.iter().any(|v| v.pos[2] < 4.0e9) {
                    continue;
                }
                let mut tall = 0;
                for c in r.chunks_exact(3) {
                    let xs = c.iter().map(|v| v.pos[0]);
                    let ys = c.iter().map(|v| v.pos[1]);
                    let (x0, x1) = (xs.clone().fold(f32::MAX, f32::min), xs.fold(f32::MIN, f32::max));
                    let (y0, y1) = (ys.clone().fold(f32::MAX, f32::min), ys.fold(f32::MIN, f32::max));
                    let (w, h) = (x1 - x0, y1 - y0);
                    // (the grid lines have no texture; the strip-by-strip logo of the unlock pages is made of tall thin textured slices)
                    if w <= 8.0 && h >= 100.0 && d.state.tex.is_none() {
                        tall += 1;
                    }
                }
                if tall >= 12 {
                    mm_grid = true;
                }
            }
            // a pop-up panel (tile / item / light pages, menus) is a page of the front end on top of the map: the game's own
            // page stack says so (drawing heuristics mistook dark boxes of the map itself for panels)
            mm_modal = frame.page_open;
        }
        if k > 1.001 && self.settings.hud != HudMode::Stretch {
            let edges = (self.settings.hud == HudMode::Edges && frame.in_game) || (mm_grid && !mm_modal);
            let centre_all = mm_grid && mm_modal;
            if std::env::var_os("TS_DRAWDBG").is_some() {
                for (i, d) in frame.draws.iter().enumerate() {
                    let r = &verts[d.first as usize..(d.first + d.count) as usize];
                    let (mut x0, mut x1, mut y0, mut y1, mut z0, mut z1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN, f32::MAX, f32::MIN);
                    for v in r {
                        x0 = x0.min(v.pos[0]); x1 = x1.max(v.pos[0]); y0 = y0.min(v.pos[1]); y1 = y1.max(v.pos[1]); z0 = z0.min(v.pos[2]); z1 = z1.max(v.pos[2]);
                    }
                    eprintln!("[draw] {i} {:?} n {} sc {:?} x {x0:.0}..{x1:.0} y {y0:.0}..{y1:.0} z {z0:.0}..{z1:.0} world {} prog {} tex {} col {:?}", d.topology, d.count, d.state.scissor, d.world, d.prog, d.state.tex.is_some(), r.first().map(|v| v.color));
                    if d.count == 24 && d.state.tex.is_none() && std::env::var_os("TS_DRAWVERTS").is_some() {
                        let pts: Vec<String> = r.iter().map(|v| format!("({:.0},{:.0})", v.pos[0], v.pos[1])).collect();
                        eprintln!("[verts] {i} {}", pts.join(" "));
                    }
                }
            }
            for (i, d) in frame.draws.iter().enumerate() {
                if menu_done[i] {
                    continue;
                }
                let st = &d.state;
                let zte = (st.test >> 16) & 1 != 0;
                let ztst = ((st.test >> 17) & 3) as u8;
                if screen_kinds[i] != 0 && !frame.in_game && d.topology == Topology::Triangles {
                    // feedback effects (the glow around the menu logo) re-draw part of the previous picture: the picture
                    // now has its 2D elements moved, so the quad and the part it reads move the same way
                    let r = &mut verts[d.first as usize..(d.first + d.count) as usize];
                    for t in r.chunks_exact_mut(3) {
                        let lo = t.iter().map(|v| v.pos[0]).fold(f32::MAX, f32::min);
                        let hi = t.iter().map(|v| v.pos[0]).fold(f32::MIN, f32::max);
                        if lo <= 2.0 && hi >= LOGICAL_W - 2.0 {
                            continue;
                        }
                        let top = t.iter().map(|v| v.pos[1]).fold(f32::MAX, f32::min) < 28.0;
                        let mid = (lo + hi) * 0.5;
                        let map = move |x: f32| {
                            if top && mid < LOGICAL_W / 3.0 {
                                x / k
                            } else if top && mid > LOGICAL_W * 2.0 / 3.0 {
                                LOGICAL_W - (LOGICAL_W - x) / k
                            } else {
                                LOGICAL_W * 0.5 + (x - LOGICAL_W * 0.5) / k
                            }
                        };
                        for v in t.iter_mut() {
                            v.pos[0] = map(v.pos[0]);
                            v.st[0] = map(v.st[0]);
                        }
                        let (mut ulo, mut uhi) = ([f32::MAX; 2], [f32::MIN; 2]);
                        for v in t.iter() {
                            for c in 0..2 {
                                ulo[c] = ulo[c].min(v.st[c]);
                                uhi[c] = uhi[c].max(v.st[c]);
                            }
                        }
                        for v in t.iter_mut() {
                            v.uvb = [ulo[0], ulo[1], uhi[0], uhi[1]];
                        }
                    }
                    continue;
                }
                let _ = (zte, ztst);
                if screen_kinds[i] != 0 || d.topology == Topology::Points || d.world {
                    continue;
                }
                // lines (the cell grids of the MapMaker tile pages) move with the rest of the 2D art, two vertices per segment
                let np = if d.topology == Topology::Lines { 2 } else { 3 };
                let r = &mut verts[d.first as usize..(d.first + d.count) as usize];
                if !frame.in_game && mm_grid && d.prog == 0 && r.iter().all(|v| v.pos[2] < 4.0e9) {
                    // 3D art that the game projects itself, without the widescreen view constants (the MapMaker map
                    // preview, the tile shown while the cursor is over one): squeeze it like the 2D box it sits in, which is
                    // anchored to the screen's side on the map screen and stays in the middle in a panel
                    let (lo, hi) = r.iter().fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(v.pos[0]), h.max(v.pos[0])));
                    let mid = (lo + hi) * 0.5;
                    if std::env::var_os("TS_MAPDBG").is_some() {
                        let (ylo, yhi) = r.iter().fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(v.pos[1]), h.max(v.pos[1])));
                        eprintln!("[map] 3d draw {i} {:?} x {lo:.0}..{hi:.0} y {ylo:.0}..{yhi:.0} tris {} tex {:?} modal {mm_modal}", d.topology, r.len() / 3, st.tex.map(|t| (t.tbp0, t.tw, t.th)));
                    }
                    for v in r.iter_mut() {
                        v.pos[0] = if mm_modal || (LOGICAL_W / 3.0..=LOGICAL_W * 2.0 / 3.0).contains(&mid) {
                            LOGICAL_W * 0.5 + (v.pos[0] - LOGICAL_W * 0.5) / k
                        } else if mid > LOGICAL_W * 2.0 / 3.0 {
                            LOGICAL_W - (LOGICAL_W - v.pos[0]) / k
                        } else {
                            v.pos[0] / k
                        };
                    }
                    continue;
                }
                if !frame.in_game && !panes && st.scissor[2] - st.scissor[0] < 600 && r.iter().all(|v| v.pos[2] < 4.0e9) {
                    // 3D art inside a clipped panel (the MapMaker map preview): the game drew it for the wider picture
                    // (about the middle of its own box), so it only needs to follow the box to its new place
                    let c = (st.scissor[0] + st.scissor[2]) as f32 * 0.5 + 0.5;
                    let dx = (LOGICAL_W * 0.5 + (c - LOGICAL_W * 0.5) / k) - c;
                    for v in r.iter_mut() {
                        v.pos[0] += dx;
                    }
                    let m = |x: f32| LOGICAL_W * 0.5 + (x - LOGICAL_W * 0.5) / k;
                    hud_scissor[i] = Some([m(st.scissor[0] as f32).floor() as i32, st.scissor[1], m(st.scissor[2] as f32 + 1.0).ceil() as i32 - 1, st.scissor[3]]);
                    continue;
                }
                if r.iter().any(|v| v.pos[2] < 4.0e9) {
                    continue;
                }
                // the black frame the game draws around each pane of the split screen (four strips: 2 units high at the top and bottom, 4 wide at
                // the sides) is made thin: only the gap between the panes' rectangles (what the 3D does not cover) stays black, and a line
                // at the screen's edge; it is placed in the field's own units, like the scissor rectangles, so it is not moved with the rest
                if panes && frame.in_game && d.count == 24 && st.tex.is_none() && r.iter().all(|v| v.color[0] == 0.0 && v.color[1] == 0.0 && v.color[2] == 0.0) {
                    thin_pane_border(r);
                    continue;
                }
                // hairlines (grid lines) thinner than two field pixels fade away when magnified: widen them
                for t in r.chunks_exact_mut(np) {
                    let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
                    for v in t.iter() {
                        x0 = x0.min(v.pos[0]);
                        x1 = x1.max(v.pos[0]);
                        y0 = y0.min(v.pos[1]);
                        y1 = y1.max(v.pos[1]);
                    }
                    // (only the grid's own lines: outlines and crosshairs are drawn as the game made them)
                    if d.topology == Topology::Lines && y1 - y0 < 1.9 && y1 - y0 > 0.0 && x1 - x0 > 6.0 {
                        let c = (y0 + y1) * 0.5;
                        for v in t.iter_mut() {
                            v.pos[1] = if v.pos[1] > c { c + 1.0 } else { c - 1.0 };
                        }
                    }
                }
                let sc = st.scissor;
                // a narrow scissor in the running game is a split-screen pane; in menus it is a clipped panel, which moves
                // with the rest of the menu
                let narrow = sc[2] - sc[0] < 600;
                let viewport = panes;
                let mut grid_tri = vec![false; r.len() / np];
                if np == 3 {
                    for (t, c) in r.chunks_exact_mut(np).enumerate() {
                        let (lo, hi) = (c.iter().map(|v| v.pos[0]).fold(f32::MAX, f32::min), c.iter().map(|v| v.pos[0]).fold(f32::MIN, f32::max));
                        let h = c.iter().map(|v| v.pos[1]).fold(f32::MIN, f32::max) - c.iter().map(|v| v.pos[1]).fold(f32::MAX, f32::min);
                        let w = hi - lo;
                        if w <= 8.0 && h >= LOGICAL_H * 0.9 && st.tex.is_none() && (lo <= 2.0 || hi >= LOGICAL_W - 2.0) {
                            // the black borders at the screen edges stay at the screen edges
                            grid_tri[t] = true;
                            let left = lo <= 2.0;
                            for v in c.iter_mut() {
                                v.pos[0] = if left { v.pos[0] / k } else { LOGICAL_W - (LOGICAL_W - v.pos[0]) / k };
                            }
                        } else if mm_grid && st.tex.is_none() && w > 8.0 && h >= LOGICAL_H * 0.9 && ((lo <= 2.0) != (hi >= LOGICAL_W - 2.0)) {
                            // a black box hiding the part of the grid outside the map (it reaches one side of the field):
                            // in the wider picture it has to reach the screen's edge too
                            grid_tri[t] = true;
                            for v in c.iter_mut() {
                                v.pos[0] = if v.pos[0] <= 2.0 {
                                    -4000.0
                                } else if v.pos[0] >= LOGICAL_W - 2.0 {
                                    4000.0
                                } else {
                                    LOGICAL_W * 0.5 + (v.pos[0] - LOGICAL_W * 0.5) / k
                                };
                            }
                        } else if mm_grid && w <= 8.0 && h >= 100.0 {
                            grid_tri[t] = true;
                            for v in c.iter_mut() {
                                v.pos[0] = LOGICAL_W * 0.5 + (v.pos[0] - LOGICAL_W * 0.5) / k;
                            }
                        }
                    }
                }
                // Group the triangles into elements: runs whose horizontal extents overlap or nearly touch belong together
                // (a line of text, a bar, the radar), so each element moves as one piece.  Triangles spanning the whole
                // width (backgrounds, fades, highlight bars) keep stretching.
                let mut tris: Vec<(f32, f32, usize)> = Vec::new();
                for (t, c) in r.chunks_exact(np).enumerate() {
                    let lo = c.iter().map(|v| v.pos[0]).fold(f32::MAX, f32::min);
                    let hi = c.iter().map(|v| v.pos[0]).fold(f32::MIN, f32::max);
                    if lo <= 2.0 && hi >= LOGICAL_W - 2.0 && st.tex.is_none() {
                        continue; // flat fills / fades covering the whole picture keep stretching
                    }
                    if grid_tri[t] {
                        continue;
                    }
                    tris.push((lo, hi, t));
                }
                tris.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                let mut clusters: Vec<(f32, f32, Vec<usize>)> = Vec::new();
                for (lo, hi, t) in tris {
                    match clusters.last_mut() {
                        Some(c) if lo <= c.1 + 20.0 => {
                            c.1 = c.1.max(hi);
                            c.2.push(t);
                        }
                        _ => clusters.push((lo, hi, vec![t])),
                    }
                }
                for (lo, hi, members) in clusters {
                    let mid = (lo + hi) * 0.5;
                    // vertical position of the cluster (the menu title row sits at the top)
                    let (mut ylo, mut yhi) = (f32::MAX, f32::MIN);
                    for &t in &members {
                        for v in &r[np * t..np * t + np] {
                            ylo = ylo.min(v.pos[1]);
                            yhi = yhi.max(v.pos[1]);
                        }
                    }
                    // Things that reach (nearly) both screen edges stretch with the picture: backgrounds (meshes), flat
                    // bars and fades, the red pain flash.  Wide single glows stay proportional.
                    let bottom_strip = !frame.in_game && !mm_grid && ylo >= 211.0 && yhi <= 225.0;
                    if !bottom_strip && lo <= 40.0 && hi >= LOGICAL_W - 40.0 && hi - lo >= LOGICAL_W * 0.9 && (st.tex.is_none() || members.len() > 20 || (lo >= -20.0 && hi <= LOGICAL_W + 20.0)) {
                        continue;
                    }
                    if !frame.in_game && lo < -4.0 && hi > LOGICAL_W + 4.0 {
                        // a front-end sprite that overhangs both sides of the picture (a glow of the title screen): its far edges
                        // were off screen on the console, but in the wider picture the texture's last texels can be seen as a thin
                        // line, so its alpha fades out over the outer texels (`trim` in the shader)
                        params[i].mode[2] |= 32;
                    }
                    let centre = move |x: f32| LOGICAL_W * 0.5 + (x - LOGICAL_W * 0.5) / k;
                    let left = move |x: f32| x / k;
                    let right = move |x: f32| LOGICAL_W - (LOGICAL_W - x) / k;
                    if std::env::var_os("TS_MAPDBG").is_some() {
                        eprintln!("[map] draw {i} {:?}: x {lo:.0}..{hi:.0} y {ylo:.0}..{yhi:.0} edges {edges} centre_all {centre_all} viewport {viewport} members {} clamp {:#x} stq {} tex {:?} blend {:?}", d.topology, members.len(), st.clamp, st.use_stq, st.tex.map(|t| (t.tbp0, t.tw, t.th)), st.blend);
                    }
                    let map: Box<dyn Fn(f32) -> f32> = if bottom_strip {
                        // the strip at the bottom of the character select screens (the bars of the teams, the icons of the players and bots
                        // under them): stretched to the edges of the picture (the bars start 16 units in and end 16 before the edge), bars and icons
                        // the same way, so each icon stays on its bar
                        let stretch = |x: f32| (x - 16.0) * LOGICAL_W / (LOGICAL_W - 32.0);
                        if st.tex.is_some() && hi - lo < 40.0 {
                            // (an icon: its place goes with the strip, its size stays the picture's proportions)
                            Box::new(move |x| stretch(mid) + (x - mid) / k)
                        } else {
                            Box::new(stretch)
                        }
                    } else if viewport {
                        // a pane of the split screen (the left or the right half): things are placed within the pane, as the whole picture
                        // is for one player
                        let (p0, p1) = if mid < LOGICAL_W * 0.5 { (0.0, LOGICAL_W * 0.5) } else { (LOGICAL_W * 0.5, LOGICAL_W) };
                        let (pw, c) = (p1 - p0, (p0 + p1) * 0.5);
                        let small = hi - lo < 60.0 && yhi - ylo < 40.0;
                        if hi - lo >= pw * 0.9 && (st.tex.is_none() || (members.len() > 20 && yhi - ylo >= LOGICAL_H * 0.4)) {
                            // the pane's borders and the flashes and fades that fill it (the pain vignette is a ring of triangles as high as the pane; a line of text is not) reach its edges
                            Box::new(move |x| if x < c { p0 + (x - p0) / k } else { p1 - (p1 - x) / k })
                        } else if !frame.in_game || frame.scores_open || self.settings.hud == HudMode::Center || hi - lo > pw * 0.5 || (st.tex.is_none() && small && (members.len() == 8 || (mid > p0 + pw * 0.25 && mid < p0 + pw * 0.75 && {
                            let rel = ((ylo + yhi) * 0.5 - sc[1] as f32) / (sc[3] - sc[1] + 1) as f32;
                            rel > 0.35 && rel < 0.65
                        }))) {
                            // (menus, big art, the aiming crosshair)
                            Box::new(move |x| c + (x - c) / k)
                        } else if mid < p0 + pw / 3.0 {
                            Box::new(move |x| p0 + (x - p0) / k)
                        } else if mid > p0 + pw * 2.0 / 3.0 {
                            Box::new(move |x| p1 - (p1 - x) / k)
                        } else {
                            Box::new(move |x| c + (x - c) / k)
                        }
                    } else if centre_all || (frame.in_game && self.settings.hud == HudMode::Center) {
                        // (in a level with the HUD placed in the centre, every piece of it goes to the centre: the bands that
                        // start at a screen edge belong to menus)
                        Box::new(centre)
                    } else if !edges && (ylo + yhi) * 0.5 < 30.0 && yhi < 110.0 {
                        // menu title row: the title (and its band) sit at the left edge, the logo with all of its glow
                        // layers (and the "CHEATS ON" under it, whose height varies a little) at the right edge, whatever the
                        // size of the individual pieces
                        if lo < LOGICAL_W * 0.25 {
                            Box::new(left)
                        } else if hi > LOGICAL_W * 0.75 {
                            Box::new(right)
                        } else {
                            Box::new(centre)
                        }
                    } else if frame.in_game && st.tex.is_none() && hi - lo < 60.0 && yhi - ylo < 40.0 && ((ylo > 60.0 && yhi < 390.0) || members.len() == 8) {
                        // the aiming crosshair (four lines, eight triangles, so it is found also in the rows of the HUD): it follows the aim
                        // over the whole picture, so it is placed like the picture is (not by the screen third it is in, which would make it
                        // jump whenever it crosses one)
                        Box::new(centre)
                    } else if hi - lo > LOGICAL_W * 0.5 {
                        // big art (scope overlays) stays centred
                        Box::new(centre)
                    } else if mm_grid && !mm_modal && (members.len() == 2 && st.tex.map_or(false, |t| t.tw <= 4 && t.th <= 4) || (st.tex.is_none() && yhi - ylo >= 12.0) || !((yhi < 62.0) || (yhi < 90.0 && (hi < 72.0 || lo > 440.0)) || (lo > 440.0 && ylo > 125.0))) {
                        // MapMaker map screen: everything that is not part of the screen's HUD (the placed tiles, the held
                        // tile, the cursor: a small picture of 16x16 texels; the held tile and the placed ones are shapes without a texture that are
                        // not a thin bar like FREE; wherever they are, also under the right side's legend) belongs to the map and moves with the grid, whichever third it is in
                        Box::new(centre)
                    } else if edges {
                        if mid < LOGICAL_W / 3.0 {
                            Box::new(left)
                        } else if mid > LOGICAL_W * 2.0 / 3.0 {
                            Box::new(right)
                        } else {
                            Box::new(centre)
                        }
                    } else if lo <= 2.0 && hi - lo > 100.0 {
                        Box::new(left) // bands that start at the left screen edge
                    } else if hi >= LOGICAL_W - 2.0 && hi - lo > 100.0 {
                        // (a small piece that ends at the edge, like the right arrow of an option in the second column, stays with the menu)
                        Box::new(right)
                    } else {
                        // menus: one consistent placement keeps labels, bars, highlights and cursors together
                        Box::new(centre)
                    };
                    for t in members {
                        for v in &mut r[np * t..np * t + np] {
                            v.pos[0] = map(v.pos[0]);
                        }
                    }
                }
                // an evenly spaced row of tall thin lines (the MapMaker grid) is repeated to the edges of the wider picture
                if mm_grid && np == 3 {
                    let lines: Vec<usize> = (0..r.len() / 3)
                        .filter(|&t| {
                            let c = &r[np * t..np * t + np];
                            let w = c.iter().map(|v| v.pos[0]).fold(f32::MIN, f32::max) - c.iter().map(|v| v.pos[0]).fold(f32::MAX, f32::min);
                            let h = c.iter().map(|v| v.pos[1]).fold(f32::MIN, f32::max) - c.iter().map(|v| v.pos[1]).fold(f32::MAX, f32::min);
                            let lo = c.iter().map(|v| v.pos[0]).fold(f32::MAX, f32::min);
                            let hi = c.iter().map(|v| v.pos[0]).fold(f32::MIN, f32::max);
                            w <= 8.0 && h >= 100.0 && lo > 2.0 && hi < LOGICAL_W - 2.0
                        })
                        .collect();
                    if std::env::var_os("TS_LINEDBG").is_some() {
                        eprintln!("[lines] draw {i}: {} tall lines of {} tris", lines.len(), r.len() / 3);
                    }
                    if lines.len() >= 12 {
                        let xs: Vec<f32> = lines.iter().map(|&t| r[3 * t].pos[0].min(r[3 * t + 1].pos[0]).min(r[3 * t + 2].pos[0])).collect();
                        let (xmin, xmax) = (xs.iter().cloned().fold(f32::MAX, f32::min), xs.iter().cloned().fold(f32::MIN, f32::max));
                        let mut uniq: Vec<f32> = xs.iter().map(|x| (x * 2.0).round() / 2.0).collect();
                        uniq.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        uniq.dedup();
                        if uniq.len() >= 6 {
                            let spacing = (uniq[uniq.len() - 1] - uniq[0]) / (uniq.len() - 1) as f32;
                            let period = (xmax - xmin) + spacing;
                            // The game draws only the grid columns that are inside its 640 pixel field, and none beyond the
                            // map's own edges.  A side where the lines reach the field's edge is a side where the map goes
                            // on (the wider picture shows more of it); a side where they stop short is the map's edge and
                            // nothing is added there.
                            let (orig_lo, orig_hi) = (LOGICAL_W * 0.5 + (xmin - LOGICAL_W * 0.5) * k, LOGICAL_W * 0.5 + (xmax - LOGICAL_W * 0.5) * k);
                            let reach = 1.6 * spacing * k;
                            let (open_left, open_right) = (orig_lo <= reach, orig_hi >= LOGICAL_W - reach);
                            if std::env::var_os("TS_LINEDBG").is_some() {
                                eprintln!("[lines] original x {orig_lo:.1}..{orig_hi:.1} spacing {:.1}: open left {open_left} right {open_right} scissor {:?}", spacing * k, sc);
                            }
                            let src: Vec<GpuVertex> = lines.iter().flat_map(|&t| r[np * t..np * t + np].to_vec()).collect();
                            if spacing > 3.0 {
                                let mut extra: Vec<GpuVertex> = Vec::new();
                                for m in 1..=3 {
                                    for dir in [-1.0f32, 1.0] {
                                        if (dir < 0.0 && !open_left) || (dir > 0.0 && !open_right) {
                                            continue;
                                        }
                                        let shift = dir * m as f32 * period;
                                        let (a, b) = (xmin + shift, xmax + shift);
                                        if b < -spacing || a > LOGICAL_W + spacing {
                                            continue;
                                        }
                                        extra.extend(src.iter().map(|v| GpuVertex { pos: [v.pos[0] + shift, v.pos[1], v.pos[2]], color: [v.color[0] * 0.45, v.color[1] * 0.45, v.color[2] * 0.45, v.color[3]], ..*v }));
                                    }
                                }
                                if !extra.is_empty() {
                                    extra_lines.push((i, extra));
                                }
                            }
                        }
                    }
                }
                if narrow && !viewport {
                    let m = |x: f32| LOGICAL_W * 0.5 + (x - LOGICAL_W * 0.5) / k;
                    hud_scissor[i] = Some([m(sc[0] as f32).floor() as i32, sc[1], m(sc[2] as f32 + 1.0).ceil() as i32 - 1, sc[3]]);
                }
            }
        }
        let mut extra_vdraws: Vec<(usize, u32, u32, bool)> = Vec::new();
        let extra_base = verts.len() as u32;
        for (i, extra) in extra_lines {
            let first = verts.len() as u32;
            let count = extra.len() as u32;
            verts.extend(extra);
            extra_vdraws.push((i, first, count, false));
        }
        for (i, d) in frame.draws.iter().enumerate() {
            if screen_kinds[i] == 1 && d.topology == Topology::Triangles && d.count % 6 == 0 {
                fuse_scratch_strips(&mut verts[d.first as usize..(d.first + d.count) as usize]);
            }
        }
        let zero_v = [GpuVertex::zeroed()];
        let zero_p = [GpuParams::zeroed()];
        let vbuf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("verts"),
            contents: bytemuck::cast_slice(if verts.is_empty() { &zero_v[..] } else { &verts[..] }),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let pbuf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("params"),
            contents: bytemuck::cast_slice(if params.is_empty() { &zero_p[..] } else { &params[..] }),
            usage: wgpu::BufferUsages::STORAGE,
        });
        // the game fills the whole internal image
        let (vx, vy, vw, vh) = (0.0f32, 0.0f32, size.0 as f32, size.1 as f32);
        // GS samples sit on integer coordinates (pixel centres): a pixel's sample point is its integer position and the game's
        // 0.5 texel offsets land exactly on texel centres.  Geometry stays at its GS position (so a sprite covers whole upscaled
        // pixels) and the shader moves texture lookups half a logical pixel back, which keeps every texel centre where it was.
        let shift = if self.no_shift { 0.0 } else { vw / LOGICAL_W };
        let xf: [f32; 8] = [2.0 / LOGICAL_W, -2.0 / LOGICAL_H, -1.0, 1.0, shift, 0.0, 0.0, 0.0];
        let sbuf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("screen"),
            contents: bytemuck::cast_slice(&xf),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let frame_bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("frame"),
            layout: &self.frame_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: pbuf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: sbuf.as_entire_binding() },
            ],
        });
        // Draws that sample the frame buffer (image-processing effects) need a snapshot of the image rendered so far.
        let uses_screen = screen_kinds.iter().any(|&k| k != 0);
        if uses_screen && std::env::var_os("TS_SCREENLOG").is_some() {
            eprintln!("[render] screen draws: cur_fbp={} kinds={:?}", CURRENT_FBP.load(std::sync::atomic::Ordering::Relaxed), screen_kinds.iter().filter(|&&k| k != 0).count());
        }
        let (cw, ch) = (vw.floor().max(1.0) as u32, vh.floor().max(1.0) as u32);
        if self.last_copy.as_ref().map(|c| c.size) != Some((cw, ch)) {
            self.last_copy = Some(self.make_screen_copy((cw, ch)));
        }
        if uses_screen && self.screen_copy.as_ref().map(|c| c.size) != Some((cw, ch)) {
            self.screen_copy = Some(self.make_screen_copy((cw, ch)));
        }
        let copy_origin = wgpu::Origin3d::ZERO;
        let copy_ok = true;
        // Virtual draws: (draw index, first vertex, vertex count, snapshot first).  A draw that samples the screen is
        // split where the geometry switches between wide strips and tall strips (a multi-pass in-place warp such as
        // the fisheye lens): each part must see the result of the previous one, as the GS reading live memory did.
        let mut vdraws: Vec<(usize, u32, u32, bool)> = Vec::new();
        for (i, d) in frame.draws.iter().enumerate() {
            if screen_kinds[i] != 1 {
                vdraws.push((i, d.first, d.count, false));
                continue;
            }
            let verts = &frame.vertices[d.first as usize..(d.first + d.count) as usize];
            let mut start = 0usize;
            let mut class = 2u8;
            let mut first_part = true;
            let tri_class = |t: &[Vertex]| -> u8 {
                let xs = t.iter().map(|v| v.pos[0]);
                let ys = t.iter().map(|v| v.pos[1]);
                let w = xs.clone().fold(f32::MIN, f32::max) - xs.fold(f32::MAX, f32::min);
                let h = ys.clone().fold(f32::MIN, f32::max) - ys.fold(f32::MAX, f32::min);
                if w > h * 4.0 { 0 } else if h > w * 4.0 { 1 } else { 2 }
            };
            let ntri = if d.topology == Topology::Triangles { verts.len() / 3 } else { 0 };
            for t in 0..ntri {
                let c = tri_class(&verts[3 * t..3 * t + 3]);
                if c == 2 {
                    continue;
                }
                if class != 2 && c != class {
                    let snap = first_part && (i == 0 || screen_kinds[i - 1] != 1) || !first_part;
                    vdraws.push((i, d.first + start as u32, (3 * t - start) as u32, snap));
                    first_part = false;
                    start = 3 * t;
                }
                class = c;
            }
            let snap = first_part && (i == 0 || screen_kinds[i - 1] != 1) || !first_part;
            vdraws.push((i, d.first + start as u32, (verts.len() - start) as u32, snap));
        }
        // (start, end, snapshot): a snapshot entry copies the target into the screen texture; others are vdraw ranges.
        // the repeated grid lines belong right behind the draw they were made from, below the rest of the 2D art
        vdraws.extend(extra_vdraws);
        vdraws.sort_by_key(|v| (v.0, v.1 >= extra_base));
        let mut segments: Vec<(usize, usize, bool)> = Vec::new();
        {
            let n = vdraws.len();
            let mut start = 0;
            for i in 0..n {
                if vdraws[i].3 {
                    if i > start {
                        segments.push((start, i, false));
                    }
                    segments.push((i, i, true));
                    start = i;
                }
            }
            if n > start || segments.is_empty() {
                segments.push((start, n, false));
            }
        }
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        let scene = self.scene.as_ref().unwrap();
        let scene_tex = &scene.resolve;
        let mut first_pass = true;
        for &(start, end, snapshot) in &segments {
            if snapshot {
                if copy_ok {
                    if let Some(sc) = &self.screen_copy {
                        enc.copy_texture_to_texture(
                            wgpu::ImageCopyTexture { texture: scene_tex, mip_level: 0, origin: copy_origin, aspect: wgpu::TextureAspect::All },
                            wgpu::ImageCopyTexture { texture: &sc.tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                            wgpu::Extent3d { width: cw, height: ch, depth_or_array_layers: 1 },
                        );
                    }
                }
                continue;
            }
            let depth_view = &scene.depth_view;
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: scene.ms_view.as_ref().unwrap_or(&scene.resolve_view),
                        resolve_target: scene.ms_view.as_ref().map(|_| &scene.resolve_view),
                        ops: wgpu::Operations {
                            load: if first_pass { wgpu::LoadOp::Clear(wgpu::Color::BLACK) } else { wgpu::LoadOp::Load },
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: scene.vel_ms_view.as_ref().unwrap_or(&scene.vel_resolve_view),
                        resolve_target: scene.vel_ms_view.as_ref().map(|_| &scene.vel_resolve_view),
                        ops: wgpu::Operations {
                            load: if first_pass { wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT) } else { wgpu::LoadOp::Load },
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: if first_pass { wgpu::LoadOp::Clear(0.0) } else { wgpu::LoadOp::Load },
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            first_pass = false;
            pass.set_viewport(vx, vy, vw, vh, 0.0, 1.0);
            pass.set_vertex_buffer(0, vbuf.slice(..));
            pass.set_bind_group(0, &frame_bind, &[]);
            let sx = vw / LOGICAL_W;
            let sy = vh / LOGICAL_H;
            for vi in start..end {
                let (i, vfirst, vcount, _) = vdraws[vi];
                let d = &frame.draws[i];
                let (texkey, pk, fix) = prepared[i];
                pass.set_pipeline(&self.pipelines[&pk]);
                let tbind = match screen_kinds[i] {
                    1 => &self.screen_copy.as_ref().unwrap().bind,
                    2 => &self.last_copy.as_ref().unwrap().bind,
                    _ => match texkey.and_then(|k| self.textures.get(&k)).and_then(|t| t.binds.get(&samps[i])) {
                        Some(b) => b,
                        None => &self.white.as_ref().unwrap().binds[&(false, false, false)],
                    },
                };
                pass.set_bind_group(1, tbind, &[]);
                if matches!(pk.blend, BlendKind::Constant | BlendKind::Gen(6 | 7, _, _) | BlendKind::Gen(_, 6 | 7, _)) {
                    pass.set_blend_constant(wgpu::Color { r: fix as f64, g: fix as f64, b: fix as f64, a: fix as f64 });
                }
                let sc = hud_scissor[i].unwrap_or(d.state.scissor);
                let x0 = (vx + sc[0].max(0) as f32 * sx).floor().max(0.0) as u32;
                let y0 = (vy + sc[1].max(0) as f32 * sy).floor().max(0.0) as u32;
                let x1 = ((vx + (sc[2] + 1).max(0) as f32 * sx).ceil() as u32).min(size.0);
                let y1 = ((vy + (sc[3] + 1).max(0) as f32 * sy).ceil() as u32).min(size.1);
                if x1 > x0 && y1 > y0 {
                    pass.set_scissor_rect(x0, y0, x1 - x0, y1 - y0);
                } else {
                    continue;
                }
                pass.draw(vfirst..vfirst + vcount, i as u32..i as u32 + 1);
            }
        }
        // keep this frame's image for "last screen" effects (before the next frame overwrites the target)
        if copy_ok && self.update_last {
            if let Some(lc) = &self.last_copy {
                enc.copy_texture_to_texture(
                    wgpu::ImageCopyTexture { texture: scene_tex, mip_level: 0, origin: copy_origin, aspect: wgpu::TextureAspect::All },
                    wgpu::ImageCopyTexture { texture: &lc.tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                    wgpu::Extent3d { width: cw, height: ch, depth_or_array_layers: 1 },
                );
            }
        }
        self.queue.submit(Some(enc.finish()));
        // every 30th scene: wait for the GPU to learn how long drawing it really takes
        self.scene_count += 1;
        if self.scene_count % 30 == 0 {
            let t = std::time::Instant::now();
            self.device.poll(wgpu::Maintain::Wait);
            self.gpu_ms = t.elapsed().as_secs_f32() * 1e3;
        }
    }

    /// The output rectangle (x, y, w, h) inside a window of `wsize`.
    fn out_rect(&self, wsize: (u32, u32)) -> (f32, f32, f32, f32) {
        let (tw_, th_) = (wsize.0 as f32, wsize.1 as f32);
        let aspect = if self.force_43 { 4.0 / 3.0 } else { self.settings.aspect.unwrap_or(tw_ / th_).max(0.1) };
        let ww = tw_.min(th_ * aspect).floor().max(1.0);
        let wh = (ww / aspect).min(th_).floor().max(1.0);
        (((tw_ - ww) * 0.5).floor(), ((th_ - wh) * 0.5).floor(), ww, wh)
    }

    /// Show the last rendered scene in `target_tex`.  With `warp`, the picture is moved back towards the previous game
    /// frame by `t` (0 = the current frame, 1 = the previous one) using the screen-velocity buffer.
    pub fn present_scene(&mut self, target_tex: &wgpu::Texture, wsize: (u32, u32), t: f32, warp: bool) {
        let target_view = target_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let target = &target_view;
        let (wx, wy, ww, wh) = self.out_rect(wsize);
        let Some(scene) = self.scene.as_ref() else { return };
        self.queue.write_buffer(&scene.warp_buf, 0, bytemuck::cast_slice(&[t, LOGICAL_W, LOGICAL_H, if warp { 1.0f32 } else { 0.0 }]));
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("present") });
        // scale the internal image into the output rectangle of the window (black bars around it)
        {
            let (pipeline, _, _, _) = self.blit.as_ref().unwrap();
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blit"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_viewport(wx, wy, ww, wh, 0.0, 1.0);
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &scene.blit_bind, &[]);
            pass.draw(0..3, 0..1);
        }
        if !self.overlay_text.is_empty() {
            if self.overlay.is_none() {
                self.overlay = Some(overlay::Overlay::new(&self.device, &self.queue, self.format));
            }
            if let Some(o) = &self.overlay {
                o.draw(&self.device, &mut enc, target, wsize, &self.overlay_text);
            }
        }
        self.queue.submit(Some(enc.finish()));
    }

    /// Benchmark helper: render into a scratch texture and wait for the GPU.
    pub fn bench_render(&mut self, frame: &Frame, w: u32, h: u32) {
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("bench"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        self.render(&tex, (w, h), frame);
        self.device.poll(wgpu::Maintain::Wait);
    }

    /// Render a frame offscreen and return RGBA8 pixels (for screenshots / tests).
    pub fn screenshot(&mut self, frame: &Frame, w: u32, h: u32) -> Vec<u8> {
        self.screenshot_t(frame, w, h, 0.0, false)
    }

    /// Like `screenshot`, but shows the picture `t` of the way back towards the previous game frame (needs a frame prepared by
    /// `interpolate`).
    pub fn screenshot_t(&mut self, frame: &Frame, w: u32, h: u32, t: f32, warp: bool) -> Vec<u8> {
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shot"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        self.render_scene((w, h), frame);
        self.present_scene(&tex, (w, h), t, warp);
        let bpr = (w * 4).div_ceil(256) * 256;
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (bpr * h) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = self.device.create_command_encoder(&Default::default());
        enc.copy_texture_to_buffer(
            wgpu::ImageCopyTexture { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::ImageCopyBuffer {
                buffer: &buf,
                layout: wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(bpr), rows_per_image: Some(h) },
            },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        self.queue.submit(Some(enc.finish()));
        let slice = buf.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);
        let data = slice.get_mapped_range();
        let mut out = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h as usize {
            out.extend_from_slice(&data[y * bpr as usize..y * bpr as usize + w as usize * 4]);
        }
        if matches!(self.format, wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb) {
            for px in out.chunks_exact_mut(4) {
                px.swap(0, 2);
            }
        }
        drop(data);
        buf.unmap();
        out
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    pub fn texture_size(&self, k: &TexKey) -> Option<(u32, u32)> {
        self.textures.get(k).map(|t| t.size)
    }
}

/// The four strips (6 vertices each: top, bottom, left, right) of a pane's black frame, made thin: they cover the gap between the panes'
/// rectangles and nothing more, and a line half a unit wide at the screen's edge.
fn thin_pane_border(r: &mut [GpuVertex]) {
    if r.len() != 24 {
        return;
    }
    let bounds = |q: &[GpuVertex]| {
        let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for v in q {
            x0 = x0.min(v.pos[0]);
            x1 = x1.max(v.pos[0]);
            y0 = y0.min(v.pos[1]);
            y1 = y1.max(v.pos[1]);
        }
        (x0, x1, y0, y1)
    };
    let left = bounds(&r[12..18]);
    let right = bounds(&r[18..24]);
    let (px0, px1, py0, py1) = (left.0, right.1, left.2, left.3);
    // (the pane's frame: the sides are 4 wide, the top and bottom 2 high)
    if !(right.1 - right.0 < 6.0 && left.1 - left.0 < 6.0) {
        return;
    }
    let (li, ri, ti, bi) = (px0 > 100.0, px1 < 600.0, py0 > 50.0, py1 < 200.0);
    let (cx0, cx1, cy0, cy1) = (px0.max(0.0), px1.min(640.0), py0.max(0.0), py1.min(224.0));
    // (the panes' 3D covers 0..318 and 322..640 across, 0..112 and 113..224 down: the gap between is what is black)
    let rects = [
        // top
        (cx0, cx1, if ti { 111.9 } else { 0.0 }, if ti { 113.1 } else { 0.5 }),
        // bottom
        (cx0, cx1, if bi { 111.9 } else { 223.5 }, if bi { 113.1 } else { 224.0 }),
        // left
        (if li { 320.0 } else { 0.0 }, if li { 322.2 } else { 0.6 }, cy0, cy1),
        // right
        (if ri { 317.8 } else { 639.4 }, if ri { 320.0 } else { 640.0 }, cy0, cy1),
    ];
    for (k, &(nx0, nx1, ny0, ny1)) in rects.iter().enumerate() {
        let q = &mut r[6 * k..6 * k + 6];
        let (ox0, ox1, oy0, oy1) = bounds(q);
        let mid = |lo: f32, hi: f32, v: f32| if (v - lo).abs() <= (v - hi).abs() { 0 } else { 1 };
        for v in q.iter_mut() {
            v.pos[0] = if mid(ox0, ox1, v.pos[0]) == 0 { nx0 } else { nx1 };
            v.pos[1] = if mid(oy0, oy1, v.pos[1]) == 0 { ny0 } else { ny1 };
        }
    }
}
