//! Small functions for the music stream, the demo and credits, the level restart and the lists of textures to preload.

use crate::*;

/// Load a texture by name: a slot is taken from the end of the current texture buffer, the texture is loaded, and the slot is recorded at `a2` (@ 0x2b79b8).
pub fn emLoadTextureName(a0: i32, a1: i32, a2: i32) {
    let buffer = lw(g::thistexbuf);
    let slot = lw(buffer + 0x1a08);
    sw(buffer + 0x1a08, slot - 1);
    emLoadTexture(slot, a0, a1);
    sw(a2, slot);
}

/// Start the music stream: the stream service is set up, then it is initialised and given a buffer size (@ 0x205c38).
pub fn musicMake() {
    if stream_RPCinit() < 0 {
        return;
    }
    stream_RPC(0x7ff0, 0);
    stream_RPC(0x8000, 0x60000);
}

/// Start the demo `a0` through a deferred reset (@ 0x225790).
pub fn demoStart(a0: i32) {
    let frame = Frame::new(144);
    let sp = frame.sp();
    memset(sp, 0, 112);
    sw(sp + 64, a0);
    sw(sp + 80, 0x2000);
    gameDeferedReset(sp);
}

/// Show the goodies page of the front end: the message is cleared, the flag 3 of the front end's state dropped, and the next goodie's message shown (@ 0x240e50).
pub fn frontmpgSetGoodiesPage() {
    splitMessageClear(0);
    let state = 0x1fc11f8;
    sw(state + 4, lw(state + 4) & -4);
    frontPageSet(0, g::goodies_page);
    let message = getNextGoodieString();
    goodieMessage(message);
}

/// Draw the map maker's 2D view: the grid without blending, the rooms with it, and (when no menu page is up) the lasso while selecting, or else the object in use (@ 0x24b1e0).
pub fn mmDraw2DView() {
    dlSetBlend(0);
    mmDrawGrid();
    dlSetBlend(1);
    mmDrawRooms();
    if lw(lw(g::front) + 20) != 0 {
        return;
    }
    if (lw(g::editflags) & 64) == 0 {
        mmDrawUsedObject();
        return;
    }
    mmDrawLasso();
}

/// Open the map maker's main menu: the page is set, the selection flags of the front end are dropped, the edit flag 8 is set and a sound plays (@ 0x24edc0).
pub fn mmStartMainMenu() {
    frontPageSet(0, g::mmmain_page);
    let front = lw(g::front);
    let flags = lw(g::editflags) | 8;
    sw(front + 12, lw(front + 12) & -4);
    sw(g::editflags, flags);
    soundStart(140, 0);
}

/// Stop the music when it is playing (the state call answers 0x5000) and close it (@ 0x205b90).
pub fn musicStopClose() {
    let state = stream_RPC(0x80e0, 0);
    if state == 0x5000 {
        stream_RPC(0x80f0, 0);
        sw(g::musicstate, 0);
    }
    sw(g::musicopened, 0);
    sw(g::tunenum, 0);
}

/// Go into a page of the menu: the depth counter rises, the page's entry is recorded and the sound 143 plays (@ 0x2301c0).
pub fn pageIn(a0: i32) {
    let selected = lhu(a0 + 4);
    let depth = lw(0x3aff2c) + 1;
    let page = lw(a0);
    sw(0x3aff2c, depth);
    sh(page + 8, selected);
    sw((depth << 3) + 0x1fc1308, 0);
    soundStart(143, 0);
}

/// The format of the image file `a0`: the file is loaded into the big buffer, which lies in the scratch buffer, and its header is read (@ 0x2b9b98).
pub fn ImageGetDetails(a0: i32) -> i32 {
    sw(g::bigbuffer, g::scratchbuffer);
    sw(g::downloadbuffer, g::scratchbuffer + 66688);
    let size = fileSize(a0);
    fileLoadTo(a0, lw(g::bigbuffer), size);
    GetFormat(lw(g::bigbuffer))
}

/// Reset the credits: the data of the tunnel is allocated and cleared, the background and the text reset, and the time is 0 (@ 0x2ae640).
pub fn creditsReset() {
    let data = memAlloc(0x32c80, 1);
    sw(g::currenttunneldata, data);
    memset(data, 0, 0x32c80);
    creditsBgReset();
    creditsTextReset();
    sw(g::creditstime, 0);
}

/// Restart everything before a level restart, in the order the game does (@ 0x225c50).
pub fn lvRestartBefore() {
    packfileRestart();
    animRestart();
    memdbRestart(0);
    emRestart();
    zbtestRestart();
    timerRestart();
    obRestart();
    propRestart();
    chrRestart();
    camRestart();
    windowRestart();
    frontRestart();
    enemyRestart();
    soundRestart();
    bglightRestart();
}

/// The tick of an explosion particle: its damage is done to what it touches (@ 0x29d530).
pub fn particleExplosionTick(a0: i32) {
    let f12_1 = lwc1(a0 + 12);
    explosionDamage(lw(a0 + 964), a0 + 28, lw(a0 + 932), a0 + 936, a0 + 948, lw(a0 + 960), f12_1);
}

/// The answer of the clear map question: with the button 64 the map is backed up, cleared and the view reset (answer 1); otherwise the answer is 2 when the button 16 was pressed
/// and 0 while waiting (@ 0x24ee98).
pub fn clearmapconfirmfunc() -> i64 {
    let pressed = joyNewBut(0);
    if (pressed & 64) == 0 {
        let other = joyNewBut(0);
        ((other as i64 as u64) >> 3) as i64 & 2_i64
    } else {
        mmBackupMap();
        mmClearMap();
        mmInitView();
        sw(0x3b004c, 0);
        1_i64
    }
}

/// The tick of the loading screen: after 10 ticks a deferred reset to the menu (102) is made (@ 0x22a148).
pub fn loadingTick() {
    let frame = Frame::new(128);
    let sp = frame.sp();
    let ticks = lw(0x3aff0c) + 1;
    sw(0x3aff0c, ticks);
    if ticks == 10 {
        memset(sp, 0, 112);
        sw(sp + 64, 102);
        gameDeferedReset(sp);
    }
}

/// Preload the three textures of the credits' tunnel (@ 0x2ae5f0).
pub fn creditsPreload() {
    let mut at = g::tunneltexturename;
    for _ in 0..3 {
        let name = lw(at);
        at += 4;
        preloadAddTexName(name, 0);
    }
}

/// Preload the twelve decal textures (@ 0x2a2488).
pub fn decalPreload() {
    let mut at = g::s_DecalTextures + 8;
    for _ in 0..12 {
        let name = lw(at);
        at += 12;
        preloadAddTexName(name, 0);
    }
}

/// The tick of the map name page: the string editor ticks, and when the text box is done or cancelled the front end's page count drops (@ 0x24d220).
pub fn mapnameedit_pageTick(_a0: i32) {
    stringeditTick(0);
    let textbox = lw(g::textbox);
    let front;
    if lw(textbox + 32) != 0 {
        front = lw(g::front);
    } else {
        front = lw(g::front);
        if lw(textbox + 36) == 0 {
            return;
        }
    }
    sw(front + 20, lw(front + 20) - 1);
}

/// Open the music file of the tune `a0` (the stream file name is made from the tune number plus 1000) and keep that it is open (@ 0x205a50).
pub fn musicOpen(a0: i32) {
    if a0 != 0 {
        sprintf(&[g::streamFilename as i64, 0x389430_i64, (a0 + 1000) as i64]);
        stream_RPC(0x8020, g::streamFilename);
        sw(g::musicopened, 1);
    }
}

/// Preload the eleven particle textures (@ 0x2968d8).
pub fn particlePreload() {
    let mut at = g::particletextures;
    for _ in 0..11 {
        let name = lw(at);
        at += 16;
        preloadAddTexName(name, 0);
    }
}

/// Draw the scores page for the game kind: the rankings table, or for the kind 4 the escort result (completed when the state is 4), or for the kind 6 the last stand result
/// (@ 0x212010).
pub fn scores_pageGfx() {
    let kind = lw(g::curgameparms + 72);
    let menu = lw(lw(g::currentplayer)) * 92 + 0x1fb3240;
    if kind != 4 {
        if kind != 6 {
            drawRankingsTable(lhu(menu + 4));
            return;
        }
        drawLastStand();
        return;
    }
    if lw(g::gamestate) != kind {
        drawEscortNotCompleted();
        return;
    }
    drawEscortCompleted();
}

/// Preload the seven flare textures (@ 0x2a65b8).
pub fn zbtestPreload() {
    let mut at = g::flareTexturenames_36a290;
    for _ in 0..7 {
        let name = lw(at);
        at += 4;
        preloadAddTexName(name, 0);
    }
}
