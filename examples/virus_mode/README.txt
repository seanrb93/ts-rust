Virus, a mode of the later TimeSplitters games, as a script of a mod (scripts/virus.rhai; docs/scripting.md says how scripts work).

A free for all: one character, chosen at random, starts infected.  An infected character that touches a survivor infects him at once, and he plays as infected from then on.  The last survivor wins.
Computer characters never shoot in this mode: the survivors run from the infected and spread out, and the infected go after the survivors on foot, a little slower than they run.

An infected character is on fire with a green flame, is lit green, and says its own lines of being hit now and then (and when it is infected); the edge of the screen of an infected player glows green.
The first 30 seconds of a match nobody can be infected (the computer characters walk about), and a character that was just infected cannot infect for 6 seconds.  The numbers are the first functions of
the script (grace, contact_time, immune, touch, infected_speed, start_infected).  The table of results lists every character with how long it lasted (a survivor: the whole match; the infected: when it was
infected), and offers Restart and Quit as the story's results do.

Switch it on from the Mods page (or enabled = true in mod.ini), start the game again, and pick "Virus" from the list of modes on the arcade Select Level page (after the game's modes).  The options of
a mode that mean nothing for Virus (the score limit and method, team play and team starts, friendly fire, one shot kill) are left off its Options page with `hide` in modes.txt.  It needs the build
with extensions (the build the mods are made for; the default build is the original game).
