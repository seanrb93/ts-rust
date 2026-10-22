//! Functions: loading, saving and copying files and the pack file.

use crate::*;

/// Save a file.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn fileSave(mut a0: i32, mut a1: i32, mut a2: i32) {
    let mut a1_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    a1_1 = 1538;
    v0_1 = sceOpen(&[a0 as i64, a1_1 as i64, a2 as i64]);
    a1_2 = a1;
    if v0_1 < 0 {
        a0_1 = 0;
        exit(a0_1);
    }
    a2_1 = a2;
    a0_2 = v0_1;
    sceWrite(&[a0_2 as i64, a1_2 as i64, a2_1 as i64]);
    a0_3 = v0_1;
    sceClose(&[a0_3 as i64]);
    return;
}

/// Load a file.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn fileLoad(mut a0: i32, mut a1: i32, mut a2: i32) -> i32 {
    let mut s1_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut v0_3: i32 = 0;
    s1_1 = 0;
    v0_1 = fileSize(a0);
    if v0_1 > 0 {
        a1_1 = a1;
        a0_1 = v0_1 + 15 & -16;
        v0_2 = memAllocAlign64(a0_1, a1_1);
        s1_1 = v0_2;
        a0_2 = a0;
        a1_2 = s1_1;
        a2_1 = v0_1;
        fileLoadTo(a0_2, a1_2, a2_1);
    }
    if a2 != 0 {
        sw(a2, v0_1);
    }
    v0_3 = s1_1;
    return v0_3;
}

/// Copy a file.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn fileCopy(mut a0: i32, mut a1: i32) {
    let mut v0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a1_2: i32 = 0;
    v0_1 = fileSize(a0);
    if lw(0x3afe80) == 0 {
        a0_1 = 0x400000;
        v0_2 = malloc(a0_1);
        sw(0x3afe80, v0_2);
    }
    if v0_1 <= 0 {
        return;
    }
    a1_1 = lw(0x3afe80);
    a0_2 = a0;
    a2_1 = v0_1;
    fileLoadTo(a0_2, a1_1, a2_1);
    a0_3 = a1;
    a2_2 = v0_1;
    a1_2 = lw(0x3afe80);
    fileSave(a0_3, a1_2, a2_2);
    return;
}

/// Size of a file.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn fileSize(mut a0: i32) -> i32 {
    let mut v0_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut a0_3: i32 = 0;
    'b1: {
        if lw(0x3afea8) == 0 {
            v0_4 = lw(g::displayallfileops);
        } else {
            v0_1 = packfileSeek(a0);
            if v0_1 != 0 {
                v0_2 = lw(v0_1 + 52);
                v0_3 = if lw(v0_1 + 56) != 0 { v0_2 + 256 } else { v0_2 };
                break 'b1;
            }
            v0_4 = lw(g::displayallfileops);
        }
        a0_1 = a0;
        if v0_4 != 0 {
            pause();
            a0_1 = a0;
        }
        a1_1 = 1;
        v0_5 = sceOpen(&[a0_1 as i64, a1_1 as i64]);
        a0_2 = v0_5;
        if v0_5 < 0 {
            v0_3 = -1;
        } else {
            a1_2 = 0;
            a2_1 = 2;
            v0_6 = sceLseek(&[a0_2 as i64, a1_2 as i64, a2_1 as i64]);
            if lw(g::displayallfileops) != 0 {
                pause();
            }
            a0_3 = v0_5;
            sceClose(&[a0_3 as i64]);
            v0_3 = v0_6;
        }
    }
    return v0_3;
}

/// Load a file to an address.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn fileLoadTo(mut a0: i32, mut a1: i32, mut a2: i32) -> i32 {
    let mut v0_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut v0_4: i32 = 0;
    if lw(0x3afea8) == 0 {
        v0_2 = 0;
    } else {
        v0_1 = packfileSeek(a0);
        v0_2 = 0;
        if v0_1 != 0 {
            a4_1 = lw(v0_1 + 56);
            s1_1 = lw(v0_1 + 52);
            if a4_1 != 0 {
                v0_3 = -16;
                s1_2 = a1 + (a2 & v0_3) - (a4_1 + 15 & v0_3);
                a0_2 = lw(lw(0x3afeb0) + 4) + (lw(v0_1 + 48) >> 11);
                a2_2 = a4_1;
                a1_2 = s1_2;
                fileLoadCD(a0_2, a1_2, a2_2);
                if lw(g::displayallfileops) == 0 {
                    a1_3 = lw(v0_1 + 56);
                } else {
                    pause();
                    a1_3 = lw(v0_1 + 56);
                }
                a0_3 = s1_2;
                a2_3 = a1;
                v0_4 = fileInflate(a0_3, a1_3, a2_3);
                v0_2 = v0_4;
            } else {
                a1_1 = a1;
                a2_1 = s1_1;
                a0_1 = lw(lw(0x3afeb0) + 4) + (lw(v0_1 + 48) >> 11);
                fileLoadCD(a0_1, a1_1, a2_1);
                v0_2 = s1_1;
                if lw(g::displayallfileops) != 0 {
                    pause();
                    v0_2 = s1_1;
                }
            }
        }
    }
    return v0_2;
}

/// Load a file into new memory.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn fileGetMem(mut a0: i32) -> i32 {
    let mut v0_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut v1_1: i32 = 0;
    'b1: {
        if lw(0x3afea8) != 0 {
            v0_1 = packfileSeek(a0);
            v0_2 = lw(0x3afeb0);
            if v0_1 != 0 {
                a0_1 = lw(v0_2 + 16);
                v0_3 = 0;
                if a0_1 == 0 {
                    break 'b1;
                }
                v0_3 = 0;
                if lw(v0_1 + 56) != 0 {
                    break 'b1;
                }
                v1_1 = lw(v0_1 + 48);
                v0_3 = 0;
                if (v1_1 & 15) != 0 {
                    break 'b1;
                }
                v0_3 = a0_1 + v1_1;
                break 'b1;
            }
        }
        v0_3 = 0;
    }
    return v0_3;
}

/// Load a file from the disc.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn fileLoadCD(mut a0: i32, mut a1: i32, mut a2: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let mut s2_1: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut s4_1: i32 = 0;
    let mut s6_1: i32 = 0;
    let mut s5_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut s0_2: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a3_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut v0_8: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a0_10: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut v0_9: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a0_11: i32 = 0;
    sw(sp, a2 & 2047);
    s2_1 = a0;
    s3_1 = a1;
    s0_1 = a2 >> 11;
    sb(0x3afe91, 1);
    sb(0x3afe90, 0);
    sb(0x3afe92, 0);
    a0_1 = 0;
    FlushCache(a0_1);
    v0_1 = lw(sp);
    if s0_1 != 0 {
        'l2: loop {
            s1_1 = if s0_1 < 0x2801 { s0_1 } else { 0x2800 };
            s4_1 = 1;
            s6_1 = s0_1 - s1_1;
            s5_1 = s2_1 + s1_1;
            'l4: loop {
                a0_2 = s2_1;
                'l6: loop {
                    a1_1 = s1_1;
                    a3_1 = 0x3afe90;
                    a2_1 = s3_1;
                    v0_2 = sceCdRead(&[a0_2 as i64, a1_1 as i64, a2_1 as i64, a3_1 as i64]);
                    if v0_2 == 0 {
                        break;
                    }
                    'l8: loop {
                        a0_3 = 0;
                        v0_3 = sceCdSync(&[a0_3 as i64]);
                        v1_1 = 0x100000;
                        if v0_3 == 0 {
                            break;
                        }
                        'l10: loop {
                            v1_1 = v1_1 - 1;
                            if v1_1 == 0 {
                                break;
                            }
                        }
                        if v0_3 == 0 {
                            break;
                        }
                    }
                    v0_4 = sceCdGetError(&[]);
                    if v0_4 != 0 {
                        a0_4 = 0x3a06e8;
                        scePrintf(&[a0_4 as i64]);
                    } else {
                        s4_1 = 0;
                    }
                    if s4_1 == 0 {
                        break 'l4;
                    }
                    a0_2 = s2_1;
                }
                v0_5 = sceCdGetError(&[]);
                a1_2 = v0_5;
                a0_5 = 0x3a06c8;
                scePrintf(&[a0_5 as i64, a1_2 as i64]);
            }
            s3_1 = s3_1 + (s1_1 << 11);
            s0_1 = s6_1;
            s2_1 = s5_1;
            if s0_1 == 0 {
                break;
            }
        }
        v0_1 = lw(sp);
    }
    s0_2 = 1;
    if v0_1 == 0 {
        return;
    }
    'l13: loop {
        a0_6 = s2_1;
        'b16: {
            'l14: loop {
                a1_3 = 1;
                a3_2 = 0x3afe90;
                a2_2 = 0x1fb4440;
                v0_6 = sceCdRead(&[a0_6 as i64, a1_3 as i64, a2_2 as i64, a3_2 as i64]);
                if v0_6 == 0 {
                    break 'b16;
                }
                'l17: loop {
                    a0_7 = 0;
                    v0_7 = sceCdSync(&[a0_7 as i64]);
                    v1_2 = 0x100000;
                    if v0_7 == 0 {
                        break;
                    }
                    'l19: loop {
                        v1_2 = v1_2 - 1;
                        if v1_2 == 0 {
                            break;
                        }
                    }
                    if v0_7 == 0 {
                        break;
                    }
                }
                v0_8 = sceCdGetError(&[]);
                if v0_8 != 0 {
                    a0_8 = 0x3a06f8;
                    scePrintf(&[a0_8 as i64]);
                } else {
                    s0_2 = 0;
                }
                a0_6 = s2_1;
                if s0_2 == 0 {
                    break;
                }
            }
            a2_3 = lw(sp);
            a0_9 = 0x1fb4440;
            a1_4 = s3_1;
            bcopy(a0_9, a1_4, a2_3);
            a0_10 = s3_1;
            a1_5 = lw(sp);
            sceSifWriteBackDCache(&[a0_10 as i64, a1_5 as i64]);
            return;
        }
        v0_9 = sceCdGetError(&[]);
        a1_6 = v0_9;
        a0_11 = 0x3a06c8;
        scePrintf(&[a0_11 as i64, a1_6 as i64]);
    }
}

/// Start the pack file.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn packfileMake() {
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a2_4: i32 = 0;
    a0_1 = 0x3a0708;
    sw(0x3afea8, 0);
    a1_1 = 0;
    a2_1 = 2;
    packfileLoad(a0_1, a1_1, a2_1);
    a1_2 = 0;
    a0_2 = 0x3a0720;
    a2_2 = 2;
    packfileLoad(a0_2, a1_2, a2_2);
    a1_3 = 0;
    a0_3 = 0x3a0738;
    a2_3 = 2;
    packfileLoad(a0_3, a1_3, a2_3);
    a1_4 = 0;
    a0_4 = 0x3a0750;
    a2_4 = 2;
    packfileLoad(a0_4, a1_4, a2_4);
    return;
}

/// Find a file in the pack file.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn packfileSeek(mut a0: i32) -> i32 {
    let frame = Frame::new(80);
    let sp = frame.sp();
    let mut a1_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut t5_1: i32 = 0;
    let mut a5_1: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut t4_1: i32 = 0;
    let mut a3_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a7_1: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut a4_1: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut v1_2: i32 = 0;
    let mut a0_3: i32 = 0;
    'b1: {
        a1_1 = 0x3ae550;
        v0_1 = strcmp(a0, a1_1);
        a1_2 = lw(g::remember);
        s0_1 = if v0_1 >= 0 { a0 + 6 } else { a0 };
        if a1_2 == 0 {
            a1_3 = s0_1;
        } else {
            a0_1 = s0_1;
            v0_2 = strcmp(a0_1, a1_2);
            a1_3 = s0_1;
            if v0_2 == 0 {
                v0_3 = lw(g::remember);
                break 'b1;
            }
        }
        'b3: {
            a0_2 = sp;
            a2_1 = 48;
            strncpy(a0_2, a1_3, a2_1);
            t5_1 = lw(0x3afea8);
            a5_1 = 0;
            if t5_1 > 0 {
                v0_4 = 0x1fb4c40;
                'l4: loop {
                    'b6: {
                        t4_1 = a5_1 << 2;
                        a3_1 = 0;
                        v1_1 = lw(t4_1 + v0_4);
                        v0_5 = lw(v1_1 + 12);
                        a2_2 = lw(v1_1 + 20);
                        if v0_5 > 0 {
                            a7_1 = lw(sp);
                            'l7: loop {
                                'b8: {
                                    v0_6 = lw(a2_2);
                                    a4_1 = 1;
                                    if v0_6 != 0 {
                                        if v0_6 != a7_1 {
                                            a4_1 = 0;
                                        } else {
                                            a1_4 = 1;
                                            'l9: loop {
                                                v1_2 = a1_4 << 2;
                                                if a1_4 >= 12 {
                                                    break 'b8;
                                                }
                                                a0_3 = lw(a2_2 + v1_2);
                                                if a0_3 == 0 {
                                                    break 'b8;
                                                }
                                                if a0_3 != lw(sp + v1_2) {
                                                    break;
                                                }
                                                a1_4 = a1_4 + 1;
                                            }
                                            a4_1 = 0;
                                        }
                                    }
                                }
                                a3_1 = a3_1 + 1;
                                if a4_1 != 0 {
                                    break 'l4;
                                }
                                a2_2 = a2_2 + 60;
                                if a3_1 >= v0_5 {
                                    break 'b6;
                                }
                            }
                        }
                    }
                    a5_1 = a5_1 + 1;
                    v0_4 = 0x1fb4c40;
                    if a5_1 >= t5_1 {
                        break 'b3;
                    }
                }
                v0_3 = a2_2;
                sw(g::remember, a2_2);
                sw(0x3afeb0, lw(t4_1 + 0x1fb4c40));
                break 'b1;
            }
        }
        v0_3 = 0;
    }
    return v0_3;
}

/// Load the pack file.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn packfileLoad(mut a0: i32, mut a1: i32, mut a2: i32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let mut s6_1: i32 = 0;
    let mut s3_1: i32 = 0;
    let mut s1_1: i32 = 0;
    let mut s0_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut v0_1: i32 = 0;
    let mut v0_2: i32 = 0;
    let mut s1_2: i32 = 0;
    let mut s2_1: i32 = 0;
    let mut v0_3: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_3: i32 = 0;
    let mut v0_4: i32 = 0;
    let mut a0_4: i32 = 0;
    let mut a1_3: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut v1_1: i32 = 0;
    let mut a0_5: i32 = 0;
    let mut a1_4: i32 = 0;
    let mut a2_2: i32 = 0;
    let mut a0_6: i32 = 0;
    let mut a1_5: i32 = 0;
    let mut v0_5: i32 = 0;
    let mut v0_6: i32 = 0;
    let mut a1_6: i32 = 0;
    let mut a0_7: i32 = 0;
    let mut v0_7: i32 = 0;
    let mut a2_3: i32 = 0;
    let mut a1_7: i32 = 0;
    let mut a0_8: i32 = 0;
    let mut a0_9: i32 = 0;
    let mut lo_1: i32 = 0;
    let mut s7_1: i32 = 0;
    'b1: {
        s6_1 = -1;
        s3_1 = -1;
        s1_1 = 0;
        if a1 != 0 {
            s0_1 = a0 + 7;
            if a2 != 2 {
                break 'b1;
            }
        }
        s0_1 = a0 + 7;
        if lw(0x3afeac) != 0 {
            a0_1 = -1;
            exit(a0_1);
        }
    }
    'l3: loop {
        a0_2 = 0x3afea0;
        a1_1 = s0_1;
        v0_1 = SCEECdSearchFile(a0_2, a1_1);
        if v0_1 == 1 {
            s3_1 = lw(0x3afea0);
            s1_1 = 1;
            s6_1 = lw(0x3afea4);
        }
        v0_2 = lw(0x3afea8);
        if s1_1 != 0 {
            break;
        }
    }
    'b6: {
        s1_2 = 0;
        if v0_2 < 8 {
            s2_1 = 0;
            if a1 == 0 {
                a0_5 = s3_1;
                a1_4 = sp;
                a2_2 = 12;
                fileLoadCD(a0_5, a1_4, a2_2);
            } else {
                s2_1 = lw(0x3afea4);
                v0_3 = s2_1 + 15;
                if s2_1 <= 0 {
                    break 'b6;
                }
                a1_2 = a2;
                a0_3 = v0_3 & -16;
                v0_4 = memAllocAlign64(a0_3, a1_2);
                s1_2 = v0_4;
                a0_4 = s3_1;
                a1_3 = s1_2;
                a2_1 = s2_1;
                fileLoadCD(a0_4, a1_3, a2_1);
                v1_1 = lw(s1_2 + 8);
                sd(sp, ld(s1_2));
                sw(sp + 8, v1_1);
            }
            if lw(sp) == 0x4b433450 {
                if (0x3c000 as u32) >= (lw(sp + 8) as u32) {
                    a0_6 = 32;
                    a1_5 = a2;
                    v0_5 = memAlloc(a0_6, a1_5);
                    if s1_2 == 0 {
                        a1_6 = 2;
                        a0_7 = lw(sp + 8) + 15 & -16;
                        v0_7 = memAllocAlign64(a0_7, a1_6);
                        a2_3 = lw(sp + 8);
                        a1_7 = v0_7;
                        sw(v0_5 + 20, v0_7);
                        a0_8 = s3_1 + ((s6_1 - a2_3) >> 11);
                        fileLoadCD(a0_8, a1_7, a2_3);
                        sw(v0_5 + 4, s3_1);
                        sw(v0_5 + 8, s6_1);
                        sw(v0_5, s7_1);
                        sw(v0_5 + 16, 0);
                    } else {
                        v0_6 = lw(sp + 8);
                        sw(v0_5 + 4, s3_1);
                        sw(v0_5 + 8, s6_1);
                        sw(v0_5 + 20, s1_2 + s2_1 - v0_6);
                        sw(v0_5, -1);
                        sw(v0_5 + 16, s1_2);
                    }
                    a0_9 = lw(0x3afea8);
                    lo_1 = ((lw(sp + 8) as u32) / (60 as u32)) as i32;
                    sw((a0_9 << 2) + 0x1fb4c40, v0_5);
                    sw(0x3afea8, a0_9 + 1);
                    sw(v0_5 + 12, lo_1);
                    if a1 != 0 {
                        if a2 == 1 {
                            sw(0x3afeac, lw(0x3afeac) + 1);
                        }
                    }
                }
            }
        }
    }
    return;
}

/// Load an image.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn ImageLoad(mut a0: i32, mut a1: i32) {
    let mut v0_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_3: i32 = 0;
    v0_1 = ImageGetDetails(a0);
    a2_1 = v0_1;
    sw(a1, 0);
    sb(a1 + 37, 1);
    if (v0_1 as u32) < (8 as u32) {
        match (v0_1 << 2) >> 2 {
            0 | 3 => {
            }
            1 => {
                a0_2 = a0;
                a1_2 = a1;
                OpenRGB(a0_2, a1_2);
                return;
            }
            2 | 4 | 5 | 6 | 7 => {
                a0_1 = a0;
                a1_1 = a1;
                OpenXPM(a0_1, a1_1, a2_1);
                return;
            }
            _ => unreachable!(),
        }
    }
    a0_3 = 1;
    exit(a0_3);
    return;
}

/// Load an image from memory.
#[allow(unused_assignments, unused_mut, unused_variables, unused_parens)]
pub fn ImageLoadFromMem(mut a0: i32, mut a1: i32) {
    let mut v0_1: i32 = 0;
    let mut a2_1: i32 = 0;
    let mut a1_1: i32 = 0;
    let mut a0_1: i32 = 0;
    let mut a1_2: i32 = 0;
    let mut a0_2: i32 = 0;
    let mut a0_3: i32 = 0;
    v0_1 = ImageGetDetailsFromMem(a0);
    a2_1 = v0_1;
    sw(a1, 0);
    sb(a1 + 37, 1);
    if (v0_1 as u32) < (8 as u32) {
        match (v0_1 << 2) >> 2 {
            0 | 3 => {
            }
            1 => {
                a1_2 = a1;
                a0_2 = g::downloadbuffer + 4;
                OpenRGB(a0_2, a1_2);
                return;
            }
            2 | 4 | 5 | 6 | 7 => {
                a1_1 = a1;
                a0_1 = g::downloadbuffer + 4;
                OpenXPM(a0_1, a1_1, a2_1);
                return;
            }
            _ => unreachable!(),
        }
    }
    a0_3 = 1;
    exit(a0_3);
    return;
}
