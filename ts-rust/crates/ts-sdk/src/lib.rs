//! Native replacements for the Sony SDK, libc, libm and compiler-runtime functions the game links against.
//! Hand-written implementations live in `native/`; `stubs.rs` is generated and contains `todo!()` stubs for
//! everything the decompiled game calls that has no native implementation yet.

pub mod native;
pub mod stubs;

pub use native::*;
pub use stubs::*;
