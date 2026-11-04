Character Heights: the camera of a player is at the height of its character.
The game gives every character the same height of its head standing (1.8) and crouching (1.0), so a small character such as Robofish had the camera of a tall one.  The whole mod is a script,
scripts/heights.rhai (docs/scripting.md), that sets both from how tall each character is: the top of its model (measured from the game's models, listed in the script) and its scale.  A character added
by a mod has its scale only.  Switch it on from the Mods page (or enabled = true) and start the game again.
