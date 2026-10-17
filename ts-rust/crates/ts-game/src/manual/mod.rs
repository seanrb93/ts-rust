//! Hand-written replacements for functions the decompiler cannot translate automatically, and the platform glue.
//! A function defined here wins over the generated one of the same name (the generator skips it).

pub mod avmenu;
pub mod boot;
pub mod boot_gen;
pub mod chrnames;
pub mod debugmenu;
pub mod gs;
pub mod hooks;
pub mod mmoptions;
pub mod mouselook;
pub mod nativetext;
pub mod soundmix;
pub mod dualwield;
pub mod botsets;
pub mod randomiser;
pub mod weaponsets;
pub mod weapontable;

pub use avmenu::*;
pub use boot::*;
pub use boot_gen::*;
pub use chrnames::*;
pub use debugmenu::*;
pub use gs::*;
pub use mmoptions::*;
pub use mouselook::*;
pub use nativetext::*;
pub use soundmix::*;
pub use dualwield::*;
pub use botsets::*;
pub use randomiser::*;
pub use weaponsets::*;
pub use weapontable::*;
