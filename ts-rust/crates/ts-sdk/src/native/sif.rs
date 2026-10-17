//! Sub-system interface to the IOP (the PS2's I/O co-processor).  The IOP ran the sound / streaming / pad / memory
//! card servers; on PC none of those exist, so RPC binds and calls succeed immediately and return empty replies.
//! Audio is replaced by a native engine (see `ts-port`), not by emulating the IOP modules.
#![allow(non_snake_case)]

use ts_rt::*;

pub fn sceSifInitRpc(_args: &[i64]) -> i32 {
    0
}

/// RPC server ids by client address (set by `sceSifBindRpc`).
static BOUND: std::sync::Mutex<Vec<(i32, u32)>> = std::sync::Mutex::new(Vec::new());

const SID_STREAM: u32 = 0x534a_4521; // "SJE!" music stream driver

/// `sceSifBindRpc(client, rpc_id, mode)`: mark the (virtual) server as bound.
pub fn sceSifBindRpc(args: &[i64]) -> i32 {
    let client = args[0] as i32;
    let sid = args[1] as u32;
    let mut b = BOUND.lock().unwrap();
    b.retain(|(c, _)| *c != client);
    b.push((client, sid));
    sw(client + 36, 1); // `serve` field: non-zero once bound
    0
}

/// `sceSifCallRpc(client, rpc, mode, send, ssize, recv, rsize, end_func, end_param)`
/// The music stream server is implemented natively by `ts-audio`; every other server just returns zeros.
pub fn sceSifCallRpc(args: &[i64]) -> i32 {
    use ts_audio::MusicCmd;
    let client = args[0] as i32;
    let cmd = args[1] as u32;
    let send = args.get(3).copied().unwrap_or(0) as i32;
    let (recv, rsize) = (args.get(5).copied().unwrap_or(0) as i32, args.get(6).copied().unwrap_or(0) as i32);
    let sid = BOUND.lock().unwrap().iter().find(|(c, _)| *c == client).map(|x| x.1).unwrap_or(0);
    let mut reply = 0;
    if sid == SID_STREAM {
        let arg = if send != 0 { lw(send) as u32 } else { 0 };
        reply = match cmd {
            0x8020 => {
                let name = cstr(send);
                match crate::native::cd::disc_host_path(&name) {
                    Some(p) => ts_audio::stream_command(MusicCmd::Open(p)),
                    None => {
                        eprintln!("[audio] cannot open music file {name}");
                        0
                    }
                }
            }
            0x30 | 0x80f0 => ts_audio::stream_command(MusicCmd::Close),
            0x8050 | 0x8060 => ts_audio::stream_command(MusicCmd::Start),
            0x8070 => ts_audio::stream_command(MusicCmd::Stop),
            0x81 => ts_audio::stream_command(MusicCmd::SetLoop(arg != 0)),
            0xb0 | 0xc0 => ts_audio::stream_command(MusicCmd::Volume(arg >> 16, arg & 0xffff)),
            0x80e0 => ts_audio::stream_command(MusicCmd::Status),
            _ => 0,
        };
    }
    if recv != 0 && rsize > 0 {
        memset_game(recv, 0, rsize as usize);
        sw(recv, reply);
    }
    0
}

pub fn sceSifCheckStatRpc(_args: &[i64]) -> i32 {
    0
}

pub fn sceSifInitIopHeap(_args: &[i64]) -> i32 {
    0
}

/// Hand out distinct fake IOP RAM addresses.
pub fn sceSifAllocIopHeap(args: &[i64]) -> i32 {
    use std::sync::atomic::{AtomicI32, Ordering};
    static NEXT: AtomicI32 = AtomicI32::new(0x0010_0000);
    let size = (args.first().copied().unwrap_or(0) as i32 + 63) & !63;
    NEXT.fetch_add(size.max(64), Ordering::Relaxed)
}

pub fn sceSifLoadModule(_args: &[i64]) -> i32 {
    1
}
pub fn sceSifRebootIop(_args: &[i64]) -> i32 {
    1
}
pub fn sceSifSyncIop(_args: &[i64]) -> i32 {
    1
}
pub fn sceSdRemoteInit(_args: &[i64]) -> i32 {
    0
}

/// `sceSdTransToIOP(ee_addr, iop_addr, size, mode)`: SIF DMA from game memory to IOP RAM.
pub fn sceSdTransToIOP(args: &[i64]) -> i32 {
    let (ee, iop, size) = (args[0] as i32, args[1] as u32, args[2] as usize);
    ts_audio::iop_write(iop, host_slice(ee, size));
    0
}

/// `sceSdRemote(sync, rpc, ...)`: libsd calls, served by the native SPU2-style mixer.
pub fn sceSdRemote(args: &[i64]) -> i32 {
    let a = |i: usize| args.get(i).copied().unwrap_or(0) as u32;
    match a(1) {
        0x8010 => ts_audio::sd_set_param(a(2), a(3)),
        0x8020 => return ts_audio::sd_get_param(a(2)) as i32,
        0x8030 => ts_audio::sd_set_switch(a(2), a(3)),
        0x8040 => return ts_audio::sd_get_switch(a(2)) as i32,
        0x8050 => ts_audio::sd_set_addr(a(2), a(3)),
        0x8070 => ts_audio::sd_set_core_attr(a(2), a(3)),
        0x80b0 => ts_audio::sd_proc_batch(a(2), a(4)),
        0x80d0 => {
            ts_audio::sd_voice_trans(a(4), a(5), a(6));
            return a(6) as i32;
        }
        _ => {}
    }
    0
}
