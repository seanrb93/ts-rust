Test scripts for network play. Each starts several instances of the port on one machine (a host and clients, headless), plays a match with scripted button presses and prints the number of desyncs and panics of each. They use the calltrace build in `build/cargo-ct`, copy it to `target/release`, and write their output to the scratchpad folder named inside them (edit `S` in the script to change it). Run one from the repository folder with the number of frames as the argument, for example `scripts/net-match-4-players.sh 4200`. A match needs about 3400 frames to start with eight or more players, so give those more (5600 for 8, 5000 or more for 10).

* `net-match-N-players.sh`: a host and N-1 clients play a match.
* `net-lobby-picture-10-players.sh`: the same with ten, and a picture of the Host Game page.
* `net-match-front-end-Nup.sh`: the front end has the number of players of a local game (2up, 3up, 4up) before the Network pages open; the match has its own players.
* `net-match-end-front-end-*.sh`: a one-kill limit ends the match, and button presses go through the end screens; the log shows the front end's number of players when the match is over.
* `net-match-end-baseline*.sh`: the same with an exe built from an earlier commit in `build/cargo-netbase` (to tell a new problem from an old one).

The presses of these scripts are those of the older Network pages (a page of rows for the host, Host before Join) and need changing to the pages of `docs/networking.md`: X, Down to Host Game and X; the level, the mode and Go on the arcade pages; X on a character; X to start.
