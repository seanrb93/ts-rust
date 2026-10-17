//! Loading the original game executable's initialised data into the address space.
//!
//! The decompiled code still addresses its global variables at their original addresses, so the executable's
//! initialised `.data`/`.rodata`/`.sdata` contents (default values, string literals, tables) are copied there once at
//! start-up.  No code from the executable is ever executed.

use crate::mem::host_slice;
use std::path::Path;

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
