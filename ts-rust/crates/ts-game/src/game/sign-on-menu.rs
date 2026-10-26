//! Making the menu of the sign-ons (the players of the saved games).

use crate::*;

/// Copy the 32 bytes of a menu item from `src` to `dst` by doublewords.
fn copy_item(src: i32, dst: i32) {
    let b = ld(src + 8);
    let c = ld(src + 16);
    sd(dst, ld(src));
    sd(dst + 8, b);
    let d = ld(src + 24);
    sd(dst + 16, c);
    sd(dst + 24, d);
}

/// Whether any of the four lists of the sign-on page's state (the bytes at 0x3aff48, 0x3aff40, 0x3aff4a and 0x3aff94) is set; the items then get flag 64.
fn any_flag() -> bool {
    lbu(0x3aff48) != 0 || lbu(0x3aff40) != 0 || lbu(0x3aff4a) != 0 || lbu(0x3aff94) != 0
}

/// Make the items of the sign-on menu (records of 32 bytes at `somi`): New Player (the first, flag 64 when there are 144 sign-ons or more), Load, then one item for each sign-on
/// (those of the groups 0 to 8 of `signoninfo`, in order, each pointing at its record of 2936 bytes in `signonarray`), the links up and down between them (the halfwords at +16
/// and +18) and the item numbers in the table at 0x1fc3310; the items of the sign-ons that are signed on get flag 64; the number of items is kept for the list (@ 0x23e088).
pub fn makesignonmenu() {
    let count = signonCount();
    let somi = g::somi;
    copy_item(g::mi_signonnew, somi);
    sh(somi + 16, count);
    let flags = lhu(somi + 10);
    sh(somi + 18, lhu(g::mi_signonload + 8));
    sh(somi + 10, if count < 144 { flags } else { flags | 64 });
    if any_flag() {
        sh(somi + 10, lhu(somi + 10) | 64);
    }
    let load = somi + 32;
    copy_item(g::mi_signonload, load);
    let up = lhu(g::mi_signonnew + 8);
    sh(load + 18, (0 < count) as i32);
    sh(load + 16, up);
    if any_flag() {
        sh(load + 10, lhu(load + 10) | 64);
    }
    let order = 0x1fc3310;
    let mut item = load + 32;
    let mut made = 0;
    let mut group = 0;
    loop {
        let mut sign_on = 0;
        if count <= 0 {
            group += 1;
        } else {
            let next_group = group + 1;
            loop {
                if lw((sign_on << 3) + g::signoninfo) == group {
                    copy_item(g::mi_signon, item);
                    let number = made + 1;
                    sh(item + 8, number);
                    sw(item, lw(g::signonarray) + sign_on * 2936);
                    if made <= 0 {
                        sh(item + 10, 0);
                    } else {
                        sh(item + 10, 32);
                    }
                    let above = if made <= 0 { lhu(g::mi_signonload + 8) } else { lhu(item + 8) - 1 };
                    sh(item + 16, above);
                    let below = if made >= count - 1 { lhu(g::mi_signonnew + 8) } else { lhu(item + 8) + 1 };
                    sh(item + 18, below);
                    if any_flag() {
                        sh(item + 10, lhu(item + 10) | 64);
                    }
                    item += 32;
                    sb(made + order, sign_on);
                    made = number;
                }
                sign_on += 1;
                if sign_on >= count {
                    break;
                }
            }
            group = next_group;
        }
        if group >= 9 {
            break;
        }
    }
    let signed_on = lw(0x3aff20);
    let mut index = 0;
    if signed_on > 0 {
        loop {
            let sign_on = signonGet(index);
            let next = index + 1;
            let total = lw(0x3aff20);
            if sign_on >= 0 && count > 0 {
                let mut k = 0;
                loop {
                    if lbu(k + order) == sign_on {
                        let at = (k << 5) + somi + 64;
                        sh(at + 10, lhu(at + 10) | 64);
                    }
                    k += 1;
                    if k >= count {
                        break;
                    }
                }
            }
            let slot = lbu(index + 0x3aff90);
            index = next;
            if slot != 0 {
                let wanted = slot - 1;
                if (wanted as u32) < (count as u32) && count > 0 {
                    let mut k = 0;
                    loop {
                        if lbu(k + order) == wanted {
                            let at = (k << 5) + somi + 64;
                            sh(at + 10, lhu(at + 10) | 64);
                        }
                        k += 1;
                        if k >= count {
                            break;
                        }
                    }
                }
            }
            if !(index < total) {
                break;
            }
        }
    }
    signon_links(made, count);
    sh(g::signonplr_list, count + 2);
}
