Character Heights: the camera of a player is at the height of its character, as a part of the character's scale.
The game gives every character the same height of its head standing (1.8) and crouching (1.0), so a small character such as Robofish had the camera of a tall one.  The whole mod is a script,
scripts/heights.rhai (docs/scripting.md), that sets both from each character's own scale (it also lifts the camera of a big character of a mod).  Switch it on from the Mods page (or enabled = true) and start the game again.
