//! GIF packet interpreter: tracks the drawing-relevant graphics-synthesizer registers and assembles primitives into
//! the backend-independent `Frame` draw list.  Only the register set this game uses is interpreted; the GS pixel
//! pipeline itself is not modelled (a GPU does the rasterisation).

use crate::scene::*;
use crate::vram::{with_vram, Image};

#[derive(Clone, Copy, Default)]
struct Ctx {
    tex0: u64,
    tex1: u64,
    clamp: u64,
    alpha: u64,
    test: u64,
    frame: u64,
    zbuf: u64,
    xyoffset: u64,
    scissor: u64,
}

#[derive(Clone, Copy, Default)]
struct Vtx {
    v: Vertex,
}

/// value of the LABEL register that starts the mirrored drawing of a model (any other value ends it)
pub const MIRROR_LABEL: u64 = 0x4d49_5252;

pub struct Gif {
    prim: u64,
    rgbaq: [u8; 4],
    q: f32,
    st: [f32; 2],
    uv: [u16; 2],
    fog: u8,
    ctx: [Ctx; 2],
    bitbltbuf: u64,
    trxpos: u64,
    trxreg: u64,
    trxdir: u64,
    queue: Vec<Vtx>,
    strip_flip: bool,
    /// `state()` result valid until a drawing register / PRIM changes
    state_cache: Option<DrawState>,
    /// identifies `state_cache` (bumped whenever it is rebuilt) so that consecutive primitives can be merged cheaply
    state_id: u64,
    id_state: Option<DrawState>,
    /// (state id, topology, program, world) of the last draw emitted in the frame being built
    last_emit: Option<(u64, Topology, u32, bool)>,
    /// entry of the VU1 microprogram currently running
    pub prog: u32,
    /// the packet being processed belongs to a world-projected sprite function
    pub world: bool,
    /// counts XGKICKs
    pub kick: u32,
    /// the drawing is mirrored left to right (the game writes the GS register LABEL with `MIRROR_LABEL` before the picture of a mirrored model)
    mirror: bool,
}

impl Default for Gif {
    fn default() -> Self {
        Gif {
            prim: 0,
            rgbaq: [128, 128, 128, 128],
            q: 1.0,
            st: [0.0; 2],
            uv: [0; 2],
            fog: 0,
            // Draw environment normally written by the GS thread / vblank handler (`sceGsPutDrawEnv`): the 640x224
            // field is centred in the 4096x4096 primitive space, i.e. offset (2048-320, 2048-112) in 12.4 fixed point.
            ctx: [Ctx { xyoffset: (1728u64 * 16) | ((1936u64 * 16) << 32), ..Ctx::default() }; 2],
            bitbltbuf: 0,
            trxpos: 0,
            trxreg: 0,
            trxdir: 0,
            queue: Vec::new(),
            prog: 0,
            world: false,
            kick: 0,
            mirror: false,
            strip_flip: false,
            state_cache: None,
            state_id: 0,
            id_state: None,
            last_emit: None,
        }
    }
}

fn u32at(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}
fn u64at(d: &[u8], o: usize) -> u64 {
    u64::from_le_bytes(d[o..o + 8].try_into().unwrap())
}

impl Gif {
    pub fn new() -> Gif {
        Gif::default()
    }

    fn cur(&self) -> &Ctx {
        &self.ctx[((self.prim >> 9) & 1) as usize]
    }

    fn state(&self) -> DrawState {
        let c = self.cur();
        let tme = (self.prim >> 4) & 1 != 0;
        let tex = tme.then(|| TexKey {
            tbp0: (c.tex0 & 0x3fff) as u32,
            tbw: ((c.tex0 >> 14) & 0x3f) as u32,
            psm: ((c.tex0 >> 20) & 0x3f) as u32,
            tw: ((c.tex0 >> 26) & 0xf) as u32,
            th: ((c.tex0 >> 30) & 0xf) as u32,
            tcc: ((c.tex0 >> 34) & 1) as u32,
            tfx: ((c.tex0 >> 35) & 3) as u32,
            cbp: ((c.tex0 >> 37) & 0x3fff) as u32,
            cpsm: ((c.tex0 >> 51) & 0xf) as u32,
            csm: ((c.tex0 >> 55) & 1) as u32,
            csa: ((c.tex0 >> 56) & 0x1f) as u32,
        });
        let x0 = (c.scissor & 0x7ff) as i32;
        let x1 = ((c.scissor >> 16) & 0x7ff) as i32;
        let y0 = ((c.scissor >> 32) & 0x7ff) as i32;
        let y1 = ((c.scissor >> 48) & 0x7ff) as i32;
        DrawState {
            tex,
            blend: Blend { enabled: (self.prim >> 6) & 1 != 0, alpha: c.alpha },
            test: c.test,
            zmsk: (c.zbuf >> 32) & 1 != 0,
            fbmsk: (c.frame >> 32) as u32,
            clamp: c.clamp,
            tex1: c.tex1,
            scissor: [x0, y0, x1, y1],
            use_stq: (self.prim >> 8) & 1 == 0,
            fog: (self.prim >> 5) & 1 != 0,
            gouraud: (self.prim >> 3) & 1 != 0,
            fbp: (c.frame & 0x1ff) as u32,
        }
    }

    fn vertex(&self, x: u32, y: u32, z: u32) -> Vtx {
        let c = self.cur();
        let ox = (c.xyoffset & 0xffff) as f32 / 16.0;
        let oy = ((c.xyoffset >> 32) & 0xffff) as f32 / 16.0;
        let fst = (self.prim >> 8) & 1 != 0;
        let st = if fst {
            [self.uv[0] as f32 / 16.0, self.uv[1] as f32 / 16.0, 1.0]
        } else {
            [self.st[0], self.st[1], self.q]
        };
        Vtx {
            v: Vertex {
                pos: [if self.mirror { 640.0 - (x as f32 / 16.0 - ox) } else { x as f32 / 16.0 - ox }, y as f32 / 16.0 - oy, z as f32],
                color: [self.rgbaq[0] as f32, self.rgbaq[1] as f32, self.rgbaq[2] as f32, self.rgbaq[3] as f32],
                st,
                fog: self.fog as f32,
                vel: [0.0, 0.0],
            },
        }
    }

    fn emit(&mut self, frame: &mut Frame, topo: Topology, vs: &[Vtx]) {
        if self.state_cache.is_none() {
            let new = self.state();
            // registers rewritten with the same values leave the state (and so its id) as it was
            if self.id_state.as_ref() != Some(&new) {
                self.state_id += 1;
                self.id_state = Some(new.clone());
            }
            self.state_cache = Some(new);
        }
        let first = frame.vertices.len() as u32;
        frame.vertices.extend(vs.iter().map(|v| v.v));
        let count = vs.len() as u32;
        let key = (self.state_id, topo, self.prog, self.world);
        if self.last_emit == Some(key) {
            if let Some(last) = frame.draws.last_mut() {
                if last.first + last.count == first {
                    last.count += count;
                    return;
                }
            }
        }
        self.last_emit = Some(key);
        let state = self.state_cache.as_ref().unwrap();
        frame.draws.push(Draw { state: state.clone(), topology: topo, first, count, prog: self.prog, world: self.world, kick: self.kick });
    }

    /// A vertex was kicked: assemble according to the primitive type.
    /// Queue a vertex; a vertex written with ADC set (XYZ3/XYZF3, or the ADC bit of packed XYZ2) is queued without
    /// triggering primitive output -- this is how strips are restarted.
    fn queue_only(&mut self, x: u32, y: u32, z: u32) {
        let v = self.vertex(x, y, z);
        self.queue.push(v);
        let ptype = self.prim & 7;
        let keep = match ptype {
            0 => 0,
            1 | 6 => 1,
            2 => 1,
            3 => 2,
            4 => 2,
            5 => 2,
            _ => 0,
        };
        // fan keeps its first vertex as the pivot: drop from index 1; others drop the oldest
        while self.queue.len() > keep.max(1) && ptype != 5 {
            self.queue.remove(0);
        }
        if ptype == 5 {
            while self.queue.len() > 2 {
                self.queue.remove(1);
            }
        }
    }

    fn kick(&mut self, frame: &mut Frame, x: u32, y: u32, z: u32) {
        tlog!("    vertex prim={} ({}, {}, {z}) rgba={:?} st={:?} q={} uv={:?}", self.prim & 7, x as f32 / 16.0, y as f32 / 16.0, self.rgbaq, self.st, self.q, self.uv);
        let v = self.vertex(x, y, z);
        self.queue.push(v);
        let ptype = self.prim & 7;
        match ptype {
            0 => {
                let q = [self.queue[0]];
                self.queue.clear();
                self.emit(frame, Topology::Points, &q);
            }
            1 => {
                if self.queue.len() == 2 {
                    let q = [self.queue[0], self.queue[1]];
                    self.queue.clear();
                    self.emit(frame, Topology::Lines, &q);
                }
            }
            2 => {
                if self.queue.len() == 2 {
                    let q = [self.queue[0], self.queue[1]];
                    self.emit(frame, Topology::Lines, &q);
                    self.queue.remove(0);
                }
            }
            3 => {
                if self.queue.len() == 3 {
                    let q = [self.queue[0], self.queue[1], self.queue[2]];
                    self.queue.clear();
                    self.emit(frame, Topology::Triangles, &q);
                }
            }
            4 => {
                if self.queue.len() == 3 {
                    let q = [self.queue[0], self.queue[1], self.queue[2]];
                    self.emit(frame, Topology::Triangles, &q);
                    self.queue.remove(0);
                }
            }
            5 => {
                if self.queue.len() == 3 {
                    let q = [self.queue[0], self.queue[1], self.queue[2]];
                    self.emit(frame, Topology::Triangles, &q);
                    self.queue.remove(1);
                }
            }
            6 => {
                if self.queue.len() == 2 {
                    let (a, b) = (self.queue[0], self.queue[1]);
                    self.queue.clear();
                    // sprite: axis-aligned rectangle; colour and depth come from the second vertex
                    let mk = |x: f32, y: f32, s: f32, t: f32| {
                        let mut v = b;
                        v.v.pos[0] = x;
                        v.v.pos[1] = y;
                        v.v.st = [s, t, b.v.st[2]];
                        if (self.prim >> 8) & 1 == 0 {
                            // STQ sprite: interpolate S,T linearly between the corners
                            v.v.st = [s, t, b.v.st[2]];
                        }
                        v
                    };
                    let (x0, y0, x1, y1) = (a.v.pos[0], a.v.pos[1], b.v.pos[0], b.v.pos[1]);
                    let (s0, t0, s1, t1) = (a.v.st[0], a.v.st[1], b.v.st[0], b.v.st[1]);
                    let tl = mk(x0, y0, s0, t0);
                    let tr = mk(x1, y0, s1, t0);
                    let bl = mk(x0, y1, s0, t1);
                    let br = mk(x1, y1, s1, t1);
                    self.emit(frame, Topology::Triangles, &[tl, tr, bl, tr, br, bl]);
                }
            }
            _ => {
                self.queue.clear();
            }
        }
        let _ = self.strip_flip;
    }

    /// Write a register using the GS-native 64-bit encoding.
    pub fn write_reg(&mut self, frame: &mut Frame, addr: u32, v: u64) {
        if !matches!(addr, 0x01 | 0x02 | 0x03 | 0x04 | 0x05) {
            tlog!("    GS reg {addr:#04x} = {v:#018x}");
        }
        let ci = |a: u32| (a & 1) as usize; // _1 / _2 register pairs: even address = context 1
        if !matches!(addr, 0x01..=0x05 | 0x0a | 0x0c | 0x0d) {
            self.state_cache = None;
        }
        match addr {
            0x00 => {
                self.prim = v & 0x7ff;
                self.queue.clear();
            }
            0x01 => {
                self.rgbaq = [v as u8, (v >> 8) as u8, (v >> 16) as u8, (v >> 24) as u8];
                self.q = f32::from_bits((v >> 32) as u32);
            }
            0x02 => {
                self.st = [f32::from_bits(v as u32), f32::from_bits((v >> 32) as u32)];
            }
            0x03 => {
                self.uv = [(v & 0x3fff) as u16, ((v >> 16) & 0x3fff) as u16];
            }
            0x04 => {
                self.fog = (v >> 56) as u8;
                self.kick(frame, (v & 0xffff) as u32, ((v >> 16) & 0xffff) as u32, ((v >> 32) & 0xff_ffff) as u32);
            }
            0x05 => {
                self.kick(frame, (v & 0xffff) as u32, ((v >> 16) & 0xffff) as u32, (v >> 32) as u32);
            }
            0x0c => {
                self.fog = (v >> 56) as u8;
                self.queue_only((v & 0xffff) as u32, ((v >> 16) & 0xffff) as u32, ((v >> 32) & 0xff_ffff) as u32);
            }
            0x0d => {
                self.queue_only((v & 0xffff) as u32, ((v >> 16) & 0xffff) as u32, (v >> 32) as u32);
            }
            0x06 | 0x07 => self.ctx[ci(addr)].tex0 = v,
            0x08 | 0x09 => self.ctx[ci(addr)].clamp = v,
            0x0a => self.fog = (v >> 56) as u8,
            0x14 | 0x15 => self.ctx[ci(addr)].tex1 = v,
            0x18 | 0x19 => self.ctx[ci(addr)].xyoffset = v,
            0x40 | 0x41 => self.ctx[ci(addr)].scissor = v,
            0x42 | 0x43 => self.ctx[ci(addr)].alpha = v,
            0x47 => self.ctx[0].test = v,
            0x48 => self.ctx[1].test = v,
            0x4c | 0x4d => self.ctx[ci(addr)].frame = v,
            0x4e | 0x4f => self.ctx[ci(addr)].zbuf = v,
            0x62 => self.mirror = (v & 0xffff_ffff) == MIRROR_LABEL,
            0x50 => self.bitbltbuf = v,
            0x51 => self.trxpos = v,
            0x52 => self.trxreg = v,
            0x53 => self.trxdir = v,
            // texture flush, fog colour, colour clamp, dither, TEXA, PABE, etc. do not change the draw list
            _ => {}
        }
    }

    fn image_data(&mut self, frame: &mut Frame, data: &[u8]) {
        let dbp = ((self.bitbltbuf >> 32) & 0x3fff) as u32;
        let dbw = ((self.bitbltbuf >> 48) & 0x3f) as u32;
        let psm = ((self.bitbltbuf >> 56) & 0x3f) as u32;
        let x = ((self.trxpos >> 32) & 0x7ff) as u32;
        let y = ((self.trxpos >> 48) & 0x7ff) as u32;
        let w = (self.trxreg & 0xfff) as u32;
        let h = ((self.trxreg >> 32) & 0xfff) as u32;
        if std::env::var_os("TS_UPLOGLOG").is_some() {
            eprintln!("[upload] dbp {dbp:#x} dbw {dbw} psm {psm} pos {x},{y} size {w}x{h} bytes {}", data.len());
        }
        let w2 = ImageWrite { dbp, dbw, psm, x, y, w, h, data: data.to_vec() };
        with_vram(|v| {
            v.write(Image { dbp, dbw, psm, w, h, data: data.to_vec(), generation: 0 });
        });
        frame.uploads.push(w2);
    }

    /// Interpret one GIF packet stream (a whole XGKICK buffer or DIRECT payload).  Returns bytes consumed.
    pub fn process(&mut self, frame: &mut Frame, d: &[u8]) -> usize {
        let mut p = 0usize;
        while p + 16 <= d.len() {
            let lo = u64at(d, p);
            let regs = u64at(d, p + 8);
            let nloop = (lo & 0x7fff) as usize;
            let eop = (lo >> 15) & 1 != 0;
            let pre = (lo >> 46) & 1 != 0;
            let prim = (lo >> 47) & 0x7ff;
            let flg = (lo >> 58) & 3;
            let mut nreg = ((lo >> 60) & 0xf) as usize;
            if nreg == 0 {
                nreg = 16;
            }
            p += 16;
            tlog!("  GIFTAG nloop={nloop} eop={eop} flg={flg} nreg={nreg} regs={regs:#x} pre={pre} prim={prim:#x}");
            if pre && flg == 0 {
                self.write_reg(frame, 0, prim);
            }
            match flg {
                0 => {
                    for _ in 0..nloop {
                        for r in 0..nreg {
                            if p + 16 > d.len() {
                                return p;
                            }
                            let reg = ((regs >> (4 * r)) & 0xf) as u32;
                            self.packed(frame, reg, &d[p..p + 16]);
                            p += 16;
                        }
                    }
                }
                1 => {
                    let total = nloop * nreg;
                    for i in 0..total {
                        if p + 8 > d.len() {
                            return p;
                        }
                        let reg = ((regs >> (4 * (i % nreg))) & 0xf) as u32;
                        let v = u64at(d, p);
                        p += 8;
                        self.write_reg(frame, reg, v);
                    }
                    p = (p + 15) & !15;
                }
                2 => {
                    let bytes = nloop * 16;
                    let end = (p + bytes).min(d.len());
                    self.image_data(frame, &d[p..end]);
                    p = end;
                }
                _ => {
                    p += nloop * 16;
                }
            }
            if eop {
                break;
            }
        }
        p
    }

    fn packed(&mut self, frame: &mut Frame, reg: u32, q: &[u8]) {
        match reg {
            0 => self.write_reg(frame, 0, u64at(q, 0)),
            1 => {
                let v = (q[0] as u64) | (q[4] as u64) << 8 | (q[8] as u64) << 16 | (q[12] as u64) << 24;
                self.rgbaq = [q[0], q[4], q[8], q[12]];
                let _ = v;
            }
            2 => {
                self.st = [f32::from_bits(u32at(q, 0)), f32::from_bits(u32at(q, 4))];
                self.q = f32::from_bits(u32at(q, 8));
            }
            3 => self.uv = [(u32at(q, 0) & 0x3fff) as u16, (u32at(q, 4) & 0x3fff) as u16],
            4 => {
                let adc = (u32at(q, 12) >> 15) & 1 != 0;
                let x = u32at(q, 0) & 0xffff;
                let y = u32at(q, 4) & 0xffff;
                let z = (u32at(q, 8) >> 4) & 0xff_ffff;
                self.fog = (u32at(q, 12) >> 4) as u8;
                if !adc {
                    self.kick(frame, x, y, z);
                } else {
                    self.queue_only(x, y, z);
                }
            }
            5 => {
                let adc = (u32at(q, 12) >> 15) & 1 != 0;
                let x = u32at(q, 0) & 0xffff;
                let y = u32at(q, 4) & 0xffff;
                let z = u32at(q, 8);
                if !adc {
                    self.kick(frame, x, y, z);
                } else {
                    self.queue_only(x, y, z);
                }
            }
            6 | 7 => self.write_reg(frame, reg, u64at(q, 0)),
            8 | 9 => self.write_reg(frame, reg, u64at(q, 0)),
            0xa => self.fog = (u32at(q, 12) >> 4) as u8,
            0xc | 0xd => {}
            0xe => {
                let addr = (q[8]) as u32;
                self.write_reg(frame, addr, u64at(q, 0));
            }
            _ => {}
        }
    }
}
