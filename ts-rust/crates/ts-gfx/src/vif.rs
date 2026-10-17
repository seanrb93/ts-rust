//! VIF1 command interpreter + VU1 data memory.  The VIF is the unpacking front-end of the vector unit: the game's
//! display list is a stream of VIF codes that fill VU1's 16 KB data memory (`UNPACK`) and start microprograms
//! (`MSCAL`).  The microprograms themselves are native Rust (decompiled from the original microcode).

use crate::gif::Gif;
use crate::programs;
use crate::vu::Regs;
use crate::scene::Frame;

pub const VU_MEM_QW: usize = 1024;

/// VU1 data memory + the VIF-side addressing registers.
#[derive(Clone)]
pub struct Vu1 {
    pub mem: [[u32; 4]; VU_MEM_QW],
    /// TOP: the buffer the running microprogram reads (XTOP)
    pub top: u32,
    /// ITOP: non-double-buffered base (XITOP)
    pub itop: u32,
}

impl Vu1 {
    pub fn new() -> Box<Vu1> {
        Box::new(Vu1 { mem: [[0; 4]; VU_MEM_QW], top: 0, itop: 0 })
    }

    pub fn f(&self, qw: usize, lane: usize) -> f32 {
        f32::from_bits(self.mem[qw % VU_MEM_QW][lane])
    }

    pub fn set_f(&mut self, qw: usize, lane: usize, v: f32) {
        self.mem[qw % VU_MEM_QW][lane] = v.to_bits();
    }

    /// Borrowed view of VU memory from quadword `qw` to the end, as little-endian bytes (what the GIF reads).
    pub fn bytes_from(&self, qw: usize) -> &[u8] {
        let q = qw % VU_MEM_QW;
        let words: &[[u32; 4]] = &self.mem[q..];
        // SAFETY: [[u32; 4]] is plain integer data; this target is little endian like the PS2.
        unsafe { std::slice::from_raw_parts(words.as_ptr() as *const u8, words.len() * 16) }
    }

    /// The bytes of `qwc` quadwords starting at `qw`, as the GIF would read them (little endian).
    pub fn bytes(&self, qw: usize, qwc: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(qwc * 16);
        for i in 0..qwc {
            for w in self.mem[(qw + i) % VU_MEM_QW] {
                out.extend_from_slice(&w.to_le_bytes());
            }
        }
        out
    }
}

pub struct Vif {
    base: u32,
    offset: u32,
    dbf: bool,
    tops: u32,
    itop: u32,
    cl: u32,
    wl: u32,
    mode: u32,
    mask: u32,
    row: [u32; 4],
    col: [u32; 4],
}

impl Default for Vif {
    fn default() -> Self {
        Vif { base: 0, offset: 0, dbf: false, tops: 0, itop: 0, cl: 1, wl: 1, mode: 0, mask: 0, row: [0; 4], col: [0; 4] }
    }
}

fn rd32(d: &[u8], o: usize) -> u32 {
    if o + 4 > d.len() {
        return 0;
    }
    u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

impl Vif {
    fn start_program(&mut self, vu: &mut Vu1, regs: &mut Regs, gif: &mut Gif, frame: &mut Frame, entry: u32) {
        vu.top = self.tops;
        vu.itop = self.itop;
        self.dbf = !self.dbf;
        self.tops = self.base + if self.dbf { self.offset } else { 0 };
        tlog!("  -> microprogram {entry:#x} (TOP={})", vu.top);
        if crate::trace::enabled() && entry == 0 && std::env::var("TS_VUIN").is_ok() {
            let t = vu.top as usize;
            for q in 0..28usize {
                tlog!("      in[top+{q:3}] = {:?} / {:x?}", vu.mem[t + q].iter().map(|w| f32::from_bits(*w)).collect::<Vec<_>>(), vu.mem[t + q]);
            }
            for q in 24..30usize {
                tlog!("      pos[{}] = {:?}  st = {:?}  col = {:x?}", q - 24, vu.mem[t + q].iter().map(|w| f32::from_bits(*w)).collect::<Vec<_>>(), vu.mem[t + q + 64].iter().map(|w| f32::from_bits(*w)).collect::<Vec<_>>(), vu.mem[t + q + 128]);
            }
        }
        if crate::trace::enabled() && std::env::var("TS_VUMEM").is_ok() {
            for q in 0..32usize {
                tlog!("      vumem[{q:3}] = {:?}", vu.mem[q].iter().map(|w| f32::from_bits(*w)).collect::<Vec<_>>());
            }
        }
        programs::run(entry, vu, regs, gif, frame);
    }

    /// Execute a VIF code stream (one DMA packet).
    pub fn process(&mut self, vu: &mut Vu1, regs: &mut Regs, gif: &mut Gif, frame: &mut Frame, d: &[u8], marks: &[(usize, bool)]) {
        let mut p = 0usize;
        let mut mi = 0usize;
        while p + 4 <= d.len() {
            if !marks.is_empty() {
                while mi + 1 < marks.len() && marks[mi + 1].0 <= p {
                    mi += 1;
                }
                gif.world = marks[mi].1;
            }
            let code = rd32(d, p);
            p += 4;
            let cmd = (code >> 24) & 0x7f;
            let imm = code & 0xffff;
            let num = ((code >> 16) & 0xff) as usize;
            if cmd != 0 {
                tlog!("VIF {:#04x} imm={imm:#06x} num={num}", cmd);
            }
            match cmd {
                0x00 => {}
                0x01 => {
                    self.cl = imm & 0xff;
                    self.wl = (imm >> 8) & 0xff;
                }
                0x02 => {
                    self.offset = imm & 0x3ff;
                    self.dbf = false;
                    self.tops = self.base;
                }
                0x03 => self.base = imm & 0x3ff,
                0x04 => self.itop = imm & 0x3ff,
                0x05 => self.mode = imm & 3,
                0x06 | 0x07 => {}
                0x10 | 0x11 | 0x13 => {}
                0x14 | 0x15 => self.start_program(vu, regs, gif, frame, imm),
                0x17 => self.start_program(vu, regs, gif, frame, 0xffff),
                0x20 => {
                    self.mask = rd32(d, p);
                    p += 4;
                }
                0x30 => {
                    for k in 0..4 {
                        self.row[k] = rd32(d, p + 4 * k);
                    }
                    p += 16;
                }
                0x31 => {
                    for k in 0..4 {
                        self.col[k] = rd32(d, p + 4 * k);
                    }
                    p += 16;
                }
                0x4a => {
                    // MPG: microprogram upload.  Programs are native; skip the code words.
                    let n = if num == 0 { 256 } else { num };
                    p += n * 8;
                }
                0x50 | 0x51 => {
                    let n = if imm == 0 { 65536 } else { imm as usize };
                    let end = (p + n * 16).min(d.len());
                    gif.process(frame, &d[p..end]);
                    p = end;
                }
                c if c & 0x60 == 0x60 => {
                    let t = crate::programs::tstart();
                    p = self.unpack(vu, d, p, c, imm, num);
                    crate::programs::tadd(&crate::programs::UNPACK_NS, t);
                }
                _ => {}
            }
        }
    }

    fn unpack(&mut self, vu: &mut Vu1, d: &[u8], mut p: usize, cmd: u32, imm: u32, num: usize) -> usize {
        let vn = ((cmd >> 2) & 3) as usize;
        let vl = (cmd & 3) as usize;
        let n = if num == 0 { 256 } else { num };
        let flg = (imm >> 15) & 1 != 0;
        let usn = (imm >> 14) & 1 != 0;
        let mut addr = (imm & 0x3ff) as usize;
        if flg {
            addr += self.tops as usize;
        }
        let comps = vn + 1;
        let (cl, wl) = (self.cl.max(1) as usize, self.wl.max(1) as usize);
        let start = p;
        // Fast path for the common case: no masking, no row addition, elements stored back to back.  32-bit elements are
        // copied word for word; 8 and 16 bit ones are widened.
        if self.mask == 0 && self.mode == 0 && (cl == wl || (cl >= wl && wl == 1 && cl == 1)) && !(vn == 3 && vl == 3) {
            let bytes_per = [4usize, 2, 1, 0][vl] * comps;
            if bytes_per != 0 && start + n * bytes_per <= d.len() && addr + n <= VU_MEM_QW {
                let src = &d[start..start + n * bytes_per];
                for i in 0..n {
                    let e = &src[i * bytes_per..(i + 1) * bytes_per];
                    let mut tmp = [0u32; 4];
                    match vl {
                        0 => {
                            for c in 0..comps {
                                tmp[c] = u32::from_le_bytes([e[4 * c], e[4 * c + 1], e[4 * c + 2], e[4 * c + 3]]);
                            }
                        }
                        1 => {
                            for c in 0..comps {
                                let v = u16::from_le_bytes([e[2 * c], e[2 * c + 1]]);
                                tmp[c] = if usn { v as u32 } else { v as i16 as i32 as u32 };
                            }
                        }
                        _ => {
                            for c in 0..comps {
                                tmp[c] = if usn { e[c] as u32 } else { e[c] as i8 as i32 as u32 };
                            }
                        }
                    }
                    vu.mem[addr + i] = match vn {
                        0 => [tmp[0]; 4],
                        1 => [tmp[0], tmp[1], 0, 0],
                        2 => [tmp[0], tmp[1], tmp[2], 0],
                        _ => tmp,
                    };
                }
                return start + ((n * bytes_per + 3) & !3);
            }
        }
        let read = |p: &mut usize| -> u32 {
            match vl {
                0 => {
                    let v = rd32(d, *p);
                    *p += 4;
                    v
                }
                1 => {
                    let v = if *p + 2 <= d.len() { u16::from_le_bytes([d[*p], d[*p + 1]]) } else { 0 };
                    *p += 2;
                    if usn { v as u32 } else { v as i16 as i32 as u32 }
                }
                _ => {
                    let v = if *p < d.len() { d[*p] } else { 0 };
                    *p += 1;
                    if usn { v as u32 } else { v as i8 as i32 as u32 }
                }
            }
        };
        for i in 0..n {
            let mut v = [0u32; 4];
            if vn == 3 && vl == 3 {
                // V4-5: RGBA5551
                let c = if p + 2 <= d.len() { u16::from_le_bytes([d[p], d[p + 1]]) as u32 } else { 0 };
                p += 2;
                v = [(c & 0x1f) << 3, ((c >> 5) & 0x1f) << 3, ((c >> 10) & 0x1f) << 3, ((c >> 15) & 1) << 7];
            } else {
                let mut tmp = [0u32; 4];
                for t in tmp.iter_mut().take(comps) {
                    *t = read(&mut p);
                }
                v = match vn {
                    0 => [tmp[0]; 4],
                    1 => [tmp[0], tmp[1], 0, 0],
                    2 => [tmp[0], tmp[1], tmp[2], 0],
                    _ => tmp,
                };
            }
            let dst = if cl >= wl { addr + (i / wl) * cl + (i % wl) } else { addr + i };
            let j = if cl >= wl { (i % wl).min(3) } else { i.min(3) };
            if dst < VU_MEM_QW {
                for k in 0..4 {
                    let sel = if self.mask == 0 { 0 } else { (self.mask >> (2 * (4 * j + k))) & 3 };
                    match sel {
                        0 => {
                            let mut x = v[k];
                            match self.mode {
                                1 => x = x.wrapping_add(self.row[k]),
                                2 => {
                                    self.row[k] = self.row[k].wrapping_add(x);
                                    x = self.row[k];
                                }
                                _ => {}
                            }
                            vu.mem[dst][k] = x;
                        }
                        1 => vu.mem[dst][k] = self.row[k],
                        2 => vu.mem[dst][k] = self.col[j],
                        _ => {}
                    }
                }
            }
        }
        // data is padded to a word boundary
        let used = p - start;
        start + ((used + 3) & !3)
    }
}
