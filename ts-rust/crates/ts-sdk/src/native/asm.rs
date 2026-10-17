//! Game-side routines that were written in assembly (EE multimedia instructions, stack tricks, variadics) and so
//! cannot be decompiled mechanically.  Each is a faithful port of the original instruction sequence.
#![allow(non_snake_case)]

use ts_rt::*;

/// Where the original keeps its scratch formatting buffer.
const VAFMT_BUF: i32 = 0x01fe_afe0;

/// `vafmt(fmt, ...)`: vsprintf into a fixed global buffer; returns the buffer.
pub fn vafmt(args: &[i64]) -> i32 {
    let mut a = Vec::with_capacity(args.len() + 1);
    a.push(VAFMT_BUF as i64);
    a.extend_from_slice(args);
    crate::native::printf::sprintf(&a);
    VAFMT_BUF
}

/// `vtxlistcolour(src, dst, count, c, f)`: per vertex colour (4 packed bytes), blend towards `src` by factor `f`,
/// exactly as the original's PEXTLB / PSUBH / PMULTH / PSRLH / PADDH / PMINH / PPACB sequence computed it:
/// `out = min(c + (((src - c) * f) & 0xffff) >> 8, src)` per channel.
pub fn vtxlistcolour(args: &[i64]) -> i32 {
    let (mut src, mut dst, mut n) = (args[0] as i32, args[1] as i32, args[2] as i32);
    let c = (args[3] as u32).to_le_bytes();
    let f = (args[4] as u32).to_le_bytes();
    loop {
        let s = (lw(src) as u32).to_le_bytes();
        let mut o = [0u8; 4];
        for k in 0..4 {
            let d = s[k] as i16 - c[k] as i16;
            let p = d as i32 * f[k] as i32;
            let low = ((p as u32 & 0xffff) >> 8) as i16;
            o[k] = (c[k] as i16).wrapping_add(low).min(s[k] as i16) as u8;
        }
        sw(dst, u32::from_le_bytes(o) as i32);
        src = src.wrapping_add(4);
        dst = dst.wrapping_add(4);
        n = n.wrapping_sub(1);
        if n == 0 {
            break;
        }
    }
    0
}

/// `bubbleSort(entries, out, n)`: `entries` are 8-byte records whose first word is an int key; writes to `out[i]`
/// the address of the record with the i-th largest key (first wins on ties; records with key == i32::MIN lose to
/// record 0).
pub fn bubbleSort(args: &[i64]) -> i32 {
    let (base, out, n) = (args[0] as i32, args[1] as i32, args[2] as i32);
    if n <= 0 {
        return 0;
    }
    let mut used = vec![false; n as usize];
    for i in 0..n {
        let mut best = i32::MIN;
        let mut idx = 0usize;
        for j in 0..n as usize {
            if used[j] {
                continue;
            }
            let k = lw(base.wrapping_add(8 * j as i32));
            if best < k {
                best = k;
                idx = j;
            }
        }
        sw(out.wrapping_add(4 * i), base.wrapping_add(8 * idx as i32));
        used[idx] = true;
    }
    0
}
