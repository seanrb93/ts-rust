//! The flags and the first choice of the menu shown in a game.

use crate::*;

/// Set the flags (+0 and +8, halfwords), the cursor (+4) and the number of rows (+6, 10) of the in-game menu of the player `player` (records of 92 bytes at 0x1fb3240) by the
/// game state, the game mode (+72 of the game parameters), the parameter flags (+80) and the number of players (@ 0x212538).
pub fn ingamemenuRoot(player: i32) {
    let state = lw(g::gamestate);
    let menu = player * 92 + 0x1fb3240;
    let old_flags = lhu(menu + 8);
    sh(menu, 64);
    sh(menu + 8, old_flags & 3);
    if state != 4 {
        sh(menu, 576);
    }
    let params = g::curgameparms;
    let flags = lw(params + 80);
    let mut apply = true;
    if ((lw(params + 72) - 6) as u32) >= 2 && flags & 32 == 0 {
        apply = false;
    }
    if apply {
        let v = lhu(menu) | 2;
        sh(menu, v | 516);
        if flags & 32 != 0 {
            sh(menu, v | 517);
        }
    }
    let ranked = gameModeDisplayRank();
    let params = g::curgameparms;
    if ranked == 0 {
        sh(menu, lhu(menu) | 8);
    }
    if lw(params + 80) & 32 != 0 {
        let mode = lw(params + 68);
        if (mode as u32) < 27 {
            match mode {
                0 | 1 | 2 | 3 | 4 | 5 | 8 | 9 | 10 | 11 | 15 | 16 | 17 | 25 | 26 => {
                    sh(menu, lhu(menu) | 8);
                }
                _ => {}
            }
        }
    }
    let params = g::curgameparms;
    let kind = lw(params + 72);
    if kind != 7 {
        sh(menu, lhu(menu) | 16);
    }
    let flags = lw(params + 80);
    let mut state_now = lw(g::gamestate);
    if flags & 32 == 0 {
        sh(menu, lhu(menu) | 32);
        state_now = lw(g::gamestate);
    }
    if state_now == 4 {
        sh(menu + 8, lhu(menu + 8) | 4);
    }
    let second;
    if kind != 7 || lw(player * 1820 + lw(g::players) + 296) != 4 {
        second = lhu(menu + 8);
    } else {
        sh(menu + 8, lhu(menu + 8) | 4);
        second = lhu(menu + 8);
    }
    let params = g::curgameparms;
    if second & 4 != 0 {
        let mode = lw(params + 72);
        let first = lhu(menu) | 128;
        sh(menu, first | 256);
        if mode == 1 || lw(params + 80) & 32 != 0 {
            sh(menu, first | 768);
        }
        let mode_less = lw(params + 72) - 1;
        if (mode_less as u32) < 7 {
            match mode_less {
                0 | 1 | 2 | 4 => {
                    sh(menu + 8, second | 8);
                }
                6 => {
                    if lw(params + 80) & 0x10000000 != 0 {
                        sh(menu + 8, second | 8);
                    }
                }
                _ => {}
            }
        }
        if lw(params + 80) & 32 != 0 {
            sh(menu + 8, lhu(menu + 8) & 0xfff7);
        }
    }
    let params = g::curgameparms;
    if lw(params + 80) & 0x40000 == 0 {
        let players = lw(g::lvnumplayers);
        if lw(params + 72) != 0 {
            if players >= 2 && player == lw(0x3afe78) {
                sh(menu + 4, 6);
            } else {
                let flags = lw(params + 80);
                let early = state_now < 4;
                if flags & 32 != 0 && !early {
                    sh(menu + 4, 5);
                } else {
                    let bits;
                    if lw(params + 72) != 7 {
                        bits = lhu(menu);
                        sh(menu + 4, 3);
                    } else if state_now >= 4 {
                        sh(menu + 4, 4);
                        sh(menu + 6, 10);
                        return;
                    } else {
                        bits = lhu(menu);
                        sh(menu + 4, 3);
                    }
                    if (bits >> 3) & 1 != 0 {
                        let mut index = 3;
                        let mut next;
                        loop {
                            next = index + 1;
                            index = next;
                            if (bits >> (index & 31)) & 1 == 0 {
                                break;
                            }
                        }
                        sh(menu + 4, next);
                    }
                }
            }
        } else {
            sh(menu + 4, 7);
        }
    } else {
        let v = lhu(menu);
        sh(menu + 4, 8);
        sh(menu, v | 527);
    }
    sh(menu + 6, 10);
}
