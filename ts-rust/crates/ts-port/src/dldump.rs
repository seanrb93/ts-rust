//! Developer tool: walk a finished display list and print what the game emitted.
//! (DMA tag chain -> VIF codes -> GIF packets.)  Used to understand the draw-call formats the native renderer must
//! reproduce; it is not part of rendering.

use ts_rt::*;

fn vif_name(cmd: u32) -> String {
    match cmd {
        0x00 => "NOP".into(),
        0x01 => "STCYCL".into(),
        0x02 => "OFFSET".into(),
        0x03 => "BASE".into(),
        0x04 => "ITOP".into(),
        0x05 => "STMOD".into(),
        0x06 => "MSKPATH3".into(),
        0x07 => "MARK".into(),
        0x10 => "FLUSHE".into(),
        0x11 => "FLUSH".into(),
        0x13 => "FLUSHA".into(),
        0x14 => "MSCAL".into(),
        0x15 => "MSCALF".into(),
        0x17 => "MSCNT".into(),
        0x20 => "STMASK".into(),
        0x30 => "STROW".into(),
        0x31 => "STCOL".into(),
        0x4a => "MPG".into(),
        0x50 => "DIRECT".into(),
        0x51 => "DIRECTHL".into(),
        c if c & 0x60 == 0x60 => format!("UNPACK(fmt={:#x})", c & 0xf),
        c => format!("VIF?{c:#x}"),
    }
}

/// Decode and print VIF codes in `n` qwords starting at `addr`.  Returns after consuming the data blocks.
fn dump_vif(addr: i32, qwc: u32, out: &mut String, depth: usize) {
    let mut p = addr;
    let end = addr + (qwc as i32) * 16;
    let pad = "    ".repeat(depth);
    while p < end {
        let code = lw(p) as u32;
        let cmd = (code >> 24) & 0x7f;
        let imm = code & 0xffff;
        let num = (code >> 16) & 0xff;
        p += 4;
        match cmd {
            0x4a => {
                // MPG: num instructions (0 = 256), 8 bytes each
                let n = if num == 0 { 256 } else { num };
                out.push_str(&format!("{pad}  MPG addr={imm:#x} n={n}\n"));
                p += (n as i32) * 8;
            }
            0x50 | 0x51 => {
                let n = if imm == 0 { 65536 } else { imm };
                out.push_str(&format!("{pad}  DIRECT qwc={n} -> GIF\n"));
                dump_gif(p, n, out, depth + 1);
                p += (n as i32) * 16;
            }
            c if c & 0x60 == 0x60 => {
                let fmt = c & 0xf;
                let n = if num == 0 { 256 } else { num };
                let bits = match fmt {
                    0 => 32 * 4,
                    1 => 16 * 4,
                    2 => 8 * 4,
                    3 => 16,
                    4 => 32 * 3,
                    5 => 16 * 3,
                    6 => 8 * 3,
                    7 => 16,
                    8 => 32 * 2,
                    9 => 16 * 2,
                    10 => 8 * 2,
                    12 => 32,
                    13 => 16,
                    14 => 8,
                    15 => 16,
                    _ => 32,
                };
                let bytes = ((n * bits + 31) / 32) * 4;
                out.push_str(&format!(
                    "{pad}  UNPACK fmt={fmt:#x}(V{}-{}) num={n} addr={:#x} flg={} usn={} -> {bytes} bytes\n",
                    fmt >> 2,
                    [32, 16, 8, 5][(fmt & 3) as usize],
                    imm & 0x3ff,
                    (imm >> 15) & 1,
                    (imm >> 14) & 1
                ));
                p += (bytes as i32 + 3) & !3;
            }
            0x01 | 0x03 | 0x04 | 0x05 | 0x02 | 0x07 | 0x14 | 0x15 | 0x17 | 0x00 | 0x10 | 0x11 | 0x13 | 0x06 => {
                if cmd != 0 {
                    out.push_str(&format!("{pad}  {} imm={imm:#x}\n", vif_name(cmd)));
                }
            }
            0x20 | 0x30 | 0x31 => {
                out.push_str(&format!("{pad}  {}\n", vif_name(cmd)));
                p += if cmd == 0x20 { 4 } else { 16 };
            }
            _ => {
                out.push_str(&format!("{pad}  {} raw={code:#010x}\n", vif_name(cmd)));
            }
        }
    }
}

fn dump_gif(addr: i32, qwc: u32, out: &mut String, depth: usize) {
    let mut p = addr;
    let end = addr + (qwc as i32) * 16;
    let pad = "    ".repeat(depth);
    let mut tags = 0;
    while p < end && tags < 4000 {
        let lo = ld(p) as u64;
        let regs = ld(p + 8) as u64;
        let nloop = (lo & 0x7fff) as u32;
        let eop = (lo >> 15) & 1;
        let pre = (lo >> 46) & 1;
        let prim = (lo >> 47) & 0x7ff;
        let flg = (lo >> 58) & 3;
        let mut nreg = ((lo >> 60) & 0xf) as u32;
        if nreg == 0 {
            nreg = 16;
        }
        out.push_str(&format!(
            "{pad}GIFTAG nloop={nloop} eop={eop} flg={flg} nreg={nreg} regs={regs:#x}{}\n",
            if pre != 0 { format!(" PRIM={prim:#x}") } else { String::new() }
        ));
        p += 16;
        tags += 1;
        let data_qw = match flg {
            0 => (nloop * nreg).div_ceil(2),
            1 => (nloop * nreg).div_ceil(2),
            _ => nloop,
        };
        if flg == 0 && nreg == 1 && (regs & 0xf) == 0xe {
            // A+D: print register writes
            for k in 0..nloop.min(24) {
                let v = ld(p + (k as i32) * 16) as u64;
                let a = ld(p + (k as i32) * 16 + 8) as u64 & 0xff;
                out.push_str(&format!("{pad}  A+D reg={a:#04x}{} data={v:#018x}\n", reg_name(a as u32)));
            }
            if nloop > 24 {
                out.push_str(&format!("{pad}  ... {} more\n", nloop - 24));
            }
        }
        p += (data_qw as i32) * 16;
        if eop != 0 {
            break;
        }
    }
}

fn reg_name(a: u32) -> String {
    let n = match a {
        0x00 => "PRIM",
        0x01 => "RGBAQ",
        0x02 => "ST",
        0x03 => "UV",
        0x04 => "XYZF2",
        0x05 => "XYZ2",
        0x06 => "TEX0_1",
        0x08 => "CLAMP_1",
        0x0a => "FOG",
        0x14 => "TEX1_1",
        0x18 => "XYOFFSET_1",
        0x1a => "PRMODECONT",
        0x34 => "MIPTBP1_1",
        0x3b => "TEXA",
        0x3d => "TEXCLUT",
        0x40 => "SCISSOR_1",
        0x42 => "ALPHA_1",
        0x44 => "COLCLAMP/DIMX",
        0x45 => "DTHE",
        0x46 => "COLCLAMP",
        0x47 => "TEST_1",
        0x48 => "PABE",
        0x4c => "FRAME_1",
        0x4e => "ZBUF_1",
        0x50 => "BITBLTBUF",
        0x51 => "TRXPOS",
        0x52 => "TRXREG",
        0x53 => "TRXDIR",
        0x3f => "TEXFLUSH",
        _ => return String::new(),
    };
    format!(" ({n})")
}

/// Walk the DMA chain at `head` and return a text description.
pub fn dump_list(head: i32, max_tags: usize) -> String {
    let mut out = String::new();
    let mut stack: Vec<i32> = Vec::new();
    let mut tag = head;
    for n in 0..max_tags {
        let w0 = lw(tag) as u32;
        let addr = (lw(tag + 4) as u32 & 0x7fff_ffff) as i32;
        let qwc = w0 & 0xffff;
        let id = (w0 >> 28) & 7;
        let name = ["refe", "cnt", "next", "ref", "refs", "call", "ret", "end"][id as usize];
        let v0 = lw(tag + 8) as u32;
        let v1 = lw(tag + 12) as u32;
        out.push_str(&format!("#{n} tag@{tag:#x} {name} qwc={qwc} addr={addr:#x} vif=[{v0:#x},{v1:#x}]\n"));
        // the two VIF codes carried in the tag
        for v in [v0, v1] {
            let c = (v >> 24) & 0x7f;
            if c != 0 {
                out.push_str(&format!("      tagVIF {}\n", vif_name(c)));
            }
        }
        let next = match id {
            0 => {
                dump_vif(addr, qwc, &mut out, 1);
                break;
            }
            1 => {
                dump_vif(tag + 16, qwc, &mut out, 1);
                tag + 16 + (qwc as i32) * 16
            }
            2 => {
                dump_vif(tag + 16, qwc, &mut out, 1);
                addr
            }
            3 => {
                dump_vif(addr, qwc, &mut out, 1);
                tag + 16
            }
            5 => {
                stack.push(tag + 16 + (qwc as i32) * 16);
                dump_vif(tag + 16, qwc, &mut out, 1);
                addr
            }
            6 => match stack.pop() {
                Some(r) => r,
                None => break,
            },
            _ => break,
        };
        tag = next;
    }
    out
}
