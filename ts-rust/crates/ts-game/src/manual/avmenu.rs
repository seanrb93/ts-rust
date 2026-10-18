//! The game's own option pages that the port adds or reshapes: Audio Options, Video Options, Game Options and Controls.
//!
//! The pages are one page of the game (`audiovideo_page`: a table of menu items, each naming a label and a "game variable" with a
//! value, limits and a list of value names).  Labels and value names may be given as pointers to text instead of text numbers, so
//! the port builds its own copy of the table in memory it owns (`BASE`, above the console's RAM) and the page uses that table
//! (`av_menu`, called by `audiovideo_pageTick`).  Which page it is follows from the page description that was opened (`Mode`):
//! the audio items; the port's graphics settings (supplied by `AvOptions`); the port's game options (`AvOptions::game_rows`); the
//! controls and preferences of the player's profile (read and written here).  A change made with the game's own arrows is noticed
//! on the next tick and handed on.

use crate::*;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

pub struct AvRow {
    pub label: String,
    pub values: Vec<String>,
    pub index: usize,
}

pub trait AvOptions: Send + Sync {
    /// the rows of the Video Options page (the labels and value names are only read when the page ticks)
    fn rows(&self) -> Vec<AvRow>;
    /// the player moved a row to the value with this index
    fn set(&self, row: usize, index: usize);
    /// the player accepted the row's value (ended the edit with X) / gave up the change (triangle)
    fn commit(&self, _row: usize) {}
    fn cancel(&self, _row: usize) {}
    /// the rows of the Game Options page and a change of one of them
    fn game_rows(&self) -> Vec<AvRow> {
        Vec::new()
    }
    fn set_game(&self, _row: usize, _index: usize) {}
    /// the rows of the port's own settings that are on the Controls page after the profile's (the control method, the sensitivities)
    fn control_rows(&self) -> Vec<AvRow> {
        Vec::new()
    }
    fn set_control(&self, _row: usize, _index: usize) {}
    fn set_mapmaker_unlimited(&self, _on: bool) {}
    /// the rows of the port's own settings on the Audio Options page's second page (the volumes of the kinds of sound effects)
    fn audio_rows(&self) -> Vec<AvRow> {
        Vec::new()
    }
    fn set_audio(&self, _row: usize, _index: usize) {}
    /// the rows of the Mods page (a mod each: On / Off) and a change of one of them
    fn mod_rows(&self) -> Vec<AvRow> {
        Vec::new()
    }
    fn set_mod(&self, _row: usize, _index: usize) {}
    /// the music chosen for the levels (a row of the arcade options page): 0 = the level's own, 1 = a random track each time, n + 1 = track n
    fn level_music(&self) -> i32 {
        0
    }
    fn set_level_music(&self, _choice: i32) {}
}

static PROVIDER: OnceLock<Box<dyn AvOptions>> = OnceLock::new();

/// The port's options (the settings the pages show), once it has handed them over.
pub(crate) fn provider() -> Option<&'static dyn AvOptions> {
    PROVIDER.get().map(|p| p.as_ref())
}

pub fn set_av_options(p: Box<dyn AvOptions>) {
    let _ = PROVIDER.set(p);
}

/// memory of the port's own, outside the 32 MiB the console had (RAM is 64 MiB in this port)
const BASE: i32 = 0x240_0000;
const GV_AT: i32 = BASE + 0x1000;
/// line slot far below any picture: an item of a page that is not shown
const OFF_SCREEN: i32 = 100;
const STR_AT: i32 = BASE + 0x2000;
const LIST_AT: i32 = BASE + 0x4000;
/// first item number of the rows (the page draws a frame for item 5, the removed screen-position item)
const FIRST_ID: i32 = 6;

fn copy(dst: i32, src: i32, bytes: i32) {
    for i in (0..bytes).step_by(4) {
        sw(dst + i, lw(src + i));
    }
}

pub(crate) fn put_str(at: &mut i32, s: &str) -> i32 {
    let start = *at;
    for b in s.bytes() {
        sb(*at, b as i32);
        *at += 1;
    }
    sb(*at, 0);
    *at += 1;
    *at = (*at + 3) & !3;
    start
}

pub(crate) fn read_str(mut at: i32) -> String {
    let mut s = String::new();
    while lbu(at) != 0 && s.len() < 60 {
        s.push(lbu(at) as u8 as char);
        at += 1;
    }
    s
}

// ---- which page -------------------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Audio,
    Video,
    Game,
    Controls,
    /// the buttons of the Custom configuration
    Custom,
    /// the mods that are installed, each On or Off
    Mods,
}

/// "pages" that the items of the game type page open (`av_front_page` swaps each for a copy of the audio / video page's description at
/// the `_FRONT_AT` place: the page then knows which one it is from where its description is)
const AUDIO_PAGE: i32 = BASE + 0x6560;
const VIDEO_PAGE: i32 = BASE + 0x6570;
const GAME_PAGE: i32 = BASE + 0x6580;
const CONTROLS_PAGE: i32 = BASE + 0x6590;
const AUDIO_FRONT_AT: i32 = BASE + 0x65c0;
const VIDEO_FRONT_AT: i32 = BASE + 0x65d0;
const GAME_FRONT_AT: i32 = BASE + 0x65e0;
const CONTROLS_FRONT_AT: i32 = BASE + 0x65f0;
/// the pause menu's page descriptions
const AUDIO_PAUSE_AT: i32 = BASE + 0x6200;
const VIDEO_PAUSE_AT: i32 = BASE + 0x6210;
const CONTROLS_PAUSE_AT: i32 = BASE + 0x6220;
const CUSTOM_PAUSE_AT: i32 = BASE + 0x6230;
const CUSTOM_FRONT_AT: i32 = BASE + 0x6240;
const CUSTOM_PAGE: i32 = BASE + 0x6250;
const MODS_PAGE: i32 = BASE + 0x65a0;
const MODS_FRONT_AT: i32 = BASE + 0x65b0;

static MODE: AtomicU8 = AtomicU8::new(1);

fn mode() -> Mode {
    match MODE.load(Ordering::Relaxed) {
        0 => Mode::Audio,
        2 => Mode::Game,
        3 => Mode::Controls,
        4 => Mode::Custom,
        5 => Mode::Mods,
        _ => Mode::Video,
    }
}

/// The page that is being opened (the description of one of the pages above): which page it is.  (The game copies the description
/// when it opens it, so the page that ticks is remembered from here.)
fn set_mode(page: i32) -> bool {
    let m = match page {
        AUDIO_FRONT_AT | AUDIO_PAUSE_AT => Mode::Audio,
        VIDEO_FRONT_AT | VIDEO_PAUSE_AT => Mode::Video,
        GAME_FRONT_AT => Mode::Game,
        CONTROLS_FRONT_AT | CONTROLS_PAUSE_AT => Mode::Controls,
        CUSTOM_FRONT_AT | CUSTOM_PAUSE_AT => Mode::Custom,
        MODS_FRONT_AT => Mode::Mods,
        _ => return false,
    };
    MODE.store(m as u8, Ordering::Relaxed);
    true
}

// ---- layout ----------------------------------------------------------------------------------------------------------
//
// Every item is placed explicitly (position = offset from the page's origin), and every value sits centred between the two arrow
// columns of its column (the arrows are drawn beside the value: the game puts them at offsets from the value's right edge).

struct Layout {
    /// distance between lines, in the page's units
    pitch: i32,
    /// (item x offset, left arrow, right arrow) per column; the value is centred between the arrows' places
    columns: &'static [(i32, i32, i32)],
    /// position of the page's origin (the header of the table)
    header: (i32, i32),
    /// where the rows go: (first row, column, line slot of that row); a row follows the last entry that starts at or before it.
    rows: &'static [(usize, usize, i32)],
    /// line slot of the hint at the bottom (pause menu pages), and its offset from that slot's line
    hint_slot: i32,
    hint_dy: i32,
    /// all lines are moved down by this much (units of the page)
    dy: i32,
    /// only one group of rows is shown at a time (each start of `rows` begins a page) and left / right turn the pages, instead of
    /// the columns
    paged: bool,
}

impl Layout {
    /// (column, line slot) of the row `i`
    fn row_pos(&self, i: usize) -> (usize, i32) {
        let &(first, col, slot) = self.rows.iter().rev().find(|r| r.0 <= i).unwrap_or(&self.rows[0]);
        (col, slot + (i - first) as i32)
    }

    /// page of the row `i` of a paged layout
    fn row_page(&self, i: usize) -> usize {
        self.rows.iter().rposition(|r| r.0 <= i).unwrap_or(0)
    }

    fn pages(&self) -> usize {
        if self.paged {
            self.rows.len()
        } else {
            1
        }
    }
}

/// The page of a paged layout being shown.
static PAGE: AtomicUsize = AtomicUsize::new(0);

fn page_shown() -> usize {
    if layout().paged {
        PAGE.load(Ordering::Relaxed).min(layout().pages() - 1)
    } else {
        0
    }
}

/// Is the item with this number shown (the audio items only on the audio page; on a paged layout only the rows of the page shown).
fn item_shown(id: i32) -> bool {
    if id < FIRST_ID {
        // (the audio items are the first page of the Audio Options)
        return mode() == Mode::Audio && (!layout().paged || page_shown() == 0);
    }
    let l = layout();
    !l.paged || l.row_page((id - FIRST_ID) as usize) == page_shown()
}

/// front end: the page covers the screen
const FRONT_VIDEO: Layout = Layout { pitch: 15, columns: &[(-49, 174, 303), (280, 500, 605)], header: (0x40, 0x20), rows: &[(0, 0, 0), (5, 1, 0)], hint_slot: 0, hint_dy: 7, dy: 24, paged: false };
const FRONT_ONE: Layout = Layout { pitch: 15, columns: &[(-19, 300, 390)], header: (0x40, 0x20), rows: &[(0, 0, 0)], hint_slot: 0, hint_dy: 7, dy: 30, paged: false };
/// pause menu: the page is drawn over the running game
const PAUSE_VIDEO: Layout = Layout { pitch: 13, columns: &[(-49, 174, 303), (280, 500, 605)], header: (0x40, 0x39), rows: &[(0, 0, 0), (5, 1, 0)], hint_slot: 11, hint_dy: 7, dy: 12, paged: false };
const PAUSE_ONE: Layout = Layout { pitch: 13, columns: &[(-19, 300, 390)], header: (0x40, 0x39), rows: &[(0, 0, 0)], hint_slot: 11, hint_dy: 7, dy: 12, paged: false };
/// pause menu of a split screen: the game draws the items' places at half the size (the text as large as ever; the values stay
/// where the page says) and every picture is at most half as high, so there is room for five lines of a readable height.  The rows
/// are on pages of five.  (A quarter-size picture of four players is narrower still: everything stays in the left half.)
const SPLIT_PAGED: Layout = Layout { pitch: 18, columns: &[(-49, 160, 256)], header: (0x40, 0x39), rows: &[(0, 0, 0), (5, 0, 0), (10, 0, 0)], hint_slot: 7, hint_dy: 7, dy: 14, paged: true };
/// the Controls page: twelve rows (the profile's controls, then the port's settings), on pages
const FRONT_CTRL: Layout = Layout { pitch: 15, columns: &[(-19, 300, 390)], header: (0x40, 0x20), rows: &[(0, 0, 0), (8, 0, 0)], hint_slot: 0, hint_dy: 7, dy: 30, paged: true };
const PAUSE_CTRL: Layout = Layout { pitch: 13, columns: &[(-19, 300, 390)], header: (0x40, 0x39), rows: &[(0, 0, 0), (8, 0, 0)], hint_slot: 11, hint_dy: 7, dy: 12, paged: true };
const SPLIT_CTRL: Layout = Layout { pitch: 18, columns: &[(-19, 160, 256)], header: (0x40, 0x39), rows: &[(0, 0, 0), (5, 0, 0), (10, 0, 0)], hint_slot: 7, hint_dy: 7, dy: 14, paged: true };
const SPLIT_ONE: Layout = Layout { pitch: 18, columns: &[(-19, 160, 256)], header: (0x40, 0x39), rows: &[(0, 0, 0)], hint_slot: 7, hint_dy: 7, dy: 14, paged: false };
/// the buttons of the Custom configuration: eighteen rows, on pages
const FRONT_CUSTOM: Layout = Layout { pitch: 15, columns: &[(-49, 250, 390)], header: (0x40, 0x20), rows: &[(0, 0, 0), (9, 0, 0)], hint_slot: 0, hint_dy: 7, dy: 24, paged: true };
/// the Mods page: a mod on each row, on pages of eight (up to four pages)
const FRONT_MODS: Layout = Layout { pitch: 15, columns: &[(-19, 300, 390)], header: (0x40, 0x20), rows: &[(0, 0, 0), (8, 0, 0), (16, 0, 0), (24, 0, 0)], hint_slot: 0, hint_dy: 7, dy: 30, paged: true };
const PAUSE_CUSTOM: Layout = Layout { pitch: 13, columns: &[(-49, 250, 390)], header: (0x40, 0x39), rows: &[(0, 0, 0), (9, 0, 0)], hint_slot: 11, hint_dy: 7, dy: 12, paged: true };
const SPLIT_CUSTOM: Layout = Layout { pitch: 18, columns: &[(-49, 160, 256)], header: (0x40, 0x39), rows: &[(0, 0, 0), (5, 0, 0), (10, 0, 0), (15, 0, 0)], hint_slot: 7, hint_dy: 7, dy: 14, paged: true };
/// the Audio Options: the game's own items on the first page, the volumes of the kinds of effects on the second (the rows of the second
/// page start at the first row: the first page has no rows)
const FRONT_AUDIO: Layout = Layout { pitch: 15, columns: &[(-19, 300, 390)], header: (0x40, 0x20), rows: &[(0, 0, 0), (0, 0, 0)], hint_slot: 0, hint_dy: 7, dy: 30, paged: true };
const PAUSE_AUDIO: Layout = Layout { pitch: 13, columns: &[(-19, 300, 390)], header: (0x40, 0x39), rows: &[(0, 0, 0), (0, 0, 0)], hint_slot: 11, hint_dy: 7, dy: 12, paged: true };
const SPLIT_AUDIO: Layout = Layout { pitch: 18, columns: &[(-19, 160, 256)], header: (0x40, 0x39), rows: &[(0, 0, 0), (0, 0, 0)], hint_slot: 7, hint_dy: 7, dy: 14, paged: true };
/// line slots of the audio items (a slider takes two lines)
const AUDIO_SLOTS: [i32; 4] = [0, 1, 3, 4];
const AUDIO_FLAGS: [i32; 4] = [0x04, 0x0c, 0x04, 0x0c];

fn in_game() -> bool {
    lw(g::gamestate) == 3
}

/// The page is also used by the pause menu (its tick is told apart by `av_ingame`).
pub fn av_ingame() -> bool {
    in_game()
}

fn layout() -> &'static Layout {
    let two = matches!(mode(), Mode::Video);
    if mode() == Mode::Custom {
        return if !in_game() {
            &FRONT_CUSTOM
        } else if lw(g::lvnumplayers) > 1 {
            &SPLIT_CUSTOM
        } else {
            &PAUSE_CUSTOM
        };
    }
    if mode() == Mode::Audio && PROVIDER.get().is_some() {
        return if !in_game() {
            &FRONT_AUDIO
        } else if lw(g::lvnumplayers) > 1 {
            &SPLIT_AUDIO
        } else {
            &PAUSE_AUDIO
        };
    }
    if mode() == Mode::Mods {
        return &FRONT_MODS;
    }
    if mode() == Mode::Controls {
        return if !in_game() {
            &FRONT_CTRL
        } else if lw(g::lvnumplayers) > 1 {
            &SPLIT_CTRL
        } else {
            &PAUSE_CTRL
        };
    }
    if in_game() {
        if lw(g::lvnumplayers) > 1 {
            if mode() == Mode::Video {
                &SPLIT_PAGED
            } else {
                &SPLIT_ONE
            }
        } else if two {
            &PAUSE_VIDEO
        } else {
            &PAUSE_ONE
        }
    } else if two {
        &FRONT_VIDEO
    } else {
        &FRONT_ONE
    }
}

/// Item number of the row being edited (selected with X: left and right then change its value), 0 for none.
static EDIT: AtomicI32 = AtomicI32::new(0);

/// The row being edited on the MapMaker's Options page (the same mark: the slider shows its arrows then).
pub fn editing_row() -> i32 {
    EDIT.load(Ordering::Relaxed)
}

pub fn set_editing_row(id: i32) {
    EDIT.store(id, Ordering::Relaxed);
}

/// The MapMaker's limit was switched off or on (told to the port, which keeps it).
pub fn mm_unlimited_changed(on: bool) {
    if let Some(p) = PROVIDER.get() {
        p.set_mapmaker_unlimited(on);
    }
}

fn editing() -> i32 {
    EDIT.load(Ordering::Relaxed)
}

/// Are the arrows of the row `id` shown: on the row being edited.
fn arrows_shown(id: i32) -> bool {
    editing() == id
}

/// The arrows next to a value are the game's own arrows, drawn at offsets from the right edge of the value; far off screen they are
/// not seen.  They sit beside the value.
fn fit_arrows(gv: i32, text: i32, col: usize, show: bool) {
    let (_, left, right) = layout().columns[col];
    let centre = (left + 24 + right) / 2;
    let w = textWidth(text);
    let edge = centre + w / 2;
    sw(gv + 28, edge); // right edge of the value text
    let left = centre - w / 2 - 28;
    let right = edge + 8;
    sw(gv + 32, if show { left - edge } else { 1500 });
    sw(gv + 36, if show { right - edge } else { 1500 });
}

fn place(item: i32, col: usize, slot: i32) {
    let l = layout();
    sw(item + 4, ((slot * l.pitch + l.dy) << 16) | (l.columns[col].0 & 0xffff));
}

/// The title of a page of the pause menu: the name of the page (`text` is the game's own, SETUP).
pub fn av_pause_title(page: i32, text: i32) -> i32 {
    let name = match page {
        AUDIO_PAUSE_AT => "Audio",
        VIDEO_PAUSE_AT => "Video",
        CONTROLS_PAUSE_AT => "Controls",
        CUSTOM_PAUSE_AT => "Custom Controls",
        _ => return text,
    };
    let mut at = CONFIRM_TEXT_AT + 0x1c0;
    put_str(&mut at, name)
}

/// The arrows beside the bar of a volume slider, while the cursor is on it (called by the item drawing after the bar).
pub fn slider_arrows(item: i32, x: i32, y: i32) {
    let shown = editing() != 0 && lw(item + 16) == lw((lw(lw(g::front)) << 2) + g::highlightitem);
    if shown {
        let width = (lw(g::window + 12) as f32 * lwc1(0x3ac984)) as i32;
        let colour = ((lbu(g::flashalpha) as u32) >> 1) as i32 + 64 | 0x800000;
        guiDrawImage(20, colour, (x - 30) as f32, (y - 3) as f32);
        guiDrawImage(21, colour, (x + width + 12) as f32, (y - 3) as f32);
    }
}

// ---- the rows ------------------------------------------------------------------------------------------------------------

/// What a row stands for.
#[derive(Clone, Copy)]
enum Src {
    Video(usize),
    Game(usize),
    Prof(Prof),
    Custom(usize),
    /// a row that opens a page of the game (View Credits, View Statistics)
    Page(i32),
    /// a setting of the port on the Controls page
    Port(usize),
    /// the volume of a kind of sound effects (the Audio Options page's second page)
    Sound(usize),
    /// a mod of the Mods page
    Mod(usize),
    /// a line of the Mods page that only says something (no mods are installed)
    Info,
}

/// The rows as they were given last time (the values, to see what the player changed, and what each stands for).
static PREV: Mutex<(Vec<usize>, Vec<Src>)> = Mutex::new((Vec::new(), Vec::new()));
static PREV_MODE: AtomicU8 = AtomicU8::new(255);

fn apply(src: Src, value: usize) {
    match (src, PROVIDER.get()) {
        (Src::Video(r), Some(p)) => p.set(r, value),
        (Src::Game(r), Some(p)) => p.set_game(r, value),
        (Src::Port(r), Some(p)) => p.set_control(r, value),
        (Src::Sound(r), Some(p)) => p.set_audio(r, value),
        (Src::Mod(r), Some(p)) => p.set_mod(r, value),
        (Src::Prof(r), _) => prof_set(r, value),
        (Src::Custom(k), _) => custom_set(k, value),
        _ => {}
    }
}

// ---- the controls and preferences of the profile ---------------------------------------------------------------------------
//
// The pause menu had two pages of them (Setup Controls, Preferences); they are one page now (and a page of the game type page too).
// Their values are bits and fields of the player's profile; the game's own pages go through the game variables `gv_*` and their
// "on change" functions, which write the same places.

#[derive(Clone, Copy, PartialEq, Eq)]
enum Prof {
    Config,
    InvLook,
    AimMode,
    CrouchMode,
    Lookahead,
    AutoAim,
    WeaponChange,
    Vibration,
    VibMode,
}

const PROF_ROWS: [(Prof, i32); 9] = [
    (Prof::Config, g::gv_config),
    (Prof::InvLook, g::gv_invlook),
    (Prof::AimMode, g::gv_aimmode),
    (Prof::CrouchMode, g::gv_crouchmode),
    (Prof::Lookahead, g::gv_lookahead),
    (Prof::AutoAim, g::gv_autoaim),
    (Prof::WeaponChange, g::gv_wpchange),
    (Prof::Vibration, g::gv_vibration),
    (Prof::VibMode, g::gv_vibmode),
];

/// The player's profile: in a level the player's own, in the menus the one that is signed on for the player of the page.
fn profile() -> Option<i32> {
    if in_game() {
        let p = lw(g::currentplayer);
        let prof = if p != 0 { lw(p + 24) } else { 0 };
        (prof != 0).then_some(prof)
    } else {
        let id = signonGet(lw(lw(g::front)));
        (id >= 0).then(|| lw(g::signonarray) + id * 2936 + 32)
    }
}

/// The value of the game variable for the row, from the profile (what `setup_vars` and `prefs_vars` do).
fn prof_get(r: Prof, prof: i32) -> i32 {
    let flags = lhu(prof + 2);
    match r {
        Prof::Config => lhu(prof),
        Prof::InvLook => flags & 1,
        Prof::AimMode => ((flags >> 1) ^ 1) & 1,
        Prof::CrouchMode => ((flags >> 2) ^ 1) & 1,
        Prof::Lookahead => (flags >> 3) & 1,
        Prof::AutoAim => (flags >> 7) & 1,
        Prof::WeaponChange => lw(prof + 88),
        Prof::Vibration => (flags >> 4) & 1,
        Prof::VibMode => ((flags >> 6) & 1) | (((flags >> 5) & 1) << 1),
    }
}

/// Put the value of the game variable into the profile (what the "on change" functions do).
fn prof_put(r: Prof, prof: i32, v: i32) {
    let flags = lhu(prof + 2);
    let bit = |flags: i32, mask: i32, on: bool| if on { flags | mask } else { flags & !mask };
    match r {
        Prof::Config => {
            sh(prof, v);
            if in_game() {
                playerSetControls(lw(g::currentplayer));
            }
        }
        Prof::InvLook => sh(prof + 2, bit(flags, 1, v != 0)),
        Prof::AimMode => sh(prof + 2, bit(flags, 2, v == 0)),
        Prof::CrouchMode => sh(prof + 2, bit(flags, 4, v == 0)),
        Prof::Lookahead => sh(prof + 2, bit(flags, 8, v != 0)),
        Prof::AutoAim => sh(prof + 2, bit(flags, 128, v != 0)),
        Prof::WeaponChange => sw(prof + 88, v),
        Prof::Vibration => sh(prof + 2, bit(flags, 16, v != 0)),
        Prof::VibMode => sh(prof + 2, bit(bit(flags, 64, v & 1 != 0), 32, v & 2 != 0)),
    }
}

/// The values of a game variable (the entries of its list: the value and the text of each) in the order the game shows them.  The list
/// ends with an entry without a text (the variable's own range can be shorter: "Vibration Mode" starts at 1 for the game's page, which
/// leaves the first entry, Off, for when the vibration is off).
fn gv_choices(gv: i32) -> Vec<(i32, String)> {
    let list = lw(gv + 16);
    let lang = lw(g::cur_language);
    let mut v = Vec::new();
    for k in 0..24 {
        let (value, text) = (lw(list + 8 * k), lw(list + 8 * k + 4));
        if text == 0 {
            break;
        }
        let s = if (0..1246).contains(&text) { read_str(lw(lw(g::languages + 4 * lang) + 4 * text)) } else { read_str(text) };
        v.push((value, s));
    }
    v
}

fn prof_set(r: Prof, index: usize) {
    if let (Some(prof), Some(&(_, gv))) = (profile(), PROF_ROWS.iter().find(|(p, _)| *p == r)) {
        if let Some(&(value, _)) = gv_choices(gv).get(index) {
            prof_put(r, prof, value);
        }
    }
}

/// The rows of the Controls page.
fn controls_rows() -> Vec<(AvRow, Src)> {
    let Some(prof) = profile() else {
        return Vec::new();
    };
    let label_ids = [("Configuration", Prof::Config), ("Inverse Look", Prof::InvLook), ("Aim Mode", Prof::AimMode), ("Crouch Mode", Prof::CrouchMode), ("Auto Lookahead", Prof::Lookahead), ("Auto Aim", Prof::AutoAim), ("Weapon Change", Prof::WeaponChange), ("Vibration", Prof::Vibration), ("Vibration Mode", Prof::VibMode)];
    label_ids
        .iter()
        .map(|&(label, r)| {
            let gv = PROF_ROWS.iter().find(|(p, _)| *p == r).map_or(0, |x| x.1);
            let choices = gv_choices(gv);
            let cur = prof_get(r, prof);
            let index = choices.iter().position(|c| c.0 == cur).unwrap_or(0);
            (AvRow { label: label.into(), values: choices.into_iter().map(|c| c.1).collect(), index }, Src::Prof(r))
        })
        .collect()
}

/// Rows of the Custom controls page: the 18 controls of the game's own page (`controlsmenu_items`, in the order `controlsorder` gives their
/// places in the profile's control array): the first eight are the sticks, the rest buttons.
const CUSTOM_ROWS: i32 = 18;

fn custom_idx(k: usize) -> i32 {
    lw(g::controlsorder + 4 * k as i32)
}

fn custom_rows() -> Vec<(AvRow, Src)> {
    let Some(prof) = profile() else {
        return Vec::new();
    };
    let lang = lw(g::cur_language);
    let text = |id: i32| read_str(lw(lw(g::languages + 4 * lang) + 4 * id));
    (0..CUSTOM_ROWS as usize)
        .map(|k| {
            let label = style_words(&text(lw(g::controlsmenu_items + 32 * k as i32)));
            let cur = lw(prof + 4 + 4 * custom_idx(k));
            let (values, index): (Vec<String>, usize) = if k < 8 {
                (vec![style_words(&text(lw(g::disp_analogstick + 4))), style_words(&text(lw(g::disp_analogstick + 12)))], (cur >= 2) as usize)
            } else {
                let names = (0..14).map(|i| style_words(&text(lw(g::disp_buttons + 8 * i + 4)))).collect();
                (names, (0..14).find(|&i| lw(g::gc_buttons + 4 * i) == cur).unwrap_or(0) as usize)
            };
            (AvRow { label, values, index }, Src::Custom(k))
        })
        .collect()
}

/// A control was given a new value: what `controlOnChange` does (the profile's array and, in a level, the player's).
fn custom_set(k: usize, value: usize) {
    let Some(prof) = profile() else { return };
    let idx = custom_idx(k);
    let v = if k < 8 {
        let even = (idx - 10) % 2 == 0;
        match (even, value != 0) {
            (true, true) => 3,
            (true, false) => 1,
            (false, true) => 2,
            (false, false) => 0,
        }
    } else {
        lw(g::gc_buttons + 4 * value as i32)
    };
    sw(prof + 4 + 4 * idx, v);
    if in_game() {
        let p = lw(g::currentplayer);
        if p != 0 {
            sw(p + 28 + 4 * idx, v);
        }
    }
}

/// The rows of the Game Options page that open a page of the game: View Credits (once the game is completed) and View Statistics, the
/// items of the game type page that the front end's main menu used to have.  (label, page)
/// "page" of the View Credits row: selecting it starts the credits (the level 105), as the game type page does
const CREDITS_START: i32 = -105;

/// Start the credits the way the game type page does when View Credits is chosen there.
fn start_credits() {
    frontmpgStoreGoodies();
    let at = BASE + 0xe000;
    for i in (0..112).step_by(4) {
        sw(at + i, 0);
    }
    sw(at + 64, 105);
    sw(at + 4, 1);
    sw(at, 1);
    sw(at + 72, 0);
    gameDeferedReset(at);
    sw(0x1fc11f8, 6);
}

fn info_pages() -> Vec<(&'static str, i32)> {
    let menu = g::gamemode_menu;
    // (the credits are there once the game has been completed or everything is unlocked: the game type page's tick decides it the same
    // way when it gives the item a label, but that only happens while the page is ticking)
    [(4, "View Credits"), (5, "View Statistics")]
        .into_iter()
        .filter_map(|(id, label)| {
            let it = (0..lw(menu)).map(|k| menu + 16 + 32 * k).find(|&it| lhu(it + 8) == id)?;
            let shown = if id == 4 { levelAvail(105) != 0 } else { lw(it) != 0 };
            // (the credits are not a page: the game type page starts them as a level; the item's page is the statistics page for both items)
            (shown && lw(it + 12) != 0).then(|| (label, if id == 4 { CREDITS_START } else { lw(it + 12) }))
        })
        .collect()
}

/// The page a row opens, for the rows that open one.
fn row_page_target(id: i32) -> Option<i32> {
    if id < FIRST_ID {
        return None;
    }
    match PREV.lock().unwrap().1.get((id - FIRST_ID) as usize) {
        Some(Src::Page(p)) => Some(*p),
        _ => None,
    }
}

/// The rows of the page being shown.
fn rows_now() -> Vec<(AvRow, Src)> {
    match (mode(), PROVIDER.get()) {
        (Mode::Video, Some(p)) => p.rows().into_iter().enumerate().map(|(i, r)| (r, Src::Video(i))).collect(),
        (Mode::Game, Some(p)) => {
            let mut rows: Vec<(AvRow, Src)> = p.game_rows().into_iter().enumerate().map(|(i, r)| (r, Src::Game(i))).collect();
            rows.extend(info_pages().into_iter().map(|(label, page)| (AvRow { label: label.into(), values: vec![String::new()], index: 0 }, Src::Page(page))));
            rows
        }
        (Mode::Controls, p) => {
            let mut rows = controls_rows();
            if let Some(p) = p {
                rows.extend(p.control_rows().into_iter().enumerate().map(|(i, r)| (r, Src::Port(i))));
            }
            rows
        }
        (Mode::Custom, _) => custom_rows(),
        (Mode::Mods, Some(p)) => {
            let rows = p.mod_rows();
            if rows.is_empty() {
                vec![(AvRow { label: "No mods installed".into(), values: vec![String::new()], index: 0 }, Src::Info)]
            } else {
                rows.into_iter().take(32).enumerate().map(|(i, r)| (r, Src::Mod(i))).collect()
            }
        }
        (Mode::Audio, Some(p)) => p.audio_rows().into_iter().enumerate().map(|(i, r)| (r, Src::Sound(i))).collect(),
        _ => Vec::new(),
    }
}

/// The menu table for the page.
pub fn av_menu(page: i32) -> i32 {
    if confirming() {
        return confirm_menu(page);
    }
    if PROVIDER.get().is_none() {
        return g::audiovideo_menu;
    }
    let mut prev = PREV.lock().unwrap();
    // (the pages show different rows: what was remembered of the page before is dropped)
    if PREV_MODE.swap(mode() as u8, Ordering::Relaxed) != mode() as u8 {
        prev.0.clear();
        prev.1.clear();
    }
    for (i, &before) in prev.0.iter().enumerate() {
        let v = lw(GV_AT + 0x40 * i as i32).max(0) as usize;
        if v != before {
            apply(prev.1[i], v);
        }
    }
    let entries = rows_now();
    if in_game() {
        av_pause_font();
    }
    let font = textSelect(lw(g::menuitem_normalfont));
    let n = entries.len() as i32;
    let src = g::audiovideo_menu;
    // header, the four audio items, then the rows, then the closing item (back button) and the hint
    let pause = in_game();
    copy(BASE, src, 16);
    sw(BASE, 4 + n + 1);
    sw(BASE + 4, ((layout().header.1) << 16) | (layout().header.0 & 0xffff));
    let item = |k: i32| BASE + 16 + 32 * k;
    for k in 0..4 {
        copy(item(k), src + 16 + 32 * k, 32);
        place(item(k), 0, if item_shown(k + 1) { AUDIO_SLOTS[k as usize] } else { OFF_SCREEN });
        sw(item(k) + 8, (AUDIO_FLAGS[k as usize] << 16) | (k + 1));
    }
    // the volume sliders are greyed out while their switch is off (the page's own tick does this to the original table)
    for (k, switch) in [(1, g::gv_playmusic), (3, g::gv_playsound)] {
        let f = lhu(item(k) + 10);
        sh(item(k) + 10, if lw(switch) == 0 { f | 0x4040 } else { f & 0xbfbf });
    }
    // the order the cursor goes through the items (down), as item numbers
    let ids: Vec<i32> = [1, 2, 3, 4].into_iter().chain((0..n).map(|i| FIRST_ID + i)).filter(|&id| item_shown(id)).collect();
    let nav = |id: i32| -> i32 {
        let Some(idx) = ids.iter().position(|&x| x == id) else {
            return (id << 16) | id;
        };
        if ids[idx] == editing() {
            // the cursor stays on the row while its value is being changed
            return (ids[idx] << 16) | ids[idx];
        }
        let up = ids[(idx + ids.len() - 1) % ids.len()];
        let down = ids[(idx + 1) % ids.len()];
        (down << 16) | up
    };
    for k in 0..4 {
        sw(item(k) + 16, nav(k + 1));
    }
    let mut cursor = lhu(page + 4);
    // (after coming back from another page the cursor may be on an item this page does not have)
    if !ids.contains(&(cursor as i32)) && editing() == 0 {
        if let Some(&first) = ids.first() {
            sh(page + 4, first);
            cursor = first as _;
        }
    }
    let (mut strs, mut lists) = (STR_AT, LIST_AT);
    let mut shown = Vec::with_capacity(entries.len());
    let mut srcs = Vec::with_capacity(entries.len());
    for (i, (row, src_of)) in entries.iter().enumerate() {
        let gv = GV_AT + 0x40 * i as i32;
        // (a volume is a slider like the game's own: its value is the percent, the bar is drawn by the item)
        let slider = matches!(src_of, Src::Sound(_));
        copy(gv, if slider { g::gv_soundvolume } else { g::gv_playsound }, 0x30);
        let index = if slider { row.index.min(100) } else { row.index.min(row.values.len().saturating_sub(1)) };
        sw(gv, index as i32);
        sw(gv + 4, 0);
        sw(gv + 8, if slider { 100 } else { row.values.len() as i32 - 1 });
        sw(gv + 12, 0); // wraps around at the ends
        sw(gv + 16, lists);
        sw(gv + 20, 0);
        let mut value_ptrs = Vec::new();
        for (v, name) in row.values.iter().enumerate() {
            let s = put_str(&mut strs, name);
            value_ptrs.push(s);
            sw(lists, v as i32);
            sw(lists + 4, s);
            lists += 8;
        }
        sw(lists, 0);
        sw(lists + 4, 0);
        lists += 8;
        let label = put_str(&mut strs, &row.label);
        let it = item(4 + i as i32);
        let (col, slot) = layout().row_pos(i);
        if slider {
            // (made from the game's own volume item: a label with the bar under it, two lines)
            copy(it, src + 16 + 32, 32);
        }
        sw(it, label);
        place(it, col, if item_shown(FIRST_ID + i as i32) { slot * if slider { 2 } else { 1 } } else { OFF_SCREEN });
        sw(it + 8, ((if slider { 0x0c } else { 0x04 }) << 16) | (FIRST_ID + i as i32));
        sw(it + 12, gv);
        sw(it + 16, nav(FIRST_ID + i as i32));
        if !slider {
            sw(it + 20, 0);
            sw(it + 24, 0);
            sw(it + 28, 0);
        }
        fit_arrows(gv, value_ptrs.get(index).copied().unwrap_or(label), col, !slider && arrows_shown(FIRST_ID + i as i32));
        shown.push(index);
        srcs.push(*src_of);
    }
    // the music and sound switches too
    for (id, gv) in [(1, g::gv_playmusic), (3, g::gv_playsound)] {
        fit_arrows(gv, gvGetStringValue(gv), 0, arrows_shown(id));
    }
    textSelect(font);
    if pause {
        // the (invisible) item the triangle button follows back to the SETUP page
        let back = item(4 + n);
        for k in 0..8 {
            sw(back + 4 * k, 0);
        }
        sw(back + 8, 0x1_8000);
        sw(back + 12, if mode() == Mode::Custom { CONTROLS_PAUSE_AT } else { g::mainsetup_page });
    } else {
        copy(item(4 + n), src + 16 + 32 * 5, 32);
    }
    // the hint line at the bottom
    let hint_at = BASE + 0x6f00;
    let mut at = hint_at;
    let on_action = cursor as i32 >= FIRST_ID && matches!(srcs.get((cursor as i32 - FIRST_ID) as usize), Some(Src::Page(_)));
    let on_custom = mode() == Mode::Controls && editing() == FIRST_ID && lw(GV_AT) == 0;
    let hint = if mode() == Mode::Mods {
        let pages = if layout().pages() > 1 && n > 8 { format!("   Left/Right: Page {}/{}", page_shown() + 1, ((n + 7) / 8).max(1)) } else { String::new() };
        let text = match entries.first().map(|e| e.1) {
            Some(Src::Info) => "Put mods in the mods folder next to the game".to_string(),
            _ => format!("^2 {}   Changes apply after a restart{}", if editing() != 0 { "OK" } else { "Edit" }, pages),
        };
        put_str(&mut at, &text)
    } else if on_action {
        put_str(&mut at, "^2 Select")
    } else if layout().columns.len() == 1 && !layout().paged {
        put_str(&mut at, if on_custom { "^2 Choose Buttons   Left/Right: Change" } else if editing() != 0 { "^2 OK   Left/Right: Change" } else { "^2 Edit" })
    } else if layout().paged {
        let text = format!("^2 {}   Left/Right: Page {}/{}", if editing() != 0 { "OK" } else { "Edit" }, page_shown() + 1, layout().pages());
        put_str(&mut at, &text)
    } else {
        put_str(&mut at, if editing() != 0 { "^2 OK   Left/Right: Change" } else { "^2 Edit   Left/Right: Column" })
    };
    let hint_item = item(4 + n + 1);
    copy(hint_item, item(4), 32);
    sw(hint_item, hint);
    sw(hint_item + 4, if pause { ((layout().hint_slot * layout().pitch + layout().hint_dy) << 16) | (layout().columns[0].0 & 0xffff) } else { (0xae << 16) | 0x68 });
    sw(hint_item + 8, (0x4000 << 16) | 20);
    sw(hint_item + 12, 0);
    sw(hint_item + 16, 0);
    sw(BASE, lw(BASE) + 1);
    *prev = (shown, srcs);
    drop(prev);
    navigate(page, n as usize);
    BASE
}

/// (item number, line slot) of the rows of each column
fn column_rows(n: usize, col: usize) -> Vec<(i32, i32)> {
    let mut v: Vec<(i32, i32)> = if col == 0 && mode() == Mode::Audio { AUDIO_SLOTS.iter().enumerate().map(|(k, &slot)| (k as i32 + 1, slot)).collect() } else { Vec::new() };
    v.extend((0..n).filter_map(|i| {
        let (c, slot) = layout().row_pos(i);
        (c == col).then_some((FIRST_ID + i as i32, slot))
    }));
    v
}

/// Open the page where the buttons of the "Custom" configuration are chosen.
fn open_custom_controls(page: i32) {
    EDIT.store(0, Ordering::Relaxed);
    PAGE.store(0, Ordering::Relaxed);
    if in_game() {
        copy(CUSTOM_PAUSE_AT, g::prefs_page, 16);
        sw(CUSTOM_PAUSE_AT, 0x2364c8);
        sw(CUSTOM_PAUSE_AT + 8, FIRST_ID);
        MODE.store(Mode::Custom as u8, Ordering::Relaxed);
        inPage(lw(lw(g::currentplayer)), CUSTOM_PAUSE_AT);
    } else {
        frontPageSet(0, CUSTOM_PAGE);
        pageIn(page);
    }
}

/// X starts editing the row the cursor is on: left and right then change its value (the arrows appear next to it, the cursor stays on
/// it), and X or triangle ends it.  (X on a row that opens a page opens it.)  Left and right move between the columns or the pages otherwise.  Runs before the page's own menu tick, which would
/// otherwise change the values with left / right (the volume sliders too: the game's own steps depend on how long the button is held).
fn navigate(page: i32, n: usize) {
    let front = lw(g::front);
    let player = lw(front);
    let id = lhu(page + 4) as i32;
    let flags = lw(front + 12);
    let pressed = joyNewBut(lw(front));
    let slider = id == 2 || id == 4 || (id >= FIRST_ID && mode() == Mode::Audio);
    let gv = match id {
        1 => Some(g::gv_playmusic),
        2 => Some(g::gv_musicvolume),
        3 => Some(g::gv_playsound),
        4 => Some(g::gv_soundvolume),
        i if i >= FIRST_ID && ((i - FIRST_ID) as usize) < n => Some(GV_AT + 0x40 * (i - FIRST_ID)),
        _ => None,
    };
    let step_value = |gv: i32| {
        // (a volume moves in steps of a twentieth of its range; the game's own steps depend on how long the button is held)
        let unit = if slider { ((lw(gv + 8) - lw(gv + 4)) / 20).max(1) } else { 1 };
        let step = if lw(front + 16) > 0 { unit } else { -unit };
        let value = (lw(gv) + step).clamp(lw(gv + 4), lw(gv + 8));
        if value != lw(gv) {
            soundStart(141, 0);
            gvSetValue(gv, value);
        }
        sw(front + 12, flags & -61);
    };
    let order = Ordering::Relaxed;
    if editing() != 0 && editing() != id {
        EDIT.store(0, order);
    }
    if editing() != 0 {
        if pressed & (16 | 64) != 0 {
            EDIT.store(0, order);
            if let (Some(p), true) = (PROVIDER.get(), id >= FIRST_ID && mode() == Mode::Video) {
                if pressed & 64 != 0 {
                    p.commit((id - FIRST_ID) as usize);
                } else {
                    p.cancel((id - FIRST_ID) as usize);
                }
            }
            if pressed & 16 != 0 {
                joyClearAll(player); // the triangle button only ends editing, it does not leave the page
            }
            soundStart(141, 0);
            if pressed & 64 != 0 && mode() == Mode::Controls && id == FIRST_ID && lw(GV_AT) == 0 {
                // X on the "Custom" configuration: choose the buttons
                open_custom_controls(page);
                joyClearAll(player);
            }
        } else if flags & 0x30 != 0 {
            if let Some(gv) = gv {
                step_value(gv);
            }
        }
        return;
    }
    if pressed & 64 != 0 {
        if let Some(target) = row_page_target(id) {
            if target == CREDITS_START {
                start_credits();
            } else {
                frontPageSet(0, target);
                pageIn(page);
            }
            joyClearAll(player);
            return;
        }
    }
    if pressed & 64 != 0 && gv.is_some() {
        EDIT.store(id, order);
        soundStart(141, 0);
        return;
    }
    if flags & 0x30 != 0 && layout().paged {
        // left and right turn the page; the cursor goes to its first line
        let pages = layout().pages();
        let to = if lw(front + 16) > 0 { (page_shown() + 1) % pages } else { (page_shown() + pages - 1) % pages };
        PAGE.store(to, order);
        let first = if mode() == Mode::Audio && to == 0 { 1 } else { FIRST_ID + (0..n as i32).find(|&i| layout().row_page(i as usize) == to).unwrap_or(0) };
        sh(page + 4, first);
        soundStart(140, 0);
        sw(front + 12, flags & -61);
        return;
    }
    if flags & 0x30 != 0 {
        let right = lw(front + 16) > 0;
        let columns = layout().columns.len();
        let from = (0..columns).find(|&c| column_rows(n, c).iter().any(|r| r.0 == id));
        if let Some(from) = from {
            let to = if right { from + 1 } else { from.wrapping_sub(1) };
            let slot = column_rows(n, from).iter().find(|r| r.0 == id).map_or(0, |r| r.1);
            if to < columns {
                let target = column_rows(n, to).into_iter().min_by_key(|r| ((r.1 - slot).abs(), r.1));
                if let Some((tid, _)) = target {
                    sh(page + 4, tid);
                    soundStart(140, 0);
                }
            }
        }
        // (a press that has no other column to go to does nothing either: it must not change the value)
        sw(front + 12, flags & -61);
    }
}

// ---- the pause menu ----------------------------------------------------------------------------------------------------
//
// The SETUP page of the pause menu gets a fourth item that opens this page (a page description of its own, with the tick
// of the front end's page: `audiovideo_pageTick` works in both places) and the page uses the game's small menu font there,
// like the splitscreen menus do; the font is put back as soon as the page is no longer ticking (`av_frame_end`).

const MAINSETUP_AT: i32 = BASE + 0x6000;
const PAUSE_TEXT_AT: i32 = BASE + 0x6300;

/// frame count of the last tick of the page in the pause menu, and the font it replaced
static PAUSE_FONT: Mutex<Option<(u64, i32)>> = Mutex::new(None);

/// "page" that an Exit Game item opens: selecting it ends the program (`frontPageSet` / `inPage` check for it)
const EXIT_PAGE: i32 = BASE + 0x6500;
const GAMEMODE_AT: i32 = BASE + 0x6600;
const EXIT_TEXT_AT: i32 = BASE + 0x6a00;

/// The question box of the pages that offer to exit the game has been shown (its answer is read when the page ticks again).
static ASKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Show the box that asks whether to exit the game (X yes, triangle no; the callback is the MapMaker's `confirmyesno`, which answers 1 or 2),
/// with the sound of a menu choice.
fn ask_to_quit(player: i32) {
    joyClearAll(player);
    let mut at = BASE + 0xe200;
    let text = put_str(&mut at, "Are you sure you want to quit the game ?
`^2 YES   ^3 NO");
    frontStartMessage(0, text, 0x6464_647f, 0x24d268);
    soundStart(143, 0);
    ASKED.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// The answer of the box, when the page that showed it ticks again: yes ends the program.  True when there was an answer to read.
fn quit_answer() -> bool {
    if ASKED.swap(false, std::sync::atomic::Ordering::Relaxed) {
        if messageGetState_2312c0(0) == 1 {
            std::process::exit(0);
        }
        return true;
    }
    false
}

/// The page where the number of players is chosen (1up, 2up, 3up, 4up), at its tick: the triangle button shows the question box.  True when the
/// page's tick is to stop (the box has just been shown).
pub fn av_nplayers_exit(_page: i32) -> bool {
    CONFIRM.store(false, std::sync::atomic::Ordering::Relaxed);
    if quit_answer() {
        return false;
    }
    let player = lw(lw(g::currentplayer));
    if joyNewBut(player) & 16 == 0 {
        return false;
    }
    ask_to_quit(player);
    true
}

/// The game type page of the front end (the main menu), at its tick: choosing Exit Game shows the same question box instead of a page.
pub fn av_gametype_exit(page: i32) -> bool {
    if quit_answer() {
        return false;
    }
    let player = lw(lw(g::currentplayer));
    // (the item of Exit Game is number 9; X or Start choose it)
    if lhu(page + 4) != 9 || joyNewBut(player) & (64 | 2048) == 0 {
        return false;
    }
    ask_to_quit(player);
    true
}

/// Selecting Exit Game asks first: the item opens a page of the same kind as the audio / video one (it ticks with
/// `audiovideo_pageTick`, which builds this page's table while `CONFIRM` is set) with a Yes item, which opens `EXIT_NOW_PAGE`,
/// and a No item.
const EXIT_NOW_PAGE: i32 = BASE + 0x6540;
const CONFIRM_FRONT_AT: i32 = BASE + 0x6b00;
const CONFIRM_MENU_AT: i32 = BASE + 0x6c00;
const CONFIRM_TEXT_AT: i32 = BASE + 0x6e00;
static CONFIRM: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// The game's end game page is open as the exit confirmation (set on opening it, cleared when SETUP is built again).
static CONFIRM_GAME: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn confirming() -> bool {
    CONFIRM.load(std::sync::atomic::Ordering::Relaxed)
}

/// Front end, `frontPageSet`: the page about to be opened.
pub fn av_front_page(page: i32) -> i32 {
    if page == EXIT_NOW_PAGE {
        std::process::exit(0);
    }
    for (marker, at, first) in [(AUDIO_PAGE, AUDIO_FRONT_AT, 1), (VIDEO_PAGE, VIDEO_FRONT_AT, FIRST_ID), (GAME_PAGE, GAME_FRONT_AT, FIRST_ID), (CONTROLS_PAGE, CONTROLS_FRONT_AT, FIRST_ID), (CUSTOM_PAGE, CUSTOM_FRONT_AT, FIRST_ID), (MODS_PAGE, MODS_FRONT_AT, FIRST_ID)] {
        if page == marker {
            EDIT.store(0, Ordering::Relaxed);
            PAGE.store(0, Ordering::Relaxed);
            copy(at, g::audiovideo_page, 16);
            sw(at + 8, first); // the cursor starts on the first row
            set_mode(at);
            return at;
        }
    }
    if page == EXIT_PAGE {
        CONFIRM.store(true, std::sync::atomic::Ordering::Relaxed);
        copy(CONFIRM_FRONT_AT, g::audiovideo_page, 16);
        sw(CONFIRM_FRONT_AT + 8, 3); // the cursor starts on No
        return CONFIRM_FRONT_AT;
    }
    page
}

/// Pause menu, `inPage`: true when the page was one of the exit pages and has been dealt with.
pub fn av_in_page(player: i32, page: i32) -> bool {
    if set_mode(page) {
        EDIT.store(0, Ordering::Relaxed);
        PAGE.store(0, Ordering::Relaxed);
        return false;
    }
    if page == EXIT_NOW_PAGE {
        std::process::exit(0);
    }
    if page == EXIT_PAGE {
        // the game's own end game page (Cancel / Confirm); `CONFIRM_GAME` makes its tick treat Confirm as exit
        CONFIRM_GAME.store(true, std::sync::atomic::Ordering::Relaxed);
        inPage(player, g::confirmendgame_page);
        return true;
    }
    false
}

/// Title of the front end's page: "Exit Game" on the confirmation.
pub fn av_title(_page: i32, title: i32) -> i32 {
    if !confirming() && !in_game() {
        let mut at = CONFIRM_TEXT_AT + 0x180;
        let name = match mode() {
            Mode::Audio => "Audio Options",
            Mode::Video => "Video Options",
            Mode::Game => "Game Options",
            Mode::Controls => "Controls",
            Mode::Custom => "Custom Controls",
            Mode::Mods => "Mods",
        };
        return put_str(&mut at, name);
    }
    if confirming() && !in_game() {
        let mut at = CONFIRM_TEXT_AT + 0x100;
        return put_str(&mut at, "Exit Game");
    }
    title
}

/// The table of the confirmation page, laid out like the game's own pages (STORY / ARCADE): the question and the two answers
/// centred, and the legend at the bottom.  The triangle button and the "No" item go back to the game type page, which opened it.
fn confirm_menu(page: i32) -> i32 {
    let mut at = CONFIRM_TEXT_AT;
    let ask = put_str(&mut at, "Are you sure you want to quit the game?");
    let yes = put_str(&mut at, "Yes");
    let no = put_str(&mut at, "No");
    let back = put_str(&mut at, "^3 Back");
    let select = put_str(&mut at, "^2 Select");
    let m = CONFIRM_MENU_AT;
    let src = g::audiovideo_menu;
    let modes = g::gamemode_menu;
    let item = |k: i32| m + 16 + 32 * k;
    let player = lw(lw(g::front));
    let pressed = joyNewBut(player);
    // the answers are chosen here: X on No or the triangle button goes back (as the game type page's own tick does)
    if pressed & 16 != 0 || (pressed & 64 != 0 && lhu(page + 4) == 3) {
        pageOut();
        joyClearAll(player);
    }
    // the game type page's first item (STORY) is the pattern for the centred items
    let story = (0..lw(modes)).map(|k| modes + 16 + 32 * k).find(|&it| lhu(it + 8) == 1).unwrap_or(modes + 16);
    copy(m, src, 16);
    sw(m, 5);
    sw(m + 4, lw(modes + 4));
    for k in 0..3 {
        copy(item(k), story, 32);
        for f in [12, 20, 24, 28] {
            sw(item(k) + f, 0);
        }
    }
    let centred = lhu(story + 4);
    let at_y = |y: i32| (y << 16) | centred;
    sw(item(0), ask);
    sw(item(0) + 4, at_y(0x36));
    sw(item(0) + 8, (0x4010 << 16) | 1);
    sw(item(0) + 16, 0);
    sw(item(1), yes);
    sw(item(1) + 4, at_y(0x58));
    sw(item(1) + 8, (0x111 << 16) | 2);
    sw(item(1) + 12, EXIT_NOW_PAGE);
    sw(item(1) + 16, (3 << 16) | 3);
    sw(item(2), no);
    sw(item(2) + 4, at_y(0x6f));
    sw(item(2) + 8, (0x110 << 16) | 3);
    sw(item(2) + 12, 0);
    sw(item(2) + 16, (2 << 16) | 2);
    // the legend: Back and Select, at the places the original back item and the other pages' legends are
    let legend = src + 16 + 32 * 5;
    let (x, y) = (lhu(legend + 4), lhu(legend + 6));
    for (k, text, dx) in [(3, back, 0), (4, select, 0x6d)] {
        copy(item(k), legend, 32);
        sw(item(k), text);
        sw(item(k) + 4, (y << 16) | (x + dx));
        sw(item(k) + 8, (0x4000 << 16) | (k + 1));
        sw(item(k) + 12, 0);
        sw(item(k) + 16, 0);
    }
    m
}

/// Append an item that opens `page` to a menu table copied to `at`: it follows the last item and the cursor goes round the
/// items in order (`ring` is the item numbers in that order, the new one last).
fn append_item(at: i32, label: i32, page: i32, flags: i32, id: i32, y: Option<i32>, ring: &[i32]) {
    let n = lw(at);
    let item = |k: i32| at + 16 + 32 * k;
    let new = item(n);
    copy(new, item(n - 1), 32);
    sw(new, label);
    if let Some(y) = y {
        sw(new + 4, y << 16);
    }
    sw(new + 8, (flags << 16) | id);
    sw(new + 12, page);
    let before = ring[ring.len() - 2];
    let first = ring[0];
    sw(new + 16, (first << 16) | before);
    sw(at, n + 1);
}

/// The table of the pause menu's SETUP page: Controls, Audio Options, Video Options and Exit Game.
pub fn av_mainsetup_menu() -> i32 {
    EDIT.store(0, Ordering::Relaxed);
    PAGE.store(0, Ordering::Relaxed);
    CONFIRM.store(false, Ordering::Relaxed);
    CONFIRM_GAME.store(false, Ordering::Relaxed);
    let src = g::mainsetup_menu;
    let n = lw(src);
    copy(MAINSETUP_AT, src, 16 + 32 * n);
    let item = |k: i32| MAINSETUP_AT + 16 + 32 * k;
    let mut ring: Vec<i32> = (1..=n).collect();
    let mut at = PAUSE_TEXT_AT;
    if PROVIDER.get().is_some() && n >= 3 {
        // the page descriptions: the tick of the front end's page, no drawing of its own
        for (desc, first) in [(CONTROLS_PAUSE_AT, FIRST_ID), (AUDIO_PAUSE_AT, 1), (VIDEO_PAUSE_AT, FIRST_ID)] {
            copy(desc, g::prefs_page, 16);
            sw(desc, 0x2364c8);
            sw(desc + 8, first);
        }
        // (the first item is the invisible one the triangle button follows back) the next two (Setup Controls, Preferences) become
        // Controls and Audio Options; Video Options and Exit Game follow
        let label = put_str(&mut at, "Controls...");
        sw(item(1), label);
        sw(item(1) + 12, CONTROLS_PAUSE_AT);
        let label = put_str(&mut at, "Audio Options...");
        sw(item(2), label);
        sw(item(2) + 12, AUDIO_PAUSE_AT);
        let label = put_str(&mut at, "Video Options...");
        ring.push(ring.len() as i32 + 1);
        append_item(MAINSETUP_AT, label, VIDEO_PAUSE_AT, 0x31, ring.len() as i32, None, &ring);
        // with more than one player only the first one changes the sound and the picture: the items are greyed out for the others
        if lw(g::lvnumplayers) > 1 && lw(lw(g::currentplayer)) != 0 {
            for k in [2, 3] {
                sh(item(k) + 10, lhu(item(k) + 10) | 0x40);
            }
        }
    }
    let label = put_str(&mut at, "Exit Game");
    ring.push(ring.len() as i32 + 1);
    append_item(MAINSETUP_AT, label, EXIT_PAGE, 0x31, ring.len() as i32, None, &ring);
    // the cursor goes round the items in order (item numbers are 1, 2, ...)
    let total = lw(MAINSETUP_AT);
    for k in 0..total {
        let (up, down) = ((k + total - 1) % total + 1, (k + 1) % total + 1);
        sw(item(k) + 16, (down << 16) | up);
    }
    MAINSETUP_AT
}

/// The table of the front end's game type page (STORY, ARCADE, ...): the small items below them are Controls, Audio Options, Video
/// Options, Game Options (with View Credits and View Statistics) and Exit Game.
pub fn av_gamemode_menu() -> i32 {
    CONFIRM.store(false, Ordering::Relaxed);
    let src = g::gamemode_menu;
    let n = lw(src);
    copy(GAMEMODE_AT, src, 16 + 32 * n);
    let item = |k: i32| GAMEMODE_AT + 16 + 32 * k;
    // the item of the audio / video page: it becomes the Audio Options item and the others are made from it
    let last = (0..n).find(|&k| lhu(item(k) + 8) == 6).unwrap_or(n - 2);
    let flags = lhu(item(last) + 10);
    // where the cursor goes up from the first of the small items: what it went up to from View Statistics (the credits item, when it
    // has a label, is the one above it, and it goes too)
    let id_of = |k: i32| lhu(item(k) + 8);
    let up_of = |id: i32| (0..n).find(|&k| id_of(k) == id).map(|k| lw(item(k) + 16) & 0xffff);
    let mut above = up_of(5).unwrap_or(3);
    if above == 4 {
        above = up_of(4).unwrap_or(3);
    }
    let mut at = EXIT_TEXT_AT;
    let label = put_str(&mut at, "Audio Options");
    sw(item(last), label);
    sw(item(last) + 12, AUDIO_PAGE);
    let new_items = [(10, "Controls", CONTROLS_PAGE), (11, "Video Options", VIDEO_PAGE), (12, "Game Options", GAME_PAGE), (13, "Mods", MODS_PAGE), (9, "Exit Game", EXIT_PAGE)];
    for (k, &(id, text, page)) in new_items.iter().enumerate() {
        let it = item(n + k as i32);
        copy(it, item(last), 32);
        let label = put_str(&mut at, text);
        sw(it, label);
        sw(it + 8, ((if id == 9 { 1 } else { flags }) << 16) | id);
        sw(it + 12, page);
    }
    // View Credits and View Statistics are on the Game Options page: their items leave the table
    let mut total = 0;
    for k in 0..n + 5 {
        if matches!(id_of(k), 4 | 5) {
            continue;
        }
        if total != k {
            copy(item(total), item(k), 32);
        }
        total += 1;
    }
    sw(GAMEMODE_AT, total);
    // the items that went down to View Credits / View Statistics now go down to Controls (and the small items' way back up is set below)
    for k in 0..total {
        if matches!(id_of(k), 1..=3) {
            let nav = lw(item(k) + 16);
            let fix = |id: i32| if matches!(id, 4 | 5) { 10 } else { id };
            sw(item(k) + 16, (fix((nav >> 16) & 0xffff) << 16) | fix(nav & 0xffff));
        }
    }
    // The small items sit one line apart, ending a little above the back / select line (Exit Game is the last one, the items
    // above it are placed from there).
    let (exit_y, pitch) = (0x9c_i32, 14);
    let order = [10, 6, 11, 12, 13, 9];
    let find = |id: i32| (0..total).find(|&k| id_of(k) == id);
    for (row, &id) in order.iter().rev().enumerate() {
        if let Some(k) = find(id) {
            let y = exit_y - pitch * row as i32;
            sw(item(k) + 4, (y << 16) | lhu(item(k) + 4));
        }
    }
    // the cursor goes down in the order of the items: the new items, Exit Game, then STORY again
    for (i, &id) in order.iter().enumerate() {
        let Some(k) = find(id) else { continue };
        let down = order.get(i + 1).copied().unwrap_or(1);
        let up = if i == 0 { above } else { order[i - 1] };
        sw(item(k) + 16, (down << 16) | up);
    }
    let first = (0..total).find(|&k| id_of(k) == 1).unwrap_or(0);
    sw(item(first) + 16, (lhu(item(first) + 18) << 16) | 9);
    GAMEMODE_AT
}

/// Called every tick of the page: in the pause menu, use the smaller font while it is shown.
pub fn av_pause_font() {
    let now = ts_sdk::frame_count() as u64;
    let mut f = PAUSE_FONT.lock().unwrap();
    match *f {
        Some((_, saved)) => *f = Some((now, saved)),
        None => *f = Some((now, lw(g::menuitem_normalfont))),
    }
    sw(g::menuitem_normalfont, 1);
}

/// Once per frame: put the font back when the page has stopped ticking.
pub fn av_frame_end() {
    let mut f = PAUSE_FONT.lock().unwrap();
    if let Some((last, saved)) = *f {
        if ts_sdk::frame_count() as u64 > last + 2 {
            sw(g::menuitem_normalfont, saved);
            *f = None;
        }
    }
}

/// The menu of the game's end game confirmation page; for the exit confirmation, the triangle button goes back to SETUP.
pub fn av_confirm_table(page: i32) -> i32 {
    if !CONFIRM_GAME.load(std::sync::atomic::Ordering::Relaxed) {
        return g::confirmendgame_menu;
    }
    let src = g::confirmendgame_menu;
    copy(CONFIRM_MENU_AT, src, 16 + 32 * lw(src));
    let back = (0..lw(src)).find(|&k| lhu(CONFIRM_MENU_AT + 16 + 32 * k + 8) == 0x8000);
    if let Some(k) = back {
        sw(CONFIRM_MENU_AT + 16 + 32 * k + 12, g::mainsetup_page);
    }
    CONFIRM_MENU_AT
}

/// Called first by the end game page's tick: when it is the exit confirmation, handle the choice here (Cancel goes back to
/// SETUP, Confirm ends the program).  True when the press was dealt with.
pub fn av_confirm_pressed(page: i32) -> bool {
    if !CONFIRM_GAME.load(std::sync::atomic::Ordering::Relaxed) {
        return false;
    }
    let player = lw(lw(g::currentplayer));
    if joyNewBut(player) & 64 == 0 {
        return false;
    }
    match lhu(page + 4) {
        2 => std::process::exit(0),
        1 => {
            outPage(player, g::mainsetup_page);
            joyClearAll(player);
            true
        }
        _ => false,
    }
}

/// The arcade level list: the console game's (MapMaker is the last of them, place 18 is special to the game's code), then Ron (25).
const ARCADE_LEVELS_AT: i32 = BASE + 0x7600;

pub fn av_arcade_levels() -> i32 {
    // Ron has no data for two of its six modes: KnockOut (mask 16) needs bag spawn points (the game divides by their number)
    // and LastStand (mode 6, mask 32) needs a per-level set-up (`enemySetup` calls exit() for a level it does not know), so they
    // are taken out of its mode list
    sw(g::setup_ron_ls, lw(g::setup_ron_ls) & !(16 | 32));
    let mut at = ARCADE_LEVELS_AT;
    for i in 0..19 {
        sw(at, lw(g::arcadelevels + 4 * i));
        at += 4;
    }
    for id in [25] {
        sw(at, id);
        at += 4;
    }
    sw(at, 0);
    ARCADE_LEVELS_AT
}

/// A page is left (`pageOut`, `outPage`): from the Custom controls page one goes back to the Controls page.
pub fn av_page_out() {
    if mode() == Mode::Custom {
        MODE.store(Mode::Controls as u8, Ordering::Relaxed);
        EDIT.store(0, Ordering::Relaxed);
        PAGE.store(0, Ordering::Relaxed);
    }
}
