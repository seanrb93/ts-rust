Weapon Balance: every weapon of the game set again from scratch, both triggers and the blasts, by data only (weapons.txt).  The blasts need an extensions build; the rest works in any build.

The choices are in settings.ini: how hard the guns hit (gun_damage, in percent), how strong the explosions are (explosion_strength, in percent), whether the shotguns are balanced, whether less ammunition can be carried, and whether the bots fire no faster than a player.  Change a value, save the file and start the game again.  The numbers below are for the settings as they come.

The figures are hits on the body from full health (1.0), at the measured rates of fire (docs/damage.md).  A hit on the head is worth 5 times and an arm or a leg 0.4.

The rules
  Every automatic kills in 0.6 to 0.9 seconds and needs 2 hits on the head.
  The guns that fire a shot at a time (the pistols, the Colt, the Raygun and the accurate second triggers) need 1 hit on the head, which rewards aim.
  A shotgun pellet is under 0.2, so one stray pellet on the head does not kill: a shotgun kills by putting most of its pellets in.
  A sniper rifle kills with 2 hits on the body or 1 on the head.
  A blast kills only near its middle, and the weapons that make one are scarce.  In the game every blast is 2.3 at its middle, whatever made it, which killed from a long way out.

The weapons (body hits to kill, and the time they take)
  Sidearms
    Pistol 4 in 1.0 s
    Mauser Pistol 5 in 0.8 s; the second trigger 3 in 0.7 s, slower and harder
    Scifi Handgun 8 in 0.9 s; the second trigger's slow bolts 5 in 0.8 s (they were 0.8 each, nearly a kill a bolt)
    Colt Pistol 2 in 0.7 s, slow and accurate
  Submachine guns (wide at range)
    Uzi 8 in 0.7 s
    Tommy Gun 7 in 0.7 s; the second trigger is a slow, accurate shot, 4 in 0.8 s
  Rifles (accurate)
    M16 7 in 0.7 s
    Assault Rifle 6 in 0.6 s, less accurate
    Scifi Autorifle 7 in 0.8 s with a clip of 50; the grenade of the second trigger has 0.35 of the blast (at most 0.8 at its middle), so it kills only with a direct hit.  It shares the plentiful plasma and fired 1.5 grenades a second
    Raygun Carbine 7 in 0.8 s; the faster second trigger 9 in 0.8 s
    Raygun 4 in 0.75 s; the second trigger's slow bolts 6 in 0.8 s
  Minigun 8 in 0.7 s with the most ammunition; the second trigger sprays faster for less, 12 in 0.8 s
  Shotguns (the quickest kills, near only)
    Shotgun: 1 shot when nearly every pellet hits (6 of 0.19), then a long reload; the second trigger spreads wider
    Assault Shotgun: 2 shots at point blank (0.33 s apart) and more at range; it killed in 1 shot before
    Blunderbuss: 1 shot, only very near
  Sniper Rifle 2 on the body (0.6 each), Scifi Sniper Rifle 2 (0.51) with the bigger clip; both 1 on the head
  Explosives (the strength of the blast as a factor of the game's, and how far out it still kills)
    Rocket Launcher 1.0, within 1.4 units of a radius of 3.  The second trigger fires 3 rockets of 0.6 each: one kills only on a direct hit, two together do
    Grenade Launcher 0.9, within 1.6 of 4; a grenade that hits a character directly kills by itself
    Proximity, Remote and Timed Mines 0.8, within 1.6 of 5 (they killed within 2.3)
    TNT keeps the game's blast: it is thrown, slow and has to be set off

The most that can be carried: pistol ammunition 500 -> 240, rifle 200 -> 180, shotgun 100 -> 48, plasma 200 -> 180, rockets 30 -> 12, grenades 20 -> 12.  The kinds are shared by their weapons.

The bots' pause between shots (weapon table +60, which only the bots read) is the rate a player is held to, so a bot does not fire a pistol ten times as fast as a player can.

A blast belongs to the kind of bullet, so weapons that fire the same kind share it (blast and alt_blast in weapons.txt).

All machines of a network game need the same mods, and this is one of them.
