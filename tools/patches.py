"""Changes applied to the decompiled game code after generation (see `apply`).

The game was written for a fixed 60 Hz frame; the port can run levels at higher rates (see `timeTickStart` in
ts-game/src/manual/boot.rs).  Most of the logic already scales with the frame time the game measures
(`updateframesf`), but a few things count whole 60 Hz ticks or add a constant on every call.  They are changed here, in
one place, so that regenerating the sources keeps them.

Every patch is (old text, new text, expected number of occurrences) and is checked, so a decompiler change that
moves the code is noticed instead of silently dropping a patch.
"""
import re

# plain textual replacements applied to every function
GLOBAL = [
    # the integer tick counter used as the time base of animations (sin-table lookups, ...): use the continuous time
    ('lwc1(g::gameframe).to_bits() as i32 as f32', 'gameframe_f()'),
    # five custom weapon sets (manual/weaponsets.rs): the game reads the chosen set through this
    ('lw(g::gv_weaponset)', 'weaponset_effective()'),
    # five custom bot sets (manual/botsets.rs)
    ('lw(g::gv_chrset)', 'chrset_effective()'),
    # the weapon combinations in the port's memory, with room for the dual versions (manual/weapontable.rs)
    ('g::guncombos', 'GUNCOMBOS'),
    ('g::gunsinventory', 'GUNSINV'),
    # the level table in the port's memory, with the levels of mods (manual/customlevels.rs)
    ('g::lvinfo', 'LVINFO'),
    # the table of props and that of their walls and floors, with room for 480 rooms (manual/proptable.rs)
    ('g::proptable', 'PROPTABLE'),
    ('g::PropWallsFloors', 'PWFTAB'),
    # the lists of the props of each room (the game's held 213 rooms)
    ('0x1fe9598', 'ROOM_PROP_COUNTS'),
    ('0x1fd48c8', 'ROOM_PROP_LISTS'),
]

# `updateframesi` (whole ticks this frame) is 0 only while the game is paused in the original, so many tests of it mean
# "the game is running".  At a high frame rate most frames complete no whole tick, so those tests use the frame time
# instead.  A one-shot event (a hit, a shot, a sound) must not be lost on a frame without a whole tick.
RUNNING_GATE = re.compile(r'lw\(g::updateframesi\) (>|<=|>=|<|!=|==) 0(?![\w.])')
# functions whose test guards a random effect that happens once per tick (flicker, smoke): they stay on whole ticks
TICK_GATED = {
    'partGfx', 'electricityPropTick', 'specialfxOccasionalRocketSmokeTick', 'clockPropTick', 'mallclockPropTick',
    'boatsignPropTick', 'gongPropTick',
}

# function name -> [(old, new, count)]
FUNCS = {
    # a value that is only tested against 0 ("running") before it is used
    'propDamage': [('v0_1 = lw(g::updateframesi);\n        f21_1 = f12;', 'v0_1 = running_i();\n        f21_1 = f12;', 1)],
    'gunChrFire': [('v0_1 = lw(g::updateframesi);\n            if lw(a0 + 12) != 0 {', 'v0_1 = running_i();\n            if lw(a0 + 12) != 0 {', 1)],
    'gunRemoteFire': [('a5_1 = lw(g::updateframesi);\n    s3_1 = lw(a0 + 352);', 'a5_1 = running_i();\n    s3_1 = lw(a0 + 352);', 1)],
    'gunUpdateParts': [('a3_2 = lw(g::updateframesi);\n                if a5_1 != 18 {', 'a3_2 = running_i();\n                if a5_1 != 18 {', 1)],
    'propGfxRoom': [('                    a1_2 = a1;'+chr(10)+'                    obInstGfx(a0_2, a1_2);'+chr(10), '                    a1_2 = a1;'+chr(10)+'                    let mirrored = punch_mirrored(s0_1);'+chr(10)+'                    if mirrored {'+chr(10)+'                        dlLabel(0x4d495252);'+chr(10)+'                    }'+chr(10)+'                    obInstGfx(a0_2, a1_2);'+chr(10)+'                    if mirrored {'+chr(10)+'                        dlLabel(0);'+chr(10)+'                    }'+chr(10), 1)],
    'gunUpdate': [
        # the left fist hits on the left (the points of the hit test are put on the other side of the player)
        (' '*60+'swc1(sp + 232, f6_4);'+chr(10), ' '*60+'swc1(sp + 232, f6_4);'+chr(10)+' '*60+'punch_mirror_points(lw(a0 + 12), s5_1, sp + 208, sp + 224);'+chr(10), 1),
        # an unarmed swipe: the arms take turns (the left one is the right one mirrored)
        (' '*56+'sw(a0 + 156, 1);'+chr(10)+' '*56+'sh(a1_47 + 292, v1_22 | 0x8000);'+chr(10), ' '*56+'sw(a0 + 156, 1);'+chr(10)+' '*56+'punch_swing(lw(a0 + 12));'+chr(10)+' '*56+'sh(a1_47 + 292, v1_22 | 0x8000);'+chr(10), 1),
        ('v0_46 = lw(g::updateframesi);', 'v0_46 = running_i();', 1),
        # a dual weapon without a clip (rocket launcher, blunderbuss, mines, TNT, brick) takes turns with its hands too
        (' '*60+'if lw(s4_1 + a0_57 + 272) > 0 {'+chr(10), ' '*60+'if lw(lo_1 + (g::guns + 20)) <= 0 {'+chr(10)+' '*64+'fp_1 = v0_25;'+chr(10)+' '*60+'} else if lw(s4_1 + a0_57 + 272) > 0 {'+chr(10), 1),
        # the dynamite and the brick are held rolled over in the left hand
        ('            swc1(s0_1 + 88, f23_1 + -2.0_f32);'+chr(10), '            swc1(s0_1 + 88, f23_1 + -2.0_f32 + if a1 != 0 && (lw(s3_1) == 18 || lw(s3_1) == 25) { 95.0_f32 } else { 0.0_f32 });'+chr(10), 1),
    ],
    'particleBurstTick': [('v0_1 = lw(g::updateframesi);\n                    if lwc1(a3_1) < f1_1 {', 'v0_1 = running_i();\n                    if lwc1(a3_1) < f1_1 {', 1)],
    'dying': [('v0_9 = lw(g::updateframesi);', 'v0_9 = running_i();', 1), ('v0_29 = lw(g::updateframesi);', 'v0_29 = running_i();', 1)],
    'enemyAmbushWallRight': [('v0_2 = lw(g::updateframesi);\n                        if v0_1 != 0 {', 'v0_2 = running_i();\n                        if v0_1 != 0 {', 1)],
    'enemyAmbushWallLeft': [('v0_2 = lw(g::updateframesi);\n                        if v0_1 != 0 {', 'v0_2 = running_i();\n                        if v0_1 != 0 {', 1)],
    'enemyAIFutureLastStandHandler': [('a0_18 = lw(g::updateframesi);', 'a0_18 = running_i();', 1)],
    # animation phases taken from the integer tick counter
    'vtxlistWaterxyz': [('f5_1 = a7_1 as f32;', 'f5_1 = gameframe_f();', 1)],
    'spaceship1PropTick': [
        ('f2_1 = a1_1 as f32;', 'f2_1 = gameframe_f();', 1),
        ('((a1_1 - lw(g::updateframesi)) as f32 * f8_1', '((gameframe_f() - lwc1(g::updateframesf)) * f8_1', 1),
    ],
    'reflectionsPropTick': [('f4_1 = a4_2 as f32 * lwc1(0x3ace98);', 'f4_1 = gameframe_f() * lwc1(0x3ace98);', 1)],
    # the ejected shotgun shells move by their velocity on every call: scale it by the frame time
    'updateShotgunReload': [
        ('lwc1(a0_6 + 52) + lwc1(a0_6 + 64)', 'lwc1(a0_6 + 52) + lwc1(a0_6 + 64) * lwc1(g::updateframesf)', 1),
        ('lwc1(a0_6 + 56) + lwc1(a0_6 + 68)', 'lwc1(a0_6 + 56) + lwc1(a0_6 + 68) * lwc1(g::updateframesf)', 1),
        ('lwc1(a0_6 + 48) + lwc1(a0_6 + 60)', 'lwc1(a0_6 + 48) + lwc1(a0_6 + 60) * lwc1(g::updateframesf)', 1),
        ('lwc1(a1_4 + 52) + lwc1(a1_4 + 64)', 'lwc1(a1_4 + 52) + lwc1(a1_4 + 64) * lwc1(g::updateframesf)', 1),
        ('lwc1(a1_4 + 56) + lwc1(a1_4 + 68)', 'lwc1(a1_4 + 56) + lwc1(a1_4 + 68) * lwc1(g::updateframesf)', 1),
        ('lwc1(a1_4 + 48) + lwc1(a1_4 + 60)', 'lwc1(a1_4 + 48) + lwc1(a1_4 + 60) * lwc1(g::updateframesf)', 1),
    ],
    # constant increments on every call (a HUD ring angle, the spin of an explosion ring): scale by the time of the frame
    'hudTick': [('f2_1 = lwc1(0x3b0190) + lwc1(0x3ad780);', 'f2_1 = lwc1(0x3b0190) + lwc1(0x3ad780) * lwc1(g::tickframesf);', 1)],
    'blastringGfx': [('f0_1 = lwc1(s2_1) + lwc1(s1_1 + lo_1 - 4);', 'f0_1 = lwc1(s2_1) + lwc1(s1_1 + lo_1 - 4) * lwc1(g::tickframesf);', 1)],
    # The player's speed is the distance moved in the last frame divided by the whole ticks that frame lasted (player +292,
    # last non-zero value in +340).  The weapon's walk bob is driven by it.  With frames of a fraction of a tick the divisor
    # must be the frame time, as a float, or the speed (and the bob) shrinks with the frame rate.
    'playerDirtyPosReset': [('sw(s0_1 + 340, 1);', 'swc1(s0_1 + 340, 1.0_f32);', 1)],
    # the sign-on list wraps round: New Player (the top) goes up to the last sign-on and the last sign-on goes down to New Player.
    # The links are made from the count of the sign-ons, but a sign-on that does not get an item (and with none at all) leaves a link
    # on an item that is not there: the arrow vanishes and stays gone.  Make the links again from the items that were made (a1_2).
    'makesignonmenu': [
        ('    sh(g::signonplr_list, v0_1 + 2);', chr(10).join([
            '    if a1_2 != v0_1 {',
            '        let somi = g::somi;',
            '        sh(somi + 16, if a1_2 > 0 { a1_2 } else { lhu(g::mi_signonload + 8) as i32 });',
            '        sh(somi + 50, if a1_2 > 0 { 1 } else { lhu(g::mi_signonnew + 8) as i32 });',
            '        if a1_2 > 0 {',
            '            sh(somi + 64 + (a1_2 - 1) * 32 + 18, lhu(g::mi_signonnew + 8) as i32);',
            '        }',
            '    }',
            '    sh(g::signonplr_list, v0_1 + 2);']), 1),
    ],
    # a menu item with a value (the Audio / Video Options page): the slider's bar and a value that has a place of its own were drawn from the
    # screen's corner, not from the corner of the pane the page is in, so in the second and later panes of a split screen they were not seen
    'draw_menuitem': [
        ('                draw_slider(a0_1, a1_2, a2_1, a3_1, a4_1);' + chr(10) + '                return;', '                draw_slider(a0_1, a1_2, a2_1, a3_1, a4_1);' + chr(10) + '                slider_arrows(a0, a0_1, a1_2);' + chr(10) + '                return;', 1),
        ('a0_1 = lhu(a0);' + chr(10) + '                a1_2 = lhu(a0 + 2) + 14;', 'a0_1 = lw(g::window) + lhu(a0);' + chr(10) + '                a1_2 = lw(g::window + 20) + lhu(a0 + 2) + 14;', 1),
        ('v0_7 = lw(s3_1 + 28);', 'v0_7 = if lw(s3_1 + 28) != 0 { lw(s3_1 + 28) + lw(g::window) } else { 0 };', 2),
        # the value and the arrows of the weapon set on the weapon page follow the box of the selected weapons to the right
        ('            s7_1 = g::window;'+chr(10)+'            s2_1 = s5_1;', '            if s3_1 == g::gv_weaponset && weapons_page_active() {'+chr(10)+'                s5_1 = s5_1 + 24;'+chr(10)+'            }'+chr(10)+'            s7_1 = g::window;'+chr(10)+'            s2_1 = s5_1;', 1),
    ],
    # the name of a dual weapon in the message of its pickup
    'pickupPropTick': [
        ('a2_9 = lw(a3_4);', 'a2_9 = pickup_name(lw(s4_1 + 12), lw(a3_4));', 1),
        # the weapon table with the added pairs (manual/weapontable.rs): the combinations go up to combo_count() - 1, what a character owns of
        # the added ones is kept by own_get / own_set
        (' '*16+'if ((a0_24 - 1) as u32) >= (32 as u32) {', ' '*16+'if ((a0_24 - 1) as u32) >= ((combo_count() - 1) as u32) {', 1),
        (' '*20+'if ((a0_25 - 1) as u32) >= (32 as u32) {', ' '*20+'if ((a0_25 - 1) as u32) >= ((combo_count() - 1) as u32) {', 1),
        (' '*28+'s0_5 = lw(v1_7);', ' '*28+'s0_5 = own_get(s6_1, lw(s4_1 + 12));', 1),
        (' '*28+'sw(v1_7, 1);', ' '*28+'own_set(s6_1, lw(s4_1 + 12));', 1),
        (' '*36+'if (lw(a1_19 * 400 + (g::guns + 28)) & 2) != 0 {', ' '*36+'if (if a1_19 < 33 { lw(a1_19 * 400 + (g::guns + 28)) } else { 0 } & 2) != 0 {', 1),
    ],
    # the weapon table with the added pairs (manual/weapontable.rs)
    # the triangle button on the page of the number of players asks whether to exit the game
    # the ammo of the left hand sits as close to the edge as the right hand's does (its block started 36 units in, the right one's ends 26 from the edge)
    'hudGfx': [(' '*28+'a2_11 = if lw(g::window + 88) != 0 { a2_10 + 60 } else { a2_10 + 80 };', ' '*28+'a2_11 = if lw(g::window + 88) != 0 { a2_10 + 50 } else { a2_10 + 70 };', 1)],
    # test aid: TS_FILELOG lists the files the game loads
    'fileLoadTo': [('pub fn fileLoadTo(mut a0: i32, mut a1: i32, mut a2: i32) -> i32 {'+chr(10), 'pub fn fileLoadTo(mut a0: i32, mut a1: i32, mut a2: i32) -> i32 {'+chr(10)+'    file_log(a0);'+chr(10), 1)],
    'nplayers_pageTick': [('    v0_1 = joyUpdatePorts();'+chr(10), '    if av_nplayers_exit(a0) {'+chr(10)+'        return;'+chr(10)+'    }'+chr(10)+'    v0_1 = joyUpdatePorts();'+chr(10), 1)],
    'getnextcombo': [
        (' '*8+'if s2_1 >= 33 {', ' '*8+'if s2_1 >= combo_count() {', 1),
        (' '*8+'s0_3 = if s0_2 >= 33 { s4_1 } else { s0_2 };', ' '*8+'s0_3 = if s0_2 >= combo_count() { s4_1 } else { s0_2 };', 1),
    ],
    'getprevcombo': [
        (' '*8+'if s2_1 >= 33 {', ' '*8+'if s2_1 >= combo_count() {', 1),
        (' '*4+'s0_3 = if s0_2 < s4_1 { 32 } else { s0_2 };', ' '*4+'s0_3 = if s0_2 < s4_1 { combo_count() - 1 } else { s0_2 };', 1),
        (' '*8+'s0_3 = if s0_4 < s4_1 { 32 } else { s0_4 };', ' '*8+'s0_3 = if s0_4 < s4_1 { combo_count() - 1 } else { s0_4 };', 1),
    ],
    'getmostpowerfulcombo': [(' '*8+'if s0_1 >= 33 {', ' '*8+'if s0_1 >= combo_count() {', 1)],
    'gunShouldSwitchTo': [(' '*12+'if lw(a0 + (a1 << 2) + 488) == 0 {', ' '*12+'if own_get(a0, a1) == 0 {', 1)],
    'gunPickedUpAmmo': [
        (' '*28+'if lw(a5_1) != 0 {', ' '*28+'if own_get(a0, a6_1) != 0 {', 1),
        (' '*24+'if a6_1 >= 33 {', ' '*24+'if a6_1 >= combo_count() {', 1),
    ],
    'playerSetStartGun': [
        (' '*4+'if (v0_2 as u32) >= (32 as u32) {', ' '*4+'if (v0_2 as u32) >= ((combo_count() - 1) as u32) {', 1),
        (' '*4+'sw(s0_1 + (a2_1 << 2) + 488, 1);', ' '*4+'own_set(s0_1, a2_1);', 1),
    ],
    'chrClearGuns': [(' '*12+'sw(a0 + 260, 0);', ' '*12+'own_clear(a0);'+chr(10)+' '*12+'sw(a0 + 260, 0);', 1)],
    'propDrop': [
        (' '*12+'if ((a0_2 - 1) as u32) < (32 as u32) {', ' '*12+'if ((a0_2 - 1) as u32) < ((combo_count() - 1) as u32) {', 1),
        (' '*16+'if ((a0_3 - 1) as u32) < (32 as u32) {', ' '*16+'if ((a0_3 - 1) as u32) < ((combo_count() - 1) as u32) {', 1),
    ],
    'StatsGetGun': [('pub fn StatsGetGun(mut a0: i32, mut a1: i32, mut a2: i32) -> f32 {'+chr(10), 'pub fn StatsGetGun(mut a0: i32, mut a1: i32, mut a2: i32) -> f32 {'+chr(10)+'    a2 = stats_combo(a2);'+chr(10), 1)],
    'StatsSetGun': [('pub fn StatsSetGun(mut a0: i32, mut a1: i32, mut a2: i32, mut f12: f32) {'+chr(10), 'pub fn StatsSetGun(mut a0: i32, mut a1: i32, mut a2: i32, mut f12: f32) {'+chr(10)+'    a2 = stats_combo(a2);'+chr(10), 1)],
    'StatsAddGun': [('pub fn StatsAddGun(mut a0: i32, mut a1: i32, mut a2: i32, mut f12: f32) {'+chr(10), 'pub fn StatsAddGun(mut a0: i32, mut a1: i32, mut a2: i32, mut f12: f32) {'+chr(10)+'    a2 = stats_combo(a2);'+chr(10), 1)],
    # the prompt that asks whether to save the game's data after a match is answered with Yes (X)
    # (and so is the message that follows it, "Save completed", which has only an OK)
    # the title of the pause menu's pages: the name of the page
    'topmenuGfx': [('    a0_4 = s0_1;'+chr(10)+'    v0_3 = textWidth(a0_4);', '    s0_1 = av_pause_title(lw(s1_1 + (lhu(s1_1 + 4) << 2) + 12), s0_1);'+chr(10)+'    a0_4 = s0_1;'+chr(10)+'    v0_3 = textWidth(a0_4);', 1)],
    # going back from the Custom controls page leads to the Controls page
    'pageOut': [('pub fn pageOut() {'+chr(10), 'pub fn pageOut() {'+chr(10)+'    av_page_out();'+chr(10), 1)],
    'outPage': [('pub fn outPage(mut a0: i32, mut a1: i32) {'+chr(10), 'pub fn outPage(mut a0: i32, mut a1: i32) {'+chr(10)+'    av_page_out();'+chr(10), 1)],
    'statsupdate_pageTick': [
        ('v0_2 = splitMessageTick(a0_3);', 'v0_2 = if lw(a0_3 * 260 + 0x1fc1488) != 0 { save_flow(true); 2 } else { 0 };', 1),
        ('                setlastgamemodepage();'+chr(10), '                save_flow(false);'+chr(10)+'                setlastgamemodepage();'+chr(10), 1),
    ],
    'mcseq_pageTick': [
        ('                            v1_8 = lw(g::mcseq_msgflags);'+chr(10)+'                            v0_19 = v1_8 & 2;', '                            if save_flow_active() && lw(g::mcseq_msgflags) == 2 {'+chr(10)+'                                sw(g::mcseq_msgflags, 2 | 32);'+chr(10)+'                            }'+chr(10)+'                            v1_8 = lw(g::mcseq_msgflags);'+chr(10)+'                            v0_19 = v1_8 & 2;', 1),
    ],
    'playerTick': [
        ('if (lhu(v1_7 + 2) & 128) != 0 {', 'if (lhu(v1_7 + 2) & 128) != 0 && !autoaim_off() {', 1),
        ('sw(s1_3 + 292, lw(g::updateframesi));', 'swc1(s1_3 + 292, lwc1(g::updateframesf));', 1),
        ('if v0_8 > 0 {\n                sw(s1_3 + 340, v0_8);', 'if f32::from_bits(v0_8 as u32) > 0.0_f32 {\n                sw(s1_3 + 340, v0_8);', 1),
        ('lwc1(s1_9 + 340).to_bits() as i32 as f32', 'lwc1(s1_9 + 340)', 1),
        # the right stick turns the view as fast as the Controls option says
        ('f31_1 * lwc1(0x3ad054) * (lwc1(s1_8 + 260) / 6e+01_f32) * f3_6', 'f31_1 * lwc1(0x3ad054) * ts_sdk::pad::look_sensitivity() * (lwc1(s1_8 + 260) / 6e+01_f32) * f3_6', 1),
        ('f22_1 * lwc1(0x3ad05c) * (lwc1(s1_8 + 260) / 6e+01_f32) * lwc1(g::updateframesf)', 'f22_1 * lwc1(0x3ad05c) * ts_sdk::pad::look_sensitivity() * (lwc1(s1_8 + 260) / 6e+01_f32) * lwc1(g::updateframesf)', 1),
    ],
    'gunTick': [
        ('if v0_5 > 0 {', 'if f32::from_bits(v0_5 as u32) > 0.0_f32 {', 1),
        ('/ v0_5 as f32 * f3_1', '/ f32::from_bits(v0_5 as u32) * f3_1', 1),
        ('lwc1(s1_1 + 340).to_bits() as i32 as f32', 'lwc1(s1_1 + 340)', 1),
    ],
    # Front end above 60 Hz: the menus' logic (and the pad sampling that feeds it) still runs once per 60 Hz tick; the frames
    # in between redraw the page and advance the animated background (`menu_effects`), whose per-call steps are scaled by the
    # frame time (`menu_dt`, 1.0 whenever the front end runs at 60 Hz).
    'bossMainLoop': [
        ('memdbTick();\n        gameTick();\n        lvTickBefore();\n        if lw(g::lvnumplayers) > 0 {',
         'menu_memdb_tick();\n        if menu_skip() {\n            menu_effects();\n        } else {\n            gameTick();\n            lvTickBefore();\n        }\n        if !menu_skip() && lw(g::lvnumplayers) > 0 {', 1),
        ('    lvTickAfter();\n    a3_2 = 0x2eeff0;', '    if !menu_skip() {\n        lvTickAfter();\n        menu_memdb_mark();\n    }\n    a3_2 = 0x2eeff0;', 1),
    ],
    'joyTick': [('pub fn joyTick() {\n', 'pub fn joyTick() {\n    if menu_skip() {\n        return;\n    }\n', 1)],
    'frontfxTick': [
        ('pub fn frontfxTick(mut a0: i32) {\n', 'pub fn frontfxTick(mut a0: i32) {\n    menu_note_fx(a0);\n', 1),
        ('f0_2 = f0_1 + f5_1;', 'f0_2 = f0_1 + f5_1 * menu_dt();', 1),
        ('f20_1 = lwc1(g::fronttime) + f5_1;', 'f20_1 = lwc1(g::fronttime) + f5_1 * menu_dt();', 1),
        # the width of the logo during its entrance is added up call by call
        ('swc1(g::globalscalex, f2_2 + f1_2 * f4_1 * f5_1);', 'swc1(g::globalscalex, f2_2 + f1_2 * f4_1 * f5_1 * menu_dt());', 1),
    ],
    'frontfxBgTick': [
        ('lwc1(g::frontbgtranslatey1) + lwc1(0x3ac938)', 'lwc1(g::frontbgtranslatey1) + lwc1(0x3ac938) * menu_dt()', 1),
        ('lwc1(g::frontbgtime) + lwc1(0x3ac93c)', 'lwc1(g::frontbgtime) + lwc1(0x3ac93c) * menu_dt()', 1),
        ('lwc1(g::frontbgtranslatey2) + lwc1(0x3ac940)', 'lwc1(g::frontbgtranslatey2) + lwc1(0x3ac940) * menu_dt()', 1),
        ('v0_2 = dpsub(a0_1, a1_1);', 'v0_2 = dpsub(a0_1, dpmul(a1_1, fptodp(menu_dt())));', 1),
    ],
    'frontfxGoodiesTick': [
        ('lwc1(g::frontfxgoodiestime) + lwc1(0x3ac968)', 'lwc1(g::frontfxgoodiestime) + lwc1(0x3ac968) * menu_dt()', 1),
        ('a1_1 = lw(g::frontfxlpos) + 3;', 'a1_1 = lw(g::frontfxlpos) + menu_steps(3);', 1),
    ],
    # drawChar (character select) moves and turns the model by one step each time it is called: only once per tick
    'drawChar': [
        ('swc1(a0 + 48, lwc1(a0 + 48) + lwc1(sp + 64));', 'if !menu_skip() {\n                swc1(a0 + 48, lwc1(a0 + 48) + lwc1(sp + 64));\n            }', 1),
        ('rotateChar(a0_4, f12_3);', 'if !menu_skip() {\n                    rotateChar(a0_4, f12_3);\n                }', 1),
        ('rotateChar(a0_5, f12_4);', 'if !menu_skip() {\n            rotateChar(a0_5, f12_4);\n        }', 1),
        ('swc1(a0 + 56, lwc1(a0 + 56) + lwc1(sp + 72));', 'if !menu_skip() {\n            swc1(a0 + 56, lwc1(a0 + 56) + lwc1(sp + 72));\n        }', 1),
        ('swc1(v1_2 + 48, lwc1(v1_2 + 48) + lwc1(sp + 64));', 'if !menu_skip() {\n                swc1(v1_2 + 48, lwc1(v1_2 + 48) + lwc1(sp + 64));\n            }', 1),
        ('swc1(v1_2 + 56, f1_1 + f2_2);', 'if !menu_skip() {\n                swc1(v1_2 + 56, f1_1 + f2_2);\n            }', 1),
        # the slide of the characters is added to their positions as they are drawn
        ('    swc1(a0 + 48, lwc1(a0 + 48) + lwc1(a2_2));\n    swc1(a1_15 + 48, lwc1(a1_15 + 48) + lwc1(a2_2));\n    if v0_9 != 0 {\n        swc1(v0_9 + 48, lwc1(v0_9 + 48) + lwc1(a2_2));\n    }\n',
         '    if !menu_skip() {\n        swc1(a0 + 48, lwc1(a0 + 48) + lwc1(a2_2));\n        swc1(a1_15 + 48, lwc1(a1_15 + 48) + lwc1(a2_2));\n        if v0_9 != 0 {\n            swc1(v0_9 + 48, lwc1(v0_9 + 48) + lwc1(a2_2));\n        }\n    }\n', 1),
    ],
    # the Audio / Video Options page uses the port's own table of items (graphics settings added, screen position removed)
    'audiovideo_pageTick': [
        ('s0_1 = g::audiovideo_menu;', 's0_1 = av_menu(a0);', 1),
        # the same tick serves the pause menu's page
        ('    sw(((lw(0x3aff2c) - 1) << 3) + 0x1fc1308, 0);\n', '    if !av_ingame() {\n        sw(((lw(0x3aff2c) - 1) << 3) + 0x1fc1308, 0);\n    }\n', 1),
        ('    pageSet(a0_1, a1_1);\n', '    if !av_ingame() {\n        pageSet(a0_1, a1_1);\n    }\n', 1),
        ('    frontfxTick(a0_2);\n', '    if !av_ingame() {\n        frontfxTick(a0_2);\n    }\n', 1),
        ('    menutick_230238(a0_3, a1_2);\n', '    if av_ingame() {\n        ingame_menutick(a0_3, a1_2);\n    } else {\n        menutick_230238(a0_3, a1_2);\n    }\n', 1),
        ('    a0_1 = lw(lw(v1_1) + 480);\n', '    a0_1 = av_title(a0, lw(lw(v1_1) + 480));\n', 1),
    ],
    'confirmendgame_pageTick': [
        ('pub fn confirmendgame_pageTick(mut a0: i32) {\n', 'pub fn confirmendgame_pageTick(mut a0: i32) {\n    if av_confirm_pressed(a0) {\n        return;\n    }\n', 1),
        ('s0_2 = g::confirmendgame_menu;', 's0_2 = av_confirm_table(a0);', 1),
    ],
    'mainsetup_pageTick': [('s0_1 = g::mainsetup_menu;', 's0_1 = av_mainsetup_menu();', 1)],
    'frontPageSet': [('pub fn frontPageSet(mut a0: i32, mut a1: i32) {\n', 'pub fn frontPageSet(mut a0: i32, mut a1: i32) {\n    a1 = av_front_page(a1);\n', 1)],
    'inPage': [('pub fn inPage(mut a0: i32, mut a1: i32) {\n', 'pub fn inPage(mut a0: i32, mut a1: i32) {\n    if av_in_page(a0, a1) {\n        return;\n    }\n', 1)],
    'gametype_pageTick': [
        ('s0_1 = g::gamemode_menu;', 's0_1 = av_gamemode_menu();', 1),
        # STORY is open to three and four players
        ('    if a3_1 < 3 {'+chr(10), '    if a3_1 < 5 {'+chr(10), 1),
        # STORY, ARCADE and CHALLENGE sit higher (the page has room above them)
        ('sh(a0_4 + 38, 70);','sh(a0_4 + 38, 70 - 14);',1),('sh(a0_4 + 6, 50);','sh(a0_4 + 6, 50 - 14);',1),
        ('sh(v0_6 + 6, 40);','sh(v0_6 + 6, 40 - 14);',1),('sh(v0_6 + 70, 80);','sh(v0_6 + 70, 80 - 14);',1),('sh(v0_6 + 38, 60);','sh(v0_6 + 38, 60 - 14);',1),
        ('                if v1_3 >= 7 {\n', '                if v1_3 >= 7 && !(9..=12).contains(&v1_3) {\n', 1),
        # choosing Exit Game asks in the question box (`av_gametype_exit`) instead of opening a page
        (' '*8+'a0_13 = a0;'+chr(10)+' '*8+'s0_1 = av_gamemode_menu();'+chr(10), ' '*8+'if av_gametype_exit(a0) {'+chr(10)+' '*12+'return;'+chr(10)+' '*8+'}'+chr(10)+' '*8+'a0_13 = a0;'+chr(10)+' '*8+'s0_1 = av_gamemode_menu();'+chr(10), 1),
    ],
    # the developer menu: the triangle button goes back one menu, like start does
    'menuTick': [('        if (v0_8 & 2048) != 0 {\n            menuEnd();', '        if (v0_8 & 2064) != 0 {\n            menuEnd();', 1)],
    # the developer menu's Exit Level hands the reset a block of stack that the console happened to leave clear
    'exitlevel': [('    a0_1 = sp;\n', '    memset(sp, 0, 112);\n    a0_1 = sp;\n', 1)],
    # noclip (developer menu): the player's move is not tested against the level, it goes where the stick points
    'moveTestDoSlide': [('pub fn moveTestDoSlide(mut a0: i32, mut a1: i32, mut a2: i32, mut a3: i32, mut a4: i32, mut f12: f32, mut f13: f32) -> i32 {\n', 'pub fn moveTestDoSlide(mut a0: i32, mut a1: i32, mut a2: i32, mut a3: i32, mut a4: i32, mut f12: f32, mut f13: f32) -> i32 {\n    if noclip_slide(a1, a2) {\n        return 0;\n    }\n', 1)],
    'obinstCalcAmbientLight': [('                \'l17: loop {\n                    #[cfg(feature = "calltrace")]\n                    ts_rt::calltrace::poll();\n                    v0_13 = lw(g::bgNumRooms);\n', '                let mut rooms_tried = 0;\n                \'l17: loop {\n                    #[cfg(feature = "calltrace")]\n                    ts_rt::calltrace::poll();\n                    // an object with an invalid position (NaN) would pass from room to room for ever (the console\'s FPU has no NaN)\n                    rooms_tried += 1;\n                    if rooms_tried > 64 {\n                        break;\n                    }\n                    v0_13 = lw(g::bgNumRooms);\n', 1)],
    # one more arcade level, Ron (the console game has an arcade set-up for it, but it is not in its list): the list is in `av_arcade_levels`, MapMaker keeps its place (index 18), it goes after it
    'arcade_pageTick': [('s1_3 = if s1_2 >= 19 { 0 } else { s1_2 };', 's1_3 = if s1_2 >= av_arcade_count() { 0 } else { s1_2 };', 1), ('lw((s1_3 << 2) + g::arcadelevels)', 'lw((s1_3 << 2) + av_arcade_levels())', 1), ('lw((s0_3 << 2) + g::arcadelevels)', 'lw((s0_3 << 2) + av_arcade_levels())', 1), ('s0_3 = if -1 >= s0_2 { 18 } else { s0_2 };', 's0_3 = if -1 >= s0_2 { av_arcade_count() - 1 } else { s0_2 };', 1), ('lw((a1_5 << 2) + g::arcadelevels)', 'lw((a1_5 << 2) + av_arcade_levels())', 1)],
    # the lists of rooms by depth (what the player can see) held 200 rooms of one byte each: 512 rooms of two bytes now, in the port's memory (a level
    # of a mod can have more than 255 rooms); the debug string of the rooms drawn, a stack buffer a room longer than the frame, is not written
    'propGfxRoomPreproc': [('a6_1 = 212;', 'a6_1 = ROOM_LAST;', 1), ('v0_1 = 0x1fe98e8;', 'v0_1 = ROOM_PROP_COUNTS + 4 * ROOM_LAST;', 1), ('(a3_1 as u32) < (214 as u32)', '(a3_1 as u32) < (ROOM_LIMIT as u32)', 1)],
    # (the instances of models: a room has one too)
    'obReset': [('{ 20 } else { 300 }', '{ 20 } else { 1000 }', 1)],
    'propReset': [('a0_9 = 712;', 'a0_9 = PROP_LAST;', 1), ('a0_10 = 712;', 'a0_10 = PROP_LAST;', 1),
                  # (a level's props, the rooms' among them: 300 of them were room for 200 rooms and the rest)
                  ('{ 20 } else { 300 }', '{ 20 } else { 1000 }', 1)],
    # the game's memory pool cannot be longer (the stacks and static data are after it): big allocations of a level of a mod are made in a pool of the port's own
    'memAlloc': [('    if a1 == 1 {'+chr(10)+'        v1_2 = lw(g::mem_level_free);', '    if a1 == 1 && big_wants(a2_1) {'+chr(10)+'        a3_1 = big_alloc(a2_1);'+chr(10)+'    } else if a1 == 1 {'+chr(10)+'        v1_2 = lw(g::mem_level_free);', 1)],
    'memReset': [('pub fn memReset() {'+chr(10), 'pub fn memReset() {'+chr(10)+'    big_reset();'+chr(10), 1)],
    'bgGfx': [
        ('sb(v1_3 + a2_1 * 200 + 0x1fc4a78, s0_1);', 'sh((v1_3 << 1) + a2_1 * 1024 + 0x2500000, s0_1);', 1),
        ('s0_2 = lbu(s2_2 * 200 + s1_1 + 0x1fc4a78);', 's0_2 = lhu(s2_2 * 1024 + (s1_1 << 1) + 0x2500000);', 1),
        ('sb(a1_2, v0_3);', '', 1),
    ],
    'bgGetRoomsTick': [
        ('sb(v1_3 + a2_1 * 200 + 0x1fc4a78, a5_1);', 'sh((v1_3 << 1) + a2_1 * 1024 + 0x2500000, a5_1);', 1),
        ('a5_2 = lbu(a4_2 * 200 + a3_2 + 0x1fc4a78);', 'a5_2 = lhu(a4_2 * 1024 + (a3_2 << 1) + 0x2500000);', 1),
    ],
    'levelAvail': [('pub fn levelAvail(mut a0: i32) -> i32 {\n', 'pub fn levelAvail(mut a0: i32) -> i32 {\n    if a0 == 25 || (31..=99).contains(&a0) {\n        return 1;\n    }\n', 1)],
    'gmKnockOutTick': [('                if v1_2 == 0 {\n                    trap("break");\n                }\n', '                if v1_2 == 0 {\n                    // no bag spawn points in the level (Ron has none): nothing to put a bag on\n                    return;\n                }\n', 1)],
    # Square on the character select page picks a random character; on the weapon set page it fills the custom set
    'chrplayer_pageTick': [('pub fn chrplayer_pageTick(mut a0: i32) {'+chr(10), 'pub fn chrplayer_pageTick(mut a0: i32) {'+chr(10)+'    char_randomise(a0);'+chr(10), 1)],
    'weapons_pageDraw': [
        # the names of the weapons of the weapon page (the new dual entries have names of their own)
        ('lw((lh((s0_2 << 3) + (GUNCOMBOS + 6)) << 2) + lw((v1_1 << 2) + g::languages))', 'combo_name(s0_2)', 1),
        ('lw((lh((a1_13 << 3) + (GUNCOMBOS + 6)) << 2) + lw((v1_4 << 2) + a0_17))', 'combo_name(a1_13)', 1),
        # the box of the available weapons is wider (the names of the dual weapons are long)
        ('a2_8 = 263;', 'a2_8 = 287;', 1), ('a2_9 = 263;', 'a2_9 = 287;', 1), ('a2_10 = 263;', 'a2_10 = 287;', 1), ('a2_12 = 263;', 'a2_12 = 287;', 1),
        ('a2_14 = 264;', 'a2_14 = 288;', 1), ('a0_19 = 200 - v0_5;', 'a0_19 = 224 - v0_5;', 1),
        # the box of the selected weapons, its title, numbers and names are moved to the right (24 units)
        ('    a0_4 = 312;', '    a0_4 = 336;', 1), ('    a2_2 = 575;', '    a2_2 = 599;', 1),
        ('>> 1) + 312;', '>> 1) + 336;', 1), ('        a0_8 = 312;', '        a0_8 = 336;', 1), ('        a2_4 = 575;', '        a2_4 = 599;', 1),
        ('        a0_10 = 302 - v0_4;', '        a0_10 = 326 - v0_4;', 1), ('            a0_11 = 314;', '            a0_11 = 338;', 1),
        ('        a0_12 = 312;', '        a0_12 = 336;', 1), ('        a2_7 = 575;', '        a2_7 = 599;', 1),
    ],
    'frontmpgGetGunComboNum': [('    return v0_3;'+chr(10), '    return dual_combo(a0, v0_3);'+chr(10), 1)],
    'weapons_pageTick': [
        ('pub fn weapons_pageTick(mut a0: i32) {'+chr(10), 'pub fn weapons_pageTick(mut a0: i32) {'+chr(10)+'    weapons_randomise(a0);'+chr(10), 1),
        ('v0_9 = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1656);', 'v0_9 = weapons_legend();', 1),
        ("    'b5: {"+chr(10)+"        'b6: {"+chr(10)+"            'b7: {"+chr(10)+"                a0_4 = lw(g::front);"+chr(10), "    weapons_list_extend();"+chr(10)+"    'b5: {"+chr(10)+"        'b6: {"+chr(10)+"            'b7: {"+chr(10)+"                a0_4 = lw(g::front);"+chr(10), 1),
    ],
    'chrsets_pageTick': [
        ('pub fn chrsets_pageTick(mut a0: i32) {'+chr(10), 'pub fn chrsets_pageTick(mut a0: i32) {'+chr(10)+'    bots_randomise(a0);'+chr(10), 1),
        ('        a2_13 = lw(a1_31 + 1680);', '        a2_13 = bots_legend(lw(a1_31 + 1680), s4_1);', 1),
    ],
    'chrsetInc': [('pub fn chrsetInc() {'+chr(10), 'pub fn chrsetInc() {'+chr(10)+'    chrset_step(1);'+chr(10)+'    return;'+chr(10), 1)],
    'chrsetDec': [('pub fn chrsetDec() {'+chr(10), 'pub fn chrsetDec() {'+chr(10)+'    chrset_step(-1);'+chr(10)+'    return;'+chr(10), 1)],
    # a bot without a weapon in a level that has none to pick up goes after its target (bare handed) instead of standing still
    'findweapon': [(' '*32+'a0_6 = a0;'+chr(10)+' '*32+'enemySetIdleAnimation(a0_6);'+chr(10)+' '*32+'return;'+chr(10), ' '*32+'if ts_sdk::bots_punch() {'+chr(10)+' '*36+'sw(s0_1 + 2720, lw(s0_1 + 2716));'+chr(10)+' '*36+'sw(s0_1 + 2716, 128);'+chr(10)+' '*36+'sw(s0_1 + 2892, 0);'+chr(10)+' '*36+'return;'+chr(10)+' '*32+'}'+chr(10)+' '*32+'a0_6 = a0;'+chr(10)+' '*32+'enemySetIdleAnimation(a0_6);'+chr(10)+' '*32+'return;'+chr(10), 1)],
    'enemyTick': [('pub fn enemyTick(mut a0: i32) {'+chr(10), 'pub fn enemyTick(mut a0: i32) {'+chr(10)+'    bot_punch(a0);'+chr(10), 1)],
    # the buttons in the texts are named for the device in use (manual/nativetext.rs)
    'textPrint': [('pub fn textPrint(mut a0: i32, mut a1: i32, mut a2: i32, mut a3: i32) {'+chr(10), 'pub fn textPrint(mut a0: i32, mut a1: i32, mut a2: i32, mut a3: i32) {'+chr(10)+'    a2 = button_text(a2);'+chr(10), 1)],
    'textOutline': [('pub fn textOutline(mut a0: i32, mut a1: i32, mut a2: i32, mut a3: i32, mut a4: i32) {'+chr(10), 'pub fn textOutline(mut a0: i32, mut a1: i32, mut a2: i32, mut a3: i32, mut a4: i32) {'+chr(10)+'    a2 = button_text(a2);'+chr(10), 1)],
    'textSize': [('pub fn textSize(mut a0: i32, mut a1: i32, mut a2: i32) {'+chr(10), 'pub fn textSize(mut a0: i32, mut a1: i32, mut a2: i32) {'+chr(10)+'    a2 = button_text(a2);'+chr(10), 1)],
    # the MapMaker's Options page (manual/mmoptions.rs): a slider changed with X, a row that takes the limit of the map away
    'settings_pageTick': [
        ('pub fn settings_pageTick(mut a0: i32) {'+chr(10), 'pub fn settings_pageTick(mut a0: i32) {'+chr(10)+'    mm_options_tick(a0);'+chr(10), 1),
        ('s0_1 = g::settings_menu;', 's0_1 = mm_options_menu();', 1),
    ],
    # the slider of the MapMaker's menus shows arrows while it is being changed
    'draw_mi': [(' '*12+'draw_slider(a0_1, a1_1, a2_1, a3_1, a4_2);'+chr(10)+' '*12+'return;', ' '*12+'draw_slider(a0_1, a1_1, a2_1, a3_1, a4_2);'+chr(10)+' '*12+'slider_arrows(a0, a0_1, a1_1);'+chr(10)+' '*12+'return;', 1)],
    'mmUpdateMapCost': [('< 0x1389', '< mm_limit()', 1)],
    'mmPlaceObject': [('< 0x1389', '< mm_limit()', 1)],
    'padact_pageTick': [('< 0x1389', '< mm_limit()', 1)],
    'mmDrawMemFree': [('pub fn mmDrawMemFree(mut a0: i32, mut a1: i32) {'+chr(10), 'pub fn mmDrawMemFree(mut a0: i32, mut a1: i32) {'+chr(10)+'    if mm_unlimited() {'+chr(10)+'        return;'+chr(10)+'    }'+chr(10), 1)],
    'weaponSetInc': [('pub fn weaponSetInc() {'+chr(10), 'pub fn weaponSetInc() {'+chr(10)+'    weapon_set_step(1);'+chr(10)+'    return;'+chr(10), 1)],
    'weaponSetDec': [('pub fn weaponSetDec() {'+chr(10), 'pub fn weaponSetDec() {'+chr(10)+'    weapon_set_step(-1);'+chr(10)+'    return;'+chr(10), 1)],
    # test aid (TS_FORCEPLAYERS=<n>): the match has n players, whatever the front end set up
    'gameReset': [('    v1_7 = lw(g::curgameparms);'+chr(10), '    dual_reset();'+chr(10)+'    v1_7 = test_players(lw(g::curgameparms));'+chr(10), 1)],
    # a particle stream emits in proportion to the elapsed time
    'particleStreamTick': [('lwc1(g::updateframesi).to_bits() as i32 as f32', 'lwc1(g::updateframesf)', 1)],
    # the music of a level: the player's choice on the arcade options page instead of the level's own
    'musicRestart': [('    v0_1 = lvGetTuneNum();'+chr(10), '    v0_1 = av_level_music(lvGetTuneNum());'+chr(10), 1)],
    # the name of a place where the maps and sign ons are kept had the number of the card's slot after it ("SAVE SLOT 1"): the name alone now
    'mcseqSlotMakeName': [('    sb(v0_2 + 1 + s4_1, s0_1 + 49);'+chr(10), '    sb(v0_2 + s4_1, 0);'+chr(10)+'    sb(v0_2 + 1 + s4_1, 0);'+chr(10), 1)],
    # the renderer is told which file each texture came from (a mod can have a picture of any size for it)
    'emLoadTexture': [('    sw(s1_31 + 4, 0);'+chr(10)+'    return;'+chr(10), '    sw(s1_31 + 4, 0);'+chr(10)+'    tex_loaded(lw(sp + 48), a1, a2);'+chr(10)+'    return;'+chr(10), 1)],
    # the arcade set-up (the modes) of a level of a mod
    'lvGetARCADESetup': [('    v1_1 = a0 - 1;'+chr(10)+'    a1_1 = 0;'+chr(10), '    if let Some(p) = custom_setup(a0) {'+chr(10)+'        return p;'+chr(10)+'    }'+chr(10)+'    v1_1 = a0 - 1;'+chr(10)+'    a1_1 = 0;'+chr(10), 1)],
    # the arcade options page has a row more (Level Music): its table is a copy in the port's memory, which has room for it; the hint line says Edit for it as for Weapon Set and Bot Set
    'arcadeoptions_pageTick': [('g::i0410', '(av_opt_menu() + 16)', 7), ('    s0_1 = g::arcadeoptions_menu;'+chr(10), '    s0_1 = av_opt_ready();'+chr(10), 1), ('            if v1_5 < 14 {'+chr(10), '            if v1_5 < 14 || v1_5 == 15 {'+chr(10), 1)],
}

used = set()


def apply(name, code):
    for old, new in GLOBAL:
        if old in code:
            used.add(('*', old))
            code = code.replace(old, new)
    if name not in TICK_GATED:
        code = RUNNING_GATE.sub(lambda m: f'lwc1(g::updateframesf) {m.group(1)} 0.0_f32', code)
    for old, new, count in FUNCS.get(name, []):
        n = code.count(old)
        if n != count:
            raise SystemExit(f'patch for {name} expects {count} occurrence(s) of {old!r}, found {n}')
        used.add((name, old))
        code = code.replace(old, new)
    return code


def check():
    for name, lst in FUNCS.items():
        for old, new, count in lst:
            if (name, old) not in used:
                raise SystemExit(f'patch for {name} was never applied: {old!r}')
