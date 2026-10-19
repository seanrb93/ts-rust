//! Byte buffers that values are packed into and read back from (the save-game and network style streams).
//!
//! A buffer is addressed by its *cursor*: the game passes the address of a word that holds the current position.  Adding a value
//! writes it there and moves the cursor on; removing reads it and moves on.  Values are little-endian and not aligned.

use crate::*;

/// Append a byte (@ 0x204060).
pub fn BuffAddu8(cursor: i32, v: i32) {
    let at = lw(cursor);
    sb(at, v);
    sw(cursor, lw(cursor) + 1);
}

/// Append a 16-bit value (@ 0x204088).
pub fn BuffAddu16(cursor: i32, v: i32) {
    let at = lw(cursor);
    sh(at, v);
    sw(cursor, lw(cursor) + 2);
}

/// Append a 32-bit value (@ 0x2040b8).
pub fn BuffAddu32(cursor: i32, v: i32) {
    let at = lw(cursor);
    sw(at, v);
    sw(cursor, lw(cursor) + 4);
}

/// Append a float (@ 0x2040e8).
pub fn BuffAddfloat(cursor: i32, v: f32) {
    let at = lw(cursor);
    swc1(at, v);
    sw(cursor, lw(cursor) + 4);
}

/// Read a byte (@ 0x204118).
pub fn BuffRemoveu8(cursor: i32) -> i32 {
    let at = lw(cursor);
    let v = lbu(at);
    sw(cursor, at + 1);
    v
}

/// Read a 16-bit value (@ 0x204140).
pub fn BuffRemoveu16(cursor: i32) -> i32 {
    let at = lw(cursor);
    let v = lhu(at);
    sw(cursor, at + 2);
    v
}

/// Read a 32-bit value (@ 0x204170).
pub fn BuffRemoveu32(cursor: i32) -> i32 {
    let at = lw(cursor);
    let v = lw(at);
    sw(cursor, at + 4);
    v
}

/// Read a float (@ 0x2041a0).
pub fn BuffRemovefloat(cursor: i32) -> f32 {
    let at = lw(cursor);
    let v = lwc1(at);
    sw(cursor, at + 4);
    v
}
