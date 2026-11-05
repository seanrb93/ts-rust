Smooth Weapons: the guns with about twice the polygons, added only where they are curved: barrels, drums, grips and the like go round, and flat sides and sharp edges stay as they are.
Run `python tools/make-smooth-weapons.py <game>/PAK/OB.PAK <game>/mods` to make the mod (about 7 MB: it holds the game's own models, so they are not kept in the repository).
With First Person Hands on, the guns in your own hands are that mod's (its models have the arms on them), so only the guns seen in the third person are smooth.  To have both, make that mod from these guns: HANDS_BASE=<game>/mods/smooth_weapons_mod for make-hands.py (over 1 GB in place of about 400 MB).
Switch it on or off from the Mods page (or enabled = true) and start the game again.
