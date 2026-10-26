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

/// An unarmed swipe starts (`gunUpdate`): the arms take turns.
#[inline]
pub fn punch_swing_hook(prop: i32) {
    #[cfg(feature = "extensions")]
    punch_swing(prop);
    #[cfg(not(feature = "extensions"))]
    let _ = prop;
}

/// The hit test points of a swipe (`gunUpdate`) are put on the other side of the player when the swipe is the left fist's.
#[inline]
pub fn punch_mirror_points_hook(prop: i32, chr: i32, p1: i32, p2: i32) {
    #[cfg(feature = "extensions")]
    punch_mirror_points(prop, chr, p1, p2);
    #[cfg(not(feature = "extensions"))]
    let _ = (prop, chr, p1, p2);
}

/// The picture of the prop is about to be drawn (`propGfxRoom`): a mirrored arm is marked in the display list.  Gives whether it is.
#[inline]
pub fn punch_label_begin(prop: i32) -> bool {
    #[cfg(feature = "extensions")]
    {
        let mirrored = punch_mirrored(prop);
        if mirrored {
            dlLabel(0x4d495252);
        }
        mirrored
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = prop;
        false
    }
}

/// The picture of the prop has been drawn: the mark of a mirrored arm ends.
#[inline]
pub fn punch_label_end(mirrored: bool) {
    #[cfg(feature = "extensions")]
    if mirrored {
        dlLabel(0);
    }
    #[cfg(not(feature = "extensions"))]
    let _ = mirrored;
}

/// A dual weapon without a clip (rocket launcher, blunderbuss, mines, TNT, brick) takes turns with its hands as the others do: `wanted` is
/// the weapon's place in the table of guns (400 bytes a gun).
#[inline]
pub fn dual_no_clip(wanted: i32) -> bool {
    #[cfg(feature = "extended-tables")]
    {
        lw(wanted + (g::guns + 20)) <= 0
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        let _ = wanted;
        false
    }
}

/// The dynamite and the brick are held rolled over in the left hand: the roll (`v`) of the weapon `gun` is turned by 95 degrees more.
#[inline]
pub fn dynamite_roll(v: f32, left: i32, gun: i32) -> f32 {
    #[cfg(feature = "extended-tables")]
    {
        v + if left != 0 && (gun == 18 || gun == 25) { 95.0_f32 } else { 0.0_f32 }
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        let _ = (left, gun);
        v
    }
}
