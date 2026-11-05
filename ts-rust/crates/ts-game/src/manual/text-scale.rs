//! The size of the text of the menus (the setting UI Size of an `extensions` build).
//!
//! `textPrint` and `textSize` give the text the size of the font in whole pixels.  With a scale set, every distance they use (the width and the
//! height of a glyph, the step to the next one, the step to the next line, a button picture) is multiplied by it, in both, so a text that a page
//! places by its width (centred, or against a right edge) is still placed right, and a text that starts at a left edge keeps its edge.  The
//! pages' own spacing (rows, columns) does not change.  The text of a level that is being played is sized by the renderer with the rest of the
//! HUD, so it is left at its size here; the pause menu and the pages that open over a level are text of a menu.

use crate::*;
use std::sync::atomic::{AtomicI32, Ordering};

/// The scale in thousandths (1000 is the font's own size).
static TEXT_SCALE: AtomicI32 = AtomicI32::new(1000);

pub fn set_text_scale(thousandths: i32) {
    TEXT_SCALE.store(thousandths.clamp(250, 4000), Ordering::Relaxed);
}

/// The scale in thousandths that the text drawn now has.
pub fn text_scale() -> i32 {
    #[cfg(feature = "extensions")]
    {
        let s = TEXT_SCALE.load(Ordering::Relaxed);
        if s == 1000 {
            return 1000;
        }
        let in_level = lw(g::gamestate) == 3 && lw(lw(g::front) + 20) == 0;
        if in_level && lw(g::curmenu) == 0 && !(0..4).any(|p| lhu(0x1fb3240 + 92 * p + 8) & 1 != 0) {
            return 1000;
        }
        s
    }
    #[cfg(not(feature = "extensions"))]
    {
        1000
    }
}

/// How far the texts drawn now are moved, in sixteenths of a pixel of the field (the outline of a text goes round it in steps smaller than a pixel).
#[cfg(feature = "extensions")]
static TEXT_NUDGE: [AtomicI32; 2] = [AtomicI32::new(0), AtomicI32::new(0)];

#[cfg(feature = "extensions")]
pub fn set_text_nudge(x: i32, y: i32) {
    TEXT_NUDGE[0].store(x, Ordering::Relaxed);
    TEXT_NUDGE[1].store(y, Ordering::Relaxed);
}

pub fn text_nudge() -> (i32, i32) {
    #[cfg(feature = "extensions")]
    {
        (TEXT_NUDGE[0].load(Ordering::Relaxed), TEXT_NUDGE[1].load(Ordering::Relaxed))
    }
    #[cfg(not(feature = "extensions"))]
    {
        (0, 0)
    }
}

/// `v` times the scale `s` (thousandths), rounded.
pub fn text_scaled(v: i32, s: i32) -> i32 {
    if s == 1000 {
        v
    } else {
        (v as f32 * s as f32 / 1000.0).round() as i32
    }
}
