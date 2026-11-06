Balance: a rebalance of the game's weapons by data only (damage, spread, ammunition, the bots' pause between shots).  It only changes the weapon table.

Both triggers are set.  The second trigger of a weapon has its own damage, which an earlier version of this mod left as the game has it, so holding that trigger avoided the rebalance (the Shotgun did 3 per shot, the Scifi Handgun 0.8 a round at 5 rounds a second).

Every weapon has a job.  The numbers are for hits on the body, from full health (1.0), at the measured rates of fire and reloads (docs/damage.md).  A hit on the head is worth 5 times and an arm or a leg 0.4.

  Sidearms (0.6 to 1 second, each with a clear strength):
    Pistol 4 hits, Mauser 5, Scifi Handgun 8, and the Colt 2 slow and accurate hits
  Submachine guns (0.8 to 0.9 seconds, wide at range):
    Uzi 10 hits, Tommy Gun 8 (its second trigger is slower, harder and very accurate)
  Rifles (0.6 to 0.8 seconds, and the most damage over time):
    M16 7 hits and the most accurate, Assault Rifle 6 and less accurate, Scifi Autorifle 7 with the big clip, Raygun Carbine 7, Raygun 4
  Minigun (0.7 seconds): the most ammunition, the second trigger sprays faster for less each
  Shotguns (the quickest kills, near only):
    Shotgun 1 shot when nearly every pellet hits, with a long reload; Assault Shotgun 2 shots; Blunderbuss 1 shot, only very near
  Snipers: 2 hits on the body, 1 on the head
  Rocket Launcher, Grenade Launcher: the blast is what kills, so their damage is as it was and they are scarce (at most 12 rounds)

The most that can be carried: pistol ammunition 500 -> 240, rifle 200 -> 180, shotgun 100 -> 48, plasma 200 -> 180, rockets 30 -> 12, grenades 20 -> 12.  The kinds are shared by their weapons.

The bots' pause between shots (weapon table +60, which only the bots read) is the rate a player is held to, so a bot does not fire a pistol ten times as fast as a player can.

All machines of a network game need the same mods, and this is one of them.
