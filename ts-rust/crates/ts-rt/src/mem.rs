//! The game address space: 32 MiB of main RAM (plus slack for native heap/stack) and the 16 KiB scratchpad.
//!
//! Accessor names follow the original load/store widths (`lw` = load word, `lbu` = load byte unsigned, ...).

use std::sync::atomic::{AtomicPtr, Ordering};

pub const RAM_SIZE: u32 = 0x1000_0000; // 256 MiB (the hardware registers start there): 32 MiB of PS2 RAM, the port's tables, the native heap, the big memory pool and the eternal overflow (ts-game bigpool.rs) and the stack
pub const SPR_BASE: u32 = 0x7000_0000;
pub const SPR_SIZE: u32 = 0x4000;
/// First address available to the native heap (`malloc`), above the game's 32 MiB.
pub const HEAP_BASE: u32 = 0x0200_0000;
pub const HEAP_END: u32 = 0x03c0_0000;
/// Native call-frame area grows downward from here.
pub const STACK_TOP: u32 = RAM_SIZE;

static RAM: AtomicPtr<u8> = AtomicPtr::new(std::ptr::null_mut());
static SPR: AtomicPtr<u8> = AtomicPtr::new(std::ptr::null_mut());
static MMIO: AtomicPtr<u8> = AtomicPtr::new(std::ptr::null_mut());
/// Hardware register windows (timers, DMA, GIF, VIF, GS privileged regs) are backed by plain scratch storage so a
/// stray access does not crash; nothing behind them is modelled.  Functions that poll hardware are replaced instead.
pub const MMIO_BASE: u32 = 0x1000_0000;
pub const MMIO_END: u32 = 0x1400_0000;
const MMIO_SIZE: usize = 0x10_0000;

/// Allocate the address space.  Must be called once before any game code runs.
pub fn init_memory() {
    if !RAM.load(Ordering::Relaxed).is_null() {
        return;
    }
    let ram = Box::leak(vec![0u8; RAM_SIZE as usize + 16].into_boxed_slice());
    let spr = Box::leak(vec![0u8; SPR_SIZE as usize + 16].into_boxed_slice());
    let mmio = Box::leak(vec![0u8; MMIO_SIZE + 16].into_boxed_slice());
    RAM.store(ram.as_mut_ptr(), Ordering::Relaxed);
    SPR.store(spr.as_mut_ptr(), Ordering::Relaxed);
    MMIO.store(mmio.as_mut_ptr(), Ordering::Relaxed);
}

#[cold]
#[inline(never)]
fn bad_addr(a: u32) -> ! {
    panic!("access to unmapped game address {a:#010x}");
}

#[inline(always)]
fn ptr(a: i32) -> *mut u8 {
    let a = a as u32;
    if a < RAM_SIZE {
        unsafe { RAM.load(Ordering::Relaxed).add(a as usize) }
    } else {
        ptr_slow(a)
    }
}

#[inline(never)]
fn ptr_slow(a: u32) -> *mut u8 {
    if (SPR_BASE..SPR_BASE + SPR_SIZE).contains(&a) {
        return unsafe { SPR.load(Ordering::Relaxed).add((a - SPR_BASE) as usize) };
    }
    if (MMIO_BASE..MMIO_END).contains(&a) {
        return unsafe { MMIO.load(Ordering::Relaxed).add(a as usize & (MMIO_SIZE - 1)) };
    }
    // uncached / accelerated mirrors of main RAM
    let m = a & 0x0fff_ffff;
    if matches!(a >> 28, 2 | 3 | 8 | 9 | 0xa | 0xb) && m < RAM_SIZE {
        return unsafe { RAM.load(Ordering::Relaxed).add(m as usize) };
    }
    bad_addr(a)
}

/// Raw pointer to a game address (for bulk copies from native services such as file I/O).
pub fn host_ptr(a: i32) -> *mut u8 {
    ptr(a)
}

pub fn host_slice(a: i32, len: usize) -> &'static mut [u8] {
    let _ = ptr(a.wrapping_add(len.saturating_sub(1) as i32));
    unsafe { std::slice::from_raw_parts_mut(ptr(a), len) }
}

/// Read a NUL-terminated string from game memory.
pub fn cstr(a: i32) -> String {
    let mut v = Vec::new();
    let mut p = a;
    loop {
        let c = lbu(p) as u8;
        if c == 0 {
            break;
        }
        v.push(c);
        p = p.wrapping_add(1);
    }
    String::from_utf8_lossy(&v).into_owned()
}

macro_rules! load {
    ($name:ident, $t:ty, $r:ty) => {
        #[inline(always)]
        pub fn $name(a: i32) -> $r {
            unsafe { (ptr(a) as *const $t).read_unaligned() as $r }
        }
    };
}
macro_rules! store {
    ($name:ident, $t:ty, $v:ty) => {
        #[inline(always)]
        pub fn $name(a: i32, v: $v) {
            let t: $t = v as $t;
            #[cfg(feature = "trace")]
            crate::trace::record(a, std::mem::size_of::<$t>(), &t as *const $t as *const u8);
            #[cfg(any(feature = "calltrace", feature = "watch"))]
            crate::watch::check(a, std::mem::size_of::<$t>(), t as u64);
            unsafe { (ptr(a) as *mut $t).write_unaligned(t) }
        }
    };
}

load!(lb, i8, i32);
load!(lbu, u8, i32);
load!(lh, i16, i32);
load!(lhu, u16, i32);
load!(lw, i32, i32);
load!(lwu, u32, i64);
load!(ld, i64, i64);
load!(lq, u128, u128);
load!(lwc1, f32, f32);
store!(sb, u8, i32);
store!(sh, u16, i32);
store!(sw, i32, i32);
store!(sd, i64, i64);
store!(sq, u128, u128);
#[inline(always)]
pub fn swc1(a: i32, v: f32) {
    #[cfg(feature = "trace")]
    crate::trace::record(a, 4, &v as *const f32 as *const u8);
    #[cfg(any(feature = "calltrace", feature = "watch"))]
    crate::watch::check(a, 4, v.to_bits() as u64);
    #[cfg(feature = "calltrace")]
    crate::calltrace::fstore(a, v);
    unsafe { (ptr(a) as *mut f32).write_unaligned(v) }
}

pub fn memcpy_game(dst: i32, src: i32, n: usize) {
    #[cfg(feature = "trace")]
    crate::trace::record_bulk(dst, n);
    unsafe { std::ptr::copy(ptr(src), ptr(dst), n) }
}

pub fn memset_game(dst: i32, c: u8, n: usize) {
    #[cfg(feature = "trace")]
    crate::trace::record_bulk(dst, n);
    unsafe { std::ptr::write_bytes(ptr(dst), c, n) }
}
