//! Hooks the generator inserts at the start of selected game functions (`WORLD_SPRITE_FUNCS` in tools/decomp/emit.py).
//!
//! Some functions draw 2D sprites at screen positions they computed by projecting 3D points (light glows, lens flares,
//! player name tags).  They look like HUD elements to the renderer (no depth test, far-plane depth), but widescreen
//! handling must leave them where the projection put them: the display-list range written while such a function runs
//! is recorded and handed to the renderer.

use crate::*;

pub struct Guard {
    start: i32,
}

pub fn enter() -> Guard {
    Guard { start: lw(g::gfxtag) }
}

impl Drop for Guard {
    fn drop(&mut self) {
        ts_sdk::world_range(self.start, lw(g::gfxtag));
    }
}
