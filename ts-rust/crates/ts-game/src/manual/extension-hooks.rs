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

/// A coordinate of the weapon page (its boxes are wider and the box of the chosen weapons is moved to the right by 24 so that the names of the
/// dual weapons fit).
#[inline]
pub fn wp_wide(x: i32) -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        x + 24
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        x
    }
}

/// The text of the weapon `combo` on the weapon page: the game's own (`text`), or the name of an added pair.
#[inline]
pub fn combo_name_or(combo: i32, text: i32) -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        let _ = text;
        combo_name(combo)
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        let _ = combo;
        text
    }
}

/// The combination a weapon set has at a place (`frontmpgGetGunComboNum`): an added pair is the single weapon when the pairs are switched off.
#[inline]
pub fn dual_combo_or(place: i32, combo: i32) -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        dual_combo(place, combo)
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        let _ = place;
        combo
    }
}

/// A game starts: nobody owns an added pair and the left fists start again.
#[inline]
pub fn dual_reset_hook() {
    #[cfg(feature = "extended-tables")]
    dual_reset();
}

/// The weapon page has made its list of the weapons that can be chosen: the added pairs follow the singles they are made from.
#[inline]
pub fn weapons_list_extend_hook() {
    #[cfg(feature = "extended-tables")]
    weapons_list_extend();
}
