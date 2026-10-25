//! Stack frames for decompiled functions whose locals have their address taken (arrays/structs passed by pointer).
//! Frames live in the game address space so that such pointers are valid game pointers.

use crate::mem::STACK_TOP;
use std::cell::Cell;

thread_local! {
    static SP: Cell<u32> = const { Cell::new(STACK_TOP - 16) };
}

pub struct Frame {
    base: u32,
    size: u32,
}

impl Frame {
    pub fn new(size: u32) -> Frame {
        let size = (size + 15) & !15;
        let base = SP.with(|s| {
            let b = s.get() - size;
            s.set(b);
            b
        });
        Frame { base, size }
    }

    #[inline(always)]
    pub fn sp(&self) -> i32 {
        self.base as i32
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        SP.with(|s| s.set(s.get() + self.size));
    }
}

/// Move the stack of the thread (the verifier does this to run a function on a captured memory image, whose stack is in use above it).
pub fn set_sp(sp: u32) {
    SP.with(|s| s.set(sp));
}

/// Current native stack pointer (lowest address in use by live frames).
pub fn current_sp() -> u32 {
    SP.with(|s| s.get())
}
