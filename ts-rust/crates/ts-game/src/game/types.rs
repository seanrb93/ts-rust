//! Typed views of the game's data in guest memory.
//!
//! The game keeps its structures in the flat address space of the console (`ts_rt::mem`).  A view is a plain address with
//! named accessors, so decompiled code reads `m.translation()` instead of `lwc1(a0 + 48)`.  Views are `Copy` and own nothing.

use crate::*;

/// A 4x4 float matrix in row-major order: rows 0..2 hold the axes, row 3 the translation (64 bytes, 16-byte aligned).
#[derive(Clone, Copy)]
pub struct Mat4(pub i32);

impl Mat4 {
    pub fn at(self, row: i32, col: i32) -> f32 {
        lwc1(self.0 + row * 16 + col * 4)
    }
    pub fn set(self, row: i32, col: i32, v: f32) {
        swc1(self.0 + row * 16 + col * 4, v)
    }
    pub fn set_translation(self, x: f32, y: f32, z: f32) {
        self.set(3, 0, x);
        self.set(3, 1, y);
        self.set(3, 2, z);
    }
    pub fn set_diagonal(self, x: f32, y: f32, z: f32) {
        self.set(0, 0, x);
        self.set(1, 1, y);
        self.set(2, 2, z);
    }
    /// Make this the identity matrix (the SDK's `sceVu0UnitMatrix`).
    pub fn set_identity(self) {
        sceVu0UnitMatrix(self.0);
    }
}

/// A quaternion `x y z w` (16 bytes).
#[derive(Clone, Copy)]
pub struct Quat(pub i32);

impl Quat {
    pub fn x(self) -> f32 {
        lwc1(self.0)
    }
    pub fn y(self) -> f32 {
        lwc1(self.0 + 4)
    }
    pub fn z(self) -> f32 {
        lwc1(self.0 + 8)
    }
    pub fn w(self) -> f32 {
        lwc1(self.0 + 12)
    }
    pub fn set_x(self, v: f32) {
        swc1(self.0, v)
    }
    pub fn set_y(self, v: f32) {
        swc1(self.0 + 4, v)
    }
    pub fn set_z(self, v: f32) {
        swc1(self.0 + 8, v)
    }
    pub fn set_w(self, v: f32) {
        swc1(self.0 + 12, v)
    }
}
