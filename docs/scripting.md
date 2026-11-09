# Scripts

A mod can have scripts: files of the mod in the [Rhai](https://rhai.rs) language.  There are two kinds.  The script of a **mode** (a `variant` of `modes.txt` with a `script` line) runs while that mode is played.  The script of a **mod** (`script = bots_punch.rhai` in its `mod.ini`) runs all the time the mod is on, in every match, and in the menus (`on_frame`).  A mod can have several.  The program knows nothing of the mode;
the rules, the computer characters' behaviour and the look of it are the script's.  `example_mods/virus_mode/` is made this way (`scripts/virus.rhai`).  Scripts need the build with extensions.

```
mode deathmatch
  variant "Virus"
  script virus.rhai
  team_play off
  hide team_play team_starts friendly_fire kill_limit score_method score_limit one_shot_kill
```

The script of a mode is read from `scripts/<file>` of the mod (or next to `modes.txt`) and starts when the match starts; a script of a mod is read when the program starts.  A script with an error is reported in the log (`[script]`) and stops; the match goes on without it.
`TS_SCRIPTLOG=1` shows what the script says with `log(...)`.

## The functions of the script

All are optional.  The state of the script is kept in `this` (a map): `this.clock = 0.0;` in `on_start`, `this.clock += dt();` in `on_tick`.  A function that uses `this` has to be called as a method of it
(`this.infect(b, a)`); the functions the program calls are given `this` already.

| function | when |
| --- | --- |
| `on_frame()` | every frame the game is run, in the menus too (scripts of mods) |
| `on_start()` | the match has its characters (and again when it is restarted) |
| `on_tick()` | every frame of the match, after the game's own |
| `on_bot_idle(i)` | the game looked for a weapon for the computer character `i` and found none to pick up: answer `true` if the script gave it something else to do |
| `on_bot_target(i)` | the game has chosen whom the computer character `i` goes after (in the modes where it chooses among all the others, such as Deathmatch): answer the number of another character, or -1 to leave the game's choice |
| `on_bot_aim(i, t, x, y, z)` | the computer character `i` fires at the character `t`, aimed at the place `x, y, z`: answer `[x, y, z]` for another place (to aim ahead of a moving target), or `()` to leave it |
| `on_bot_unseen_hit(i, t)` | the computer character `i` fires at `t` where no player sees, and the game makes such a shot hit at once with no bullet: answer `false` to make it miss |
| `on_bot_weapon(i)` | the computer character `i` has no weapon and is choosing one to fetch: answer the `id` of a weapon of `pickups()`, or -1 for the game's choice |
| `file_prefixes()` and `on_file(name)` | the game loads a file whose name starts with one of the prefixes: `on_file` gives the names to try instead (a list; the plain file is used when none is in the packs) |
| `on_gun_model(name, left)` | a gun is made for a hand when a gun is held in each: the name of the file of another model for it (or "") |
| `has_model(name)` | the game loads a file whose name starts with one of `file_prefixes`, or looks for the model `on_gun_model` named: `true` if `make_model` can make it, and then it is made in place of any file of that name. `name` is the name in the pack of objects, such as `ob/gun3/gun3_c00.raw`; the names `on_file` gives are asked about in turn |
| `make_model(name)` | the game loads a model that `has_model` said yes to: call `model_add` for what to add, and answer with the name of the model in the packs to add it to (or "" for none). The last eight models made are kept, so this is not run again each time a gun is switched |
| `model_names()` | a list of every name `make_model` can make, for the test aid `TS_MODELDUMP` |
| `lists_pair(single)` | the weapon page may list the dual (x2) version of the weapon `single` (a number of the game's table of weapon combinations): answer `true` for it |
| `allows_pair(pair, single)` | a set of weapons may have the pair `pair` of the weapon `single` (when not, it has the single weapon): answer `true` |
| `fixes()` | every frame: a list of the names of the fixes of the original's bugs to switch on (`docs/original-bugs.md`; the others are off) |
| `alternate_fists()` | an unarmed player punches with the left fist and the right in turn: answer `true` |
| `on_hud(player)` | when the screen of a player is drawn (`hud_glow` and `pad_info` work here) |
| `on_input()` | every frame, in the menus too (`in_match()` tells them apart), when the pads have been read and before the game sees them: `consume` takes buttons from the game |
| `on_gfx(player)` | when the view of a player is drawn, where the original's editors drew (`draw_cylinder` works here) |
| `results_per_character()` | the results: `true` for a row for every character instead of every team |
| `result_key(i)` | the number the results rank the character `i` by (the highest first) |
| `result_text(i)` | what the results say in place of the score of the character `i` |

## What the script can ask

Characters are numbered from 0, the players first, then the computer characters.

`chars()`, `players()`, `is_player(i)`, `alive(i)`, `team(i)`, `char_name(i)` (the character's name as the results show it; a player's is the name of its profile), `x(i)`, `y(i)`, `z(i)`, `score(i)`, `bot_state(i)` (the game's own state of a computer character: 8 is going to a pad),
`player_character(p)` (the number of the character of the player `p`, -1 before the player chose), `bot_unarmed(i)` (a computer character with no weapon in either hand), `bot_target(i)` (the character it goes after, -1 for none), `peek(address)` (a word of the game's memory), `peekf(address)` (a float), `chr_view_ratio(c, ratio)` (says that the script scaled the heights of the view of character `c` by `ratio`, so that the arm of the fists stays where it is in the view), `chr_entry(c)` (the address of the entry of the character `c` in the game's table of characters: +20 the height of its head standing, +24 crouching, +28 the floor, +40 its scale),
`setting(name, default)` (a value of the `[settings]` of the mod's `mod.ini`: a number of the default's kind, true or false for yes and no, or text; the default when the file does not have it), `setting(name)` (the same, `()` when it is not there), and for the script of a mode the options of its variant (`option` in `modes.txt`) as the Options page left them, in place of the settings of the same names,
`dt()` (frames of 1/60 second since the last call), `rand_int(n)` (0 to n-1, by the game's random numbers), `env_int(name, default)` (a number from the environment: test aids),
`pad_count()`, `pad_x(k)`, `pad_y(k)`, `pad_z(k)` (the pads of the level: the places computer characters go between).
The buttons of a player's controller: `buttons(p)` (held) and `pressed(p)` (newly pressed), in the bits of the PlayStation 2 pad (select 0x100, start 0x800, up 0x1000, right 0x2000, down 0x4000, left 0x8000, L2 1, R2 2, L1 4, R1 8, triangle 0x10, circle 0x20, cross 0x40, square 0x80).
The pads and links as the original's pad editor sees them: `pad_used(k)` (not deleted), `pad_extref(k)` (the pad's own number, which links and the files use), `pad_flags(k)`, `link_count()`, `link_used(i)`, `link_from(i)` and `link_to(i)` (the numbers of its pads), `pad_file()` (the name of the file the pads were loaded from), `pad_file_bytes()` (the pads and links as that file has them, without the deleted ones), `level()`.
The props: `aim_point(p)` (where the view of a player meets the level, `[x, y, z]`: 3 ahead when it meets nothing within 50), `prop_nearest(x, y, z, most)` (the external reference of the prop nearest a place, -1 for none), `prop_exists(e)`, `prop_kind(e)`, `prop_x(e)`, `prop_y(e)`, `prop_z(e)`, `prop_angle(e)` (by the prop's external reference), `prop_kind_editable(kind)` (a kind the original's prop editor can make: the characters of 0 to 158 and things such as crates from 159), `prop_kind_name(kind)`.
`health(i)` and `armour(i)` (each a part of 1; a blow takes the armour first), `bot_skill(i)` (the skill the match gives a computer character, 0 the easiest to 4), `game_mode()` (1 BagTag, 2 Capture the Bag, 3 Deathmatch, 4 Escort, 5 KnockOut, 6 Last Stand, 7 the story) and `mode_script()` (the script of a mode of a mod is playing it: a script of a mod can leave the computer characters to it).
What a character sees: `can_see(i, j)` (nothing is in the way from the eyes of `i` to the body of `j`), `line_clear(i, x, y, z)` (nothing is in the way from its eyes to the place) and `eye_y(i)` (the height of its eyes).
Weapons: a weapon (a "combination" of the game) is one gun or a gun in each hand, and a gun has two shots.  `weapon(i)` (the weapon a character holds), `owns(i, w)`, `weapon_gun(w)` (the gun in its right hand, -1 for the fists), `weapon_rank(w)` (the game's order of weapons, the lowest the best),
`shot_kind(gun, second)` (the kind of bullet: 8 and 9 rockets, 10 to 13 grenades, 14 to 16 mines, 17 and 18 thrown things), `shot_speed(gun, second)` (units a frame), `shot_damage(gun, second)`, `shot_spread(gun, second)`, `gun_wait(gun)` (the frames a computer character waits between shots),
`ammo(i, gun)` and `ammo_max(gun)` (the ammunition a character carries for a gun, not counting what is in it).
`shots()`: the shots in flight, a list of maps with `kind`, `x`, `y`, `z`, `vx`, `vy`, `vz` (units a frame), `age` (seconds) and `owner` (the character that fired it, or -1).
`pickups()`: the weapon, health and armour pickups of the level, a list of maps with `id`, `kind` ("weapon", "health", "armour" or "ammo"), `weapon` (for a weapon), `pad` (the pad a computer character goes to for it), `x`, `y`, `z` and `here` (false while it is taken).
`mod_file(path)` (a file of the script's mod as a blob, empty when there is none), `mod_text(path)` (the same as a string), `mod_has_file(path)` (paths are from the mod's folder; `..` is refused. Keep such files under `scripts`, or the program takes the folder for files of a pack), `f32_at(blob, at)` (a little-endian 32-bit float of a blob: Rhai's own `parse_le_float` reads 4 bytes as the low half of a 64-bit float, which gives numbers near zero).

## What it can do

* `set_team(i, t)`, `set_score(i, v)`: the side of a character (the game uses it to know whom a computer character goes after; the teams of the Options page are separate), and its score.
* `message(i, "text")`: a line on the screen of a player.  `sound(id, i)`: a sound of the game at the character.  `pain_voice(i)`: one of the character's own lines of being hit.
* `damage(target, attacker, amount)`: a blow (of the game's damage: a fraction of a character's health, a pistol shot is 0.3).  `poke(address, value)` and `pokef(address, number)`: set a word or a float of the game's memory (a mod's script can reach what the other functions do not).
* `finish()`: the match is over.
* `keep_out(i, true)`: the character `i` does not come back into the match once it is dead (a player's button does not bring it back; a computer character stays out of sight and out of the tests of shots), for modes with lives; `keep_out(i, false)` lets it back in, and `kept_out(i)` asks.  `example_mods/elimination_mode` is made with it.
* Computer characters: `bot_passive(i, true)` (never attacks, looks for no weapon: it only goes to the pads it is sent to), `bot_pad(i, k)` (go to that pad; -1 gives the choice back to the game),
  `bot_set_state(i, state)` (a state of the game's own machine: 128 chooses a target), `bot_speed(i, 0.8)` (a part of its own speed), `bot_steer(i, x, y, z)` and `bot_steer_clear(i)` (run straight at a place when going to a pad),
  `bot_run_to(i, x, y, z)` and `bot_run_clear(i)` (run straight at a place now, whatever it was doing, and not attack meanwhile; clearing it has it choose what to do again), `bot_switch(i, w)` (hold a weapon it owns),
  `bot_hold_fire(i, true)`, `bot_fire_mode(i, m)` (which shot of its gun it fires: 0 the first, 1 the second, -1 the game's choice, which is the first where a player sees), `bot_spread(i, part)` (a part of the spread of its own aim: 0.5 aims twice as well),
  `bot_reaction(i, part)` (a part of the time it takes to react), `bot_uses_ammo(i, true)` (it uses up the ammunition of its gun as a player does, and does not fire with none), `set_ammo(i, gun, n)`.
* Looks: `flame(i, true)` (a green flame), `tint(i, r, g, b, add)` and `tint_clear(i)` (the light of the character's model: its parts of red, green and blue, and a part of the strongest colour added to green),
  `radar_colour(i, team)` and `radar_colour_clear(i)` (the colour of its dot on the radar: a team's colour; the game gives every character one colour when it is not a team game),
  `set_colour(i, team)` (its colour in the results and the HUD: 1 red, 2 blue, 3 yellow, 4 green), `hud_glow(r, g, b, a)` (in `on_hud`: a glow at the edge of the screen, each 0 to 127).
* `hide_score(true)`: the score and the rank are left off the screen.
* `consume(p, mask)`: the game does not see those buttons until they are let go (in `on_input`, before the game reads them).
* Editing the level, with the original's pad and prop editor, which the game has but never calls: `pad_add(p)` (a pad where the player stands, facing where it looks; its number, -1 when the list is full), `pad_delete(k)` (and its links), `pad_link(a, b)` and `pad_unlink(a, b)` (true when done), `pad_toggle_flag(k, mask)`, `pad_info(k)` (in `on_hud`: the original's text about a pad),
  `prop_place(p, kind)` (a prop of an editable kind where the player looks, turned the way it looks; its external reference, -1 when it could not be made), `prop_change(p, e, step)` (the prop made again as the next or last editable kind; the kind), `prop_turn(e, degrees)` (made again turned, as its walls are worked out when it is made), `prop_delete(e)`, `prop_new(kind, x, y, z, angle)` (for putting back saved props).
  New pads and links are used by the computer characters only once the level is loaded again: a script saves them with `mod_write(pad_file(), pad_file_bytes())`, and the file in the mod's folder takes the place of the game's from the next start.
* `mod_write(path, blob)` and `mod_write_text(path, text)`: write a file in the mod's folder (true when written; `..` is refused).  A file at the path of a file of the packs is used in its place from the next start; keep other files in a folder whose name starts with `_` (or in `scripts`).
* The original's sound test (`soundtoolStart`, which nothing in the game calls): `sound_test_open()` (on the front end: a page that steps through the 1215 sounds, plays them and changes their envelope, volume and note while the game runs; true when it opened), `sound_test_close()` (back to the pages it was opened from), `sound_test_open_now()`, `sound_test_showing()` (its own page is on top, not the list of sounds it opens).  Its SAVE item, which wrote the table to the developers' machine, does nothing in the build with extensions.  `in_match()` tells the menus from a match.
* Drawing: `draw_cylinder(x, y, z, height, radius)` (in `on_gfx`: the original's eight-sided debug cylinder, each side its own colour; not drawn where it would reach the eye, as it is drawn without clipping), `blast_ring(x, y, z)` (the original's unused blast ring: an upright see-through ring that grows for a second; one at a time, true when it started), `ray(x1, y1, z1, x2, y2, z2, seconds)` (the original's unused ray: a line of 40 turned squares of green light between two places, for so long; the original's `particleNewRay` left the number of squares at 0 and kept the end where the drawing reads a direction).
* The camera of a player's view: `camera_set(p, ex, ey, ez, tx, ty, tz)` (the view of the player `p` is from the place `ex, ey, ez`, looking at `tx, ty, tz`, until `camera_clear(p)` or the match starts again; the level is drawn from the room under the eye, and the player's own body and the guns in its hands are drawn in that view as in the game's third person camera, the gun of the first person view is not), `camera_on(p)`, `room_at(x, y, z)` (the room of the level under a place, as the camera finds it; 0 outside the level).
  The sticks: `stick(p, axis)` (0 the left stick across, 1 the left up, 2 the right across, 3 the right up; -1 to 1, after the dead zone) and `consume_sticks(p)` (in `on_input`: the game sees them centred this frame).  Where a player looks: `look_yaw(p)` and `look_pitch(p)` (degrees; the turn grows to the left, 0 looks along z; the mouse turns them) and `set_look(p, yaw, pitch)`.
  `example_mods/spectator_mod` is made with these: a camera that follows any character or flies about the level.
* `model_add(part, values)` (in `make_model`): triangles to add to a part of the model. `values` is a flat list of 28 numbers for each triangle: the number of its texture, a point its faces turn away from (x, y, z), then three corners of x, y, z, s, t, r, g, b. The program works out the normals, the chunks and the strips. A texture the model does not have is added to its table of textures.
* `log("text")`.

A script has 5 million operations a call, then it is stopped.

In a network game with rollback what the scripts keep (`this`, their variables, and what they asked for the characters) is kept for each tick and put back with the game's memory when a tick is played again or a picture is made between ticks, so a script plays the same on every machine.  A resync sends the game's memory only: a script that keeps something of its own may differ after one.  `example_mods/smarter_bots_mod` uses most of the calls for the computer characters.

`TS_MODELDUMP=<folder>` writes every model of `model_names()` to that folder once the scripts run, and prints how many milliseconds each took. `example_mods/first_person_hands_mod` is made this way: a model takes it about 8 ms, and it needs no models of its own.
