# How the game modes work

Read from the hand-decompiled code in `ts-rust/crates/ts-game/src/game`; it is the basis for the mode part of `docs/mod-roadmap.md`.  Everything here was read from the code, none of
it has been changed by a test, so the names are the ones the code suggests.

## Where a mode runs

* `curgameparms` (a record at a fixed address) holds the game's parameters.  `+72` is the mode: 1 BagTag, 2 Capture the Bag, 3 Deathmatch, 4 Escort, 5 KnockOut, 6 Last Stand, 7 the
  story.  `game-status-and-level-tick.rs` ends the level's tick with a `match` on it that calls one of `gmBagTagTick`, `gmCtfTick`, `gmDeathmatchTick`, `gmEscortTick`,
  `gmKnockOutTick`, `gmLastStandTick`, `gmStoryTick`.  The time limit is checked earlier in the same function (`+88` counts down by `updateframesi`; at 0 the level is completed).
* The options a player chooses for a mode (team play, friendly fire, radar, time limit, kill limit, score limit, scoring method, one shot kills) are kept in one record of 32 bytes
  for each mode at `0x1fc2c98` (`gameoptionsMake` fills them, `storeGameModeOptions` and `getGameModeOptions` move them to and from the menus' variables `gv_*`); `modes.txt` writes into them.
* The arcade page lists the modes from a table of six `(mode, level mask)` pairs (`gmtab`) against the bits of the level's set-up; the name of a mode is a text number in `gamemodedata`
  (12 bytes a mode: the text, then two flags the menus read for team starts and score).

## The state the ticks use

| Field | What it is |
|---|---|
| `curgameparms + 72` | the mode |
| `+80` | flags: `0x8000` the game is over (the last second is being counted), `0x1000000` and `0x2000000` the warnings of "three" and "two points short" were given, `0x8000000` the first kill was announced, `0x10000000` the level was completed in the story or the escortee arrived, `0x20000000` a team scored (Escort) |
| `+88` | the time left (the time limit counted down by the frames of the tick) |
| `+92` | the score a team has to reach in Deathmatch (the score limit) |
| `+96` | the goal of Capture the Bag and KnockOut (0: no goal) |
| a character (`chrinfo + 0x1210 * n`): `+16` name, `+24` team, `+248` the score the mode keeps (the ticks BagTag holds, 1 when the escortee has arrived, a count in the story), `+484` non zero for the one who holds the bag |
| `teamGetScore(team)` | the score of a team 1 to 5 |
| `chrCountKills(chr)` | the kills of a character |
| `0x3afee8` | the holder of the bag that BagTag has seen last |
| `lvnumplayers`, `numactivechars` | the numbers of players and of other characters (the loop of a tick goes over both) |

Each tick ends with `UnlockCombo()` (a combination of weapons that is unlocked completes the level), and a level is completed with `gameLevelCompleted()`.

## The ticks

* **BagTag.**  Finds the holder of the bag (the first character with `+484` set), tells the players when it changes (a message with the holder's name, a different one for the holder), adds the
  frames of the tick to the holder's `+248` and shows it on the holder's screen.  No score limit of its own; the time limit ends it.
* **Capture the Bag.**  Completes the level when a team's score equals the goal at `+96` (if it is not 0).
* **Deathmatch.**  Announces the first kill; when `0x8000` is set (the last second) finds the best team score and completes the level; completes it when a team has reached `+92`; warns when a team is three or
  two points short.
* **Escort.**  Completes the level when any team has a score above 0, or when the escortee has reached its destination (`gmEscorteeReachedDest`).
* **KnockOut.**  Completes the level when a team's score equals `+96`; keeps two bags in play: when fewer than two are, a countdown ends with a new bag on a random pad that no bag uses, or a free
  bag is moved there.
* **Last Stand.**  The bases that the players defend (`enemyGetNumBases`, `gmLastStandAllBasesDestroyed`) and the waves are driven here: messages, the count of the waves by `gameframe`, and the end
  when all bases are destroyed.
* **Story.**  Completes the level (and sets `0x10000000`) when a character's `+248` is positive.

## What this says about new rules

* A win condition is a few lines over `teamGetScore`, a character's `+248` or `chrCountKills`, and a call to `gameLevelCompleted()`.  A mode of parts could be made of: a score source (kills,
  ticks held, objects), a limit (score, time, last alive), and the messages.
* Scoring is spread: the kills are counted by the characters, the bag by the pickups (`+484`), the teams by `teamGetScore`.  A new kind of score would add to one of these or to a field the port keeps
  (the way the port keeps what a character owns of the added combinations, `weapontable.rs`).
* Nothing in the ticks creates the objects of a mode except KnockOut's bags; bases, bags and the escortee come from the level's set-up and from code that runs when a level starts
  (`gmEscortSetup`, `knockoutSetup`), so a mode that needs other objects needs that code read too.
* What is not read yet: the end-of-game pages and the statistics of each mode (they are keyed by the mode number), `UnlockCombo`, and the set-up functions.
