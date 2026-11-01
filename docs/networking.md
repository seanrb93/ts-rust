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

Choose Network Game on the front end's Select Game Type page (the program starts again at the lobby), or start the game with `--lobby`.  The lobby is a plain text screen (keyboard: arrows, Enter, Esc, and the digits and dots of an address; pad: the D-pad,
X and circle):

* **Host a game** listens on UDP port 7777 and shows the address to give out.  Left and Right choose the level; the players that join are listed;
  Enter starts the game for everybody (2 to 4 players).
* **Join a game** asks for the host's address (`192.168.1.20`, or with a port), sends JOIN and waits for the host to start.

When the host starts, every machine gets the level, the number of the player, the others' addresses and the random seed, and starts the game at once.
The host also chooses the weapon set (one of its own custom sets, or a preset); the five weapons of a custom set go to the other machines with the rest of the match, so the sets saved on each machine do not matter.  The bot sets are not needed: the bots' characters are sent.

A host announces its game once a second to the local network (UDP broadcast to port 7778, a different port from the game's 7777), and the Join screen lists the games it hears (address, players, level) so that one can be picked with Up/Down and Enter; typing an address still works for a host on another network.  The broadcast leaves by the network the PC uses first, and routers do not pass it on.  `TS_LOBBY_AUTO=find` joins the first game found.

For tests without a window: `TS_LOBBY_AUTO=host,3,Prison` (starts when 3 players are there) and `TS_LOBBY_AUTO=join,127.0.0.1:7777`, with `--lobby --headless`.

The host's rows (Up/Down to move, Left/Right to change): Level, Mode (Deathmatch, Bag Tag, Capture the Bag, Knockout), Time limit (minutes), Score limit
(the kills of Deathmatch, the goal of Capture the Bag and Knockout), Bots (0 to 6), Friendly fire and Your character.  A player who has joined picks
their own character with Left/Right and the others see it.  These are the options of the game's own Options page for the mode, written into the game's records
when the match starts (`netlink_options`, `netlink_parms`, `netlink_player_char`, `netlink_bots` in `netlink.rs`); the lobby is still separate from the
game's menus, and bot sets, handicaps and the other options are not in it yet.  Not every level suits every mode (a level needs the pads
of a mode's set-up); a mode that does not fit its level may misbehave.

For tests the same setup can be given without a lobby with `--setup mode=5,time=2,limit=7,friendly=0,bots=11/12,chars=3/9` (the numbers are characters),
and `TS_LOBBY_SETUP`, `TS_LOBBY_CHAR` with the auto lobby.

## Trying it by hand (all the machines on one PC, or a LAN)

```
ts-rust.exe --level Prison --players 2 --net local=0,peers=192.168.1.10:7000/192.168.1.11:7000,seed=7,delay=3
ts-rust.exe --level Prison --players 2 --net local=1,peers=192.168.1.10:7000/192.168.1.11:7000,seed=7,delay=3
```

`local` is the number of this machine's player (0 to 3), `peers` the address of every player's machine in the order of the players (a machine listens
on its own entry), `seed` and `delay` are the same everywhere.  On one PC use different ports (`127.0.0.1:7000/127.0.0.1:7001`).  `TS_NETLOG=1` prints a
line with the checksum and where the players are every 30 ticks, to compare the machines.

## What must be the same on every machine

Every machine plays every player, so whatever the game reads from this machine's settings must not change what happens.  Found by trying it (a client with
other settings fell out of step, as the checksums showed): the field of view and the picture's shape (they change what the game takes to be in view), the
look sensitivity, the aim assistance of a keyboard and mouse player, and the mouse itself (it turned player 1 only on the machine it was moved on).  In a
network game the picture and the field of view are the machine's own, but the game's workings (the aim, the sway of the weapon) use the original's, as `camTick` stores them for the player; the look sensitivity is applied to the right stick before the pad
is sent; the mouse's movement is sent in the pad (two numbers more) and every machine turns that player by it; the aim assistance is on for everybody.
When the checksums differ three samples running (one a second) a message "Out of step with the others" shows on that machine; every difference is a DESYNC line in the log.  A lone difference can come from a field only one machine's picture touches (what is in view).

Each player has a profile of their own in a network game, named as the lobby says (the name of the user on each computer), with the character the lobby set; before, all the players shared one called DEBUG and so one character.  `saves/net-stats.txt` has a line every 300 ticks: the time a tick takes and how much of it is waiting for the others.

## What works and what does not (yet)

Works: 2 and 3 players, with different pads on each machine, in the same game with identical checksums; each machine shows its own view.
A machine with other game files or other mods than the host's is turned away by the lobby (a hash of the build, the packs' sizes and the files of the mods
that are on).  A player whose machine stops answering for 4 seconds is dropped: the machines that are left agree on the tick from which the player's pad is nothing pressed (the most of the player's pads any of them has, passed on between them), and go on; on that tick everybody sees "Player N has left" and the player's character falls.  With only two players, or when no other machine answers, the game ends with a message.  Each machine hears the sounds from its own
player's place.  Pause does not stop a network game: the pause menu opens on that machine (it still says Game Paused) while the game goes on for everybody,
and while the menu is open the sticks are not passed on (the buttons are: the menu reads the same pad, so a menu choice with X also jumps).  The lobby (above) with 3 machines on one PC reaches the same state on all of them.
Not done: a game set-up that comes from the game's own front end pages (mode and options, characters, bots, mods must match by hand),
two players leaving together (that can desync), reaching a machine behind a router, and a test of play between
different systems.  In a network game the maths functions (`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `pow`) come from the `libm` crate, whose
results are the same on every platform (the platforms' own libraries differ in the last bit); square roots, floors and the rest are exact on any machine.
That makes play between Windows, Linux and Android possible in principle, but it has only been run between copies on one Windows PC.
