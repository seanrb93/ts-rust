//! `sscanf` against the game address space (subset of C semantics: d i u x o c s [set] f e g n %).
#![allow(non_snake_case)]

use ts_rt::*;

/// `sscanf(str, fmt, ptr...)`; arguments arrive as 64-bit argument registers (pointers into game memory).
pub fn sscanf(args: &[i64]) -> i32 {
    let input = cstr(args[0] as i32).into_bytes();
    let fmt = cstr(args[1] as i32).into_bytes();
    let mut outs = args[2..].iter();
    let (mut ip, mut fp) = (0usize, 0usize);
    let mut count = 0;
    while fp < fmt.len() {
        let c = fmt[fp];
        if c.is_ascii_whitespace() {
            while ip < input.len() && input[ip].is_ascii_whitespace() {
                ip += 1;
            }
            fp += 1;
            continue;
        }
        if c != b'%' {
            if ip < input.len() && input[ip] == c {
                ip += 1;
                fp += 1;
                continue;
            }
            break;
        }
        fp += 1;
        let mut suppress = false;
        if fp < fmt.len() && fmt[fp] == b'*' {
            suppress = true;
            fp += 1;
        }
        let mut width = 0usize;
        while fp < fmt.len() && fmt[fp].is_ascii_digit() {
            width = width * 10 + (fmt[fp] - b'0') as usize;
            fp += 1;
        }
        let (mut h, mut l) = (0, 0);
        while fp < fmt.len() && matches!(fmt[fp], b'h' | b'l' | b'L') {
            if fmt[fp] == b'h' {
                h += 1
            } else {
                l += 1
            }
            fp += 1;
        }
        if fp >= fmt.len() {
            break;
        }
        let conv = fmt[fp];
        fp += 1;
        let maxw = if width == 0 { usize::MAX } else { width };
        match conv {
            b'%' => {
                if ip < input.len() && input[ip] == b'%' {
                    ip += 1
                } else {
                    break;
                }
            }
            b'n' => {
                if let (false, Some(p)) = (suppress, outs.next()) {
                    sw(*p as i32, ip as i32);
                }
            }
            b'c' => {
                let n = if width == 0 { 1 } else { width };
                if ip + n > input.len() {
                    break;
                }
                if !suppress {
                    let Some(p) = outs.next() else { break };
                    for k in 0..n {
                        sb(*p as i32 + k as i32, input[ip + k] as i32);
                    }
                    count += 1;
                }
                ip += n;
            }
            b's' | b'[' => {
                let mut set: Option<(bool, Vec<u8>)> = None;
                if conv == b'[' {
                    let mut neg = false;
                    if fp < fmt.len() && fmt[fp] == b'^' {
                        neg = true;
                        fp += 1;
                    }
                    let mut members = Vec::new();
                    let mut first = true;
                    while fp < fmt.len() && (fmt[fp] != b']' || first) {
                        if fp + 2 < fmt.len() && fmt[fp + 1] == b'-' && fmt[fp + 2] != b']' {
                            for ch in fmt[fp]..=fmt[fp + 2] {
                                members.push(ch);
                            }
                            fp += 3;
                        } else {
                            members.push(fmt[fp]);
                            fp += 1;
                        }
                        first = false;
                    }
                    fp += 1;
                    set = Some((neg, members));
                } else {
                    while ip < input.len() && input[ip].is_ascii_whitespace() {
                        ip += 1;
                    }
                }
                let start = ip;
                while ip < input.len() && ip - start < maxw {
                    let ch = input[ip];
                    let ok = match &set {
                        Some((neg, m)) => m.contains(&ch) != *neg,
                        None => !ch.is_ascii_whitespace(),
                    };
                    if !ok {
                        break;
                    }
                    ip += 1;
                }
                if ip == start {
                    break;
                }
                if !suppress {
                    let Some(p) = outs.next() else { break };
                    for (k, ch) in input[start..ip].iter().enumerate() {
                        sb(*p as i32 + k as i32, *ch as i32);
                    }
                    sb(*p as i32 + (ip - start) as i32, 0);
                    count += 1;
                }
            }
            b'd' | b'i' | b'u' | b'x' | b'X' | b'o' => {
                while ip < input.len() && input[ip].is_ascii_whitespace() {
                    ip += 1;
                }
                let start = ip;
                let mut neg = false;
                if ip < input.len() && (input[ip] == b'-' || input[ip] == b'+') {
                    neg = input[ip] == b'-';
                    ip += 1;
                }
                let mut radix = match conv {
                    b'x' | b'X' => 16,
                    b'o' => 8,
                    _ => 10,
                };
                if (conv == b'i' || radix == 16)
                    && ip + 1 < input.len()
                    && input[ip] == b'0'
                    && matches!(input[ip + 1], b'x' | b'X')
                {
                    radix = 16;
                    ip += 2;
                } else if conv == b'i' && ip < input.len() && input[ip] == b'0' {
                    radix = 8;
                }
                let ds = ip;
                let mut v: i64 = 0;
                while ip < input.len() && ip - start < maxw {
                    match (input[ip] as char).to_digit(radix) {
                        Some(d) => v = v.wrapping_mul(radix as i64).wrapping_add(d as i64),
                        None => break,
                    }
                    ip += 1;
                }
                if ip == ds {
                    break;
                }
                if neg {
                    v = -v;
                }
                if !suppress {
                    let Some(p) = outs.next() else { break };
                    let p = *p as i32;
                    if h >= 2 {
                        sb(p, v as i32)
                    } else if h == 1 {
                        sh(p, v as i32)
                    } else if l >= 2 {
                        sd(p, v)
                    } else {
                        sw(p, v as i32)
                    }
                    count += 1;
                }
            }
            b'f' | b'e' | b'g' | b'E' | b'G' => {
                while ip < input.len() && input[ip].is_ascii_whitespace() {
                    ip += 1;
                }
                let start = ip;
                while ip < input.len()
                    && ip - start < maxw
                    && (input[ip].is_ascii_digit() || matches!(input[ip], b'.' | b'-' | b'+' | b'e' | b'E'))
                {
                    ip += 1;
                }
                let Ok(v) = std::str::from_utf8(&input[start..ip]).unwrap_or("").parse::<f64>() else { break };
                if !suppress {
                    let Some(p) = outs.next() else { break };
                    if l >= 1 {
                        sd(*p as i32, v.to_bits() as i64)
                    } else {
                        swc1(*p as i32, v as f32)
                    }
                    count += 1;
                }
            }
            _ => break,
        }
    }
    count
}
