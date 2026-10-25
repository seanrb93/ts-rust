//! Hook points for differential testing inside the running game (feature `shadow`): every decompiled function
//! announces its entry arguments; a registered hook may run the original machine code on the same state and compare.
use std::sync::OnceLock;

/// Called at function entry; the returned closure runs when the function returns.
pub type Hook = fn(u32, &'static str, [i64; 8], [f32; 8]) -> Option<Box<dyn FnOnce()>>;

static HOOK: OnceLock<Hook> = OnceLock::new();

pub fn set_hook(h: Hook) {
    let _ = HOOK.set(h);
}

pub struct Guard(Option<Box<dyn FnOnce()>>);

impl Drop for Guard {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f();
        }
    }
}

#[inline]
pub fn enter(addr: u32, name: &'static str, ints: [i64; 8], floats: [f32; 8]) -> Guard {
    match HOOK.get() {
        Some(h) => Guard(h(addr, name, ints, floats)),
        None => Guard(None),
    }
}

/// Called at the start of a platform routine (the Sony SDK, the sound and pad servers; the development build puts the call there): a hook may answer instead of the routine, so that
/// the original code and the port see the same replies and the calls they make can be compared.
pub type ExternHook = fn(&'static str, &[i64]) -> Option<i64>;

static EXTERN_HOOK: OnceLock<ExternHook> = OnceLock::new();

pub fn set_extern_hook(h: ExternHook) {
    let _ = EXTERN_HOOK.set(h);
}

#[inline]
pub fn extern_hook(name: &'static str, args: &[i64]) -> Option<i64> {
    EXTERN_HOOK.get().and_then(|h| h(name, args))
}
