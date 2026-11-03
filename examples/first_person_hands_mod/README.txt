First person hands: eleven weapons are seen with the hands of a character on them: Pistol, Scifi Handgun, Mauser, Colt, Uzi (one hand on the grip) and M16, Assault Rifle, Tommy Gun,
Shotgun, Assault Shotgun, Minigun (a hand on the grip and a hand under the front, as the guns are held).  The hands are the low polygon hands of the game's characters with the
character's own skin and sleeve (Det. Harry Tipper by default; HANDS_CHARACTER=<number> for another), put on the body of each gun, so they move with the gun.
Limits: the hands are not animated for the reload, a pair of the same two handed gun (dual wield) shows the front hand on both guns, and the skin is one character's.
Run `python examples/first_person_hands_src/make-hands.py <game>/PAK/OB.PAK <game>/mods/first_person_hands_mod` to make the mod (it holds the game's own models and picture, so it is not kept
in the repository).  WEAPONS in that file lists the guns and where the hands go; plot_hands.py shows a gun with its hands without starting the game, shot.py takes pictures in the game.
