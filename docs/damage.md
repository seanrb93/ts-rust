# Damage in the game

Everything that takes health from a character or a prop, read from the decompiled code (`propDamage` in `game/prop-damage-and-special-effect-creation.rs` and what calls it).  A character's health is a number from 1 (full) down to 0, kept at +520 of its prop; armour is a second number at +12 of its record, and takes damage before health does.  Numbers marked (reading) were read from the code and not measured in the game.

## What a hit is

Every source of damage ends in `propDamage(target, attacker, point, kind, ..., damage)`.  `kind` is the kind of bullet (2 to 7 are shots, 8 and 9 rockets, 10 and up grenades and mines, 1 is a blow with the hand or a monster's bite, -1 is a fall or a shot from off the screen).  The damage number is a fraction of a full health bar.

The order of what happens to a hit on a character:

1. For the shots and blows (kinds 1 to 7) the part that was hit gives a multiplier (below).  Explosions and falls are not scaled by the part.
2. With the Handicaps option, the damage is multiplied by the handicap of the character that is hit (not for headshots or explosions).
3. In the story levels and challenges the difficulty changes it (below).
4. Armour takes what it can first.
5. With the One Shot Kill option, a hit by a character takes all the health that is left.
6. Health goes down by what is left, and a character at 0 dies.  A hit with no health left changes nothing.
7. A team mate's hit does nothing unless Friendly Fire is on (and then it counts only when teams are on).

## The part that was hit (shots and blows)

By the model's table of parts, a hit on the head is worth 5 times the damage.  The two or three parts of the body (hips and spine) are 1 times; every other part (arms, legs, hands, feet) 0.4 times (reading: the bones are grouped in `propDamage`, 0 to 2 full, 11 the head, all others 0.4).  A headshot also sets a head flying in the single player and some modes.

## The weapons

`damage` is the field of the weapon's table (+72); each bullet carries it.  The kinds of bullet and how many are made per shot are code: the shotgun kind (4) makes 6 bullets per shot, kind 9 (the rocket launcher's second fire) makes 3, all others 1.  Every pellet is a bullet of its own and does the full damage of the weapon.  The pellets are spread by `spread` (+84): each goes off the aim by `(random + random - 1) * spread` on each side, so the pattern is a square that gets wider with distance (a spread of 0.03 is +-0.3 units at 10 units).

| # | Weapon | Damage | Bullets per shot | Clip | Reserve | Spread |
|---|---|---|---|---|---|---|
| 0 | Pistol | 0.30 | 1 | 10 | 50 | 0.01 |
| 1 | M16 | 0.20 | 1 | 30 | 90 | 0.01 |
| 2 | Assault Rifle | 0.25 | 1 | 30 | 90 | 0.01 |
| 3 | Sniper Rifle | 0.45 | 1 | 5 | 30 | 0.001 |
| 4 | Raygun | 0.40 | 1 | 20 | 60 | 0.01 |
| 5 | Raygun Carbine | 0.15 | 1 | 20 | 60 | 0.01 |
| 6 | Scifi Autorifle | 0.30 | 1 | 50 | 150 | 0.01 |
| 7 | Scifi Handgun | 0.20 | 1 | 20 | 60 | 0.01 |
| 8 | Rocket Launcher | 0.50 (the blast is separate) | 1 (3 on the second fire) | none | 15 | 0.01 |
| 9 | Mauser Pistol | 0.30 | 1 | 15 | 45 | 0.01 |
| 10 | Minigun | 0.20 | 1 | 60 | 300 | 0.01 |
| 11 | Grenade Launcher | 1 (the blast is separate) | 1 | 8 | 24 | 0.05 |
| 12 | Scifi Sniper Rifle | 0.75 | 1 | 10 | 30 | 0.001 |
| 13 | (no name) | 1 | 1 | 8 | 24 | 0.05 |
| 14 | Proximity Mine | 1 | 1 | none | 10 | 0.01 |
| 15 | Remote Mine | 1 | 1 | none | 10 | 0.05 |
| 16 | Timed Mine | 1 | 1 | none | 10 | 0.05 |
| 17 | (no name) | 1 | 1 | none | 10 | 0.05 |
| 18 | TNT | 5 | 1 | none | 10 | 0.05 |
| 19 | Tommy Gun | 0.20 | 1 | 32 | 128 | 0.01 |
| 20 | Assault Shotgun | 0.30 | 6 | 8 | 40 | 0.03 |
| 21 | Uzi | 0.20 | 1 | 20 | 160 | 0.01 |
| 22 | Blunderbuss | 0.20 | 6 | none | 30 | 0.05 |
| 23 | Colt Pistol | 0.50 | 1 | 6 | 36 | 0.002 |
| 24 | Shotgun | 0.50 | 6 | 2 | 24 | 0.03 |
| 25 | Brick | 0.40 | 1 | none | 20 | 0.05 |
| 26 | (no name) | 0.50 | 1 | 20 | 20 | 0.01 |
| 27 | (no name) | 0.50 | 1 | 30 | 90 | 0.03 |
| 28, 29 | (no name) | 0.20 | 1 | 10 | 40 | 0.01 |

(`python tools/dump-guns.py <game folder>` lists the same fields.  Weapons 13 and 26 to 29 have no name text; the bullets of the mines, TNT and the grenade are not hits but things that explode.)

A full shot of the Shotgun (6 pellets of 0.5) is 3 full health bars if all pellets hit the body, and one pellet to the head does 2.5, so one lucky pellet kills; the Assault Shotgun's six torso hits are 1.8 and a head pellet 1.5; the Blunderbuss's are 1.2 and 1.0.

## Explosions

`explosionDamage` hurts every prop and character with a clear line (a line test with the point pulled in by 0.95) inside the radius.  The size of the explosion by its kind sets the radius: 3 units for the kinds 8 and 9 (rockets), 4 for the next three, 5 for the rest.  The strength starts at 2.3 and falls over four stages of 30 frames: 2.3 in the first, 0.345 in the second, 0.115 in the third and none in the fourth, each time times a random number between 0.8 and 1.0.  What a prop takes is the strength times `(1 - distance / radius)`: at the middle of a rocket's blast that is 2.3, and anything within about 1.7 units (more than half the radius) of the centre gets a full bar or more.  Props that are not characters take half of it (2.3 times for the autoguns 420 and 423).  Mines near the blast are set off in its first frames.  The grenade, mine and TNT damage fields (1, 1 and 5) are not what the blast does: they are the damage of the thing hitting, and the blast is the numbers above (reading).

## Blows and falls

- A blow of the player's hand: 0.8 (0.5 when the target is not facing within 60 degrees in the player's reading of the angle, and 0.4 for the other kind of blow in the same function); counts as kind 1, so the part multipliers apply (reading).
- The left fist the port adds (`manual/dualwield.rs`): 0.5.
- A monster's bite or blow (`enemy-attack.rs`): 0.3.
- A fall or a thing that hits a character at speed (`prop-ticks-after-guns.rs`): the speed plus 0.1, not scaled by the part.
- A bot shooting at the player from where the player cannot see it (`character-line-test-and-sets-page.rs`; not in mode 7): for each bullet of the shot, the weapon's damage times a factor for the bot's skill (0.25, 0.75, 1.05, 1.75 and 2.25 from the lowest skill to the highest), times one minus the bot's aim number.

## Game options and modes

- **Handicaps** (the option): damage taken is multiplied by the factor of the character's handicap, from the table at 0x35df10: 0.162, 0.402, 0.833, 1.0 (none), 1.2, 2.488, 6.192 and 0 (cannot be hurt).  Headshots and explosions are not scaled by it.
- **One Shot Kill**: a hit by a character takes everything the target has left.
- **Teamplay** and **Friendly Fire**: with teams on, a hit by a team mate does no damage unless friendly fire is on.
- **Challenges** (the options with the flag 32): the damage a character takes and does is multiplied by two numbers per character in the challenge's own table (`challengeGetSetup`).
- **Escort** mode (4): the escorted character takes a tenth of the damage (0.1).
- **Story levels** (mode 7), by difficulty (0 easy, 1 normal, 2 hard): what the player does to an enemy is multiplied by 5, 3 and 1.75; what the player takes is multiplied by 0.4, 0.6 and 0.9 and by a number from the health (at least 0.04, 0.06 and 0.08), and a single hit can take no more than 0.1, 0.2 and 0.4 of a bar (reading).
- The **Invincible** cheat sets the damage the player takes to 0.

## What a mod can change

`weapons.txt`: `damage` (the fraction of a bar for each bullet; the pellets, the multipliers and the explosions are code and are not in it).  The handicap and challenge tables are data and can be changed by `patches.txt` (`<symbol>+offset <f> value`).
