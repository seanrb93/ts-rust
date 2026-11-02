pub mod asm;
pub mod cd;
pub mod gfx;
pub mod gs;
pub mod kernel;
pub mod levels;
pub mod libc;
pub mod math;
pub mod misc;
pub mod net;
pub mod mc;
pub mod pad;
pub mod scanf;
pub mod sif;
pub mod softfloat;
pub mod printf;
pub mod vu0;

pub use asm::*;
pub use cd::*;
pub use gfx::*;
pub use gs::*;
pub use kernel::*;
pub use levels::*;
pub use libc::*;
pub use math::*;
pub use misc::*;
pub use mc::*;
pub use pad::*;
pub use scanf::*;
pub use sif::*;
pub use softfloat::*;
pub use printf::*;
pub use vu0::*;

/// A test aid (`TS_NATIVELOG=1`): a native service the game called while a picture between ticks was being made.
pub(crate) fn note(name: &str) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *ON.get_or_init(|| std::env::var_os("TS_NATIVELOG").is_some()) && net::muted() {
        eprintln!("[native] {name}");
    }
}
