#!/bin/bash
# usage: run5.sh <frames>  : host + 4 clients with the calltrace/symbol build; output in scratchpad
S="C:/Users/sbran/AppData/Local/Temp/claude/C--Users-sbran-Documents-ts-rust/3b0e4081-b679-4543-b93b-85959b5a1af5/scratchpad"
cd /c/Users/sbran/Documents/ts-rust/ts-rust
powershell -Command "Get-Process ts-rust* -ErrorAction SilentlyContinue | Stop-Process -Force"
(cargo build --release -p ts-port --features extended-tables,extensions,high-framerate,calltrace --target-dir build/cargo-ct 2>&1 | grep -E "^error" -A6 | head -20)
cd target/release
cp ../../build/cargo-ct/release/ts-rust.exe ./ts-rust-ct.exe; cp ../../build/cargo-ct/release/ts_rust.pdb ./ts-rust-ct.pdb; cp ../../build/cargo-ct/release/ts_rust.pdb ./ts_rust.pdb
FR=${1:-3000}
X=""; for f in $(seq 4300 90 8800); do X="$X --press $f:cross:3 --press $((f+30)):start:3 --press $((f+60)):right:3"; done
for n in A B; do rm -rf $S/sv$n; cp -r saves $S/sv$n; done
A=""; for f in 650 720 800 880 960; do A="$A --press $f:cross:3"; done; A="$A --press 1020:down:3"
H=""; for f in 1340 1360 1380 1400; do H="$H --press $f:down:3"; done
D=""; for i in 0 1 2 3 4 5 6; do D="$D --press $((1540+i*15)):down:3"; done
C="--press 1200:cross:3 --press 1320:down:3 --press 1340:cross:3 --press 1400:down:3 --press 1420:down:3 --press 1440:cross:3"
TS_KILLLIMIT=1 TS_WINLOG=1 TS_NETDROP=120 TS_BADLOG=1 TS_BT=1 TS_NETLOG=1 timeout 700 ./ts-rust-ct.exe --save-dir $S/svA --headless --frames $FR $X $A --press 1200:cross:3 --press 1320:cross:3 $H --press 1420:cross:3 --press 1440:right:3 --press 1460:cross:3 --press 1520:right:3 $D --press 2600:cross:3 --press 2900:cross:3 --screenshot $((FR-300)) $S/p5a1.png --screenshot $((FR-200)) $S/p5a2.png --screenshot $((FR-100)) $S/p5a.png > $S/h0.txt 2>&1 &
sleep 2
for n in B; do
TS_LOBBY_ADDR=127.0.0.1 TS_KILLLIMIT=1 TS_WINLOG=1 TS_NETDROP=120 TS_BADLOG=1 TS_BT=1 TS_NETLOG=1 timeout 700 ./ts-rust-ct.exe --save-dir $S/sv$n --headless --frames $FR $X $A $C --screenshot $((FR-100)) $S/p5$n.png > $S/c$n.txt 2>&1 &
sleep 1
done
wait
for f in h0 cB; do echo $f desync $(grep -c DESYNC $S/$f.txt) panic $(grep -c "PANIC" $S/$f.txt); done
grep -h -m1 -A22 "PANIC" $S/h0.txt | grep -E "PANIC|unmapped|at C:" | head -14
grep "\[net\] tick" $S/h0.txt | tail -1 | cut -c1-120
