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

NETWORK is one of the big items of the front end's Select Game Type page (between ARCADE and CHALLENGE).  It opens a page with **Join Game** and **Host Game** (`avmenu.rs`, `netmenu.rs`; the rows come from `lobby.rs`).

Joining: Join Game searches the local network and lists the games found (the host's name, the level, the mode, the players); a host on another network is reached by typing its address with the keyboard (digits, `.` and `:`; Delete erases) and choosing the row with it.  Choosing a game connects to it and opens Game Details: the host, the level, the mode, its options and the players, which follow what the host does.  Select Character opens the game's own character select page.  A player who has chosen waits there for the host; the triangle button goes back to choosing, and again to Game Details.

Hosting: Host Game opens the game's own arcade pages: the level and the mode on the arcade page (a Map Maker level offers Load Map, but not Create Map, and the modes that mods add and the cheats are left out, as the other machines are not told of them), then the match settings on the arcade options page (with the game's Setup Weapons and Setup Bots pages), then the character select page.  The game is open to players from the moment Host Game is chosen: it listens on UDP port 7777 and announces itself to the local network.  Once the host has chosen a character the page says what it waits for: the host's address while nobody has joined, then the players who have not chosen yet.  When everybody has, X starts the match.  Stepping back from the arcade page closes the game, and the players are told.

Handicaps and teams are not chosen on the character select page of a network match (the players take the teams in turn, as do the bots).

The lobby keeps running on all these pages (`av_net_background`, called from `frontTick`), and a player who is not heard from for 8 seconds is dropped, so that nobody waits for a game that has gone.

After a match the statistics are saved without asking and each machine goes back by itself: the host to the match settings (`RESUME` and `HOP` in `netmenu.rs`), and the others join it again and see Game Details.  A player who is back before the host is told it is connecting until the host is back too.

* The name of a player is the name of the profile signed in on that machine (the computer's user name when there is none; 12 letters at most).

When the host starts, each machine starts the level from the front end at once (`netlink_start_match`): every machine gets the level, the number of the player, the others' addresses and the random seed; the players get profiles of their own, and the game goes back to being a game of one machine when the level is left.
`--lobby` is still there, for tests: a plain text screen before the game starts (`TS_LOBBY_AUTO` and the others below belong to it).

When the host starts, every machine gets the level, the number of the player, the others' addresses and the random seed, and starts the game at once.
The host also chooses the weapon set (one of its own custom sets, or a preset); the five weapons of a custom set go to the other machines with the rest of the match, so the sets saved on each machine do not matter.  The bot sets are not needed: the bots' characters are sent.

The input delay is fixed (2 ticks); with rollback it is not a setting.  `saves/net-stats.txt` (below) shows how long the stalls are.

The match is set up on the host by the game's own start function (`frontmpgStartGame`, whose block of 112 bytes is captured at `gameDeferedReset`), and that block goes to every machine in the roster with the values of the option variables; each machine sets the variables and starts the level with the block.  Level music is not part of it: it is a setting of each machine (the Options page).  The checksum leaves out the four buffer addresses of a player, which depend on what each machine had in memory before the level.

Map Maker maps: the arcade page's Map Maker level has Load Map, which opens the game's own page of the saved maps; the modes of the map are those its set-up allows.  A map is 8192 bytes as the game keeps it in memory.  The roster carries the map's checksum, and the host sends the map in eight pieces of 1024 bytes (UDP, over and over) to each machine that has not said it has the whole of it; Start waits until every machine has it ("Sending the map to ..." on the character select page; Game Details shows "Map: receiving n of 8").  Each machine puts the same bytes in the game's buffer for maps and the arcade state the host's pages left (level, mode, set-up, tileset, teams) before the level starts.  Map Maker levels are played with the pictures between ticks like any other.  (They once were not: bots played them differently on machines that drew different numbers of pictures, and with lag the machines parted.  The cause was the mods "Bots Can Punch" and "Every Weapon (x2)", which kept what they remember while a game is played (when a bot may punch again, whose swipe is the left fist's, who owns an added weapon pair) in the program's own memory, which going back some ticks does not put back; `memmap.rs` now keeps it in the game's memory.  A level with no weapons to pick up makes the bots punch, which is why a Map Maker map showed it.)  Test aids that found it: `TS_SUBFORCE=<n>` (a picture every nth tick on one machine), `TS_SUBX`, `TS_SUBCHECK=2`, `TS_REPLAYCHECK=1` (the tick before is played again and what differs is told), `TS_NETLAG`, `TS_NATIVELOG=1`, `TS_NOSND=1`, `TS_NETDUMP3`, `TS_NOBOTS=1`.  The map's name is kept in the map itself and shown on the pages.

The host's mods are named on Game Details, and a machine whose mods or game files differ is refused with both lists in the message.

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

## Coming back after losing the connection

A machine that hears nothing from a player for 4 seconds gives them up, and when all have agreed the player is out: their character falls and the others play on.

A machine that has given up all the others holds the game, shows "Connection lost. Waiting for the others" with the seconds left, and says five times a second that it is there (`wait_for_others`, `RS_HELLO`).  With two players both machines do this.  The first machine still playing, on hearing from a player who is out, brings them back with a resync (below) that names the player: the machine coming back takes the game at the tick the others stop at, whatever tick it had, and from that tick its pads count again and it gets the others' (`rs_welcome`).  Its character is as the others have it, so a player who fell presses a button to come back to life.  "Player 2 is back" shows to everybody.  After a minute with nobody answering, or when the player presses triangle, the match is over for that machine and it goes back to its Network page.

A machine that quit or crashed comes back by the Join Game page: while a player is out the host shows its game there ("under way"), and answers a JOIN with what its lobby told that player when the match began (`Rejoin`, `rejoin_take`), which is the slot of the same name if there is one, and otherwise of a player who is out.  The machine starts the level as at the beginning, with that player's character (a different one would load another model and move everything in memory after it), holds at its first tick and asks to come back as above.  This needs the host to be there still, and does not work for a Map Maker level, whose map would have to be sent as well.

Test aid: `TS_NETCUT=<tick>,<seconds>` cuts a machine off from that tick for that long; `TS_RSSTART=<file>` writes the memory at the start of the level, to compare machines.  Tried with 2 and 3 machines, and with a machine that quit and started again; nothing out of step in the thousands of ticks after.

## How long a packet takes

The machines ask each other twice a second (`RS_PING`), and the view of a match shows, at the lower left, the time to the machine that takes longest: green, yellow from 100 ms, red from 180 ms.  The table of results (the pause menu's and the one at the end of a match) shows the time on each other player's row; this machine's player, the bots and a player who is out have none, and at the end of a match the times are the last ones known, as the machines have stopped asking.  Game Details shows each player's time to the host (from the lobby's JOIN and roster messages), and the host's character select shows the longest when it can start.

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

What travels: a machine sends one packet a tick to each other machine, with its pads that machine has not yet confirmed.  A pad is a byte saying which of its fields differ from the pad before it in the packet, then those fields, so a held stick or button costs one byte; a packet needs nothing from any other packet.  A checksum goes in the next 12 rounds of packets after it is made.  At 60 ms of lag that is about 4.7 KB/s to each other player, with the IP and UDP headers (it was 30.8 KB/s, two packets a tick with every pad whole).  A machine that is waiting sends its packets again every 25 ms.  `net-stats.txt` says what was sent.

Testing that rollback is exact: `TS_NETPADS=<seed>` makes each machine's pad from the tick and the player, so the same match is played whatever the timing, and the checksums of a tick must then be the same with and without `TS_NETLAG` and `TS_NETLOSS=<n>` (one in n of the packets that come is lost), on every machine and in every build that plays the same.  (`TS_NETMENUS=1` lets those pads press Start too.)  `TS_ROLLCHECK=<every n ticks>` plays the last ticks with wrong pads, undoes them and plays them right, and lists the words of memory left different; `TS_NETDUMP=<tick>` lists the words of the state the checksum covers, and with `TS_NETMEM=<file>` writes the whole memory, to compare two runs.

Each player's controls: a match's players have profiles of their own, made new for the match, so each machine tells the host what its signed-in profile has for the controls and preferences (the words of the 96 bytes that differ from a new profile's), the host passes them all on with the list of players, and every machine puts them into the new profiles before the level.  Every machine plays every player, so they must all know each player's settings.  A change in the pause menu during a match lasts for that match.  `TS_NETPREFS=<configuration>,<switches>` gives a machine's player other settings for a test, and with `TS_NETLOG` each machine lists what it has for every player.

Who may join: a host turns a machine away if it differs in the program (its version and the form of the packets), the game's packs (names and sizes) or the mods that are running, and the machine is told which of the three.  For a mod, the text files are compared by their lines and other files by their sizes; a README, the way lines end, and the name and the switch in `mod.ini` do not count.  A mod switched on or off counts from the next start of the program, as that is when it starts or stops running.

The option pages in a match: every machine plays every player's menu.  Where the pages are lives in the program's memory, outside the game's (`avmenu.rs`), so `Rollback` keeps it for each tick and going back puts it back.  What a page changes in this machine's own settings (the picture, the sound, Control Method) is only done for this machine's player and not while ticks are played again, and Exit Game ends only the machine of the player who chose it.  To test it, `TS_NETMENUS=1` lets the test pads press Start; those are real presses, so such a run can change `ts-rust.ini`: keep a copy.  Two players in the menus at the same time still share one place in the pages: every machine agrees on it, but it is awkward for those two players.

What is seen: a guess that was wrong shows for a moment (a shot that did not happen, a step the other way) and is put right when the pad comes; the input delay can be low (2 ticks by
default) because the game does not wait.  Not done: the character's animation and the sounds of a wrong guess are not hidden; a level that ends while a tick is a guess.

## What must be the same on every machine

What is drawn does not count: a machine draws its own player's view, and a flickering glow draws a random number when it is in view, so in a network game the random numbers are put back after the drawing (`netlink_random_keep_or`).  The end screens of a match are not the same on every machine either (each goes on by itself), so a new match starts with the statistics cleared (`netlink_reseed`).

Every machine plays every player, so whatever the game reads from this machine's settings must not change what happens.  Found by trying it (a client with
other settings fell out of step, as the checksums showed): the field of view and the picture's shape (they change what the game takes to be in view), the
look sensitivity, the aim assistance of a keyboard and mouse player, and the mouse itself (it turned player 1 only on the machine it was moved on).  In a
network game the picture and the field of view are the machine's own, but the game's workings (the aim, the sway of the weapon) use the original's, as `camTick` stores them for the player; the look sensitivity is applied to the right stick before the pad
is sent; the mouse's movement is sent in the pad (two numbers more) and every machine turns that player by it; the aim assistance is on for everybody.
When the checksums differ three samples running the machines bring themselves back into step (below); every difference is a DESYNC line in the log, which is still worth reading, as it shows a fault to be found.  A lone difference can come from a field only one machine's picture touches (what is in view).

## Coming back into step

The first machine still playing is in charge.  When it sees the checksums differ three times running it names a tick about two seconds on (just after a checksum's), and tells the others (`TSRS` packets on the game's socket).  A machine already past that tick says so, and a later one is named.  Each machine plays up to the tick and stops there with every pad before it known and played, so the rewinding is over.  The machine in charge sends its game, packed and in pieces; each of the others says ten times a second which pieces it has, and lost ones are sent again.  The game goes on at the same tick with the same pads, the journal starts afresh, "Back in step" shows for every player (`netlink_boundary`, `rs_stop` in `netlink.rs`, `rs_*` in `net.rs`).  The checksum just after the tick shows at once whether it worked.

What is sent: at the start of a level each machine keeps a copy of the game's memory (64 MB).  The words that differ from that copy now are what the match has done on that machine.  The machine in charge sends those; each of the others first puts back the words it changed itself, then writes what it was sent.  What no machine changed is left alone, so what has differed between the machines since before the level (where its profiles are, what it had in memory) stays as it is on each.  Also left alone, as they are each machine's own (`rs_own`): the lists and buffers of what is being drawn, the stacks of the threads, the profiles, the variables of drawing, frame timing and music, and the addresses of a player's buffers and preferences.

It takes 70 to 420 ms on one PC, 45 to 165 KB packed.  If there are 5 in a minute it does not help: no more are tried, and the old message "Out of step with the others" shows.  Not copied: where a player's option pages are (kept outside the game's memory), and the sounds playing, so a sound can stop or run on for a moment.

Test aids: `TS_NETBREAK=<tick>[,<ticks>]` makes this machine's game differ from that tick (a second, or as long as is said); `TS_RSFORCE=<tick>` (on the machine in charge) asks for a resync with nothing wrong; `TS_RSDUMP=<file>` writes what each machine changed, at each resync, to compare the machines and find what must be left alone.  Tried with 3 machines, 60 ms of lag, 1 packet in 20 lost and different saves; with a fault that does not stop (it gives up after 5); and with the host gone, when the next machine takes charge.

Each player has a profile of their own in a network game, named as the lobby says (the name of the user on each computer), with the character the lobby set; before, all the players shared one called DEBUG and so one character.  `saves/net-stats.txt` has a line every 300 ticks: the time a tick takes and how much of it is waiting for the others.

## What works and what does not (yet)

Works: 2 and 3 players, with different pads on each machine, in the same game with identical checksums; each machine shows its own view.
A machine with other game files or other mods than the host's is turned away by the lobby (a hash of the build, the packs' sizes and the files of the mods
that are on).  A player whose machine stops answering for 4 seconds is dropped: the machines that are left agree on the tick from which the player's pad is nothing pressed (the most of the player's pads any of them has, passed on between them), and go on; on that tick everybody sees "Player N has left" and the player's character falls.  With only two players, or when no other machine answers, the game ends with a message.  Each machine hears the sounds from its own
player's place.  Pausing stops nothing for the others: when a player presses Start only that player's own in-game menu opens (`netlink_pause`; the original opens the menus of every player and stops the time), the time is not stopped (`timeGetPause`), and the character of the player who paused stands still while their menu is open (their sticks are not sent).  Each player's menu is their own: it closes when that player resumes (`netlink_unpause`), whatever the others' menus do, and the time flag is cleared when none is open.  The other machines carry on, show no menu, and the mouse and sticks of the other players are not held back by it.
