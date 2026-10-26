//! The HUD boxes of the challenge and last stand, the fillets of a model's bones, the start of the decals, and the pages of the split screen.

use crate::*;

/// Draw the box of the time left of the challenge (hours and minutes of the ticks left, rounded up, with the language's title at +3248) at height `y` (@ 0x2b1780).
pub fn hudChallengeTimeLeftGfx(y: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let x = lw(g::window) + (32 >> (lw(g::window + 88) & 31));
    let left = GetChallengeTimeLeft();
    let minutes = (left + 59).wrapping_div(60).wrapping_rem(60);
    let hours = (left + 59).wrapping_div(60).wrapping_div(60).wrapping_rem(60);
    let text = vafmt(&[0x3a9a38, hours as i64, minutes as i64]);
    let language = lw(g::cur_language);
    sw(sp, 1);
    drawValueBox(lw(lw((language << 2) + g::languages) + 3248), text, x, y, 112, 22, 0x50505080, 0x7f7f7f80);
}

/// Draw the box of the last stand's score (the player's +248 with the language's title at +3252) at height `y` and `across` more to the right, in the player's colour (@ 0x2b1900).
pub fn hudLSScoreGfx(_unused: i32, y: i32, across: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let character = lw(lw(g::currentplayer) + 384);
    let record = lw(character + 352);
    let x = lw(g::window) + (32 >> (lw(g::window + 88) & 31)) + across;
    let text = vafmt(&[0x3afba8, lw(record + 248) as i64, across as i64, character as i64, record as i64]);
    let record = lw(lw(lw(g::currentplayer) + 384) + 352);
    let language = (lw(g::cur_language) << 2) + g::languages;
    sw(sp, 1);
    drawValueBox(lw(lw(language) + 3252), text, x, y, 125, 22, lw(record + 20), 0x7f7f7f80);
}

/// Make the fillet matrices of the bones of the model `model` from the bone `bone` down (the bone records of 80 bytes; kind 2 is a fillet joint whose matrix is made between its
/// own and its parent's by `calInbetweenMatrix`; the children at +3 and the next bones at +4 are followed) (@ 0x214a88).
pub fn calDoubleFillets(model: i32, bone: i32, amount: f32) {
    let mut bone = bone;
    let mut skeleton = lw(model);
    loop {
        let bones = skeleton - lw(skeleton) * 80;
        let record = bones + bone * 80;
        if lb(record) == 2 {
            let matrices = lw(model + 4);
            let to = matrices + (lb(record + 1) << 6);
            let parent = lb(lb(record + 3) * 80 + bones + 1) << 6;
            calInbetweenMatrix(lw(lw(model + 88)) + parent, matrices + parent, to);
        }
        let child = lb(record + 3);
        if child >= 0 && lb(record) != 3 {
            calDoubleFillets(model, child, amount);
        }
        bone = lb(record + 4);
        if bone < 0 {
            break;
        }
        skeleton = lw(model);
    }
}

/// Get and clear the memory for the decals (240 bytes each: 200 for 4 players, 265 for 3, 400 for 2, else 500) and restart them (@ 0x2a2658).
pub fn decalReset() {
    let players = lw(g::lvnumplayers);
    let max = if players == 2 {
        400
    } else if players == 3 {
        265
    } else if players == 4 {
        200
    } else {
        500
    };
    sw(g::s_MaxNumDecals, max);
    let memory = memAlloc(lw(g::s_MaxNumDecals) * 240, 1);
    let count = lw(g::s_MaxNumDecals);
    sw(g::s_Decals, memory);
    memset(memory, 0, count * 240);
    decalRestart();
}

/// Draw the box of the challenge's score (the display score text and value from `challengeGetDisplayScore`) at height `y`; 1 if one was drawn (@ 0x2b1858).
pub fn hudChallengeScoreGfx(y: i32) -> i32 {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let x = lw(g::window) + (32 >> (lw(g::window + 88) & 31));
    challengeGetDisplayScore(sp + 16, sp + 20);
    if lw(sp + 16) == 0 {
        return 0;
    }
    let dither = dlSetDitherMatrix(2);
    sw(sp, 0);
    drawValueBox(lw(sp + 16), lw(sp + 20), x, y, 112, 22, 0x50505080, 0x7f7f7f80);
    dlSetDitherMatrix(dither);
    1
}

/// Draw a page of the split screen: the front end's drawing for each other player's window (the players 1 up), then for the first, and the page's own draw function (+4 of `def`) (@ 0x231028).
pub fn splitpageDraw(page: i32, def: i32) {
    let mut player = 1;
    if player < lw(0x3aff20) {
        loop {
            frontSet(player);
            let mode = lw(0x3aff20);
            let this = player;
            player += 1;
            SetWindow(this, mode, 1);
            frontGfx();
            if player >= lw(0x3aff20) {
                break;
            }
        }
    }
    frontSet(0);
    SetWindow(0, lw(0x3aff20), 1);
    let draw = lw(def + 4);
    if draw != 0 {
        call_indirect(draw, [page, draw, 0, 0, 0, 0, 0, 0], [0.0; 8]);
    }
}

/// Set the box (at 0x1fc33a0: x, y, right, bottom) of the zoom on the player `player`'s window: in one-player games a fixed box, else a box inside that window (moved up 32
/// for the lower windows of three or four players); and the zoom's time and flag (@ 0x23f190).
pub fn soZoomStart(player: i32) {
    let players = lw(0x3aff20);
    if players != 1 {
        SetWindow(player, players, 0);
        let x = lw(g::window);
        let y = lw(g::window + 20);
        let record = 0x1fc33a0;
        sw(0x1fc33a0, x + 40);
        sw(record + 4, y + 14);
        sw(record + 8, x + 280);
        sw(record + 12, y + 98);
        SetWindow(0, 1, 0);
        if lw(0x3aff20) >= 3 && player >= 2 {
            let bottom = lw(record + 12) - 32;
            sw(record + 4, lw(record + 4) - 32);
            sw(record + 12, bottom);
        }
    } else {
        let record = 0x1fc33a0;
        sw(0x1fc33a0, 256);
        sw(record + 4, 88);
        sw(record + 12, 135);
        sw(record + 8, 383);
    }
    sw(0x3aff98, 10);
    sw(0x3aff9c, 1);
}

/// Draw the split screen message `index` (records of 260 bytes at 0x1fc1488: a flag, then the text at +4): when it is on, a zoom box and the text in the middle of it (@ 0x230b00).
pub fn splitMessageDraw(index: i32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let record = index * 260 + 0x1fc1488;
    if lw(record) == 0 {
        return;
    }
    sw(sp + 8, 559);
    sw(sp + 12, 167);
    sw(sp, 80);
    sw(sp + 4, 56);
    let text = record + 4;
    zoomboxDraw(80, 56, 559, 167, 0x2880, 0x7f7f7f80);
    let height = textHeight(text);
    let fg = lw(g::menuitem_fgcol);
    let spare = 224 - height;
    let bg = lw(g::menuitem_bgcol);
    guiDrawFormattedText(text, 0, (spare + ((spare as u32) >> 31) as i32) >> 1, 1, fg, bg);
}

/// The tick of a page of the split screen: flag 256 of the front end is set when the current page has a title function, the first player's windows are set and its page ticked
/// (the page's tick function at +0 of `def`), then each other player's front end is ticked in its window (@ 0x2310d0).
pub fn splitpageTick(page: i32, def: i32) {
    let flags = lw(0x1fc11fc) & -257;
    let titled = lw((lw(0x3aff2c) << 3) + 0x1fc1308);
    sw(0x1fc11fc, flags);
    if titled != 0 {
        sw(0x1fc11fc, flags | 256);
    }
    let mut player = 1;
    frontSet(0);
    SetWindow(0, lw(0x3aff20), 0);
    call_indirect(lw(def), [page, 0, 0, 0, 0, 0, 0, 0], [0.0; 8]);
    let mut players = lw(0x3aff20);
    while player < players {
        frontSet(player);
        let mode = lw(0x3aff20);
        let this = player;
        player += 1;
        SetWindow(this, mode, 0);
        frontTick();
        players = lw(0x3aff20);
    }
    frontSet(0);
}
