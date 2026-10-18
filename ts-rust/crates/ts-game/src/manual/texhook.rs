//! The game's texture loader tells the renderer which file each texture came from.

use crate::*;

/// `emLoadTexture` has put the picture of the file `name` into texture memory at block address `tbp0`.  The renderer asks where a texture came
/// from, to show a replacement picture of a mod instead (`ts_rt::texnames`).  `parms` is the loader's description of the texture when it is
/// made from a picture in memory (then `name` is not a file's name).
pub fn tex_loaded(tbp0: i32, name: i32, parms: i32) {
    if parms != 0 && (lw(parms + 12) & 0x1000_0000) != 0 {
        return;
    }
    let mut s = String::new();
    let mut at = name;
    while lbu(at) != 0 && s.len() < 160 {
        s.push(lbu(at) as u8 as char);
        at += 1;
    }
    if std::env::var_os("TS_TEXNAMES").is_some() {
        eprintln!("[texname] {:#x} {s}", tbp0 & 0x3fff);
    }
    ts_rt::texnames::note_texture((tbp0 & 0x3fff) as u32, &s);
}
