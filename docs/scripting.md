# Scripts of modes

A mode that a mod adds (a `variant` of `modes.txt`) can have a script: a file of the mod in the [Rhai](https://rhai.rs) language that runs while the mode is played.  The program knows nothing of the mode;
the rules, the computer characters' behaviour and the look of it are the script's.  `examples/virus_mode/` is made this way (`scripts/virus.rhai`).  Scripts need the build with extensions.

```
mode deathmatch
  variant "Virus"
  script virus.rhai
  team_play off
  hide team_play team_starts friendly_fire kill_limit score_method score_limit one_shot_kill
```

The script is read from `scripts/<file>` of the mod (or next to `modes.txt`) and starts when the match starts.  A script with an error is reported in the log (`[script]`) and stops; the match goes on without it.
`TS_SCRIPTLOG=1` shows what the script says with `log(...)`.

## The functions of the script

All are optional.  The state of the script is kept in `this` (a map): `this.clock = 0.0;` in `on_start`, `this.clock += dt();` in `on_tick`.  A function that uses `this` has to be called as a method of it
(`this.infect(b, a)`); the functions the program calls are given `this` already.

| function | when |
| --- | --- |
| `on_start()` | the match has its characters (and again when it is restarted) |
| `on_tick()` | every frame of the match, after the game's own |
| `on_hud(player)` | when the screen of a player is drawn (`hud_glow` works here) |
| `results_per_character()` | the results: `true` for a row for every character instead of every team |
| `result_key(i)` | the number the results rank the character `i` by (the highest first) |
| `result_text(i)` | what the results say in place of the score of the character `i` |

## What the script can ask

Characters are numbered from 0, the players first, then the computer characters.

`chars()`, `players()`, `is_player(i)`, `alive(i)`, `team(i)`, `x(i)`, `y(i)`, `z(i)`, `score(i)`, `bot_state(i)` (the game's own state of a computer character: 8 is going to a pad),
`dt()` (frames of 1/60 second since the last call), `rand_int(n)` (0 to n-1, by the game's random numbers), `env_int(name, default)` (a number from the environment: test aids),
`pad_count()`, `pad_x(k)`, `pad_y(k)`, `pad_z(k)` (the pads of the level: the places computer characters go between).

## What it can do

* `set_team(i, t)`, `set_score(i, v)`: the side of a character (the game uses it to know whom a computer character goes after; the teams of the Options page are separate), and its score.
* `message(i, "text")`: a line on the screen of a player.  `sound(id, i)`: a sound of the game at the character.  `pain_voice(i)`: one of the character's own lines of being hit.
* `finish()`: the match is over.
* Computer characters: `bot_passive(i, true)` (never attacks, looks for no weapon: it only goes to the pads it is sent to), `bot_pad(i, k)` (go to that pad; -1 gives the choice back to the game),
  `bot_speed(i, 0.8)` (a part of its own speed), `bot_steer(i, x, y, z)` and `bot_steer_clear(i)` (run straight at a place when going to a pad).
* Looks: `flame(i, true)` (a green flame), `tint(i, r, g, b, add)` and `tint_clear(i)` (the light of the character's model: its parts of red, green and blue, and a part of the strongest colour added to green),
  `set_colour(i, team)` (its colour on the radar and in the results: 1 red, 2 blue, 3 yellow, 4 green), `hud_glow(r, g, b, a)` (in `on_hud`: a glow at the edge of the screen, each 0 to 127).
* `hide_score(true)`: the score and the rank are left off the screen.
* `log("text")`.

A script has 5 million operations a call, then it is stopped.
