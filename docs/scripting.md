# Scripts

A mod can have scripts: files of the mod in the [Rhai](https://rhai.rs) language.  There are two kinds.  The script of a **mode** (a `variant` of `modes.txt` with a `script` line) runs while that mode is played.  The script of a **mod** (`script = bots_punch.rhai` in its `mod.ini`) runs all the time the mod is on, in every match, and in the menus (`on_frame`).  A mod can have several.  The program knows nothing of the mode;
the rules, the computer characters' behaviour and the look of it are the script's.  `examples/virus_mode/` is made this way (`scripts/virus.rhai`).  Scripts need the build with extensions.

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
| `file_prefixes()` and `on_file(name)` | the game loads a file whose name starts with one of the prefixes: `on_file` gives the names to try instead (a list; the plain file is used when none is in the packs) |
| `on_gun_model(name, left)` | a gun is made for a hand when a gun is held in each: the name of the file of another model for it (or "") |
| `has_model(name)` | the game loads a file whose name starts with one of `file_prefixes`, or looks for the model `on_gun_model` named: `true` if `make_model` can make it, and then it is made in place of any file of that name. `name` is the name in the pack of objects, such as `ob/gun3/gun3_c00.raw`; the names `on_file` gives are asked about in turn |
| `make_model(name)` | the game loads a model that `has_model` said yes to: call `model_add` for what to add, and answer with the name of the model in the packs to add it to (or "" for none). The last eight models made are kept, so this is not run again each time a gun is switched |
| `model_names()` | a list of every name `make_model` can make, for the test aid `TS_MODELDUMP` |
| `lists_pair(single)` | the weapon page may list the dual (x2) version of the weapon `single` (a number of the game's table of weapon combinations): answer `true` for it |
| `allows_pair(pair, single)` | a set of weapons may have the pair `pair` of the weapon `single` (when not, it has the single weapon): answer `true` |
| `alternate_fists()` | an unarmed player punches with the left fist and the right in turn: answer `true` |
| `on_hud(player)` | when the screen of a player is drawn (`hud_glow` works here) |
| `results_per_character()` | the results: `true` for a row for every character instead of every team |
| `result_key(i)` | the number the results rank the character `i` by (the highest first) |
| `result_text(i)` | what the results say in place of the score of the character `i` |

## What the script can ask

Characters are numbered from 0, the players first, then the computer characters.

`chars()`, `players()`, `is_player(i)`, `alive(i)`, `team(i)`, `x(i)`, `y(i)`, `z(i)`, `score(i)`, `bot_state(i)` (the game's own state of a computer character: 8 is going to a pad),
`player_character(p)` (the number of the character of the player `p`, -1 before the player chose), `bot_unarmed(i)` (a computer character with no weapon in either hand), `bot_target(i)` (the character it goes after, -1 for none), `peek(address)` (a word of the game's memory), `peekf(address)` (a float), `chr_view_ratio(c, ratio)` (says that the script scaled the heights of the view of character `c` by `ratio`, so that the arm of the fists stays where it is in the view), `chr_entry(c)` (the address of the entry of the character `c` in the game's table of characters: +20 the height of its head standing, +24 crouching, +28 the floor, +40 its scale),
`setting(name, default)` (a value of the mod's `settings.ini`: a number of the default's kind, true or false for yes and no, or text; the default when the file does not have it), `setting(name)` (the same, `()` when it is not there),
`dt()` (frames of 1/60 second since the last call), `rand_int(n)` (0 to n-1, by the game's random numbers), `env_int(name, default)` (a number from the environment: test aids),
`pad_count()`, `pad_x(k)`, `pad_y(k)`, `pad_z(k)` (the pads of the level: the places computer characters go between).
`mod_file(path)` (a file of the script's mod as a blob, empty when there is none), `mod_text(path)` (the same as a string), `mod_has_file(path)` (paths are from the mod's folder; `..` is refused. Keep such files under `scripts`, or the program takes the folder for files of a pack), `f32_at(blob, at)` (a little-endian 32-bit float of a blob: Rhai's own `parse_le_float` reads 4 bytes as the low half of a 64-bit float, which gives numbers near zero).

## What it can do

* `set_team(i, t)`, `set_score(i, v)`: the side of a character (the game uses it to know whom a computer character goes after; the teams of the Options page are separate), and its score.
* `message(i, "text")`: a line on the screen of a player.  `sound(id, i)`: a sound of the game at the character.  `pain_voice(i)`: one of the character's own lines of being hit.
* `damage(target, attacker, amount)`: a blow (of the game's damage: a fraction of a character's health, a pistol shot is 0.3).  `poke(address, value)` and `pokef(address, number)`: set a word or a float of the game's memory (a mod's script can reach what the other functions do not).
* `finish()`: the match is over.
* Computer characters: `bot_passive(i, true)` (never attacks, looks for no weapon: it only goes to the pads it is sent to), `bot_pad(i, k)` (go to that pad; -1 gives the choice back to the game),
  `bot_set_state(i, state)` (a state of the game's own machine: 128 chooses a target), `bot_speed(i, 0.8)` (a part of its own speed), `bot_steer(i, x, y, z)` and `bot_steer_clear(i)` (run straight at a place when going to a pad).
* Looks: `flame(i, true)` (a green flame), `tint(i, r, g, b, add)` and `tint_clear(i)` (the light of the character's model: its parts of red, green and blue, and a part of the strongest colour added to green),
  `radar_colour(i, team)` and `radar_colour_clear(i)` (the colour of its dot on the radar: a team's colour; the game gives every character one colour when it is not a team game),
  `set_colour(i, team)` (its colour in the results and the HUD: 1 red, 2 blue, 3 yellow, 4 green), `hud_glow(r, g, b, a)` (in `on_hud`: a glow at the edge of the screen, each 0 to 127).
* `hide_score(true)`: the score and the rank are left off the screen.
* `model_add(part, values)` (in `make_model`): triangles to add to a part of the model. `values` is a flat list of 28 numbers for each triangle: the number of its texture, a point its faces turn away from (x, y, z), then three corners of x, y, z, s, t, r, g, b. The program works out the normals, the chunks and the strips. A texture the model does not have is added to its table of textures.
* `log("text")`.

A script has 5 million operations a call, then it is stopped.

`TS_MODELDUMP=<folder>` writes every model of `model_names()` to that folder once the scripts run, and prints how many milliseconds each took. `examples/first_person_hands_mod` is made this way: a model takes it about 8 ms, and it needs no models of its own.
