//! C library functions operating on the game address space.
#![allow(non_snake_case)]

use std::sync::atomic::{AtomicI32, Ordering};
use ts_rt::*;

pub fn memset(a0: i32, a1: i32, a2: i32) -> i32 {
    memset_game(a0, a1 as u8, a2 as u32 as usize);
    a0
}

pub fn memcpy(a0: i32, a1: i32, a2: i32) -> i32 {
    memcpy_game(a0, a1, a2 as u32 as usize);
    a0
}

/// `bcopy(src, dst, n)`
pub fn bcopy(a0: i32, a1: i32, a2: i32) -> i32 {
    memcpy_game(a1, a0, a2 as u32 as usize);
    0
}

pub fn strlen(a0: i32) -> i32 {
    let mut n = 0;
    while lbu(a0.wrapping_add(n)) != 0 {
        n += 1;
    }
    n
}

pub fn strcpy(a0: i32, a1: i32) -> i32 {
    let mut i = 0;
    loop {
        let c = lbu(a1.wrapping_add(i));
        sb(a0.wrapping_add(i), c);
        if c == 0 {
            break;
        }
        i += 1;
    }
    a0
}

pub fn strncpy(a0: i32, a1: i32, a2: i32) -> i32 {
    let mut hit_nul = false;
    for i in 0..a2 {
        let c = if hit_nul { 0 } else { lbu(a1.wrapping_add(i)) };
        if c == 0 {
            hit_nul = true;
        }
        sb(a0.wrapping_add(i), c);
    }
    a0
}

pub fn strcat(a0: i32, a1: i32) -> i32 {
    strcpy(a0.wrapping_add(strlen(a0)), a1);
    a0
}

pub fn strcmp(a0: i32, a1: i32) -> i32 {
    let mut i = 0;
    loop {
        let (x, y) = (lbu(a0.wrapping_add(i)), lbu(a1.wrapping_add(i)));
        if x != y || x == 0 {
            return x - y;
        }
        i += 1;
    }
}

pub fn strncmp(a0: i32, a1: i32, a2: i32) -> i32 {
    for i in 0..a2 {
        let (x, y) = (lbu(a0.wrapping_add(i)), lbu(a1.wrapping_add(i)));
        if x != y || x == 0 {
            return x - y;
        }
    }
    0
}

pub fn strcasecmp(a0: i32, a1: i32) -> i32 {
    let mut i = 0;
    loop {
        let x = (lbu(a0.wrapping_add(i)) as u8).to_ascii_lowercase() as i32;
        let y = (lbu(a1.wrapping_add(i)) as u8).to_ascii_lowercase() as i32;
        if x != y || x == 0 {
            return x - y;
        }
        i += 1;
    }
}

pub fn strchr(a0: i32, a1: i32) -> i32 {
    let mut p = a0;
    loop {
        let c = lbu(p);
        if c == (a1 & 0xff) {
            return p;
        }
        if c == 0 {
            return 0;
        }
        p = p.wrapping_add(1);
    }
}

pub fn atoi(a0: i32) -> i32 {
    let s = cstr(a0);
    let t = s.trim_start();
    let (neg, rest) = match t.as_bytes().first() {
        Some(b'-') => (true, &t[1..]),
        Some(b'+') => (false, &t[1..]),
        _ => (false, t),
    };
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    let v: i64 = digits.parse().unwrap_or(0);
    (if neg { -v } else { v }) as i32
}

static HEAP_PTR: AtomicI32 = AtomicI32::new(HEAP_BASE as i32);

/// Simple bump allocator in the native heap region of the game address space.
pub fn malloc(a0: i32) -> i32 {
    let size = ((a0 as u32 + 15) & !15) as i32;
    let p = HEAP_PTR.fetch_add(size + 16, Ordering::Relaxed);
    if (p as u32 + size as u32) > HEAP_END {
        return 0;
    }
    sw(p, size);
    p + 16
}

pub fn free(_a0: i32) -> i32 {
    0
}

pub fn exit(a0: i32) -> i32 {
    eprintln!("[libc] the game called exit({a0})");
    #[cfg(feature = "calltrace")]
    ts_rt::calltrace::dump_now();
    std::process::exit(a0)
}

pub fn Exit(a0: i32) -> i32 {
    std::process::exit(a0)
}

pub fn __main() -> i32 {
    0
}
