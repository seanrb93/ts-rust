//! Wide block addresses of texture memory.  The graphics synthesizer addresses it with 14 bits (16384 blocks); the port keeps up to 17 bits so a level can have
//! more textures than the console's 4 MB hold.  The low 14 bits stay where the hardware has them and the three higher ones go to bits the game never uses:
//! TEX0 has them at 18, 19 (the buffer width is at most 8 units, so bits 14 to 17) and 56 (the palette offset, not used with 8-bit pictures), and the palette
//! address (the CBP of a small texture) at 57 to 59; the MIPTBP registers have them at bits 17 to 19 above each address; BITBLTBUF has them at 46, 47 and 54.
//! An address under 16384 is written exactly as the hardware does, so nothing changes for it.

/// The most blocks of texture memory the port addresses.
pub const WIDE_BLOCKS: u32 = 1 << 17;

/// The TEX0 bits that hold the block address `addr` (the buffer width and the other fields are or'ed to it by the caller).
pub fn tex0_tbp_bits(addr: u64) -> u64 {
    (addr & 0x3fff) | ((addr >> 14) & 3) << 18 | ((addr >> 16) & 1) << 56
}

/// The block address in a TEX0 value.
pub fn tex0_tbp(tex0: u64) -> u32 {
    ((tex0 & 0x3fff) | ((tex0 >> 18) & 3) << 14 | ((tex0 >> 56) & 1) << 16) as u32
}

/// The TEX0 bits (already shifted) that hold the palette address `addr` of a texture of 8 bits.
pub fn tex0_cbp_bits(addr: u64) -> u64 {
    (addr & 0x3fff) << 37 | ((addr >> 14) & 7) << 57
}

/// The palette address in a TEX0 value of a picture of 8 bits.
pub fn tex0_cbp8(tex0: u64) -> u32 {
    (((tex0 >> 37) & 0x3fff) | ((tex0 >> 57) & 7) << 14) as u32
}

/// The bits of one address field of MIPTBP1 or MIPTBP2 (at bit 0 of the field; the caller shifts it up).
pub fn mip_tbp_bits(addr: u64) -> u64 {
    (addr & 0x3fff) | ((addr >> 14) & 7) << 17
}

/// The BITBLTBUF bits (in place) of the destination block address `addr`.
pub fn bitbltbuf_dbp_bits(addr: u64) -> u64 {
    (addr & 0x3fff) << 32 | ((addr >> 14) & 3) << 46 | ((addr >> 16) & 1) << 54
}

/// The destination block address in a BITBLTBUF value.
pub fn bitbltbuf_dbp(v: u64) -> u32 {
    (((v >> 32) & 0x3fff) | ((v >> 46) & 3) << 14 | ((v >> 54) & 1) << 16) as u32
}
