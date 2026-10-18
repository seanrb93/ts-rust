//! Render front-end for the decompiled TimeSplitters.
//!
//! The engine describes each frame as a DMA display list of VIF packets.  This crate walks that list, unpacks the
//! vertex data into VU1 memory, runs the (natively ported) microprograms and interprets the resulting GIF vertex
//! streams into a backend-independent `Frame` of draw calls that a GPU renderer draws at any resolution.

#[macro_use]
pub mod trace;
pub mod gif;
pub mod hd;
pub mod depth;
pub mod png;
pub mod programs;
pub mod scene;
pub mod texture;
pub mod vif;
pub mod vram;
pub mod vu;
pub mod vu1_gen;
#[cfg(feature = "vucheck")]
pub mod vu_ref;
#[cfg(feature = "vucheck")]
pub mod vu1_gen_ref;

use gif::Gif;
use scene::Frame;
use ts_rt::*;
use vif::{Vif, Vu1};
use vu::Regs;

pub struct Gfx {
    vu: Box<Vu1>,
    regs: Box<Regs>,
    vif: Vif,
    gif: Gif,
    /// decoded textures by key, reused while their texture memory is unchanged
    tex_cache: std::collections::HashMap<scene::TexKey, std::sync::Arc<scene::DecodedTex>>,
    /// the VIF byte stream of the frame being translated (kept between frames so that it is not allocated every time)
    stream_buf: Vec<u8>,
    /// size of the previous frame's draw list: the next one is allocated with room for about as much
    last_sizes: (usize, usize),
}

impl Default for Gfx {
    fn default() -> Self {
        Gfx { vu: Vu1::new(), regs: Box::default(), vif: Vif::default(), gif: Gif::new(), tex_cache: Default::default(), stream_buf: Vec::new(), last_sizes: (0, 0) }
    }
}

impl Gfx {
    pub fn new() -> Gfx {
        Gfx::default()
    }

    /// Append one DMA payload to the VIF input stream.  The VIF sees a single continuous byte stream: a command
    /// (e.g. UNPACK) may begin in one DMA packet and its data continue in the next.
    fn packet(&mut self, stream: &mut Vec<u8>, addr: i32, qwc: usize) {
        if qwc == 0 {
            return;
        }
        let d = host_slice(addr, qwc * 16);
        if trace::enabled() {
            let mut line = format!("PACKET @{addr:#x} qwc={qwc}:");
            for q in 0..qwc.min(16) {
                line += &format!(
                    " [{:08x} {:08x} {:08x} {:08x}]",
                    lw(addr + q as i32 * 16) as u32,
                    lw(addr + q as i32 * 16 + 4) as u32,
                    lw(addr + q as i32 * 16 + 8) as u32,
                    lw(addr + q as i32 * 16 + 12) as u32
                );
            }
            trace::line(line);
        }
        stream.extend_from_slice(d);
    }

    /// Process the DMA chain beginning at `head` and return the frame's draw list.
    pub fn run_list(&mut self, head: i32, world: &[(i32, i32)]) -> Frame {
        vu::enter_vu_mode();
        let t_stream = programs::tstart();
        let mut frame = Frame::default();
        frame.vertices.reserve(self.last_sizes.0 + self.last_sizes.0 / 8 + 4096);
        frame.draws.reserve(self.last_sizes.1 + self.last_sizes.1 / 8 + 256);
        // (stream offset, tag is inside a world-sprite range): lets the VIF attribute draws to the tag that produced them
        let mut marks: Vec<(usize, bool)> = vec![(0, false)];
        let mut stream = std::mem::take(&mut self.stream_buf);
        stream.clear();
        let mut stack: Vec<i32> = Vec::new();
        let mut tag = head;
        for _ in 0..200_000 {
            let w0 = lw(tag) as u32;
            let addr = (lw(tag + 4) as u32 & 0x7fff_ffff) as i32;
            let qwc = (w0 & 0xffff) as usize;
            let id = (w0 >> 28) & 7;
            // (the game clears the channel's TTE bit, so the VIF codes in the tag's upper half are not transferred)
            let body = tag + 16;
            let in_world = !world.is_empty() && world.iter().any(|&(a, b)| tag >= a && tag < b);
            if marks.last().map(|m| m.1) != Some(in_world) {
                marks.push((stream.len(), in_world));
            }
            let next = match id {
                0 => {
                    self.packet(&mut stream, addr, qwc);
                    break;
                }
                1 => {
                    self.packet(&mut stream, body, qwc);
                    body + (qwc as i32) * 16
                }
                2 => {
                    self.packet(&mut stream, body, qwc);
                    addr
                }
                3 | 4 => {
                    self.packet(&mut stream, addr, qwc);
                    body
                }
                5 => {
                    self.packet(&mut stream, body, qwc);
                    stack.push(body + (qwc as i32) * 16);
                    addr
                }
                6 => {
                    self.packet(&mut stream, body, qwc);
                    match stack.pop() {
                        Some(r) => r,
                        None => break,
                    }
                }
                _ => {
                    self.packet(&mut stream, body, qwc);
                    break;
                }
            };
            tag = next;
        }
        programs::tadd(&programs::STREAM_NS, t_stream);
        self.vif.process(&mut self.vu, &mut self.regs, &mut self.gif, &mut frame, &stream, &marks);
        self.stream_buf = stream;
        let t_tex = programs::tstart();
        // snapshot the textures the frame uses (texture memory changes while later frames are built)
        let keys: std::collections::HashSet<scene::TexKey> = frame.draws.iter().filter_map(|d| d.state.tex).collect();
        vram::with_vram(|v| {
            static TEXLOG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
            let texlog = *TEXLOG.get_or_init(|| std::env::var_os("TS_TEXLOG").is_some());
            for k in keys {
                if texlog {
                    let exact = v.images.get(&k.tbp0).is_some();
                    match v.find(k.tbp0) {
                        Some(i) => {
                            let (tw, th) = (1u32 << k.tw.min(11), 1u32 << k.th.min(11));
                            let bpp = match k.psm { 0 => 32, 1 => 24, 2 | 10 => 16, 19 => 8, 20 => 4, _ => 32 };
                            let need = (tw * th * bpp / 8) as usize;
                            if !exact || i.w != tw || i.h != th || i.data.len() != need || i.psm != k.psm {
                                eprintln!("[texlog] key tbp0 {:#x} psm {} {}x{} tbw {} | image dbp {:#x} {}x{} psm {} dbw {} len {} need {} exact {}", k.tbp0, k.psm, tw, th, k.tbw, i.dbp, i.w, i.h, i.psm, i.dbw, i.data.len(), need, exact);
                            }
                        }
                        None => eprintln!("[texlog] key tbp0 {:#x} psm {} has no image", k.tbp0, k.psm),
                    }
                }
                let Some((_, _, sig)) = texture::signature(&k, v) else { continue };
                let hit = self.tex_cache.get(&k).filter(|t| t.sig == sig).cloned();
                let t = match hit {
                    Some(t) => t,
                    None => match texture::decode(&k, v) {
                        Some((w, h, rgba, sig)) => {
                            let (lw, lh) = (1u32 << k.tw.min(11), 1u32 << k.th.min(11));
                            let t = std::sync::Arc::new(scene::DecodedTex { w, h, lw, lh, rgba, sig });
                            self.tex_cache.insert(k, t.clone());
                            t
                        }
                        None => continue,
                    },
                };
                frame.textures.insert(k, t);
            }
        });
        programs::tadd(&programs::TEX_NS, t_tex);
        self.last_sizes = (frame.vertices.len(), frame.draws.len());
        frame
    }
}
