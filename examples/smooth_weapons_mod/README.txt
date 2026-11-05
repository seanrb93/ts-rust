Smooth Weapons: the guns with more polygons, added only where they are curved: barrels, drums, grips and the rims of tubes go round, and flat sides and sharp corners stay as they are.
Run `python tools/make-smooth-weapons.py <game>/PAK/OB.PAK <game>/mods` to make the mod (about 8 MB: it holds the game's own models, so they are not kept in the repository).
With First Person Hands on, these guns are the ones its arms are put on, so the guns in your hands are smooth as well.  Finer curves (a smaller TOLERANCE in the script) run out of the game's memory in a match, so check a real match after changing it: a test level does not show it.
Switch it on or off from the Mods page (or enabled = true) and start the game again.
