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

NETWORK is one of the big items of the front end's Select Game Type page (between ARCADE and CHALLENGE).  It opens pages of the game's own kind (`avmenu.rs`, `netmenu.rs`; the rows come from `lobby.rs`):

* **Host a game**: the page Host Game is set up like a local arcade game (`netsetup.rs`): the levels are those of the arcade page that this machine has open (the Map Maker's level is not offered), the modes are those the chosen level has, and the rows of the mode's options (score limit, score method, time limit, teamplay and so on), the weapon set and the bot set are the game's own tables and variables, so they read and behave as in a local game.  With a custom weapon set or bot set chosen, the square button on that row (the hint line says so) opens the game's own Setup Weapons or Setup Bots page, and its triangle button comes back to Host Game.  After them come the number of bots, your character, Start Game, and the players that have joined.
  The game listens on UDP port 7777 and shows the address to give out; a host also announces itself to the local network.
* **Join a game**: the page Join Game lists the games found on the network (X joins one); to join a host on another network type its address with the keyboard (digits, `.` and `:`; Delete erases) and choose Connect.
  Once joined, the page shows the host's settings and the players; your character can be chosen there.
* The name of a player is the name of the profile signed in on that machine (the computer's user name when there is none; 12 letters at most).

The triangle button steps back (from a hosted or joined game to the first page, then out of the page).  When the host starts, each machine starts the level from the front end at once (`netlink_start_match`):
every machine gets the level, the number of the player, the others' addresses and the random seed; the players get profiles of their own, and the game goes back to being a game of one machine when the level is left.
`--lobby` is still there, for tests: a plain text screen before the game starts (`TS_LOBBY_AUTO` and the others below belong to it).

When the host starts, every machine gets the level, the number of the player, the others' addresses and the random seed, and starts the game at once.
The host also chooses the weapon set (one of its own custom sets, or a preset); the five weapons of a custom set go to the other machines with the rest of the match, so the sets saved on each machine do not matter.  The bot sets are not needed: the bots' characters are sent.

The input delay is fixed (2 ticks); with rollback it is not a setting.  `saves/net-stats.txt` (below) shows how long the stalls are.

The match is set up on the host by the game's own start function (`frontmpgStartGame`, whose block of 112 bytes is captured at `gameDeferedReset`), and that block goes to every machine in the roster with the values of the option variables; each machine sets the variables and starts the level with the block.  Level music is not part of it: it is a setting of each machine (the Options page).  The checksum leaves out the four buffer addresses of a player, which depend on what each machine had in memory before the level.

Map Maker maps: "Map Maker" is a level of Host Game.  Choosing it adds a row "Choose a Map", which opens the game's own page of the saved maps (and returns to Host Game); the modes of the map are those its set-up allows.  A map is 8192 bytes as the game keeps it in memory.  The roster carries the map's checksum, and the host sends the map in eight pieces of 1024 bytes (UDP, over and over) to each machine that has not said it has the whole of it; Start waits until every machine has it ("Sending the map to ..." on the page; Joined Game shows "Map: receiving n of 8").  Each machine puts the same bytes in the game's buffer for maps and the arcade state the host's pages left (level, mode, set-up, tileset, teams) before the level starts.  Map Maker levels are played with the pictures between ticks like any other.  (They once were not: bots played them differently on machines that drew different numbers of pictures, and with lag the machines parted.  The cause was the mods "Bots Can Punch" and "Every Weapon (x2)", which kept what they remember while a game is played (when a bot may punch again, whose swipe is the left fist's, who owns an added weapon pair) in the program's own memory, which going back some ticks does not put back; `memmap.rs` now keeps it in the game's memory.  A level with no weapons to pick up makes the bots punch, which is why a Map Maker map showed it.)  Test aids that found it: `TS_SUBFORCE=<n>` (a picture every nth tick on one machine), `TS_SUBX`, `TS_SUBCHECK=2`, `TS_REPLAYCHECK=1` (the tick before is played again and what differs is told), `TS_NETLAG`, `TS_NATIVELOG=1`, `TS_NOSND=1`, `TS_NETDUMP3`, `TS_NOBOTS=1`.  The map's name is kept in the map itself and shown on the pages.

The mods that are on are named on the pages: Host Game ends with a line "Mods: ...", Joined Game shows "Host mods: ...", and a machine whose mods or game files differ is refused with both lists in the message.

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

## Rollback

A game started from the lobby plays with rollback (`TS_LOCKSTEP=1` asks for the lockstep above instead; `--net ...,rollback=1` for a hand start).  The game does not wait
for the other players' pads.  Each frame it plays the pads that are known and, for a player whose pad has not come yet, a guess: the last pad known, with the buttons
held still held and no button newly pressed.  When the real pad comes and is not what was played, the game goes back to that tick and plays the frames up to now again
with the pads as they are known, so that what happens is what the others' machines have.  It runs ahead with guesses for at most 12 ticks (200 ms); beyond that it waits.

How it goes back: `ts-rt/src/journal.rs` keeps, for each of the last 24 ticks, a copy of every 4 KiB page of the game's memory as it was before the tick first changed it
(every store checks a page mark; only in builds with the `extensions`, and only while a network level is on).  `ts-game/src/manual/netlink.rs` (`netlink_boundary`, between two
frames, from `ilinkOKToTick`) undoes the last ticks, then plays each frame with `sim_frame` (`boot.rs`: the main loop's frame without showing it).  The ticks played again make no sounds
(`sceSdRemote`).  The checksums are of the state after a tick once every pad up to it is known.  `TS_NETLAG=<ms>` holds back what comes, to test.  With 120 ms of it and pads
that change all the time (about 225 times in 300 ticks the game went back, 7 ticks each) the ticks stayed 16.7 ms and the checksums of the two machines agreed.

Sounds: each tick keeps what sound commands it sent the first time (`net::sound_new`); a tick played again sends only the ones that are new, so that a shot or a hit that was only learned of in the replay is heard once, and nothing is heard twice.  The shells of the guns (cartridges) are placed in the view of the player on this machine (`netlink_view_player_or`), not in that of whoever was current when the gun was ticked.  `net-stats.txt` also gives the longest time without a packet from the others.

Pictures between the ticks: the game's ticks are 60 a second on every machine, but each machine shows pictures at its own rate (`framerate` in `ts-rust.ini`; 0 is the monitor's rate).  After a tick's
picture the machine makes more (`netlink_sub_frames`, called from `present_frame` in the time before the next tick): for each, the journal marks the memory, the game plays a frame that
stands for the part of a tick that has passed (the way `high-framerate` frames do: what moves smoothly moves, nothing that happens on a whole tick happens), the picture is shown, and
the journal puts the memory back as it was.  So the game is changed only by the ticks, and a machine can show 60, 144 or 240 pictures a second whatever the others do; the pictures
start no sounds.  The picture's shape, field of view and quality are the machine's own too (`camTick` keeps the original's for what happens in the game).  `net-stats.txt` says how many pictures were wanted, how many ticks had none for the state of the game (a pause or a menu on top of the level) or for lack of time (a machine too slow for the rate), and how many sound commands were started or left out in ticks played again.  `TS_SNDLOG=1` lists every sound started (`TS_SNDLOG=names` the table of the sounds first), `TS_SNDTEST=<sound>,<tick>` starts one.  `TS_NOSUBFRAMES=1` turns it off,
`TS_SUBCHECK=1` checks that a picture between ticks leaves the checksum as it was.  Not done: the mouse's turning is still applied on the tick, so your own view is turned 60 times a second.

What is seen: a guess that was wrong shows for a moment (a shot that did not happen, a step the other way) and is put right when the pad comes; the input delay can be low (2 ticks by
default) because the game does not wait.  Not done: the character's animation and the sounds of a wrong guess are not hidden; a level that ends while a tick is a guess.

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
player's place.  Pausing stops nothing for the others: when a player presses Start only that player's own in-game menu opens (`netlink_pause`; the original opens the menus of every player and stops the time), the time is not stopped (`timeGetPause`), and the character of the player who paused stands still while their menu is open (their sticks are not sent).  Each player's menu is their own: it closes when that player resumes (`netlink_unpause`), whatever the others' menus do, and the time flag is cleared when none is open.  The other machines carry on, show no menu, and the mouse and sticks of the other players are not held back by it.
