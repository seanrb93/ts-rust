Virus, a mode of the later TimeSplitters games, as a way to play Deathmatch.

A free for all: one character, chosen at random, starts infected.  An infected character that touches a survivor for a moment infects him, and he plays as infected from then on.  The last survivor wins.
Computer characters never shoot in this mode: the survivors run from the infected and spread out, and the infected go after the survivors on foot, slower than they run.

An infected character is on fire with a green flame, is lit green, and groans now and then; the edge of the screen of an infected player glows green.  The first 30 seconds of a match nobody can be
infected, and a character that was just infected cannot infect for 6 seconds (the numbers are at the top of manual/virus.rs: GRACE, CONTACT_TIME, IMMUNE, INFECTED_SPEED).  The table of results lists every
character with how long it lasted (a survivor: the whole match; the infected: when it was infected), and offers Restart and Quit as the story's results do.

Switch it on from the Mods page (or enabled = true in mod.ini), start the game again, and pick "Virus" from the list of modes on the arcade Select Level page (after the game's modes).  The options of
a mode that mean nothing for Virus (the score limit and method, team play and team starts, friendly fire, one shot kill) are left off its Options page.  It needs the build with extensions (the build the
mods are made for; the default build is the original game).

The groans are four sounds of the mod's own (SFX.PAK/sfx/*.wav, replacing four silent sounds of the game); `python make-groans.py` makes them (numpy), as the PAK folders are not kept in the repository.
It is made of `rule virus 1` in modes.txt (the number is how many characters start infected) and `hide` for the options of the page to leave out (docs/modding-guide.md).
