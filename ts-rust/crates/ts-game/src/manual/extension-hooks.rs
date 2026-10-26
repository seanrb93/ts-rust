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

// ---- the Audio / Video Options page, the pause menu and the front end pages (`avmenu.rs`) -----------------------------------------------

/// The Game Options page is shown inside a level (the pause menu).
#[inline]
pub fn av_ingame_x() -> bool {
    #[cfg(feature = "extensions")]
    {
        av_ingame()
    }
    #[cfg(not(feature = "extensions"))]
    {
        false
    }
}

/// The menu table of the Audio / Video Options page: the port's own (graphics settings added, screen position removed) or the game's (`own`).
#[inline]
pub fn av_menu_or(page: i32, own: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        av_menu(page)
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = page;
        own
    }
}

/// The title of the Audio / Video Options page.
#[inline]
pub fn av_title_or(page: i32, title: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        av_title(page, title)
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = page;
        title
    }
}

/// The title of a page of the pause menu.
#[inline]
pub fn av_pause_title_or(page: i32, title: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        av_pause_title(page, title)
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = page;
        title
    }
}

/// The page the front end goes to.
#[inline]
pub fn av_front_page_or(page: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        av_front_page(page)
    }
    #[cfg(not(feature = "extensions"))]
    {
        page
    }
}

/// The triangle button on the page of the number of players asks whether to exit the game: true when it was dealt with.
#[inline]
pub fn av_nplayers_exit_x(page: i32) -> bool {
    #[cfg(feature = "extensions")]
    {
        av_nplayers_exit(page)
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = page;
        false
    }
}

/// Choosing Exit Game asks in the question box instead of opening a page: true when it was dealt with.
#[inline]
pub fn av_gametype_exit_x(page: i32) -> bool {
    #[cfg(feature = "extensions")]
    {
        av_gametype_exit(page)
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = page;
        false
    }
}

/// The menu of the end game confirmation page.
#[inline]
pub fn av_confirm_table_or(page: i32, own: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        av_confirm_table(page)
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = page;
        own
    }
}

/// The end game page's press was dealt with by the exit confirmation.
#[inline]
pub fn av_confirm_pressed_x(page: i32) -> bool {
    #[cfg(feature = "extensions")]
    {
        av_confirm_pressed(page)
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = page;
        false
    }
}

/// The menu of the SETUP page.
#[inline]
pub fn av_mainsetup_menu_or(own: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        av_mainsetup_menu()
    }
    #[cfg(not(feature = "extensions"))]
    {
        own
    }
}

/// The menu of the game type page.
#[inline]
pub fn av_gamemode_menu_or(own: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        av_gamemode_menu()
    }
    #[cfg(not(feature = "extensions"))]
    {
        own
    }
}

/// Going into a page of the pause menu was dealt with.
#[inline]
pub fn av_in_page_x(player: i32, page: i32) -> bool {
    #[cfg(feature = "extensions")]
    {
        av_in_page(player, page)
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = player;
        let _ = page;
        false
    }
}

/// A page is left (`pageOut`, `outPage`): the page of the pause menu that was opened in the port's own way is closed.
#[inline]
pub fn av_page_out_hook() {
    #[cfg(feature = "extensions")]
    av_page_out();
}

/// The number of players that STORY is open to (the console game: one or two; the port: up to four).
#[inline]
pub fn story_players() -> i32 {
    #[cfg(feature = "extensions")]
    {
        5
    }
    #[cfg(not(feature = "extensions"))]
    {
        3
    }
}

/// STORY, ARCADE and CHALLENGE sit 14 higher on the game type page (the page has room above them).
#[inline]
pub fn gt_y(y: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        y - 14
    }
    #[cfg(not(feature = "extensions"))]
    {
        y
    }
}

/// An item of the game type page from 7 up that has no text of its own (the port's rows 9 to 12 have).
#[inline]
pub fn gt_item_far(item: i32) -> bool {
    #[cfg(feature = "extensions")]
    {
        item >= 7 && !(9..=12).contains(&item)
    }
    #[cfg(not(feature = "extensions"))]
    {
        item >= 7
    }
}

// ---- the item drawing --------------------------------------------------------------------------------------------------------------

/// The arrows beside the bar of a slider while the cursor is on it.
#[inline]
pub fn slider_arrows_hook(item: i32, x: i32, y: i32) {
    #[cfg(feature = "extensions")]
    slider_arrows(item, x, y);
    #[cfg(not(feature = "extensions"))]
    let _ = (item, x, y);
}

/// The slider's bar is drawn from the corner of the pane the page is in (not from the screen's corner: in the second and later panes of a split
/// screen it was not seen).
#[inline]
pub fn pane_x(x: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        lw(g::window) + x
    }
    #[cfg(not(feature = "extensions"))]
    {
        x
    }
}

/// As `pane_x`, for the height.
#[inline]
pub fn pane_y(y: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        lw(g::window + 20) + y
    }
    #[cfg(not(feature = "extensions"))]
    {
        y
    }
}

/// A value of a menu item that has a place of its own is placed from the corner of the pane too.
#[inline]
pub fn value_place(place: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        if place != 0 {
            place + lw(g::window)
        } else {
            0
        }
    }
    #[cfg(not(feature = "extensions"))]
    {
        place
    }
}

/// The value and the arrows of the weapon set on the weapon page follow the box of the selected weapons to the right.
#[inline]
pub fn weapons_set_shift(value: i32, right: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        if value == g::gv_weaponset && weapons_page_active() {
            right + 24
        } else {
            right
        }
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = value;
        right
    }
}

// ---- the save flow, the arcade pages, the MapMaker options -----------------------------------------------------------------------------

/// The state of the message about saving the game's data after a match (`statsupdate_pageTick`): the port answers it with Yes at once.
#[inline]
pub fn save_prompt_state(player: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        if lw(player * 260 + 0x1fc1488) != 0 {
            save_flow(true);
            2
        } else {
            0
        }
    }
    #[cfg(not(feature = "extensions"))]
    {
        splitMessageTick(player)
    }
}

/// The save flow ends (`statsupdate_pageTick`).
#[inline]
pub fn save_flow_hook(on: bool) {
    #[cfg(feature = "extensions")]
    save_flow(on);
    #[cfg(not(feature = "extensions"))]
    let _ = on;
}

/// The message of the memory card sequencer that has only an OK is answered with it while the save flow runs.
#[inline]
pub fn save_flow_active_x() -> bool {
    #[cfg(feature = "extensions")]
    {
        save_flow_active()
    }
    #[cfg(not(feature = "extensions"))]
    {
        false
    }
}

/// The ammo of the left hand sits as close to the edge as the right hand's does.
#[inline]
pub fn ammo_left_edge(offset: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        offset - 10
    }
    #[cfg(not(feature = "extensions"))]
    {
        offset
    }
}

/// An arcade level of the port that the console game does not list (Ron, 25) is available.
#[inline]
pub fn level_added_avail(level: i32) -> bool {
    #[cfg(feature = "extensions")]
    {
        level == 25
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = level;
        false
    }
}

/// The tune of a level: the player's choice on the arcade options page, or the level's own (`own`).
#[inline]
pub fn av_level_music_or(own: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        av_level_music(own)
    }
    #[cfg(not(feature = "extensions"))]
    {
        own
    }
}

/// The number of the arcade levels (the console game lists 19; the port adds Ron after MapMaker).
#[inline]
pub fn av_arcade_count_or(own: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        let _ = own;
        av_arcade_count()
    }
    #[cfg(not(feature = "extensions"))]
    {
        own
    }
}

/// The list of the arcade levels in memory.
#[inline]
pub fn av_arcade_levels_or(own: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        let _ = own;
        av_arcade_levels()
    }
    #[cfg(not(feature = "extensions"))]
    {
        own
    }
}

/// The items of the arcade options page: a copy in the port's memory with a row more (Level Music), or the game's own (`own`).
#[inline]
pub fn opt_items_or(own: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        let _ = own;
        av_opt_menu() + 16
    }
    #[cfg(not(feature = "extensions"))]
    {
        own
    }
}

/// The menu of the arcade options page.
#[inline]
pub fn av_opt_ready_or(own: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        let _ = own;
        av_opt_ready()
    }
    #[cfg(not(feature = "extensions"))]
    {
        own
    }
}

/// The rows of the arcade options page that show the Edit legend (Weapon Set, Bot Set and, in the port, Level Music).
#[inline]
pub fn arcade_edit_row(row: i32) -> bool {
    #[cfg(feature = "extensions")]
    {
        row < 14 || row == 15
    }
    #[cfg(not(feature = "extensions"))]
    {
        row < 14
    }
}

/// The menu of the MapMaker's Options page.
#[inline]
pub fn mm_options_menu_or(own: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        let _ = own;
        mm_options_menu()
    }
    #[cfg(not(feature = "extensions"))]
    {
        own
    }
}

/// The MapMaker's Options page: a slider changed with X, a row that takes the limit of the map away.
#[inline]
pub fn mm_options_tick_hook(page: i32) {
    #[cfg(feature = "extensions")]
    mm_options_tick(page);
    #[cfg(not(feature = "extensions"))]
    let _ = page;
}

/// The cost a MapMaker map may have (5001 in the console game).
#[inline]
pub fn mm_cost_limit() -> i32 {
    #[cfg(feature = "extensions")]
    {
        mm_limit()
    }
    #[cfg(not(feature = "extensions"))]
    {
        0x1389
    }
}

/// The map has no limit: its cost is not shown.
#[inline]
pub fn mm_unlimited_x() -> bool {
    #[cfg(feature = "extensions")]
    {
        mm_unlimited()
    }
    #[cfg(not(feature = "extensions"))]
    {
        false
    }
}
