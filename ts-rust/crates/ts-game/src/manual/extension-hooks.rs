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

// ---- the texts, the textures, the developer menu, the input, the bots, the big pool and the room tables ------------------------------

/// The text with the names of the buttons for the device in use (the console's texts name the PlayStation controller's).
#[inline]
pub fn button_text_or(text: i32) -> i32 {
    #[cfg(feature = "extensions")]
    {
        button_text(text)
    }
    #[cfg(not(feature = "extensions"))]
    {
        text
    }
}

/// The name of a memory card slot (`mcseqSlotMakeName`) had the number of the slot after it ("SAVE SLOT 1"): the name alone now.  `buffer` holds
/// the name and a space, `length` is the length of the name, `port` the number of the port.
#[inline]
pub fn slot_name_number(buffer: i32, length: i32, port: i32) {
    #[cfg(feature = "extensions")]
    {
        let _ = port;
        sb(length + buffer, 0);
        sb(length + 1 + buffer, 0);
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = (buffer, length);
        sb(length + 1 + buffer, port + 49);
    }
}

/// The texture loader has loaded a picture: the renderer is told which file it came from (a mod can have a picture of any size for it).
#[inline]
pub fn tex_loaded_hook(tbp0: i32, name: i32, parms: i32) {
    #[cfg(feature = "extensions")]
    tex_loaded(tbp0, name, parms);
    #[cfg(not(feature = "extensions"))]
    let _ = (tbp0, name, parms);
}

/// A level with no bag spawn points (Ron has none) has nothing to put a bag on: the console game stops at a break there.
#[inline]
pub fn no_bag_spawns() -> bool {
    cfg!(feature = "extensions")
}

/// The developer menu's Exit Level hands the reset a block of stack that the console happened to leave clear.
#[inline]
pub fn clear_reset_block(sp: i32) {
    #[cfg(feature = "extensions")]
    memset(sp, 0, 112);
    #[cfg(not(feature = "extensions"))]
    let _ = sp;
}

/// Noclip (developer menu): the player's move is not tested against the level, it goes where the stick points.
#[inline]
pub fn noclip_slide_x(from: i32, target: i32) -> bool {
    #[cfg(feature = "extensions")]
    {
        noclip_slide(from, target)
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = (from, target);
        false
    }
}

/// The buttons that go back one menu in the developer menu: the console's start (2048) and, in the port, the triangle (16) too.
#[inline]
pub fn menu_back_buttons() -> i32 {
    #[cfg(feature = "extensions")]
    {
        2048 | 16
    }
    #[cfg(not(feature = "extensions"))]
    {
        2048
    }
}

/// The aim assistance is off for the player who uses the keyboard and the mouse.
#[inline]
pub fn autoaim_off_x() -> bool {
    #[cfg(feature = "extensions")]
    {
        autoaim_off()
    }
    #[cfg(not(feature = "extensions"))]
    {
        false
    }
}

/// The right stick turns the view as fast as the Controls option says.
#[inline]
pub fn look_scale(speed: f32) -> f32 {
    #[cfg(feature = "extensions")]
    {
        speed * ts_sdk::pad::look_sensitivity()
    }
    #[cfg(not(feature = "extensions"))]
    {
        speed
    }
}

/// A bot without a weapon in a level that has none to pick up goes after its target (bare handed) instead of standing still (`findweapon`): gives
/// true when the bot was set to do it.
#[inline]
pub fn bot_idle_punch(record: i32) -> bool {
    #[cfg(feature = "extensions")]
    {
        if ts_sdk::bots_punch() {
            sw(record + 2720, lw(record + 2716));
            sw(record + 2716, 128);
            sw(record + 2892, 0);
            return true;
        }
        false
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = record;
        false
    }
}

/// An allocation of the level pool is made in the port's big pool (a level of a mod).
#[inline]
pub fn big_wants_x(size: i32) -> bool {
    #[cfg(feature = "extended-tables")]
    {
        big_wants(size)
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        let _ = size;
        false
    }
}

/// The address of an allocation in the big pool.
#[inline]
pub fn big_alloc_x(size: i32) -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        big_alloc(size)
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        let _ = size;
        0
    }
}

/// The level pool is reset: the big pool is too.
#[inline]
pub fn big_reset_hook() {
    #[cfg(feature = "extended-tables")]
    big_reset();
}

/// The arcade set-up of a level of a mod.
#[inline]
pub fn custom_setup_x(level: i32) -> Option<i32> {
    #[cfg(feature = "extended-tables")]
    {
        custom_setup(level)
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        let _ = level;
        None
    }
}

/// The number of the props and of the model instances a level has room for (300 in the console game).
#[inline]
pub fn level_instances() -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        1000
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        300
    }
}

/// The room `room` is `index` rooms deep in the list of the rooms at the distance `distance` (`bgGfx`): a byte for each room of the game's
/// level of 200 rooms, a halfword for each of the port's 480.
#[inline]
pub fn room_slot_write(index: i32, distance: i32, room: i32) {
    #[cfg(feature = "extended-tables")]
    sh((index << 1) + distance * 1024 + 0x2500000, room);
    #[cfg(not(feature = "extended-tables"))]
    sb(index + distance * 200 + 0x1fc4a78, room);
}

/// The room at place `index` of the rooms at the distance `distance`.
#[inline]
pub fn room_slot_read(distance: i32, index: i32) -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        lhu(distance * 1024 + (index << 1) + 0x2500000)
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        lbu(distance * 200 + index + 0x1fc4a78)
    }
}

/// Put the room `room` at the end of the list of rooms of the player whose record is `record` (up to 212 from +912, their number at +908; `bgGetRoomsTick`).  The
/// game's levels have too few rooms in view for it to be full; a level of the port with 500 rooms can have more, and the list would run over the limits of the stick
/// at +1784 and the rest of the record.  The rooms come from the farthest level to the nearest, so a full list loses its first, the farthest.
/// The presets of the game modes that mods give are written into the records of the options, after the game has made them (`gameoptionsMake`).
#[inline]
pub fn custom_rules_x() {
    #[cfg(feature = "extensions")]
    custom_rules_tick();
}

#[inline]
pub fn mode_presets_x() {
    #[cfg(feature = "extensions")]
    mode_presets_apply();
}

#[inline]
pub fn room_list_push(record: i32, room: i32) {
    #[cfg(feature = "extended-tables")]
    {
        let count = lw(record + 908);
        if count >= 212 {
            for i in 1..212 {
                sw(record + ((i - 1) << 2) + 912, lw(record + (i << 2) + 912));
            }
            sw(record + (211 << 2) + 912, room);
            return;
        }
    }
    sw(record + (lw(record + 908) << 2) + 912, room);
    sw(record + 908, lw(record + 908) + 1);
}

/// The debug string of the rooms drawn is a stack buffer that is a room longer than the frame when a level has more than the game's rooms: it is
/// written only for the game's own.
#[inline]
pub fn room_letter_write(at: i32, letter: i32) {
    #[cfg(feature = "extended-tables")]
    let _ = (at, letter);
    #[cfg(not(feature = "extended-tables"))]
    sb(at, letter);
}

// ---- frame rates above 60 Hz (`boot.rs`: `timeTickStart` hands the game fractions of a tick) -------------------------------------------

/// The time this frame stands for, as the game's code asks "is the game running?" (`updateframesi` compared with 0): at a high frame rate most
/// frames complete no whole tick, so the frame time is used then.
#[inline]
pub fn frames_gate() -> f32 {
    #[cfg(feature = "high-framerate")]
    {
        lwc1(g::updateframesf)
    }
    #[cfg(not(feature = "high-framerate"))]
    {
        lw(g::updateframesi) as f32
    }
}

/// The integer tick counter as a float, used as the time base of animations: the continuous time at a high frame rate.
#[inline]
pub fn gameframe_x() -> f32 {
    #[cfg(feature = "high-framerate")]
    {
        gameframe_f()
    }
    #[cfg(not(feature = "high-framerate"))]
    {
        lw(g::gameframe) as f32
    }
}

/// A value that is only tested against 0 ("the game is running") before it is used: at least 1 whenever time passes at a high frame rate.
#[inline]
pub fn running_x() -> i32 {
    #[cfg(feature = "high-framerate")]
    {
        running_i()
    }
    #[cfg(not(feature = "high-framerate"))]
    {
        lw(g::updateframesi)
    }
}

/// A constant that the original adds on every call (a HUD ring angle, a spin) scaled by the time of the frame at a high frame rate.
#[inline]
pub fn per_frame(v: f32) -> f32 {
    #[cfg(feature = "high-framerate")]
    {
        v * lwc1(g::tickframesf)
    }
    #[cfg(not(feature = "high-framerate"))]
    {
        v
    }
}

/// As `per_frame`, scaled by `updateframesf`.
#[inline]
pub fn per_update(v: f32) -> f32 {
    #[cfg(feature = "high-framerate")]
    {
        v * lwc1(g::updateframesf)
    }
    #[cfg(not(feature = "high-framerate"))]
    {
        v
    }
}

/// The animation time of the frame before the last one (`spaceship1PropTick`): `now` is the integer tick counter.
#[inline]
pub fn gameframe_before(now: i32) -> f32 {
    #[cfg(feature = "high-framerate")]
    {
        let _ = now;
        gameframe_f() - lwc1(g::updateframesf)
    }
    #[cfg(not(feature = "high-framerate"))]
    {
        (now - lw(g::updateframesi)) as f32
    }
}

/// The tick counter `tick` as a float for an animation (the continuous time at a high frame rate).
#[inline]
pub fn gameframe_tick_x(tick: i32) -> f32 {
    #[cfg(feature = "high-framerate")]
    {
        let _ = tick;
        gameframe_f()
    }
    #[cfg(not(feature = "high-framerate"))]
    {
        tick as f32
    }
}

/// The ticks of this frame as a float (the whole ticks in the console game, the frame time at a high frame rate).
#[inline]
pub fn update_ticks_f() -> f32 {
    #[cfg(feature = "high-framerate")]
    {
        lwc1(g::updateframesf)
    }
    #[cfg(not(feature = "high-framerate"))]
    {
        lw(g::updateframesi) as f32
    }
}

/// The ticks of this frame are stored in the word `at` (the player's: the speed is the distance moved divided by them): an integer in the console
/// game, a float at a high frame rate.
#[inline]
pub fn store_update_ticks(at: i32) {
    #[cfg(feature = "high-framerate")]
    swc1(at, lwc1(g::updateframesf));
    #[cfg(not(feature = "high-framerate"))]
    sw(at, lw(g::updateframesi));
}

/// The ticks word `at` is set to 1.
#[inline]
pub fn store_one_tick(at: i32) {
    #[cfg(feature = "high-framerate")]
    swc1(at, 1.0_f32);
    #[cfg(not(feature = "high-framerate"))]
    sw(at, 1);
}

/// A ticks word of the player (the integer of the console game, the float's bits at a high frame rate) as a float.
#[inline]
pub fn ticks_of(raw: i32) -> f32 {
    #[cfg(feature = "high-framerate")]
    {
        f32::from_bits(raw as u32)
    }
    #[cfg(not(feature = "high-framerate"))]
    {
        raw as f32
    }
}

// ---- the front end above 60 Hz (the menus' logic runs once per whole tick; the frames in between redraw) ------------------------------

/// This frame completes no whole tick of the front end: its logic and pad sampling are skipped.
#[inline]
pub fn menu_skip_x() -> bool {
    #[cfg(feature = "high-framerate")]
    {
        menu_skip()
    }
    #[cfg(not(feature = "high-framerate"))]
    {
        false
    }
}

/// The time of this front end frame in ticks (1.0 whenever the front end runs at 60 Hz).
#[inline]
pub fn menu_dt_x() -> f32 {
    #[cfg(feature = "high-framerate")]
    {
        menu_dt()
    }
    #[cfg(not(feature = "high-framerate"))]
    {
        1.0_f32
    }
}

/// A per call step of the front end's animations scaled by the time of the frame.
#[inline]
pub fn menu_scale(v: f32) -> f32 {
    #[cfg(feature = "high-framerate")]
    {
        v * menu_dt()
    }
    #[cfg(not(feature = "high-framerate"))]
    {
        v
    }
}

/// A per call step in double precision scaled by the time of the frame.
#[inline]
pub fn menu_dp_scale(v: i64) -> i64 {
    #[cfg(feature = "high-framerate")]
    {
        dpmul(v, fptodp(menu_dt()))
    }
    #[cfg(not(feature = "high-framerate"))]
    {
        v
    }
}

/// A whole number step of `base` per tick, handed out in the fractions that fit the frame time.
#[inline]
pub fn menu_steps_x(base: i32) -> i32 {
    #[cfg(feature = "high-framerate")]
    {
        menu_steps(base)
    }
    #[cfg(not(feature = "high-framerate"))]
    {
        base
    }
}

/// `frontfxTick` notes the mode of the front end for the frames in between.
#[inline]
pub fn menu_note_fx_x(mode: i32) {
    #[cfg(feature = "high-framerate")]
    menu_note_fx(mode);
    #[cfg(not(feature = "high-framerate"))]
    let _ = mode;
}

/// The per frame arena reset of the game's tick (`memdbTick`): the front end's frames in between continue after the logic's allocations.
#[inline]
pub fn menu_memdb_tick_x() {
    #[cfg(feature = "high-framerate")]
    menu_memdb_tick();
    #[cfg(not(feature = "high-framerate"))]
    memdbTick();
}

/// After the logic of a tick the end of its allocations is marked.
#[inline]
pub fn menu_memdb_mark_x() {
    #[cfg(feature = "high-framerate")]
    menu_memdb_mark();
}

/// The animated background of the front end advances on the frames that have no logic.
#[inline]
pub fn menu_effects_x() {
    #[cfg(feature = "high-framerate")]
    menu_effects();
}

/// The sign-on list wraps round: New Player (the top) goes up to the last sign-on and the last sign-on goes down to New Player.  The links are made
/// from the count of the sign-ons, but a sign-on that does not get an item leaves a link on an item that is not there: the arrow vanishes and
/// stays gone.  The links are made again from the items that were made (`made`) when they differ from the count.
#[inline]
pub fn signon_links(made: i32, count: i32) {
    #[cfg(feature = "extensions")]
    if made != count {
        let somi = g::somi;
        sh(somi + 16, if made > 0 { made } else { lhu(g::mi_signonload + 8) as i32 });
        sh(somi + 50, if made > 0 { 1 } else { lhu(g::mi_signonnew + 8) as i32 });
        if made > 0 {
            sh(somi + 64 + (made - 1) * 32 + 18, lhu(g::mi_signonnew + 8) as i32);
        }
    }
    #[cfg(not(feature = "extensions"))]
    let _ = (made, count);
}

/// Test aid (`TS_FILELOG`): the files the game loads are listed.
#[inline]
pub fn file_log_hook(name: i32) {
    #[cfg(feature = "extensions")]
    file_log(name);
    #[cfg(not(feature = "extensions"))]
    let _ = name;
}

/// A function that draws sprites at screen positions it worked out by projecting 3D points (the glows of lights, lens flares) is running: the display
/// list it writes is recorded so that the widescreen handling leaves the sprites where the projection put them.
#[inline]
pub fn world_sprite_guard() -> Option<crate::manual::hooks::Guard> {
    #[cfg(feature = "extensions")]
    {
        Some(crate::manual::hooks::enter())
    }
    #[cfg(not(feature = "extensions"))]
    {
        None
    }
}

/// An allocation of the eternal memory that does not fit in the game's block is made in the port's own (a level of a mod with many textures).
#[inline]
pub fn eternal_overflow_x(size: i32) -> Option<i32> {
    #[cfg(feature = "extended-tables")]
    {
        eternal_overflow(size)
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        let _ = size;
        None
    }
}

/// The TEX0 bits of the block address of a texture (the game or'ed the address in as it was): with the extension the address can be wider than the 14 bits of the hardware,
/// and the bits above them go to the bits of TEX0 that the game does not use (`ts_rt::gswide`).
#[inline]
pub fn gs_tex0_tbp(addr: i64) -> i64 {
    #[cfg(feature = "extended-tables")]
    {
        ts_rt::gswide::tex0_tbp_bits(addr as u64) as i64
    }
    #[cfg(not(feature = "extended-tables"))]
    addr
}

/// The TEX0 bits of the palette address of a small texture (the game shifted the address up to the CBP field).
#[inline]
pub fn gs_tex0_cbp(addr: i64) -> i64 {
    #[cfg(feature = "extended-tables")]
    {
        ts_rt::gswide::tex0_cbp_bits(addr as u64) as i64
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        addr << 37
    }
}

/// An address field of MIPTBP1 or MIPTBP2 (before the game shifts it to its place): as the game or'ed it in, or with the bits above the 14 of the hardware.
#[inline]
pub fn gs_mip_tbp(addr: i64) -> i64 {
    #[cfg(feature = "extended-tables")]
    {
        ts_rt::gswide::mip_tbp_bits(addr as u64) as i64
    }
    #[cfg(not(feature = "extended-tables"))]
    addr
}

/// A block address kept in the game's memory: a halfword in the original, a word when the addresses are wider than 16 bits.
#[inline]
pub fn gs_address_at(at: i32) -> i64 {
    #[cfg(feature = "extended-tables")]
    {
        lw(at) as i64
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        lh(at) as i64
    }
}

/// The number of slots the texture buffer of a level gets (`slots` is the original's).  With the extension a level can use all the 17 bits of block address.
#[inline]
pub fn tex_slot_count(slots: i32) -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        if slots == 352 { (ts_rt::gswide::WIDE_BLOCKS as i32 >> 5) - 160 } else { slots }
    }
    #[cfg(not(feature = "extended-tables"))]
    slots
}

/// Where the records of the slots of a texture buffer are, as the address that `+ 8 + slot * 12` is added to: the buffer itself in the original, with the extension a
/// block of its own whose address is kept at +8 of the buffer (the 512 slots inside the buffer are too few).
#[inline]
pub fn tex_slots(buffer: i32) -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        lw(buffer + 8) - 8
    }
    #[cfg(not(feature = "extended-tables"))]
    buffer
}

/// Make the records of the slots of a texture buffer that has `lw(buffer + 0x1a10)` slots (nothing in the original: they are inside the buffer).
#[inline]
pub fn tex_slots_create(n: i32, buffer: i32) {
    #[cfg(feature = "extended-tables")]
    {
        let at = tex_memory(n, 2, (lw(buffer + 0x1a10) * 12 + 15 & -16) + lw(buffer + 0x1a14));
        sw(buffer + 8, at);
    }
    #[cfg(not(feature = "extended-tables"))]
    let _ = (n, buffer);
}

/// Empty the slots of a texture buffer.
#[inline]
pub fn tex_slots_clear(buffer: i32) {
    #[cfg(feature = "extended-tables")]
    {
        let at = lw(buffer + 8);
        if at != 0 {
            memset(at, 0, lw(buffer + 0x1a10) * 12);
        }
    }
    #[cfg(not(feature = "extended-tables"))]
    memset(buffer + 8, 0, 0x1800);
}

/// The size entry of the table `tsizes` for a texture of `width` by `height` that has no entry of its own (256 by 16 and some others are missing): the smallest entry that holds it (the
/// original gives such a texture the address 0, which is the screen), or 0.
#[inline]
pub fn tex_size_entry_fallback(width: i32, height: i32) -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        let mut best = 0;
        let mut best_area = i64::MAX;
        for index in 0..28 {
            let at = g::tsizes + index * 44;
            let (w, h) = (lw(at), lw(at + 4));
            if w >= width && h >= height && (w as i64) * (h as i64) < best_area {
                best = at;
                best_area = (w as i64) * (h as i64);
            }
        }
        best
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        let _ = (width, height);
        0
    }
}

/// The number of textures of a texture buffer (`count` is the original's: 512 for a level).  A level of the extension can have all the 5000 textures the game numbers.
#[inline]
pub fn tex_count(count: i32) -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        if count == 512 { 5000 } else { count }
    }
    #[cfg(not(feature = "extended-tables"))]
    count
}

/// The address of the kind byte of the texture `index` of a texture buffer: in the buffer at +0x1808 in the original (512 bytes), with the extension after the records of the slots.
#[inline]
pub fn tex_kind_addr(buffer: i32, index: i32) -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        lw(buffer + 8) + (lw(buffer + 0x1a10) * 12 + 15 & -16) + index
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        buffer + index + 0x1808
    }
}

/// Empty the kind bytes of the textures of a texture buffer.
#[inline]
pub fn tex_kinds_clear(buffer: i32) {
    #[cfg(feature = "extended-tables")]
    {
        if lw(buffer + 8) != 0 {
            memset(tex_kind_addr(buffer, 0), 0, lw(buffer + 0x1a14));
        }
    }
    #[cfg(not(feature = "extended-tables"))]
    memset(buffer + 0x1808, 0, 512);
}

/// The table of the preload records (24 bytes each): the game's own, which holds 500, or with the extension a table of its own above the memory pools that holds 8192 (a level of a mod
/// can have thousands of textures, each of which is a record).
#[inline]
pub fn preload_table() -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        0xc00_0000
    }
    #[cfg(not(feature = "extended-tables"))]
    0x1fb4c60
}

/// The table of the records of the second preload pass (16 bytes each; 1000 in the game's own, 8192 with the extension).
#[inline]
pub fn preload_pass2_table() -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        0xc10_0000
    }
    #[cfg(not(feature = "extended-tables"))]
    0x1fb7b40
}

/// How many records the table of preload records holds.
#[inline]
pub fn preload_capacity() -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        8192
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        0x2ee0 / 24
    }
}

/// The table of the texture slots by texture number (a halfword each, -1 for a texture that is not loaded): the game's own holds the 5000 numbers it uses, with the extension
/// a table of its own holds 16384 (a level of a mod numbers its textures from 4000 up).
#[inline]
pub fn texnum_table() -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        0xc20_0000
    }
    #[cfg(not(feature = "extended-tables"))]
    0x1feb3e0
}

/// How many texture numbers `texnum_table` holds.
#[inline]
pub fn texnum_count() -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        16384
    }
    #[cfg(not(feature = "extended-tables"))]
    5000
}

/// Memory for part `part` (0 the packets, 1 the records, 2 the slots and kinds) of the texture buffer `n`: `memAlloc(size, 1)` in the original; with the extension a block of its own that no
/// reset of the level's pools reaches (a pool of the level is emptied while the texture buffer lives on, and the packets of more than 256 KB would be taken to the big pool).
#[inline]
pub fn tex_memory(n: i32, part: i32, size: i32) -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        let _ = size;
        0xc40_0000 + n * 0x40_0000 + part * 0x10_0000
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        let _ = (n, part);
        memAlloc(size, 1)
    }
}

/// How many portals deep the rooms are drawn (`bgGfx`: the game draws them from the farthest): the rooms of a level of a mod are many and the open ones show
/// hundreds of them, more than the translation of the console's graphics does at 60 frames a second, so the depth follows the work of the last frames: a room is
/// drawn only up to the depth at which the frame stays under about 100,000 vertices.  The levels of the game are drawn as they always were.
#[inline]
pub fn room_depth_limit(max_level: i32) -> i32 {
    #[cfg(feature = "extended-tables")]
    {
        use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
        static LIMIT: AtomicI32 = AtomicI32::new(24);
        static FRAME: AtomicU32 = AtomicU32::new(0);
        if !(31..=99).contains(&lw(g::lvlevelnum)) {
            return max_level;
        }
        let f = FRAME.fetch_add(1, Ordering::Relaxed);
        if f % 6 == 0 {
            let verts = ts_sdk::perf::last_verts();
            let l = LIMIT.load(Ordering::Relaxed);
            if verts > 110_000 {
                LIMIT.store((l - 1).max(3), Ordering::Relaxed);
            } else if verts < 75_000 {
                LIMIT.store((l + 1).min(24), Ordering::Relaxed);
            }
        }
        max_level.min(LIMIT.load(Ordering::Relaxed))
    }
    #[cfg(not(feature = "extended-tables"))]
    {
        max_level
    }
}
