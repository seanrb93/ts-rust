Virus, a mode of the later TimeSplitters games, as a way to play Deathmatch.

A free for all: one character, chosen at random, starts infected.  An infected character that touches a survivor infects him, and he plays as infected from then on (he is lit green, and the screen of an infected player has a green glow at its edges).  The last
survivor wins.  Computer characters never shoot in this mode: the survivors run from the infected and the infected go after the survivors on foot.

Switch it on from the Mods page (or enabled = true in mod.ini), start the game again, and pick "Virus" from the list of modes on the arcade Select Level page (after the game's modes).  The options of
a mode that mean nothing for Virus (the score limit and method, team play and team starts, friendly fire, one shot kill) are left off its Options page.  It needs the build with extensions (the build the
mods are made for; the default build is the original game).
It is made of `rule virus 1` in modes.txt (the number is how many characters start infected) and `hide` for the options of the page to leave out (docs/modding-guide.md).

The table of results at the end lists every character with its status (the survivor first, then the infected, the last infected first).  An infected character groans now and then, and a character that is infected groans.
