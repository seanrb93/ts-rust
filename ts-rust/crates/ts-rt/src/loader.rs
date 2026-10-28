//! The game's initialised data in the address space.
//!
//! The decompiled code addresses its global variables at their original addresses, so the executable's initialised `.rodata`/`.data`/`.sdata` contents (default values, string literals,
//! tables, the graphics microcode) are copied there once at start-up.  They are part of this repository (`data/game-data.bin`, written by `tools/extract-game-data.py`), so the program needs no
//! copy of the original executable, and none of its code is ever executed (every function of the game is Rust).  `load_elf_data` and `read_symbols` read the original executable and are for the
//! verifier and the tools.

use crate::mem::host_slice;
use std::path::Path;

/// The first address of the data (the end of the code of the executable) and the data itself.
pub const DATA_START: i32 = 0x2eeff0;
static GAME_DATA: &[u8] = include_bytes!("../data/game-data.bin");
static SYMBOLS: &str = include_str!("../data/symbols.tsv");

/// Put the game's initialised data at its addresses.
pub fn load_game_data() {
    host_slice(DATA_START, GAME_DATA.len()).copy_from_slice(GAME_DATA);
}

/// The names and addresses of the original executable's symbols (what a mod's `patches.txt` may name).
pub fn embedded_symbols() -> Vec<Symbol> {
    SYMBOLS
        .lines()
        .filter_map(|l| {
            let mut f = l.split('\t');
            let name = f.next()?.to_string();
            let addr = u32::from_str_radix(f.next()?.trim_start_matches("0x"), 16).ok()?;
            let size = f.next()?.parse().ok()?;
            let is_func = f.next()? == "f";
            Some(Symbol { name, addr, size, is_func })
        })
        .collect()
}

/// Parse the ELF32 program headers and copy every PT_LOAD segment's file contents to its virtual address.
pub fn load_elf_data(path: &Path) -> std::io::Result<u32> {
    let d = std::fs::read(path)?;
    let u16le = |o: usize| u16::from_le_bytes([d[o], d[o + 1]]) as usize;
    let u32le = |o: usize| u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]]) as usize;
    if &d[0..4] != b"\x7fELF" {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "not an ELF file"));
    }
    let entry = u32le(24);
    let phoff = u32le(28);
    let phentsize = u16le(42);
    let phnum = u16le(44);
    for i in 0..phnum {
        let ph = phoff + i * phentsize;
        let (typ, off, vaddr, filesz) = (u32le(ph), u32le(ph + 4), u32le(ph + 8), u32le(ph + 16));
        if typ == 1 && filesz > 0 {
            host_slice(vaddr as i32, filesz).copy_from_slice(&d[off..off + filesz]);
        }
    }
    Ok(entry as u32)
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub addr: u32,
    pub size: u32,
    pub is_func: bool,
}

/// Read the executable's symbol table (names are only used for tooling, never at game run time).
pub fn read_symbols(path: &Path) -> std::io::Result<Vec<Symbol>> {
    let d = std::fs::read(path)?;
    let u16le = |o: usize| u16::from_le_bytes([d[o], d[o + 1]]) as usize;
    let u32le = |o: usize| u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]]) as usize;
    let shoff = u32le(32);
    let shentsize = u16le(46);
    let shnum = u16le(48);
    let mut out = Vec::new();
    for i in 0..shnum {
        let sh = shoff + i * shentsize;
        if u32le(sh + 4) != 2 {
            continue;
        }
        let (off, size, link) = (u32le(sh + 16), u32le(sh + 20), u32le(sh + 24));
        let stroff = u32le(shoff + link * shentsize + 16);
        for k in 0..size / 16 {
            let e = off + k * 16;
            let (n, v, sz, info) = (u32le(e), u32le(e + 4), u32le(e + 8), d[e + 12]);
            let mut end = stroff + n;
            while d[end] != 0 {
                end += 1;
            }
            out.push(Symbol {
                name: String::from_utf8_lossy(&d[stroff + n..end]).into_owned(),
                addr: v as u32,
                size: sz as u32,
                is_func: info & 15 == 2,
            });
        }
    }
    Ok(out)
}
