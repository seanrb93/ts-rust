First person hands: the first person guns are held by the arms of the player's own character, as they would be held in real life.  The right hand is on the grip with its fingers bent round it, the left hand is palm up under the front of a two handed gun (or on its foregrip or handle) with the arm coming in from the lower left, and mines, TNT and the brick lie on the upturned palm.  The arms are the character's own model, with its own skin and sleeve.  Where each hand goes on each gun is WEAPONS in make-hands.py; the hand bends at the wrist and the fingers are bent to the size of what they hold, measured from the gun.  There is a set of guns for each of the characters of the
game that have a model (66 of them, the aliens, robots and mutants included; the others have none); the game loads the set of the character of the first player (the file with _cNN before .raw), and the first character's arms for the others.
Limits: split screen shows the first player's arms in every view, the hands do not move for a reload, and a character added by a mod has the first character's arms.
Run `python examples/first_person_hands_src/make-hands.py <game>/PAK/OB.PAK <game>/mods/first_person_hands_mod` to make the mod (about 100 MB: it holds the game's own models, so it is not kept
in the repository).  WEAPONS in that file lists the guns and where the hands go; HANDS_CHARACTER=<number> makes it for one character only.

Dual wielding: when a gun is held in each hand, the guns have only the hand on the grip (each gun has a second model, gunN_cNN_l.raw, that the game uses then), so a two handed gun does not show four arms.

The mod makes many model files (a set for each of the 66 characters, with a left hand for the gun of the left hand), more than the 4096 that the game's own list of a pack holds: the build with extensions takes bigger lists.

Which of the files the game loads is the script of the mod, scripts/hands.rhai (docs/scripting.md): it gives the model of each gun with the arms of the first player's character (`gunN_cNN.raw`), and the
model with the hand on the grip only for a gun held in both hands.  The program has no knowledge of these names.
