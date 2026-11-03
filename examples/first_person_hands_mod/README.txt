First person hands: eleven weapons are seen with a hand and forearm on the grip (and, for the two handed ones, a hand under the front): Pistol, Scifi Handgun, Mauser, Colt, Uzi,
M16, Assault Rifle, Tommy Gun, Shotgun, Assault Shotgun and Minigun.  The hands are the low polygon hands of the game's characters in one skin colour, put on the body of each gun
(the shotgun's front hand rides on the sliding forend), so they move with the gun.  They are not animated for the reload and are not the colour of the character.
Run `python examples/first_person_hands_src/make-hands.py <game>/PAK/OB.PAK <game>/mods/first_person_hands_mod` to make the mod (it holds the game's own models, so it is not kept in the
repository); WEAPONS in that file lists the guns, and `shot.py` takes the pictures to check them by.
