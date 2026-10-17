// Draw shader for GS-style vertex streams.
//
// Vertices arrive in GS screen space (x,y in pixels of the 640x224 field, z a 24-bit depth value); colour channels use
// 128 = 1.0 for texture modulation.  Texture coordinates are either perspective S,T,Q or U,V texel positions.

struct Params {
    // x: alpha test mode (0 never,1 always,2 less,3 lequal,4 equal,5 gequal,6 greater,7 notequal)
    // y: texture function (0 modulate, 1 decal, 2 highlight, 3 highlight2)
    // z: flags bit0 textured, bit1 use STQ, bit2 texture alpha used (TCC), bit3 alpha test enabled, bit4 texture is a frame-buffer copy
    // w: alpha test fail action (0 keep, 1 fb only, 2 zb only, 3 rgb only)
    mode: vec4<u32>,
    // x: alpha reference (0..255), y: texture width, z: texture height, w: fixed alpha (unused)
    misc: vec4<f32>,
};

struct Screen {
    // xy: scale, zw: offset: ndc = pos.xy * scale + offset
    xf: vec4<f32>,
    // x: screen pixels per logical pixel (texture lookups move half a logical pixel back), 0 = off
    extra: vec4<f32>,
};

@group(0) @binding(0) var<storage, read> params: array<Params>;
@group(0) @binding(1) var<uniform> screen: Screen;
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

struct VsIn {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) st: vec3<f32>,
    @location(3) fog: f32,
    @location(4) uvb: vec4<f32>,
    @location(5) vel: vec2<f32>,
    @builtin(instance_index) draw: u32,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) st: vec3<f32>,
    @location(2) @interpolate(flat) draw: u32,
    @location(3) @interpolate(flat) uvb: vec4<f32>,
    @location(4) vel: vec2<f32>,
};

struct FsOut {
    @location(0) color: vec4<f32>,
    // xy: screen movement since the previous game frame (written by opaque 3D draws, and as zero by 2D draws, which blend
    // it by their alpha); w: alpha for that blend
    @location(1) vel: vec4<f32>,
};

@vertex
fn vs_main(v: VsIn) -> VsOut {
    var o: VsOut;
    let z = clamp(v.pos.z / 16777215.0, 0.0, 1.0);
    o.clip = vec4<f32>(v.pos.x * screen.xf.x + screen.xf.z, v.pos.y * screen.xf.y + screen.xf.w, z, 1.0);
    o.color = v.color / 128.0;
    o.st = v.st;
    o.draw = v.draw;
    o.uvb = v.uvb;
    o.vel = v.vel;
    return o;
}

@fragment
fn fs_main(i: VsOut) -> FsOut {
    let p = params[i.draw];
    // derivatives must run in uniform control flow
    let stq = (p.mode.z & 2u) != 0u;
    let raw = select(i.st.xy, i.st.xy / i.st.z, stq);
    let d = (dpdx(raw) + dpdy(raw)) * (0.5 * screen.extra.x);
    let tsz = select(vec2<f32>(p.misc.y, p.misc.z), vec2<f32>(1.0), stq);
    let gx = dpdx(raw) / tsz;
    let gy = dpdy(raw) / tsz;
    var uv: vec2<f32>;
    var rgb = min(i.color.rgb, vec3<f32>(2.0));
    var a = i.color.a;
    if ((p.mode.z & 1u) != 0u) {
        uv = (raw - d);
        if ((p.mode.z & 2u) == 0u) {
            // The GS draws a sprite's last pixel one step before its far UV edge, so keep lookups on the outermost texel centres
            // instead of letting the filter blend in the neighbouring texture (which may be another glyph).
            let step = vec2<f32>(dpdx(raw).x, dpdy(raw).y) * screen.extra.x;
            let lo = i.uvb.xy + max(-step, vec2<f32>(0.0));
            let hi = i.uvb.zw - max(step, vec2<f32>(0.0));
            uv = clamp(uv, lo, max(hi, lo)) / vec2<f32>(p.misc.y, p.misc.z);
        }
        var t = textureSampleGrad(tex, samp, uv, gx, gy);
        // texture alpha is stored as the raw GS byte (128 = 1.0, values up to 255 act as 1.99)
        t.a = t.a * (255.0 / 128.0);
        if ((p.mode.z & 32u) != 0u) {
            // sprite that overhangs the picture: fade the outermost texels so that its far edge does not show
            let e = min(uv.x, 1.0 - uv.x) * p.misc.y;
            t.a = t.a * clamp((e - 0.5) / 3.0, 0.0, 1.0);
        }
        if (p.misc.w > 0.5) { t.a = min(t.a, 1.0); }
        if ((p.mode.z & 16u) != 0u) { t.a = 1.0; }   // frame-buffer sample: colour only
        var ta = a;
        if ((p.mode.z & 4u) != 0u) {
            ta = t.a;
        }
        switch (p.mode.y) {
            case 0u: {
                rgb = t.rgb * rgb;
                if ((p.mode.z & 4u) != 0u) { a = t.a * a; }
            }
            case 1u: {
                rgb = t.rgb;
                a = ta;
            }
            default: {
                rgb = t.rgb * rgb + vec3<f32>(a);
                a = ta;
            }
        }
    }
    if ((p.mode.z & 8u) != 0u) {
        let a8 = a * 128.0;     // alpha on the 0..128 scale of the original
        let r = p.misc.x * 128.0 / 255.0;
        var pass_ = true;
        switch (p.mode.x) {
            case 0u: { pass_ = false; }
            case 1u: { pass_ = true; }
            case 2u: { pass_ = a8 < r; }
            case 3u: { pass_ = a8 <= r; }
            case 4u: { pass_ = abs(a8 - r) < 0.5; }
            case 5u: { pass_ = a8 >= r; }
            case 6u: { pass_ = a8 > r; }
            default: { pass_ = abs(a8 - r) >= 0.5; }
        }
        if (!pass_) { discard; }
    }
    var o: FsOut;
    o.color = vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), clamp(a, 0.0, 1.0));
    o.vel = vec4<f32>(i.vel, 0.0, select(0.0, 1.0, o.color.a > 0.02));
    return o;
}
