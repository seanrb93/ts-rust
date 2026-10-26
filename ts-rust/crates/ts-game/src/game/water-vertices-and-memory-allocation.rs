//! The moving surface of water, and the memory allocator.

use crate::*;

/// Make `count` water vertices at `out` from the vertices at `source` (three floats each): x and z stay, y gets two sines added, one of the vertex's x and the game's time and one
/// of its z and the time (with the water's periods, speeds and amplitude), times the vertex's own factor from the table `waterMemory[table]`, less the factor times the water's
/// reduction (@ 0x278888).
pub fn vtxlistWaterxyz(source: i32, out: i32, count: i32, table: i32) {
    let mut from = source;
    let mut to = out;
    let mut i = 0;
    if count > 0 {
        let now = lw(g::gameframe);
        loop {
            let x = lwc1(from);
            let time = gameframe_x();
            let z = lwc1(from + 8);
            swc1(to, x);
            let at = i << 2;
            let k = lwc1(0x3ace6c);
            let sines = g::g_SinTable;
            i += 1;
            let half = 0.5_f32;
            let amplitude = lwc1(g::wateramplitude);
            let y = lwc1(from + 4);
            let factor = lwc1(at + lw((table << 2) + g::waterMemory));
            from += 12;
            let reduced = ps2float::mul(lwc1(g::waterreduction), factor);
            let wave_x = lwc1(((ps2float::add(ps2float::mul(ps2float::add(ps2float::mul(x, lwc1(g::waterperiodx)), ps2float::mul(time, lwc1(g::waterspeedx))), k), half) as i32 & 2047) << 2) + sines);
            let wave_z = lwc1(((ps2float::add(ps2float::mul(ps2float::add(ps2float::mul(z, lwc1(g::waterperiodz)), ps2float::mul(time, lwc1(g::waterspeedz))), k), half) as i32 & 2047) << 2) + sines);
            swc1(to + 8, z);
            swc1(to + 4, ps2float::sub(ps2float::add(y, ps2float::mul(ps2float::mul(ps2float::add(wave_x, wave_z), amplitude), factor)), reduced));
            to += 12;
            if i >= count {
                break;
            }
        }
    }
}

/// Allocate `size` bytes of the memory area `area` and return the address: 1 is the level's (taken from `mem_level_at` up), 2 the eternal (from `mem_eternal_at` up), 0 the local
/// (taken from the end of the level's free space down, with a 16-byte header "LOCAMEML", the size and the address); an area with too little left exits the game. Another area gives 0
/// (@ 0x201c38).
pub fn memAlloc(size: i32, area: i32) -> i32 {
    let at;
    if area == 1 && big_wants_x(size) {
        at = big_alloc_x(size);
    } else if area == 1 {
        let free = lw(g::mem_level_free);
        at = lw(g::mem_level_at);
        if free < size {
            exit(1);
        }
        sw(g::mem_level_free, free - size);
        sw(g::mem_level_at, at + size);
    } else if area >= 2 {
        let mut result = 0;
        if area == 2 {
            let free = lw(g::mem_eternal_free);
            result = lw(g::mem_eternal_at);
            if free < size {
                exit(1);
            }
            sw(g::mem_eternal_free, free - size);
            sw(g::mem_eternal_at, result + size);
        }
        at = result;
    } else {
        let mut result = 0;
        if area == 0 {
            let free = lw(g::mem_level_free);
            let total = size + 16;
            let mut local = lw(g::mem_local_at);
            if free < total {
                local = exit(1);
            }
            let header = local - total;
            sw(header, 0x41434f4c);
            result = header + 16;
            sw(header + 4, 0x4d454d4c);
            sw(g::mem_local_at, header);
            sw(header + 8, size);
            sw(g::mem_level_free, free - 16 - size);
            sw(header + 12, header);
        }
        at = result;
    }
    at
}
