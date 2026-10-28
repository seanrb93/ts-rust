Zap Blaster: a weapon made from nothing (a ray gun), as an example of a new weapon with its own model in the first person.
examples/zap_blaster_mod_src/make-zap-blaster.py draws the model; tools/obedit.py (meshes, on ob/gun1/gun1.raw with --tex-base 9100) put it in the pistol's file, which is saved as ob/gun90/gun90.raw.
It is a copy of the pistol with its own name, clip and damage (weapons.txt).  The gun a character holds (model_file) is the pistol's.
Copy this folder into mods/ (a build with extended-tables).
