//! The places where the game's code asks the port's extensions.  Each function is what the original does when the extension is off
//! (the original's own read, its own text) and what the extension says when it is on, so the game's code calls them in place of the
//! original's expression and stays the original's logic in a default build.

use crate::*;

/// The weapon set the game's code is told (a custom set is 0 and its weapons are in `gunzpack`).
#[inline]
pub fn weaponset_read() -> i32 {
    #[cfg(feature = "extensions")]
    {
        weaponset_effective()
    }
    #[cfg(not(feature = "extensions"))]
    {
        lw(g::gv_weaponset)
    }
}

/// The bot set the game's code is told (a custom set is 8 and its characters are in the game's custom set).
#[inline]
pub fn chrset_read() -> i32 {
    #[cfg(feature = "extensions")]
    {
        chrset_effective()
    }
    #[cfg(not(feature = "extensions"))]
    {
        lw(g::gv_chrset)
    }
}

/// The legend of the weapon page for the selected row: `text` is the game's own, the extension adds the Random button for a custom set.
#[inline]
pub fn weapons_legend_or(text: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        let _ = text;
        weapons_legend()
    }
    #[cfg(not(feature = "extensions"))]
    {
        text
    }
}

/// The legend of the bot set page while the cursor is on a set: `text` is the game's own, `preset` is non zero for a built in set.
#[inline]
pub fn bots_legend_or(text: i32, preset: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        bots_legend(text, preset)
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = preset;
        text
    }
}
