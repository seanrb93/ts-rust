//! The game, decompiled by hand.
//!
//! Every function here is readable source for a function of the original executable (`SLUS_200.90`) and replaces the
//! machine-translated version of the same name (`tools/gen_rust.py` skips it).  Rules, workflow and the list of what is
//! done are in `docs/decompilation.md`.  A function keeps the original's name, address (in its doc comment) and
//! signature, so the code that is still translated can call it.

#![allow(non_snake_case)] // functions keep the names the game gave them

pub mod bcd;
pub mod buffer;
pub mod debugflags;
pub mod empty;
pub mod enemies;
pub mod level;
pub mod list;
pub mod matrix;
pub mod boxes_guns;
pub mod misc;
pub mod numeric;
pub mod props;
pub mod random;
pub mod setters;
pub mod setters2;
pub mod setters3;
pub mod setters4;
pub mod setters5;
pub mod setters6;
pub mod setters7;
pub mod setters8;
pub mod setters9;
pub mod setters10;
pub mod setters11;
pub mod setters12;
pub mod setters13;
pub mod state;
pub mod stats;
pub mod types;
pub mod vec;

pub use bcd::*;
pub use buffer::*;
pub use debugflags::*;
pub use empty::*;
pub use enemies::*;
pub use level::*;
pub use list::*;
pub use matrix::*;
pub use boxes_guns::*;
pub use misc::*;
pub use numeric::*;
pub use props::*;
pub use random::*;
pub use setters::*;
pub use setters2::*;
pub use setters3::*;
pub use setters4::*;
pub use setters5::*;
pub use setters6::*;
pub use setters7::*;
pub use setters8::*;
pub use setters9::*;
pub use setters10::*;
pub use setters11::*;
pub use setters12::*;
pub use setters13::*;
pub use state::*;
pub use stats::*;
pub use types::*;
pub use vec::*;
