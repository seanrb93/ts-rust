# Network play

Status: a test form on the `networking` branch.  Two to four machines run the same game in lockstep over UDP; there is no lobby yet, so a game is
started by hand on each machine.

## How it works

Every machine runs the whole game.  What travels is each player's pad, 16 bytes a tick (`ts-sdk/src/native/net.rs`): the buttons, their new
presses and releases, the four stick bytes and the dead zone, the same fields the console game sent over its i.Link cable
(`PutControllerInBuff`).  A tick waits until the pads of every player are there; the pad read on this machine is used `delay` ticks later (by
everybody), which gives the packets that long to arrive.  Each packet carries the sender's pads from the first tick the receiver still lacks, so a
lost packet is made good by the next one.  Every 30 ticks the machines compare a checksum of the players, the characters, the random numbers and the
frame, and report a `DESYNC` when they differ.

The game's own link functions do the rest (`ts-game/src/manual/netlink.rs`, the `ilink` functions of the console game): the pads of the players are
written into the controller records 8 to 11, the player count of the machine is one (so each machine draws its own player's view full screen) and
the game ticks one frame at a time at 60 Hz (no high frame rate in a network game).  The random seed is given on the command line, the same on every
machine.

## Trying it (all the machines on one PC, or a LAN)

```
ts-rust.exe --level Prison --players 2 --net local=0,peers=192.168.1.10:7000/192.168.1.11:7000,seed=7,delay=3
ts-rust.exe --level Prison --players 2 --net local=1,peers=192.168.1.10:7000/192.168.1.11:7000,seed=7,delay=3
```

`local` is the number of this machine's player (0 to 3), `peers` the address of every player's machine in the order of the players (a machine listens
on its own entry), `seed` and `delay` are the same everywhere.  On one PC use different ports (`127.0.0.1:7000/127.0.0.1:7001`).  `TS_NETLOG=1` prints a
line with the checksum and where the players are every 30 ticks, to compare the machines.

## What works and what does not (yet)

Works: 2 and 3 players, with different pads on each machine, in the same game with identical checksums; each machine shows its own view.
Not done: a lobby and a game set-up that comes from the front end (the level, mode, characters, weapon sets and mods must be the same by hand), pausing,
the sound listener (it hears the first player), a player leaving, reaching a machine behind a router, and a check that a platform's maths
functions give the same results as another's (play between different systems may desync; two machines of the same build are fine).
