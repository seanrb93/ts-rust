Zap Blaster: a weapon made from nothing (a ray gun): its own model in the first person, in a character's hand and as the ammunition that is picked up.
examples/zap_blaster_mod_src/make-zap-blaster.py draws the models; tools/obedit.py (meshes, with --tex-base 9100) put them into the files of the pistol (ob/gun1/gun1.raw), of the gun a character holds (ob/chrgun1/chrgun1.raw) and of the ammunition box (ob/ammo/ammosmall.raw), which are saved as ob/gun90, ob/chrgun90 and ob/ammo90.
It is a copy of the pistol with its own name, clip and damage (weapons.txt).  It is on the Setup Weapons page (with its pair, when Dual Wield Everything is on).
Copy this folder into mods/ (a build with extended-tables).
