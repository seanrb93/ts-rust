# Network play

Status: a first version on the `networking` branch.  Two to four machines run the same game in lockstep over UDP.  A lobby (`--lobby`) sets a game up
without a command line; the game can also be started by hand on each machine (below).

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

## The lobby

Start the game with `--lobby`.  The lobby is a plain text screen (keyboard: arrows, Enter, Esc, and the digits and dots of an address; pad: the D-pad,
X and circle):

* **Host a game** listens on UDP port 7777 and shows the address to give out.  Left and Right choose the level; the players that join are listed;
  Enter starts the game for everybody (2 to 4 players).
* **Join a game** asks for the host's address (`192.168.1.20`, or with a port), sends JOIN and waits for the host to start.

When the host starts, every machine gets the level, the number of the player, the others' addresses and the random seed, and starts the game at once.
For tests without a window: `TS_LOBBY_AUTO=host,3,Prison` (starts when 3 players are there) and `TS_LOBBY_AUTO=join,127.0.0.1:7777`, with `--lobby --headless`.

The lobby is separate from the game's own menus for now: it starts a Deathmatch on the chosen level with default settings and the first characters.
Choosing the mode, the options, the characters, bots and weapon sets from the arcade pages is still to do.

## Trying it by hand (all the machines on one PC, or a LAN)

```
ts-rust.exe --level Prison --players 2 --net local=0,peers=192.168.1.10:7000/192.168.1.11:7000,seed=7,delay=3
ts-rust.exe --level Prison --players 2 --net local=1,peers=192.168.1.10:7000/192.168.1.11:7000,seed=7,delay=3
```

`local` is the number of this machine's player (0 to 3), `peers` the address of every player's machine in the order of the players (a machine listens
on its own entry), `seed` and `delay` are the same everywhere.  On one PC use different ports (`127.0.0.1:7000/127.0.0.1:7001`).  `TS_NETLOG=1` prints a
line with the checksum and where the players are every 30 ticks, to compare the machines.

## What works and what does not (yet)

Works: 2 and 3 players, with different pads on each machine, in the same game with identical checksums; each machine shows its own view.
The lobby (above) with 3 machines on one PC reaches the same state on all of them.
Not done: a game set-up that comes from the game's own front end pages (mode and options, characters, bots, weapon sets; mods must match by hand), pausing,
the sound listener (it hears the first player), a player leaving, reaching a machine behind a router, and a check that a platform's maths
functions give the same results as another's (play between different systems may desync; two machines of the same build are fine).
