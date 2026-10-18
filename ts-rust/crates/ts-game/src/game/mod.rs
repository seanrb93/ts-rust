//! The game, decompiled by hand.
//!
//! Every function here is readable source for a function of the original executable (`SLUS_200.90`) and replaces the
//! machine-translated version of the same name (`tools/gen_rust.py` skips it).  Rules, workflow and the list of what is
//! done are in `docs/decompilation.md`.  A function keeps the original's name, address (in its doc comment) and
//! signature, so the code that is still translated can call it.

pub mod bcd;
pub mod math;
pub mod random;
pub mod types;

pub use bcd::*;
pub use math::*;
pub use random::*;
pub use types::*;
