First person hands: the first person guns are held by the arms of the player's own character.  The arms are the character's own model in the pose of its animation for a gun held in
both hands (a hand on the grip, a hand under the front for the two handed guns), with its own skin and sleeve, put on each gun.  The pose is taken from each character's own skeleton (so the arms have that character's shape), and each arm is turned so that it comes up into the view from the bottom of the screen.  There is a set of guns for each of the characters of the
game that have a model (66 of them, the aliens, robots and mutants included; the others have none); the game loads the set of the character of the first player (the file with _cNN before .raw), and the first character's arms for the others.
Limits: split screen shows the first player's arms in every view, the hands do not move for a reload, and a character added by a mod has the first character's arms.
Run `python examples/first_person_hands_src/make-hands.py <game>/PAK/OB.PAK <game>/mods/first_person_hands_mod` to make the mod (about 100 MB: it holds the game's own models, so it is not kept
in the repository).  WEAPONS in that file lists the guns and where the hands go; HANDS_CHARACTER=<number> makes it for one character only.

Dual wielding: when a gun is held in each hand, the guns have only the hand on the grip (each gun has a second model, gunN_cNN_l.raw, that the game uses then), so a two handed gun does not show four arms.

The mod makes many model files (a set for each of the 66 characters, with a left hand for the gun of the left hand), more than the 4096 that the game's own list of a pack holds: the build with extensions takes bigger lists.

Which of the files the game loads is the script of the mod, scripts/hands.rhai (docs/scripting.md): it gives the model of each gun with the arms of the first player's character (`gunN_cNN.raw`), and the
model with the hand on the grip only for a gun held in both hands.  The program has no knowledge of these names.
